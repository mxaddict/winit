//! Public types for `zwlr_layer_surface_v1` support.
//!
//! Build a window with [`LayerShellAttributes`] via
//! [`WindowAttributesWayland::with_layer_shell`] to create a layer surface
//! instead of an `xdg_toplevel`. The compositor must advertise
//! `wlr-layer-shell-unstable-v1`; on compositors that don't (notably GNOME
//! Mutter), [`crate::Window::new`] returns an error and the caller is expected
//! to fall back to a regular toplevel.

use bitflags::bitflags;

use winit_core::monitor::MonitorHandle;

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
        const TOP    = 1;
        const BOTTOM = 2;
        const LEFT   = 4;
        const RIGHT  = 8;
    }
}

/// How a layer surface participates in keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardInteractivity {
    /// Surface receives no keyboard input.
    None,
    /// Surface receives keyboard input when focused by the compositor.
    /// This is what rofi, wofi, and similar pickers use: the compositor
    /// grants focus on click / per-anchor policy and lets compositor
    /// bindings continue to fire alongside.
    OnDemand,
    /// Surface captures all keyboard input while visible. Used by
    /// screen lockers and login prompts; the compositor sends key events
    /// here even when the previously-focused toplevel would otherwise
    /// have them.
    Exclusive,
}

impl Default for KeyboardInteractivity {
    fn default() -> Self {
        // OnDemand by default — Exclusive behaves like a lockscreen and
        // suppresses every compositor binding including window-close
        // shortcuts. Callers like screen lockers can opt in explicitly.
        KeyboardInteractivity::OnDemand
    }
}

/// Attributes describing a `zwlr_layer_surface_v1`.
///
/// Build a window with these attributes via
/// [`WindowAttributesWayland::with_layer_shell`] to create a layer surface
/// instead of an `xdg_toplevel`.
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
            keyboard_interactivity: KeyboardInteractivity::default(),
            namespace: String::from("winit"),
            output: None,
        }
    }
}
