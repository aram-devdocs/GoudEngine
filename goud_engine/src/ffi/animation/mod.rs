//! FFI exports for the animation system.
//!
//! Provides C-compatible functions for:
//! - **Animation Controller**: State machine component on entities
//! - **Tween**: Standalone value interpolation with easing
//! - **Skeleton2D**: Bone hierarchy and skeletal animation playback
//! - **Animation Events**: Keyframe event configuration and reading
//! - **Animation Layers**: Multi-layer blended animation stacks
//!
//! ## Module Layout
//!
//! - `controller` -- AnimationController component operations
//! - `control` -- High-level animation playback/state/parameter controls
//! - `tween` -- Standalone tween interpolation with easing
//! - `skeletal` -- Skeleton2D and SkeletalAnimator operations
//! - `events` -- Animation event add/read operations
//! - `layer` -- AnimationLayerStack component operations

pub(crate) mod control;
pub(crate) mod controller;
pub(crate) mod events;
pub(crate) mod layer;
pub(crate) mod skeletal;
pub(crate) mod tween;

use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};

/// Converts a raw `*const u8` + length into a `&str`, returning an error code on failure.
///
/// # Safety
///
/// Caller must ensure `ptr` points to valid memory of at least `len` bytes.
pub(super) unsafe fn str_from_raw<'a>(ptr: *const u8, len: i32) -> Result<&'a str, i32> {
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
pub use control::{
    goud_animation_play, goud_animation_set_parameter_bool, goud_animation_set_parameter_float,
    goud_animation_set_state, goud_animation_stop,
};
pub use controller::{
    goud_animation_controller_add_state, goud_animation_controller_add_transition,
    goud_animation_controller_create, goud_animation_controller_get_state,
    goud_animation_controller_set_state, goud_animation_controller_update,
};
pub use events::{
    goud_animation_clip_add_event, goud_animation_events_count, goud_animation_events_read,
};
pub use layer::{
    goud_animation_layer_add, goud_animation_layer_add_frame, goud_animation_layer_play,
    goud_animation_layer_reset, goud_animation_layer_set_clip, goud_animation_layer_set_weight,
    goud_animation_layer_stack_create,
};
pub use skeletal::{
    goud_skeleton_add_bone, goud_skeleton_create, goud_skeleton_play_clip,
    goud_skeleton_set_bone_transform,
};
pub use tween::{
    goud_tween_create, goud_tween_destroy, goud_tween_is_complete, goud_tween_reset,
    goud_tween_update, goud_tween_value,
};
