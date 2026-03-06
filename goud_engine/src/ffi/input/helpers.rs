//! Shared helper utilities for the input FFI layer.

use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId};
use glfw::{Key, MouseButton};

use super::codes::{GoudKeyCode, GoudMouseButton};

/// Converts an FFI key code to a GLFW Key.
pub(super) fn key_from_code(code: GoudKeyCode) -> Key {
    // GLFW Key enum uses the same values as the raw key codes
    // SAFETY: GoudKeyCode values are defined to match GLFW key discriminants exactly.
    unsafe { std::mem::transmute(code) }
}

/// Converts an FFI mouse button code to a GLFW MouseButton.
pub(super) fn mouse_button_from_code(code: GoudMouseButton) -> MouseButton {
    match code {
        0 => MouseButton::Button1,
        1 => MouseButton::Button2,
        2 => MouseButton::Button3,
        3 => MouseButton::Button4,
        4 => MouseButton::Button5,
        5 => MouseButton::Button6,
        6 => MouseButton::Button7,
        7 => MouseButton::Button8,
        _ => MouseButton::Button1, // Default to left button
    }
}

/// Helper to access InputManager from context.
pub(super) fn with_input<F, R>(context_id: GoudContextId, f: F) -> Option<R>
where
    F: FnOnce(&InputManager) -> R,
{
    let registry = get_context_registry().lock().ok()?;
    let context = registry.get(context_id)?;
    let input = context.world().resource::<InputManager>()?;
    Some(f(&input))
}
