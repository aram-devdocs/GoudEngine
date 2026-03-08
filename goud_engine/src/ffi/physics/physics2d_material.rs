//! 2D physics material and gravity FFI exports.
//!
//! Provides C-compatible functions for querying and modifying gravity scale,
//! collider friction, and collider restitution on the 2D physics provider.

use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::types::{BodyHandle, ColliderHandle};
use crate::ffi::context::GoudContextId;

use super::physics2d::{with_provider, with_provider_mut};

// =============================================================================
// Gravity
// =============================================================================

/// Gets the current gravity vector for the 2D physics world.
///
/// # Safety
///
/// `out_x` and `out_y` must be valid, non-null pointers to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_gravity(
    ctx: GoudContextId,
    out_x: *mut f32,
    out_y: *mut f32,
) -> i32 {
    if out_x.is_null() || out_y.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_x or out_y is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| {
        let g = p.gravity();
        // SAFETY: Null check above guarantees these are valid writable pointers.
        *out_x = g[0];
        *out_y = g[1];
        0
    })
}

// =============================================================================
// Body Gravity Scale
// =============================================================================

/// Sets the gravity scale for a rigid body.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_body_gravity_scale(
    ctx: GoudContextId,
    handle: u64,
    scale: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.set_body_gravity_scale(BodyHandle(handle), scale) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Gets the gravity scale for a rigid body.
///
/// # Safety
///
/// `out_scale` must be a valid, non-null pointer to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_body_gravity_scale(
    ctx: GoudContextId,
    handle: u64,
    out_scale: *mut f32,
) -> i32 {
    if out_scale.is_null() {
        set_last_error(GoudError::InvalidState("out_scale is null".to_string()));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| match p.body_gravity_scale(BodyHandle(handle)) {
        Ok(scale) => {
            // SAFETY: Null check above guarantees out_scale is valid.
            *out_scale = scale;
            0
        }
        Err(e) => {
            let code = e.error_code();
            set_last_error(e);
            code
        }
    })
}

// =============================================================================
// Collider Friction
// =============================================================================

/// Sets the friction coefficient for a collider.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_collider_friction(
    ctx: GoudContextId,
    handle: u64,
    friction: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.set_collider_friction(ColliderHandle(handle), friction) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Gets the friction coefficient for a collider.
///
/// # Safety
///
/// `out_friction` must be a valid, non-null pointer to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_collider_friction(
    ctx: GoudContextId,
    handle: u64,
    out_friction: *mut f32,
) -> i32 {
    if out_friction.is_null() {
        set_last_error(GoudError::InvalidState("out_friction is null".to_string()));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| {
        match p.collider_friction(ColliderHandle(handle)) {
            Ok(friction) => {
                // SAFETY: Null check above guarantees out_friction is valid.
                *out_friction = friction;
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

// =============================================================================
// Collider Restitution
// =============================================================================

/// Sets the restitution (bounciness) for a collider.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_collider_restitution(
    ctx: GoudContextId,
    handle: u64,
    restitution: f32,
) -> i32 {
    with_provider_mut(ctx, |p| {
        match p.set_collider_restitution(ColliderHandle(handle), restitution) {
            Ok(()) => 0,
            Err(e) => {
                let code = e.error_code();
                set_last_error(e);
                code
            }
        }
    })
}

/// Gets the restitution (bounciness) for a collider.
///
/// # Safety
///
/// `out_restitution` must be a valid, non-null pointer to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_collider_restitution(
    ctx: GoudContextId,
    handle: u64,
    out_restitution: *mut f32,
) -> i32 {
    if out_restitution.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_restitution is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| {
        match p.collider_restitution(ColliderHandle(handle)) {
            Ok(restitution) => {
                // SAFETY: Null check above guarantees out_restitution is valid.
                *out_restitution = restitution;
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

// =============================================================================
// Timestep Configuration
// =============================================================================

/// Sets the fixed timestep for the 2D physics simulation.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_timestep(ctx: GoudContextId, dt: f32) -> i32 {
    with_provider_mut(ctx, |p| {
        p.set_timestep(dt);
        0
    })
}

/// Gets the current fixed timestep for the 2D physics simulation.
///
/// # Safety
///
/// `out_dt` must be a valid, non-null pointer to writable `f32`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_get_timestep(ctx: GoudContextId, out_dt: *mut f32) -> i32 {
    if out_dt.is_null() {
        set_last_error(GoudError::InvalidState("out_dt is null".to_string()));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| {
        // SAFETY: Null check above guarantees out_dt is valid.
        unsafe {
            *out_dt = p.timestep();
        }
        0
    })
}
