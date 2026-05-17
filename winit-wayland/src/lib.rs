//! # Winit's Wayland backend.
//!
//! **Note:** Windows don't appear on Wayland until you draw/present to them.
//!
//! By default, Winit loads system libraries using `dlopen`. This can be
//! disabled by disabling the `"wayland-dlopen"` cargo feature.
//!
//! ## Client-side decorations
//!
//! Winit provides client-side decorations by default, but the behaviour can
//! be controlled with the following feature flags:
//!
//! * `wayland-csd-adwaita` (default).
//! * `wayland-csd-adwaita-crossfont`.
//! * `wayland-csd-adwaita-notitle`.
//! * `wayland-csd-adwaita-notitlebar`.
use std::ffi::c_void;
use std::ptr::NonNull;

use dpi::{LogicalSize, PhysicalSize};
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::Proxy;
use sctk::shm::slot::{Buffer, CreateBufferError, SlotPool};
use wayland_client::protocol::wl_shm::Format;
use winit_core::event_loop::ActiveEventLoop as CoreActiveEventLoop;
use winit_core::window::{
    ActivationToken, PlatformWindowAttributes, Window as CoreWindow, WindowId,
};

macro_rules! os_error {
    ($error:expr) => {{
        winit_core::error::OsError::new(line!(), file!(), $error)
    }};
}

mod event_loop;
pub mod layer_shell;
mod output;
mod seat;
mod state;
mod types;
mod window;

pub use self::event_loop::{ActiveEventLoop, EventLoop};
pub use self::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellAttributes};
pub use self::window::Window;

/// Additional methods on [`ActiveEventLoop`] that are specific to Wayland.
pub trait ActiveEventLoopExtWayland {
    /// True if the [`ActiveEventLoop`] uses Wayland.
    fn is_wayland(&self) -> bool;
}

impl ActiveEventLoopExtWayland for dyn CoreActiveEventLoop + '_ {
    #[inline]
    fn is_wayland(&self) -> bool {
        self.cast_ref::<ActiveEventLoop>().is_some()
    }
}

/// Additional methods on [`EventLoop`] that are specific to Wayland.
pub trait EventLoopExtWayland {
    /// True if the [`EventLoop`] uses Wayland.
    fn is_wayland(&self) -> bool;
}

/// Additional methods when building event loop that are specific to Wayland.
pub trait EventLoopBuilderExtWayland {
    /// Force using Wayland.
    fn with_wayland(&mut self) -> &mut Self;

    /// Whether to allow the event loop to be created off of the main thread.
    ///
    /// By default, the window is only allowed to be created on the main
    /// thread, to make platform compatibility easier.
    fn with_any_thread(&mut self, any_thread: bool) -> &mut Self;
}

/// Additional methods on [`Window`] that are specific to Wayland.
///
/// [`Window`]: crate::window::Window
pub trait WindowExtWayland {
    /// Returns `xdg_toplevel` of the window or [`None`] if this is a layer
    /// surface.
    fn xdg_toplevel(&self) -> Option<NonNull<c_void>>;

    /// Returns the raw `zwlr_layer_surface_v1` pointer if this window is a
    /// layer surface built with [`WindowAttributesWayland::with_layer_shell`],
    /// or [`None`] if it is an xdg toplevel.
    fn wl_layer_surface(&self) -> Option<NonNull<c_void>>;
}

impl WindowExtWayland for dyn CoreWindow + '_ {
    #[inline]
    fn xdg_toplevel(&self) -> Option<NonNull<c_void>> {
        self.cast_ref::<Window>()?.xdg_toplevel()
    }

    #[inline]
    fn wl_layer_surface(&self) -> Option<NonNull<c_void>> {
        self.cast_ref::<Window>()?.wl_layer_surface()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApplicationName {
    pub(crate) general: String,
    pub(crate) instance: String,
}

/// Window attributes methods specific to Wayland.
#[derive(Debug, Default, Clone)]
pub struct WindowAttributesWayland {
    pub(crate) name: Option<ApplicationName>,
    pub(crate) activation_token: Option<ActivationToken>,
    pub(crate) layer_shell: Option<LayerShellAttributes>,
}

impl WindowAttributesWayland {
    /// Build window with the given name.
    ///
    /// The `general` name sets an application ID, which should match the `.desktop`
    /// file distributed with your program. The `instance` is a `no-op`.
    ///
    /// For details about application ID conventions, see the
    /// [Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#desktop-file-id)
    #[inline]
    pub fn with_name(mut self, general: impl Into<String>, instance: impl Into<String>) -> Self {
        self.name = Some(ApplicationName { general: general.into(), instance: instance.into() });
        self
    }

    #[inline]
    pub fn with_activation_token(mut self, token: ActivationToken) -> Self {
        self.activation_token = Some(token);
        self
    }

    /// Build the window as a `zwlr_layer_surface_v1` instead of an
    /// `xdg_toplevel`. Used for rofi-style overlays, panels, and lockers.
    ///
    /// The compositor must support the `wlr-layer-shell-unstable-v1`
    /// protocol; on compositors that don't (notably GNOME Mutter),
    /// [`crate::Window::new`] returns an error and the caller is expected to
    /// fall back to a regular toplevel.
    ///
    /// On X11 and other platforms this attribute is silently ignored.
    #[inline]
    pub fn with_layer_shell(mut self, attrs: LayerShellAttributes) -> Self {
        self.layer_shell = Some(attrs);
        self
    }
}

impl PlatformWindowAttributes for WindowAttributesWayland {
    fn box_clone(&self) -> Box<dyn PlatformWindowAttributes> {
        Box::from(self.clone())
    }
}

/// Get the WindowId out of the surface.
#[inline]
fn make_wid(surface: &WlSurface) -> WindowId {
    WindowId::from_raw(surface.id().as_ptr() as usize)
}

/// The default routine does floor, but we need round on Wayland.
fn logical_to_physical_rounded(size: LogicalSize<u32>, scale_factor: f64) -> PhysicalSize<u32> {
    let width = size.width as f64 * scale_factor;
    let height = size.height as f64 * scale_factor;
    (width.round(), height.round()).into()
}

/// Converts an image buffer to a Wayland buffer (`wl_buffer`)
fn image_to_buffer(
    width: i32,
    height: i32,
    data: &[u8],
    format: Format,
    pool: &mut SlotPool,
) -> Result<Buffer, CreateBufferError> {
    let (buffer, canvas) = pool.create_buffer(width, height, 4 * width, format)?;

    for (canvas_chunk, rgba) in canvas.chunks_exact_mut(4).zip(data.chunks_exact(4)) {
        // Alpha in buffer is premultiplied.
        let alpha = rgba[3] as f32 / 255.;
        let r = (rgba[0] as f32 * alpha) as u32;
        let g = (rgba[1] as f32 * alpha) as u32;
        let b = (rgba[2] as f32 * alpha) as u32;
        let color = ((rgba[3] as u32) << 24) + (r << 16) + (g << 8) + b;
        let array: &mut [u8; 4] = canvas_chunk.try_into().unwrap();
        *array = color.to_le_bytes();
    }

    Ok(buffer)
}
