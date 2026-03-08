//! FFI functions for the 3D camera.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

// ============================================================================
// FFI: Camera
// ============================================================================

/// Sets the 3D camera position.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_camera_position(
    context_id: GoudContextId,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if ensure_renderer3d_state(context_id).is_err() {
        set_last_error(GoudError::InternalError(
            "Renderer state not found".to_string(),
        ));
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_camera_position(x, y, z);
        true
    })
    .unwrap_or(false)
}

/// Sets the 3D camera rotation (pitch, yaw, roll in degrees).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_camera_rotation(
    context_id: GoudContextId,
    pitch: f32,
    yaw: f32,
    roll: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if ensure_renderer3d_state(context_id).is_err() {
        set_last_error(GoudError::InternalError(
            "Renderer state not found".to_string(),
        ));
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_camera_rotation(pitch, yaw, roll);
        true
    })
    .unwrap_or(false)
}
