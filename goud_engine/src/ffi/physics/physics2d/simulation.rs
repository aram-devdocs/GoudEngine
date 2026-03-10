use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::types::BodyHandle;
use crate::ffi::context::GoudContextId;

use super::super::physics2d_common::with_provider;
use super::super::physics2d_common::with_provider_mut;
use super::super::physics2d_events::step_and_dispatch_collision_events;

/// Steps the 2D physics simulation by `dt` seconds.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_step(ctx: GoudContextId, dt: f32) -> i32 {
    step_and_dispatch_collision_events(ctx, dt)
}

/// Writes the position of a body into the provided output pointers.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid, non-null pointers to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_position(
    ctx: GoudContextId,
    handle: u64,
    out_x: *mut f32,
    out_y: *mut f32,
) -> i32 {
    if out_x.is_null() || out_y.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_x or out_y is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| match p.body_position(BodyHandle(handle)) {
        Ok(pos) => {
            // SAFETY: Caller guarantees out_x and out_y are valid writable pointers.
            *out_x = pos[0];
            *out_y = pos[1];
            0
        }
        Err(e) => {
            let code = e.error_code();
            set_last_error(e);
            code
        }
    })
}

/// Writes the velocity of a body into the provided output pointers.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid, non-null pointers to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_velocity(
    ctx: GoudContextId,
    handle: u64,
    out_x: *mut f32,
    out_y: *mut f32,
) -> i32 {
    if out_x.is_null() || out_y.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_x or out_y is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| match p.body_velocity(BodyHandle(handle)) {
        Ok(vel) => {
            // SAFETY: Caller guarantees out_x and out_y are valid writable pointers.
            *out_x = vel[0];
            *out_y = vel[1];
            0
        }
        Err(e) => {
            let code = e.error_code();
            set_last_error(e);
            code
        }
    })
}

/// Sets the linear velocity of a body.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_velocity(
    ctx: GoudContextId,
    handle: u64,
    vx: f32,
    vy: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.set_body_velocity(BodyHandle(handle), [vx, vy]) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Applies a force to a body (accumulated over the frame).
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_apply_force(
    ctx: GoudContextId,
    handle: u64,
    fx: f32,
    fy: f32,
) -> i32 {
    with_provider_mut(ctx, |p| match p.apply_force(BodyHandle(handle), [fx, fy]) {
        Ok(()) => 0,
        Err(e) => {
            let code = e.error_code();
            set_last_error(e);
            code
        }
    })
}

/// Applies an instantaneous impulse to a body.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_apply_impulse(
    ctx: GoudContextId,
    handle: u64,
    ix: f32,
    iy: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.apply_impulse(BodyHandle(handle), [ix, iy]) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Casts a ray and writes the hit point into the provided output pointers.
///
/// # Arguments
///
/// * `ox`, `oy` - Ray origin
/// * `dx`, `dy` - Ray direction (will be used as-is, should be normalized)
/// * `max_dist` - Maximum ray length
/// * `out_hit_x`, `out_hit_y` - Output hit point (written only on hit)
///
/// # Safety
///
/// `out_hit_x` and `out_hit_y` must be valid, non-null pointers to writable
/// `f32`, or both null if the hit point is not needed.
///
/// # Returns
///
/// 1 if a hit occurred, 0 if no hit, negative on error.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_raycast(
    ctx: GoudContextId,
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
    max_dist: f32,
    out_hit_x: *mut f32,
    out_hit_y: *mut f32,
) -> i32 {
    with_provider(ctx, |p| match p.raycast([ox, oy], [dx, dy], max_dist) {
        Some(hit) => {
            if !out_hit_x.is_null() {
                // SAFETY: Caller guarantees out_hit_x is valid if non-null.
                *out_hit_x = hit.point[0];
            }
            if !out_hit_y.is_null() {
                // SAFETY: Caller guarantees out_hit_y is valid if non-null.
                *out_hit_y = hit.point[1];
            }
            1
        }
        None => 0,
    })
}
