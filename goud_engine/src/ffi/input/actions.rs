//! Action mapping FFI functions.
//!
//! Actions provide a semantic layer over raw input, allowing the same
//! action to be triggered by multiple inputs.

use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::codes::GoudKeyCode;
use super::helpers::{key_from_code, with_input};

/// Maps an action name to a keyboard key.
///
/// Actions provide a semantic layer over raw input, allowing the same
/// action to be triggered by multiple inputs.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `action_name` - Null-terminated action name string
/// * `key` - The key code to bind
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `action_name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_input_map_action_key(
    context_id: GoudContextId,
    action_name: *const std::os::raw::c_char,
    key: GoudKeyCode,
) -> bool {
    use crate::core::input_manager::InputBinding;
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || action_name.is_null() {
        return false;
    }

    // SAFETY: caller guarantees action_name is a valid null-terminated C string.
    let name = match unsafe { CStr::from_ptr(action_name) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return false,
    };

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return false,
    };

    let context = match registry.get_mut(context_id) {
        Some(c) => c,
        None => return false,
    };

    let input = match context.world_mut().resource_mut::<InputManager>() {
        Some(i) => i,
        None => return false,
    };

    input
        .into_inner()
        .map_action(&name, InputBinding::Key(key_from_code(key)));
    true
}

/// Returns `true` if the specified action is currently pressed.
///
/// An action is pressed if ANY of its bound inputs are pressed.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `action_name` - Null-terminated action name string
///
/// # Returns
///
/// `true` if action is pressed, `false` otherwise.
///
/// # Safety
///
/// `action_name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_input_action_pressed(
    context_id: GoudContextId,
    action_name: *const std::os::raw::c_char,
) -> bool {
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || action_name.is_null() {
        return false;
    }

    // SAFETY: caller guarantees action_name is a valid null-terminated C string.
    let name = match unsafe { CStr::from_ptr(action_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    with_input(context_id, |input| input.action_pressed(name)).unwrap_or(false)
}

/// Returns `true` if the specified action was just pressed this frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `action_name` - Null-terminated action name string
///
/// # Returns
///
/// `true` if action was just pressed, `false` otherwise.
///
/// # Safety
///
/// `action_name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_input_action_just_pressed(
    context_id: GoudContextId,
    action_name: *const std::os::raw::c_char,
) -> bool {
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || action_name.is_null() {
        return false;
    }

    // SAFETY: caller guarantees action_name is a valid null-terminated C string.
    let name = match unsafe { CStr::from_ptr(action_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    with_input(context_id, |input| input.action_just_pressed(name)).unwrap_or(false)
}

/// Returns `true` if the specified action was just released this frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `action_name` - Null-terminated action name string
///
/// # Returns
///
/// `true` if action was just released, `false` otherwise.
///
/// # Safety
///
/// `action_name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_input_action_just_released(
    context_id: GoudContextId,
    action_name: *const std::os::raw::c_char,
) -> bool {
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || action_name.is_null() {
        return false;
    }

    // SAFETY: caller guarantees action_name is a valid null-terminated C string.
    let name = match unsafe { CStr::from_ptr(action_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    with_input(context_id, |input| input.action_just_released(name)).unwrap_or(false)
}
