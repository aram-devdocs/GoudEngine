//! FFI functions for skeletal animation playback, blending, and transitions.

use super::state::with_renderer;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use std::os::raw::c_char;

/// Returns the number of animations in a model, or -1 if invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_count(
    context_id: GoudContextId,
    model_id: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .get_animation_count(model_id)
            .map(|c| c as i32)
            .unwrap_or(-1)
    })
    .unwrap_or(-1)
}

/// Copies the animation name to the output buffer.
///
/// If `out_buf` is null or `buf_len` is 0, returns the required byte count
/// (name length + 1 for null terminator). This supports the standard probe-then-fill pattern:
/// 1. First call with null/0 to get required size.
/// 2. Allocate buffer of that size.
/// 3. Second call with allocated buffer.
///
/// Returns the actual length of the name (excluding null terminator) on success,
/// or -1 if the model or animation index is invalid.
///
/// # Safety
/// `out_buf` must point to at least `buf_len` writable bytes (if not null).
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_get_animation_name(
    context_id: GoudContextId,
    model_id: u32,
    anim_index: i32,
    out_buf: *mut c_char,
    buf_len: i32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if anim_index < 0 {
        return -1;
    }

    let name = with_renderer(context_id, |renderer| {
        renderer.get_animation_name(model_id, anim_index as usize)
    })
    .flatten();

    match name {
        Some(name) => {
            let name_bytes = name.as_bytes();
            let required_size = (name_bytes.len() + 1) as i32;

            // If null probe, return required size.
            if out_buf.is_null() || buf_len <= 0 {
                return required_size;
            }

            // Copy the name into the buffer.
            let copy_len = name_bytes.len().min((buf_len as usize) - 1);
            // SAFETY: Caller guarantees out_buf points to buf_len writable bytes.
            std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), out_buf as *mut u8, copy_len);
            *out_buf.add(copy_len) = 0; // Null terminate.
            name_bytes.len() as i32
        }
        None => -1,
    }
}

/// Starts playing an animation on a model instance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_play_animation(
    context_id: GoudContextId,
    instance_id: u32,
    anim_index: i32,
    looping: bool,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if anim_index < 0 {
        return false;
    }

    with_renderer(context_id, |renderer| {
        if let Some(player) = renderer.animation_player_mut(instance_id) {
            player.play(anim_index as usize, looping);
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

/// Stops animation playback on a model instance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_stop_animation(
    context_id: GoudContextId,
    instance_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        if let Some(player) = renderer.animation_player_mut(instance_id) {
            player.stop();
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

/// Sets the playback speed for a model instance's animation.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_animation_speed(
    context_id: GoudContextId,
    instance_id: u32,
    speed: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        if let Some(player) = renderer.animation_player_mut(instance_id) {
            player.set_speed(speed);
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

/// Sets up blending between two animation clips.
#[no_mangle]
pub extern "C" fn goud_renderer3d_blend_animations(
    context_id: GoudContextId,
    instance_id: u32,
    anim_a: i32,
    anim_b: i32,
    blend_factor: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if anim_a < 0 || anim_b < 0 {
        return false;
    }

    with_renderer(context_id, |renderer| {
        if let Some(player) = renderer.animation_player_mut(instance_id) {
            player.blend(anim_a as usize, anim_b as usize, blend_factor);
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

/// Starts a timed transition from the current animation to a target clip.
#[no_mangle]
pub extern "C" fn goud_renderer3d_transition_animation(
    context_id: GoudContextId,
    instance_id: u32,
    target_anim: i32,
    duration: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if target_anim < 0 {
        return false;
    }

    with_renderer(context_id, |renderer| {
        if let Some(player) = renderer.animation_player_mut(instance_id) {
            player.transition(target_anim as usize, duration);
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

/// Returns whether an animation is currently playing on a model instance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_is_animation_playing(
    context_id: GoudContextId,
    instance_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .animation_player(instance_id)
            .is_some_and(|p| p.is_playing())
    })
    .unwrap_or(false)
}

/// Returns the playback progress (0.0 to 1.0) of the primary animation.
///
/// Returns -1.0 if the instance has no animation player.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_progress(
    context_id: GoudContextId,
    instance_id: u32,
) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1.0;
    }

    with_renderer(context_id, |renderer| {
        if let Some(player) = renderer.animation_player(instance_id) {
            if let Some(animations) = renderer.get_model_animations(instance_id) {
                player.progress(animations)
            } else {
                -1.0
            }
        } else {
            -1.0
        }
    })
    .unwrap_or(-1.0)
}

/// Advances all animation players by `delta_time` seconds.
///
/// Call this once per frame before rendering.
#[no_mangle]
pub extern "C" fn goud_renderer3d_update_animations(
    context_id: GoudContextId,
    delta_time: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.update_animations(delta_time);
        true
    })
    .unwrap_or(false)
}
