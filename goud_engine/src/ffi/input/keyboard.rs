//! Keyboard input FFI functions.

use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::codes::GoudKeyCode;
use super::helpers::{key_from_code, with_input};

/// Returns `true` if the specified key is currently pressed.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `key` - The key code to check
///
/// # Returns
///
/// `true` if key is pressed, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_key_pressed(context_id: GoudContextId, key: GoudKeyCode) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_input(context_id, |input| input.key_pressed(key_from_code(key))).unwrap_or(false)
}

/// Returns `true` if the specified key was just pressed this frame.
///
/// This returns `true` only on the first frame the key is pressed.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `key` - The key code to check
///
/// # Returns
///
/// `true` if key was just pressed, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_key_just_pressed(context_id: GoudContextId, key: GoudKeyCode) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.key_just_pressed(key_from_code(key))
    })
    .unwrap_or(false)
}

/// Returns `true` if the specified key was just released this frame.
///
/// This returns `true` only on the first frame after the key is released.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `key` - The key code to check
///
/// # Returns
///
/// `true` if key was just released, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_key_just_released(
    context_id: GoudContextId,
    key: GoudKeyCode,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.key_just_released(key_from_code(key))
    })
    .unwrap_or(false)
}
