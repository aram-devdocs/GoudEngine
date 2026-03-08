//! Mouse input FFI functions (buttons, position, delta, scroll).

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::codes::GoudMouseButton;
use super::helpers::{mouse_button_from_code, with_input};

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
        set_last_error(GoudError::InvalidContext);
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
        set_last_error(GoudError::InvalidContext);
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
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_input(context_id, |input| {
        input.mouse_button_just_released(mouse_button_from_code(button))
    })
    .unwrap_or(false)
}

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
/// `out_x` and `out_y` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_get_mouse_position(
    context_id: GoudContextId,
    out_x: *mut f32,
    out_y: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if out_x.is_null() || out_y.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    // SAFETY: caller guarantees out_x and out_y are valid non-null pointers.
    with_input(context_id, |input| {
        let pos = input.mouse_position();
        // SAFETY: out_x and out_y are non-null and valid, checked above.
        unsafe {
            *out_x = pos.x;
            *out_y = pos.y;
        }
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
/// `out_dx` and `out_dy` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_get_mouse_delta(
    context_id: GoudContextId,
    out_dx: *mut f32,
    out_dy: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if out_dx.is_null() || out_dy.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    // SAFETY: caller guarantees out_dx and out_dy are valid non-null pointers.
    with_input(context_id, |input| {
        let delta = input.mouse_delta();
        // SAFETY: out_dx and out_dy are non-null and valid, checked above.
        unsafe {
            *out_dx = delta.x;
            *out_dy = delta.y;
        }
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
/// `out_dx` and `out_dy` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_get_scroll_delta(
    context_id: GoudContextId,
    out_dx: *mut f32,
    out_dy: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if out_dx.is_null() || out_dy.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    // SAFETY: caller guarantees out_dx and out_dy are valid non-null pointers.
    with_input(context_id, |input| {
        let delta = input.scroll_delta();
        // SAFETY: out_dx and out_dy are non-null and valid, checked above.
        unsafe {
            *out_dx = delta.x;
            *out_dy = delta.y;
        }
        true
    })
    .unwrap_or(false)
}
