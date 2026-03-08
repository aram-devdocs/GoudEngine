//! # Playback Query Functions for SpriteAnimator
//!
//! Read-only FFI functions for querying the state of an `FfiSpriteAnimator`.
//! These operate on pointers to stack- or heap-allocated `FfiSpriteAnimator`
//! structs and return safe default values when given null pointers.

use crate::core::error::{set_last_error, GoudError};
use crate::core::types::FfiSpriteAnimator;

/// Returns the current frame index of the animator.
///
/// # Parameters
///
/// - `animator`: Pointer to the animator (caller-owned, read-only)
///
/// # Returns
///
/// The current frame index, or 0 if `animator` is null.
///
/// # Safety
///
/// - `animator` must be a valid pointer to an `FfiSpriteAnimator` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_animator_get_current_frame(
    animator: *const FfiSpriteAnimator,
) -> u32 {
    if animator.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return 0;
    }
    // SAFETY: animator is non-null and points to a valid FfiSpriteAnimator
    (*animator).current_frame
}

/// Returns whether the animator is currently playing.
///
/// # Parameters
///
/// - `animator`: Pointer to the animator (caller-owned, read-only)
///
/// # Returns
///
/// `true` if playing, `false` if paused/stopped or if `animator` is null.
///
/// # Safety
///
/// - `animator` must be a valid pointer to an `FfiSpriteAnimator` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_animator_is_playing(
    animator: *const FfiSpriteAnimator,
) -> bool {
    if animator.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: animator is non-null and points to a valid FfiSpriteAnimator
    (*animator).playing
}

/// Returns whether the animator has finished (OneShot completed).
///
/// # Parameters
///
/// - `animator`: Pointer to the animator (caller-owned, read-only)
///
/// # Returns
///
/// `true` if the animation is finished, `false` otherwise or if `animator` is null.
///
/// # Safety
///
/// - `animator` must be a valid pointer to an `FfiSpriteAnimator` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_animator_is_finished(
    animator: *const FfiSpriteAnimator,
) -> bool {
    if animator.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: animator is non-null and points to a valid FfiSpriteAnimator
    (*animator).finished
}
