//! FFI exports for the animation system.
//!
//! Provides C-compatible functions for:
//! - **Animation Controller**: State machine component on entities
//! - **Tween**: Standalone value interpolation with easing
//! - **Skeleton2D**: Bone hierarchy and skeletal animation playback
//!
//! ## Module Layout
//!
//! - `controller` -- AnimationController component operations
//! - `tween` -- Standalone tween interpolation with easing
//! - `skeletal` -- Skeleton2D and SkeletalAnimator operations

pub mod controller;
pub mod skeletal;
pub mod tween;

use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};

/// Converts a raw `*const u8` + length into a `&str`, returning an error code on failure.
///
/// # Safety
///
/// Caller must ensure `ptr` points to valid memory of at least `len` bytes.
pub(super) unsafe fn str_from_raw(ptr: *const u8, len: i32) -> Result<&'static str, i32> {
    if ptr.is_null() {
        set_last_error(GoudError::InvalidState("string pointer is null".into()));
        return Err(-ERR_INVALID_STATE);
    }
    if len < 0 {
        set_last_error(GoudError::InvalidState("negative string length".into()));
        return Err(-ERR_INVALID_STATE);
    }
    // SAFETY: Caller guarantees ptr is valid for len bytes.
    let bytes = std::slice::from_raw_parts(ptr, len as usize);
    std::str::from_utf8(bytes).map_err(|_| {
        set_last_error(GoudError::InvalidState("string is not valid UTF-8".into()));
        -ERR_INVALID_STATE
    })
}

// Re-export all FFI functions for flat namespace access.
pub use controller::{
    goud_animation_controller_add_state, goud_animation_controller_add_transition,
    goud_animation_controller_create, goud_animation_controller_get_state,
    goud_animation_controller_set_state, goud_animation_controller_update,
};
pub use skeletal::{
    goud_skeleton_add_bone, goud_skeleton_create, goud_skeleton_play_clip,
    goud_skeleton_set_bone_transform,
};
pub use tween::{
    goud_tween_create, goud_tween_is_complete, goud_tween_reset, goud_tween_update,
    goud_tween_value,
};
