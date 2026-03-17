//! Shared helper utilities for the input FFI layer.

use crate::core::providers::input_types::{KeyCode as Key, MouseButton};
use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId};

use super::codes::{GoudKeyCode, GoudMouseButton};

/// Converts an FFI key code to a platform-neutral key code.
pub(super) fn key_from_code(code: GoudKeyCode) -> Key {
    // SAFETY: GoudKeyCode values are defined to match the platform-neutral key
    // discriminants used by the engine's input contract.
    unsafe { std::mem::transmute(code) }
}

/// Converts an FFI mouse button code to a platform-neutral mouse button.
pub(super) fn mouse_button_from_code(code: GoudMouseButton) -> MouseButton {
    match code {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Button4,
        4 => MouseButton::Button5,
        _ => MouseButton::Left,
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
