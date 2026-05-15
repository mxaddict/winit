use bitflags::bitflags;

use crate::{
    event_loop::{EventLoopBuilder, EventLoopWindowTarget},
    monitor::MonitorHandle,
    window::{Window, WindowBuilder},
};

use crate::platform_impl::{ApplicationName, Backend};

pub use crate::window::Theme;

/// Z-layer for a `zwlr_layer_surface_v1`.
///
/// Matches the four values defined by the `wlr-layer-shell-unstable-v1`
/// protocol. `Top` and `Overlay` paint above normal toplevels; `Bottom`
/// and `Background` paint below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Layer {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

bitflags! {
    /// Edges of the output a layer surface is anchored to.
    ///
    /// Setting both opposite anchors (`TOP | BOTTOM` or `LEFT | RIGHT`)
    /// stretches the surface across that axis. Setting all four anchors
    /// makes the surface fullscreen on the output.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Anchor: u32 {
        const TOP    = 1 << 0;
        const BOTTOM = 1 << 1;
        const LEFT   = 1 << 2;
        const RIGHT  = 1 << 3;
    }
}

/// How a layer surface participates in keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KeyboardInteractivity {
    /// Surface receives no keyboard input.
    #[default]
    None,
    /// Surface receives keyboard input when focused by the compositor.
    OnDemand,
    /// Surface captures all keyboard input while visible. Used by
    /// rofi-style launchers; the compositor sends key events here even
    /// when the previously-focused toplevel would otherwise have them.
    Exclusive,
}

/// Attributes describing a `zwlr_layer_surface_v1`.
///
/// Building a window with these attributes (via
/// [`WindowBuilderExtWayland::with_layer_shell`]) creates a layer surface
/// instead of an `xdg_toplevel`. The compositor must support the
/// `wlr-layer-shell-unstable-v1` protocol; on compositors that don't
/// (notably GNOME Mutter), `Window::new` returns an error and the caller
/// is expected to fall back to a regular toplevel.
#[derive(Debug, Clone)]
pub struct LayerShellAttributes {
    pub layer: Layer,
    pub anchor: Anchor,
    /// Distance the surface reserves on its anchor edge. `-1` means
    /// "ignore other exclusive zones on this output" (typical for
    /// overlays like launchers). `0` requests no reservation.
    pub exclusive_zone: i32,
    /// `(top, right, bottom, left)` margins in surface-local pixels.
    pub margin: (i32, i32, i32, i32),
    pub keyboard_interactivity: KeyboardInteractivity,
    /// Used by the compositor for debugging / IPC (e.g. `hyprctl layers`).
    pub namespace: String,
    /// Output to place the surface on. `None` lets the compositor pick.
    pub output: Option<MonitorHandle>,
}

impl Default for LayerShellAttributes {
    fn default() -> Self {
        Self {
            layer: Layer::default(),
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            exclusive_zone: -1,
            margin: (0, 0, 0, 0),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            namespace: String::from("winit"),
            output: None,
        }
    }
}

/// Additional methods on [`EventLoopWindowTarget`] that are specific to Wayland.
pub trait EventLoopWindowTargetExtWayland {
    /// True if the [`EventLoopWindowTarget`] uses Wayland.
    fn is_wayland(&self) -> bool;
}

impl<T> EventLoopWindowTargetExtWayland for EventLoopWindowTarget<T> {
    #[inline]
    fn is_wayland(&self) -> bool {
        self.p.is_wayland()
    }
}

/// Additional methods on [`EventLoopBuilder`] that are specific to Wayland.
pub trait EventLoopBuilderExtWayland {
    /// Force using Wayland.
    fn with_wayland(&mut self) -> &mut Self;

    /// Whether to allow the event loop to be created off of the main thread.
    ///
    /// By default, the window is only allowed to be created on the main
    /// thread, to make platform compatibility easier.
    fn with_any_thread(&mut self, any_thread: bool) -> &mut Self;
}

impl<T> EventLoopBuilderExtWayland for EventLoopBuilder<T> {
    #[inline]
    fn with_wayland(&mut self) -> &mut Self {
        self.platform_specific.forced_backend = Some(Backend::Wayland);
        self
    }

    #[inline]
    fn with_any_thread(&mut self, any_thread: bool) -> &mut Self {
        self.platform_specific.any_thread = any_thread;
        self
    }
}

/// Additional methods on [`Window`] that are specific to Wayland.
pub trait WindowExtWayland {}

impl WindowExtWayland for Window {}

/// Additional methods on [`WindowBuilder`] that are specific to Wayland.
pub trait WindowBuilderExtWayland {
    /// Build window with the given name.
    ///
    /// The `general` name sets an application ID, which should match the `.desktop`
    /// file destributed with your program. The `instance` is a `no-op`.
    ///
    /// For details about application ID conventions, see the
    /// [Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#desktop-file-id)
    fn with_name(self, general: impl Into<String>, instance: impl Into<String>) -> Self;

    /// Build the window as a `zwlr_layer_surface_v1` instead of an
    /// `xdg_toplevel`. Used for rofi-style overlays, panels, and lockers.
    ///
    /// On X11 and other platforms this attribute is ignored.
    fn with_layer_shell(self, attrs: LayerShellAttributes) -> Self;
}

impl WindowBuilderExtWayland for WindowBuilder {
    #[inline]
    fn with_name(mut self, general: impl Into<String>, instance: impl Into<String>) -> Self {
        self.platform_specific.name = Some(ApplicationName::new(general.into(), instance.into()));
        self
    }

    #[inline]
    fn with_layer_shell(mut self, attrs: LayerShellAttributes) -> Self {
        self.platform_specific.wayland.layer_shell = Some(attrs);
        self
    }
}

/// Additional methods on `MonitorHandle` that are specific to Wayland.
pub trait MonitorHandleExtWayland {
    /// Returns the inner identifier of the monitor.
    fn native_id(&self) -> u32;
}

impl MonitorHandleExtWayland for MonitorHandle {
    #[inline]
    fn native_id(&self) -> u32 {
        self.inner.native_identifier()
    }
}
