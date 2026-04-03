//! Tests for gamepad FFI functions: invalid context, null pointer, and boundary checks.

use super::*;

// -----------------------------------------------------------------------
// Invalid context tests — all gamepad functions must reject INVALID_CONTEXT_ID
// -----------------------------------------------------------------------

#[test]
fn button_pressed_invalid_context_returns_false() {
    assert!(!goud_input_gamepad_button_pressed(
        GOUD_INVALID_CONTEXT_ID,
        0,
        0
    ));
}

#[test]
fn button_just_pressed_invalid_context_returns_false() {
    assert!(!goud_input_gamepad_button_just_pressed(
        GOUD_INVALID_CONTEXT_ID,
        0,
        0
    ));
}

#[test]
fn button_just_released_invalid_context_returns_false() {
    assert!(!goud_input_gamepad_button_just_released(
        GOUD_INVALID_CONTEXT_ID,
        0,
        0
    ));
}

#[test]
fn axis_invalid_context_returns_zero() {
    assert_eq!(goud_input_gamepad_axis(GOUD_INVALID_CONTEXT_ID, 0, 0), 0.0);
}

#[test]
fn connected_invalid_context_returns_false() {
    assert!(!goud_input_gamepad_connected(GOUD_INVALID_CONTEXT_ID, 0));
}

#[test]
fn connected_count_invalid_context_returns_zero() {
    assert_eq!(
        goud_input_gamepad_connected_count(GOUD_INVALID_CONTEXT_ID),
        0
    );
}

#[test]
fn set_vibration_invalid_context_returns_false() {
    assert!(!goud_input_gamepad_set_vibration(
        GOUD_INVALID_CONTEXT_ID,
        0,
        0.5
    ));
}

#[test]
fn left_trigger_invalid_context_returns_zero() {
    assert_eq!(
        goud_input_gamepad_left_trigger(GOUD_INVALID_CONTEXT_ID, 0),
        0.0
    );
}

#[test]
fn right_trigger_invalid_context_returns_zero() {
    assert_eq!(
        goud_input_gamepad_right_trigger(GOUD_INVALID_CONTEXT_ID, 0),
        0.0
    );
}

// -----------------------------------------------------------------------
// Null-pointer tests for stick output functions
// -----------------------------------------------------------------------

#[test]
fn left_stick_null_out_x_returns_false() {
    let mut y: f32 = 0.0;
    // SAFETY: Passing null for out_x deliberately to test the null guard.
    let result = unsafe {
        goud_input_gamepad_left_stick(GOUD_INVALID_CONTEXT_ID, 0, std::ptr::null_mut(), &mut y)
    };
    assert!(!result);
}

#[test]
fn left_stick_null_out_y_returns_false() {
    let mut x: f32 = 0.0;
    // SAFETY: Passing null for out_y deliberately to test the null guard.
    let result = unsafe {
        goud_input_gamepad_left_stick(GOUD_INVALID_CONTEXT_ID, 0, &mut x, std::ptr::null_mut())
    };
    assert!(!result);
}

#[test]
fn left_stick_both_null_returns_false() {
    // SAFETY: Passing null for both pointers to test the null guard.
    let result = unsafe {
        goud_input_gamepad_left_stick(
            GOUD_INVALID_CONTEXT_ID,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(!result);
}

#[test]
fn right_stick_null_out_x_returns_false() {
    let mut y: f32 = 0.0;
    // SAFETY: Passing null for out_x deliberately to test the null guard.
    let result = unsafe {
        goud_input_gamepad_right_stick(GOUD_INVALID_CONTEXT_ID, 0, std::ptr::null_mut(), &mut y)
    };
    assert!(!result);
}

#[test]
fn right_stick_null_out_y_returns_false() {
    let mut x: f32 = 0.0;
    // SAFETY: Passing null for out_y deliberately to test the null guard.
    let result = unsafe {
        goud_input_gamepad_right_stick(GOUD_INVALID_CONTEXT_ID, 0, &mut x, std::ptr::null_mut())
    };
    assert!(!result);
}

#[test]
fn right_stick_both_null_returns_false() {
    // SAFETY: Passing null for both pointers to test the null guard.
    let result = unsafe {
        goud_input_gamepad_right_stick(
            GOUD_INVALID_CONTEXT_ID,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(!result);
}
