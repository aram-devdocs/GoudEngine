//! 3D physics FFI exports.
//!
//! Provides C-compatible functions for Rapier3D physics: body creation,
//! collider attachment, forces, impulses, simulation stepping, and raycasting.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::BodyHandle;
use crate::core::providers::types3d::{BodyDesc3D, ColliderDesc3D};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

use super::get_physics_registry_3d;

/// Error sentinel for handle-returning functions.
const INVALID_HANDLE: i64 = -1;

// =============================================================================
// Provider Lifecycle
// =============================================================================

/// Creates a 3D physics provider for the given context with initial gravity.
///
/// Must be called before any other `goud_physics3d_*` function for this context.
///
/// **Cleanup:** The caller MUST call `goud_physics3d_destroy` before destroying
/// the context. Physics providers are stored in a global registry separate
/// from `GoudContext` and are NOT automatically cleaned up on context destroy.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_create(ctx: GoudContextId, gx: f32, gy: f32, gz: f32) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let mut provider = Rapier3DPhysicsProvider::new();
    provider.set_gravity([gx, gy, gz]);

    let mut registry = match get_physics_registry_3d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock 3D physics registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };
    registry.providers.insert(ctx, Box::new(provider));
    0
}

/// Destroys the 3D physics provider for the given context.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_destroy(ctx: GoudContextId) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let mut registry = match get_physics_registry_3d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock 3D physics registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };
    registry.providers.remove(&ctx);
    0
}

// =============================================================================
// Gravity
// =============================================================================

/// Sets the gravity vector for the 3D physics provider.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_set_gravity(ctx: GoudContextId, x: f32, y: f32, z: f32) -> i32 {
    with_provider_mut(ctx, |p| {
        p.set_gravity([x, y, z]);
        0
    })
}

// =============================================================================
// Rigid Body Management
// =============================================================================

/// Creates a rigid body in the 3D physics world.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_type` - 0 = static, 1 = dynamic, 2 = kinematic
/// * `x`, `y`, `z` - Initial position
/// * `gravity_scale` - Per-body gravity multiplier (1.0 = normal)
///
/// # Returns
///
/// A positive body handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_add_rigid_body(
    ctx: GoudContextId,
    body_type: u32,
    x: f32,
    y: f32,
    z: f32,
    gravity_scale: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = BodyDesc3D {
            position: [x, y, z],
            body_type,
            gravity_scale,
            ..BodyDesc3D::default()
        };
        match p.create_body(&desc) {
            Ok(handle) => handle.0 as i64,
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

/// Removes a rigid body from the 3D physics world.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_remove_body(ctx: GoudContextId, handle: u64) -> i32 {
    with_provider_mut(ctx, |p| {
        p.destroy_body(BodyHandle(handle));
        0
    })
}

// =============================================================================
// Collider Management
// =============================================================================

/// Attaches a collider to a rigid body in 3D.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_handle` - Handle of the body to attach to
/// * `shape_type` - 0 = sphere, 1 = box, 2 = capsule
/// * `hx`, `hy`, `hz` - Half-extents for box shapes
/// * `radius` - Radius for sphere/capsule shapes
/// * `friction` - Friction coefficient (e.g. 0.5)
/// * `restitution` - Bounciness coefficient (e.g. 0.0)
///
/// # Returns
///
/// A positive collider handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_add_collider(
    ctx: GoudContextId,
    body_handle: u64,
    shape_type: u32,
    hx: f32,
    hy: f32,
    hz: f32,
    radius: f32,
    friction: f32,
    restitution: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = ColliderDesc3D {
            shape: shape_type,
            half_extents: [hx, hy, hz],
            radius,
            friction,
            restitution,
            is_sensor: false,
            ..ColliderDesc3D::default()
        };
        match p.create_collider(BodyHandle(body_handle), &desc) {
            Ok(handle) => handle.0 as i64,
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

// =============================================================================
// Simulation
// =============================================================================

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

// =============================================================================
// Position and Velocity
// =============================================================================

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

// =============================================================================
// Forces and Impulses
// =============================================================================

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

// =============================================================================
// Internal Helpers
// =============================================================================

/// Acquires a lock on the 3D physics registry and calls `f` with the
/// provider for the given context.
pub(super) fn with_provider<F, R>(ctx: GoudContextId, f: F) -> R
where
    F: FnOnce(&dyn crate::core::providers::physics3d::PhysicsProvider3D) -> R,
    R: From<i32>,
{
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return R::from(GoudError::InvalidContext.error_code());
    }

    let registry = match get_physics_registry_3d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock 3D physics registry".to_string(),
            ));
            return R::from(ERR_INTERNAL_ERROR);
        }
    };

    match registry.providers.get(&ctx) {
        Some(provider) => f(provider.as_ref()),
        None => {
            set_last_error(GoudError::NotInitialized);
            R::from(GoudError::NotInitialized.error_code())
        }
    }
}

/// Acquires a lock on the 3D physics registry and calls `f` with
/// a mutable reference to the provider for the given context.
pub(super) fn with_provider_mut<F, R>(ctx: GoudContextId, f: F) -> R
where
    F: FnOnce(&mut dyn crate::core::providers::physics3d::PhysicsProvider3D) -> R,
    R: From<i32>,
{
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return R::from(GoudError::InvalidContext.error_code());
    }

    let mut registry = match get_physics_registry_3d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock 3D physics registry".to_string(),
            ));
            return R::from(ERR_INTERNAL_ERROR);
        }
    };

    match registry.providers.get_mut(&ctx) {
        Some(provider) => f(provider.as_mut()),
        None => {
            set_last_error(GoudError::NotInitialized);
            R::from(GoudError::NotInitialized.error_code())
        }
    }
}
