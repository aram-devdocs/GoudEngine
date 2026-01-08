//! # FFI Input Module
//!
//! This module provides C-compatible functions for querying input state.
//! It integrates with the ECS InputManager resource to expose keyboard,
//! mouse, and gamepad state to the C# SDK.
//!
//! ## Design
//!
//! The input FFI provides query functions that read from the InputManager
//! resource. The InputManager is updated by the window FFI during event polling.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // In game loop after goud_window_poll_events:
//! if (goud_input_key_pressed(contextId, KeyCode.W)) {
//!     MoveForward(speed * deltaTime);
//! }
//!
//! if (goud_input_key_just_pressed(contextId, KeyCode.Space)) {
//!     Jump();
//! }
//!
//! float mouseX, mouseY;
//! goud_input_get_mouse_position(contextId, out mouseX, out mouseY);
//! ```

use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use glfw::{Key, MouseButton};

// ============================================================================
// Key Codes (FFI-compatible)
// ============================================================================

/// FFI-compatible key code.
///
/// These map directly to GLFW key codes for compatibility.
pub type GoudKeyCode = i32;

// Key code constants (matching GLFW key codes).
// These are self-documenting by name and intentionally have no docs.
#[allow(missing_docs)]
mod key_codes {
    use super::GoudKeyCode;
    pub const KEY_UNKNOWN: GoudKeyCode = -1;
    pub const KEY_SPACE: GoudKeyCode = 32;
    pub const KEY_APOSTROPHE: GoudKeyCode = 39;
    pub const KEY_COMMA: GoudKeyCode = 44;
    pub const KEY_MINUS: GoudKeyCode = 45;
    pub const KEY_PERIOD: GoudKeyCode = 46;
    pub const KEY_SLASH: GoudKeyCode = 47;
    pub const KEY_0: GoudKeyCode = 48;
    pub const KEY_1: GoudKeyCode = 49;
    pub const KEY_2: GoudKeyCode = 50;
    pub const KEY_3: GoudKeyCode = 51;
    pub const KEY_4: GoudKeyCode = 52;
    pub const KEY_5: GoudKeyCode = 53;
    pub const KEY_6: GoudKeyCode = 54;
    pub const KEY_7: GoudKeyCode = 55;
    pub const KEY_8: GoudKeyCode = 56;
    pub const KEY_9: GoudKeyCode = 57;
    pub const KEY_A: GoudKeyCode = 65;
    pub const KEY_B: GoudKeyCode = 66;
    pub const KEY_C: GoudKeyCode = 67;
    pub const KEY_D: GoudKeyCode = 68;
    pub const KEY_E: GoudKeyCode = 69;
    pub const KEY_F: GoudKeyCode = 70;
    pub const KEY_G: GoudKeyCode = 71;
    pub const KEY_H: GoudKeyCode = 72;
    pub const KEY_I: GoudKeyCode = 73;
    pub const KEY_J: GoudKeyCode = 74;
    pub const KEY_K: GoudKeyCode = 75;
    pub const KEY_L: GoudKeyCode = 76;
    pub const KEY_M: GoudKeyCode = 77;
    pub const KEY_N: GoudKeyCode = 78;
    pub const KEY_O: GoudKeyCode = 79;
    pub const KEY_P: GoudKeyCode = 80;
    pub const KEY_Q: GoudKeyCode = 81;
    pub const KEY_R: GoudKeyCode = 82;
    pub const KEY_S: GoudKeyCode = 83;
    pub const KEY_T: GoudKeyCode = 84;
    pub const KEY_U: GoudKeyCode = 85;
    pub const KEY_V: GoudKeyCode = 86;
    pub const KEY_W: GoudKeyCode = 87;
    pub const KEY_X: GoudKeyCode = 88;
    pub const KEY_Y: GoudKeyCode = 89;
    pub const KEY_Z: GoudKeyCode = 90;
    pub const KEY_ESCAPE: GoudKeyCode = 256;
    pub const KEY_ENTER: GoudKeyCode = 257;
    pub const KEY_TAB: GoudKeyCode = 258;
    pub const KEY_BACKSPACE: GoudKeyCode = 259;
    pub const KEY_INSERT: GoudKeyCode = 260;
    pub const KEY_DELETE: GoudKeyCode = 261;
    pub const KEY_RIGHT: GoudKeyCode = 262;
    pub const KEY_LEFT: GoudKeyCode = 263;
    pub const KEY_DOWN: GoudKeyCode = 264;
    pub const KEY_UP: GoudKeyCode = 265;
    pub const KEY_PAGE_UP: GoudKeyCode = 266;
    pub const KEY_PAGE_DOWN: GoudKeyCode = 267;
    pub const KEY_HOME: GoudKeyCode = 268;
    pub const KEY_END: GoudKeyCode = 269;
    pub const KEY_F1: GoudKeyCode = 290;
    pub const KEY_F2: GoudKeyCode = 291;
    pub const KEY_F3: GoudKeyCode = 292;
    pub const KEY_F4: GoudKeyCode = 293;
    pub const KEY_F5: GoudKeyCode = 294;
    pub const KEY_F6: GoudKeyCode = 295;
    pub const KEY_F7: GoudKeyCode = 296;
    pub const KEY_F8: GoudKeyCode = 297;
    pub const KEY_F9: GoudKeyCode = 298;
    pub const KEY_F10: GoudKeyCode = 299;
    pub const KEY_F11: GoudKeyCode = 300;
    pub const KEY_F12: GoudKeyCode = 301;
    pub const KEY_LEFT_SHIFT: GoudKeyCode = 340;
    pub const KEY_LEFT_CONTROL: GoudKeyCode = 341;
    pub const KEY_LEFT_ALT: GoudKeyCode = 342;
    pub const KEY_LEFT_SUPER: GoudKeyCode = 343;
    pub const KEY_RIGHT_SHIFT: GoudKeyCode = 344;
    pub const KEY_RIGHT_CONTROL: GoudKeyCode = 345;
    pub const KEY_RIGHT_ALT: GoudKeyCode = 346;
    pub const KEY_RIGHT_SUPER: GoudKeyCode = 347;
}
pub use key_codes::*;

