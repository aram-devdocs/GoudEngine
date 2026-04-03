//! FFI functions for touch input queries.

use crate::ffi::context::GoudContextId;

use super::helpers::with_input;

/// Returns the number of currently active touch points.
#[no_mangle]
pub extern "C" fn goud_input_touch_count(context_id: GoudContextId) -> u32 {
    with_input(context_id, |input| input.touch_count() as u32).unwrap_or(0)
}

/// Returns `true` if the given touch ID is currently active.
#[no_mangle]
pub extern "C" fn goud_input_touch_active(context_id: GoudContextId, touch_id: u64) -> bool {
    with_input(context_id, |input| input.touch_active(touch_id)).unwrap_or(false)
}

/// Writes the position of the given touch to the output pointers.
///
/// Returns `true` if the touch is active and the position was written.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid, aligned, non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_touch_position(
    context_id: GoudContextId,
    touch_id: u64,
    out_x: *mut f32,
    out_y: *mut f32,
) -> bool {
    if out_x.is_null() || out_y.is_null() {
        return false;
    }
    with_input(context_id, |input| {
        if let Some(pos) = input.touch_position(touch_id) {
            // SAFETY: Caller guarantees pointers are valid and aligned.
            unsafe {
                *out_x = pos.x;
                *out_y = pos.y;
            }
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

/// Returns `true` if the given touch began this frame.
#[no_mangle]
pub extern "C" fn goud_input_touch_just_pressed(context_id: GoudContextId, touch_id: u64) -> bool {
    with_input(context_id, |input| input.touch_just_pressed(touch_id)).unwrap_or(false)
}

/// Returns `true` if the given touch ended this frame.
#[no_mangle]
pub extern "C" fn goud_input_touch_just_released(context_id: GoudContextId, touch_id: u64) -> bool {
    with_input(context_id, |input| input.touch_just_released(touch_id)).unwrap_or(false)
}

/// Writes the movement delta of the given touch to the output pointers.
///
/// Returns `true` if the touch is active and the delta was written.
///
/// # Safety
///
/// `out_dx` and `out_dy` must be valid, aligned, non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_input_touch_delta(
    context_id: GoudContextId,
    touch_id: u64,
    out_dx: *mut f32,
    out_dy: *mut f32,
) -> bool {
    if out_dx.is_null() || out_dy.is_null() {
        return false;
    }
    with_input(context_id, |input| {
        if input.touch_position(touch_id).is_none() {
            return false;
        }
        let delta = input.touch_delta(touch_id);
        // SAFETY: Caller guarantees pointers are valid and aligned.
        unsafe {
            *out_dx = delta.x;
            *out_dy = delta.y;
        }
        true
    })
    .unwrap_or(false)
}
