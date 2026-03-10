use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::types::BodyHandle;
use crate::ffi::context::GoudContextId;

use super::{with_provider, with_provider_mut};

/// Steps the 3D physics simulation by `dt` seconds.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_step(ctx: GoudContextId, dt: f32) -> i32 {
    with_provider_mut(ctx, |p| match p.step(dt) {
        Ok(()) => 0,
        Err(e) => {
            let code = e.error_code();
            set_last_error(e);
            code
        }
    })
}

/// Writes the position of a 3D body into the provided output pointers.
///
/// # Safety
///
/// `out_x`, `out_y`, and `out_z` must be valid, non-null pointers to
/// writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics3d_get_position(
    ctx: GoudContextId,
    handle: u64,
    out_x: *mut f32,
    out_y: *mut f32,
    out_z: *mut f32,
) -> i32 {
    if out_x.is_null() || out_y.is_null() || out_z.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| match p.body_position(BodyHandle(handle)) {
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
    })
}

/// Sets the linear velocity of a 3D body.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_set_velocity(
    ctx: GoudContextId,
    handle: u64,
    vx: f32,
    vy: f32,
    vz: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.set_body_velocity(BodyHandle(handle), [vx, vy, vz]) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Applies a force to a 3D body (accumulated over the frame).
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_apply_force(
    ctx: GoudContextId,
    handle: u64,
    fx: f32,
    fy: f32,
    fz: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.apply_force(BodyHandle(handle), [fx, fy, fz]) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Applies an instantaneous impulse to a 3D body.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_apply_impulse(
    ctx: GoudContextId,
    handle: u64,
    ix: f32,
    iy: f32,
    iz: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.apply_impulse(BodyHandle(handle), [ix, iy, iz]) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}