// ============================================================================
// Mouse Button Codes (FFI-compatible)
// ============================================================================

/// FFI-compatible mouse button code.
pub type GoudMouseButton = i32;

// Mouse button constants (matching GLFW button codes).
#[allow(missing_docs)]
mod mouse_buttons {
    use super::GoudMouseButton;
    pub const MOUSE_BUTTON_LEFT: GoudMouseButton = 0;
    pub const MOUSE_BUTTON_RIGHT: GoudMouseButton = 1;
    pub const MOUSE_BUTTON_MIDDLE: GoudMouseButton = 2;
    pub const MOUSE_BUTTON_4: GoudMouseButton = 3;
    pub const MOUSE_BUTTON_5: GoudMouseButton = 4;
    pub const MOUSE_BUTTON_6: GoudMouseButton = 5;
    pub const MOUSE_BUTTON_7: GoudMouseButton = 6;
    pub const MOUSE_BUTTON_8: GoudMouseButton = 7;
}
pub use mouse_buttons::*;

// ============================================================================
// Helper Functions
// ============================================================================

/// Converts an FFI key code to a GLFW Key.
fn key_from_code(code: GoudKeyCode) -> Key {
    // GLFW Key enum uses the same values as the raw key codes
    unsafe { std::mem::transmute(code) }
}

/// Converts an FFI mouse button code to a GLFW MouseButton.
fn mouse_button_from_code(code: GoudMouseButton) -> MouseButton {
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
fn with_input<F, R>(context_id: GoudContextId, f: F) -> Option<R>
where
    F: FnOnce(&InputManager) -> R,
{
    let registry = get_context_registry().lock().ok()?;
    let context = registry.get(context_id)?;
    let input = context.world().resource::<InputManager>()?;
    Some(f(&input))
}

// ============================================================================
// Keyboard Input FFI
// ============================================================================

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

// ============================================================================
// Mouse Button Input FFI
// ============================================================================

/// Returns `true` if the specified mouse button is currently pressed.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `button` - The mouse button code (0=left, 1=right, 2=middle)
///
/// # Returns
///
/// `true` if button is pressed, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_mouse_button_pressed(
    context_id: GoudContextId,
    button: GoudMouseButton,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.mouse_button_pressed(mouse_button_from_code(button))
    })
    .unwrap_or(false)
}

/// Returns `true` if the specified mouse button was just pressed this frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `button` - The mouse button code
///
/// # Returns
///
/// `true` if button was just pressed, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_mouse_button_just_pressed(
    context_id: GoudContextId,
    button: GoudMouseButton,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.mouse_button_just_pressed(mouse_button_from_code(button))
    })
    .unwrap_or(false)
}

/// Returns `true` if the specified mouse button was just released this frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `button` - The mouse button code
///
/// # Returns
///
/// `true` if button was just released, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_mouse_button_just_released(
    context_id: GoudContextId,
    button: GoudMouseButton,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.mouse_button_just_released(mouse_button_from_code(button))
    })
    .unwrap_or(false)
}

// ============================================================================
// Mouse Position FFI
// ============================================================================

/// Gets the current mouse position in screen coordinates.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `out_x` - Pointer to store X coordinate
/// * `out_y` - Pointer to store Y coordinate
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_get_mouse_position(
    context_id: GoudContextId,
    out_x: *mut f32,
    out_y: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || out_x.is_null() || out_y.is_null() {
        return false;
    }

    with_input(context_id, |input| {
        let pos = input.mouse_position();
        *out_x = pos.x;
        *out_y = pos.y;
        true
    })
    .unwrap_or(false)
}

/// Gets the mouse movement delta since the last frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `out_dx` - Pointer to store X movement
/// * `out_dy` - Pointer to store Y movement
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `out_dx` and `out_dy` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_get_mouse_delta(
    context_id: GoudContextId,
    out_dx: *mut f32,
    out_dy: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || out_dx.is_null() || out_dy.is_null() {
        return false;
    }

    with_input(context_id, |input| {
        let delta = input.mouse_delta();
        *out_dx = delta.x;
        *out_dy = delta.y;
        true
    })
    .unwrap_or(false)
}

/// Gets the scroll wheel delta since the last frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `out_dx` - Pointer to store horizontal scroll (usually 0)
/// * `out_dy` - Pointer to store vertical scroll
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `out_dx` and `out_dy` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_get_scroll_delta(
    context_id: GoudContextId,
    out_dx: *mut f32,
    out_dy: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || out_dx.is_null() || out_dy.is_null() {
        return false;
    }

    with_input(context_id, |input| {
        let delta = input.scroll_delta();
        *out_dx = delta.x;
        *out_dy = delta.y;
        true
    })
    .unwrap_or(false)
}

// ============================================================================
// Action Mapping FFI
// ============================================================================

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
    use crate::ecs::input_manager::InputBinding;
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || action_name.is_null() {
        return false;
    }

    let name = match CStr::from_ptr(action_name).to_str() {
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

    let name = match CStr::from_ptr(action_name).to_str() {
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

    let name = match CStr::from_ptr(action_name).to_str() {
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

    let name = match CStr::from_ptr(action_name).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    with_input(context_id, |input| input.action_just_released(name)).unwrap_or(false)
}
