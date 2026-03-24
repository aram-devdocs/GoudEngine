//! FFI exports for the 3D character controller.
//!
//! Provides C-compatible functions for creating, moving, querying, and
//! destroying capsule-based 3D character controllers backed by Rapier3D.

use crate::core::error::set_last_error;
use crate::core::providers::types::{CharacterControllerDesc3D, CharacterControllerHandle};
use crate::ffi::context::GoudContextId;

use super::{with_provider, with_provider_mut, INVALID_HANDLE};

/// Creates a capsule-based 3D character controller.
///
/// The character controller uses a kinematic rigid body and capsule collider
/// for collision-aware movement with slope limits, step climbing, and ground
/// detection.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `radius` - Capsule radius
/// * `half_height` - Capsule half-height (total height = half_height * 2 + radius * 2)
/// * `x`, `y`, `z` - Initial position
/// * `max_slope_angle` - Maximum slope angle in radians (e.g., 0.785 for 45 degrees)
/// * `step_height` - Maximum step height for stair climbing
///
/// # Returns
///
/// A positive controller handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_create_character_controller(
    ctx: GoudContextId,
    radius: f32,
    half_height: f32,
    x: f32,
    y: f32,
    z: f32,
    max_slope_angle: f32,
    step_height: f32,
) -> i64 {
    use crate::ffi::context::GOUD_INVALID_CONTEXT_ID;

    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(crate::core::error::GoudError::InvalidContext);
        return INVALID_HANDLE;
    }

    with_provider_mut(ctx, |p| {
        let desc = CharacterControllerDesc3D {
            radius,
            half_height,
            position: [x, y, z],
            max_slope_angle,
            step_height,
        };
        match p.create_character_controller(&desc) {
            Ok(handle) => handle.0 as i64,
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

/// Moves a 3D character controller by the given displacement.
///
/// The controller applies gravity, slope limits, and step climbing
/// automatically. Ground detection is updated after the move.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `controller_id` - Handle returned by `goud_physics3d_create_character_controller`
/// * `dx`, `dy`, `dz` - Desired displacement (not velocity)
/// * `dt` - Delta time in seconds
/// * `out_x`, `out_y`, `out_z` - Output pointers for the resulting position
/// * `out_grounded` - Output pointer for whether the character is on the ground
///
/// # Safety
///
/// All output pointers must be valid, non-null, and writable.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics3d_move_character(
    ctx: GoudContextId,
    controller_id: u64,
    dx: f32,
    dy: f32,
    dz: f32,
    dt: f32,
    out_x: *mut f32,
    out_y: *mut f32,
    out_z: *mut f32,
    out_grounded: *mut bool,
) -> i32 {
    if out_x.is_null() || out_y.is_null() || out_z.is_null() || out_grounded.is_null() {
        set_last_error(crate::core::error::GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return crate::core::error::GoudError::InvalidState(String::new()).error_code();
    }

    with_provider_mut(ctx, |p| {
        let handle = CharacterControllerHandle(controller_id);
        match p.move_character(handle, [dx, dy, dz], dt) {
            Ok(result) => {
                // SAFETY: Caller guarantees pointers are valid and writable.
                *out_x = result.position[0];
                *out_y = result.position[1];
                *out_z = result.position[2];
                *out_grounded = result.grounded;
                0
            }
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Gets the current position of a 3D character controller.
///
/// # Safety
///
/// `out_x`, `out_y`, and `out_z` must be valid, non-null pointers to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics3d_get_character_position(
    ctx: GoudContextId,
    controller_id: u64,
    out_x: *mut f32,
    out_y: *mut f32,
    out_z: *mut f32,
) -> i32 {
    if out_x.is_null() || out_y.is_null() || out_z.is_null() {
        set_last_error(crate::core::error::GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return crate::core::error::GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| {
        let handle = CharacterControllerHandle(controller_id);
        match p.character_position(handle) {
            Ok(pos) => {
                // SAFETY: Caller guarantees pointers are valid and writable.
                *out_x = pos[0];
                *out_y = pos[1];
                *out_z = pos[2];
                0
            }
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Checks if a 3D character controller is touching the ground.
///
/// # Returns
///
/// `true` if grounded, `false` otherwise. Returns `false` on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_is_character_grounded(
    ctx: GoudContextId,
    controller_id: u64,
) -> bool {
    let result: i32 = with_provider(ctx, |p| {
        let handle = CharacterControllerHandle(controller_id);
        match p.is_character_grounded(handle) {
            Ok(grounded) => {
                if grounded {
                    1
                } else {
                    0
                }
            }
            Err(e) => {
                set_last_error(e);
                0
            }
        }
    });
    result == 1
}

/// Destroys a 3D character controller and its associated physics objects.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_destroy_character_controller(
    ctx: GoudContextId,
    controller_id: u64,
) -> i32 {
    use crate::ffi::context::GOUD_INVALID_CONTEXT_ID;

    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(crate::core::error::GoudError::InvalidContext);
        return crate::core::error::GoudError::InvalidContext.error_code();
    }

    with_provider_mut(ctx, |p| {
        let handle = CharacterControllerHandle(controller_id);
        p.destroy_character_controller(handle);
        0
    })
}
