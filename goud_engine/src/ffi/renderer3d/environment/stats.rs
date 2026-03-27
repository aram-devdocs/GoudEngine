//! FFI renderer stats getter functions.

use super::super::state::with_renderer;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// Returns the number of draw calls issued during the last `render()` call.
///
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_draw_calls(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| renderer.stats().draw_calls as i32).unwrap_or(-1)
}

/// Returns the number of objects that passed frustum culling during the last
/// `render()` call.
///
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_visible_object_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer.stats().visible_objects as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of objects rejected by frustum culling during the last
/// `render()` call.
///
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_culled_object_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer.stats().culled_objects as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of instanced draw calls issued during the last
/// `render()` call.
///
/// Each instanced draw call renders multiple instances of the same mesh in a
/// single GPU submission. Resets to zero at the start of each `render()`.
/// Returns `-1` if the context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_instanced_draw_calls(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().instanced_draw_calls as i32
    })
    .unwrap_or(-1)
}

/// Returns the total number of instances submitted via instanced draw calls
/// during the last `render()` call.
///
/// This is the sum of per-draw instance counts, not the number of draw calls.
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_active_instance_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().active_instances as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of animation players that were fully evaluated (bone
/// matrix computation) during the last `update_animations()` call.
///
/// Resets to zero at the start of each `update_animations()`. Returns `-1`
/// if the context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_evaluation_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().animation_evaluations as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of animation evaluations that were skipped during the
/// last `update_animations()` call thanks to shared evaluation cache hits,
/// baked animation lookups, and animation LOD distance culling.
///
/// Resets to zero at the start of each `update_animations()`. Returns `-1`
/// if the context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_evaluation_saved_count(
    context_id: GoudContextId,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().animation_evaluations_saved as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of bone matrix buffer uploads performed during the last
/// `render()` call (GPU skinning path only).
///
/// Each upload transfers one model's bone matrices to the GPU storage buffer.
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_bone_matrix_upload_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().bone_matrix_uploads as i32
    })
    .unwrap_or(-1)
}
