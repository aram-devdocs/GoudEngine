//! Gamepad input FFI functions: buttons, axes, sticks, triggers, and connection status.

use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::input_types::GamepadAxis;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::helpers::with_input;

use crate::core::input_manager::MAX_GAMEPAD_SLOTS;

/// Maximum supported gamepad index (exclusive). IDs >= this value are rejected.
const MAX_GAMEPAD_ID: u32 = MAX_GAMEPAD_SLOTS as u32;

/// Converts a raw FFI axis code to a [`GamepadAxis`].
///
/// Returns `None` for unknown axis values.
fn axis_from_code(code: u32) -> Option<GamepadAxis> {
    // SAFETY: GamepadAxis is #[repr(u32)] with discriminants 0..=5 matching the
    // FFI constants defined in codes.rs. We validate the range explicitly.
    match code {
        0 => Some(GamepadAxis::LeftStickX),
        1 => Some(GamepadAxis::LeftStickY),
        2 => Some(GamepadAxis::RightStickX),
        3 => Some(GamepadAxis::RightStickY),
        4 => Some(GamepadAxis::LeftTrigger),
        5 => Some(GamepadAxis::RightTrigger),
        _ => None,
    }
}

/// Returns `true` if the specified gamepad button is currently pressed.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `button` - Button code (see `GAMEPAD_BUTTON_*` constants)
///
/// # Returns
///
/// `true` if the button is pressed, `false` otherwise or on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_button_pressed(
    context_id: GoudContextId,
    gamepad_id: u32,
    button: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.gamepad_button_pressed(gamepad_id as usize, button)
    })
    .unwrap_or(false)
}

/// Returns `true` if the specified gamepad button was just pressed this frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `button` - Button code (see `GAMEPAD_BUTTON_*` constants)
///
/// # Returns
///
/// `true` if the button was just pressed, `false` otherwise or on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_button_just_pressed(
    context_id: GoudContextId,
    gamepad_id: u32,
    button: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.gamepad_button_just_pressed(gamepad_id as usize, button)
    })
    .unwrap_or(false)
}

/// Returns `true` if the specified gamepad button was just released this frame.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `button` - Button code (see `GAMEPAD_BUTTON_*` constants)
///
/// # Returns
///
/// `true` if the button was just released, `false` otherwise or on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_button_just_released(
    context_id: GoudContextId,
    gamepad_id: u32,
    button: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.gamepad_button_just_released(gamepad_id as usize, button)
    })
    .unwrap_or(false)
}

/// Returns the current value of a gamepad analog axis (-1.0 to 1.0).
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `axis` - Axis code (see `GAMEPAD_AXIS_*` constants)
///
/// # Returns
///
/// The axis value, or `0.0` on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_axis(
    context_id: GoudContextId,
    gamepad_id: u32,
    axis: u32,
) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0.0;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return 0.0;
    }
    let Some(engine_axis) = axis_from_code(axis) else {
        return 0.0;
    };

    with_input(context_id, |input| {
        input.gamepad_axis(gamepad_id as usize, engine_axis)
    })
    .unwrap_or(0.0)
}

/// Returns `true` if the specified gamepad is currently connected.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
///
/// # Returns
///
/// `true` if the gamepad is connected, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_connected(context_id: GoudContextId, gamepad_id: u32) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    with_input(context_id, |input| {
        input.is_gamepad_connected(gamepad_id as usize)
    })
    .unwrap_or(false)
}

/// Returns the number of currently connected gamepads.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
///
/// # Returns
///
/// The count of connected gamepads (0-4), or `0` on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_connected_count(context_id: GoudContextId) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    with_input(context_id, |input| input.connected_gamepad_count() as u32).unwrap_or(0)
}

/// Sets the vibration intensity for a gamepad (0.0-1.0).
///
/// Note: Actual vibration requires platform-layer support. This stores the
/// requested intensity in the InputManager for the platform to read.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `intensity` - Vibration intensity (0.0 = off, 1.0 = max)
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_set_vibration(
    context_id: GoudContextId,
    gamepad_id: u32,
    intensity: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    // Vibration requires mutable access to the InputManager.
    let mut registry = match crate::ffi::context::get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let Some(context) = registry.get_mut(context_id) else {
        return false;
    };
    let Some(input) = context
        .world_mut()
        .resource_mut::<crate::ecs::InputManager>()
    else {
        return false;
    };
    input
        .into_inner()
        .set_gamepad_vibration(gamepad_id as usize, intensity);
    true
}

/// Writes the left stick position to the output pointers.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `out_x` - Pointer to store X axis value (-1.0 to 1.0)
/// * `out_y` - Pointer to store Y axis value (-1.0 to 1.0)
///
/// # Returns
///
/// `true` on success, `false` on error or null pointers.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid, aligned, non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_gamepad_left_stick(
    context_id: GoudContextId,
    gamepad_id: u32,
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
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    with_input(context_id, |input| {
        let stick = input.gamepad_left_stick(gamepad_id as usize);
        // SAFETY: out_x and out_y are non-null and valid, checked above.
        unsafe {
            *out_x = stick.x;
            *out_y = stick.y;
        }
        true
    })
    .unwrap_or(false)
}

/// Writes the right stick position to the output pointers.
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
/// * `out_x` - Pointer to store X axis value (-1.0 to 1.0)
/// * `out_y` - Pointer to store Y axis value (-1.0 to 1.0)
///
/// # Returns
///
/// `true` on success, `false` on error or null pointers.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid, aligned, non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_gamepad_right_stick(
    context_id: GoudContextId,
    gamepad_id: u32,
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
    if gamepad_id >= MAX_GAMEPAD_ID {
        return false;
    }

    with_input(context_id, |input| {
        let stick = input.gamepad_right_stick(gamepad_id as usize);
        // SAFETY: out_x and out_y are non-null and valid, checked above.
        unsafe {
            *out_x = stick.x;
            *out_y = stick.y;
        }
        true
    })
    .unwrap_or(false)
}

/// Returns the left trigger value (0.0 to 1.0).
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
///
/// # Returns
///
/// The trigger value (0.0 = released, 1.0 = fully pressed), or `0.0` on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_left_trigger(
    context_id: GoudContextId,
    gamepad_id: u32,
) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0.0;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return 0.0;
    }

    with_input(context_id, |input| {
        input.gamepad_left_trigger(gamepad_id as usize)
    })
    .unwrap_or(0.0)
}

/// Returns the right trigger value (0.0 to 1.0).
///
/// # Arguments
///
/// * `context_id` - The context with InputManager
/// * `gamepad_id` - Gamepad index (0-3)
///
/// # Returns
///
/// The trigger value (0.0 = released, 1.0 = fully pressed), or `0.0` on error.
#[no_mangle]
pub extern "C" fn goud_input_gamepad_right_trigger(
    context_id: GoudContextId,
    gamepad_id: u32,
) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0.0;
    }
    if gamepad_id >= MAX_GAMEPAD_ID {
        return 0.0;
    }

    with_input(context_id, |input| {
        input.gamepad_right_trigger(gamepad_id as usize)
    })
    .unwrap_or(0.0)
}

#[cfg(test)]
#[path = "gamepad_tests.rs"]
mod tests;
