//! 2D physics FFI exports.
//!
//! Provides C-compatible functions for Rapier2D physics: body creation,
//! collider attachment, forces, impulses, simulation stepping, and raycasting.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::providers::types::{BodyDesc, BodyHandle, ColliderDesc};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;

use super::get_physics_registry_2d;

/// Error sentinel for handle-returning functions.
const INVALID_HANDLE: i64 = -1;

// =============================================================================
// Provider Lifecycle
// =============================================================================

/// Creates a 2D physics provider for the given context with initial gravity.
///
/// Must be called before any other `goud_physics_*` function for this context.
///
/// **Cleanup:** The caller MUST call `goud_physics_destroy` before destroying
/// the context. Physics providers are stored in a global registry separate
/// from `GoudContext` and are NOT automatically cleaned up on context destroy.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_create(ctx: GoudContextId, gx: f32, gy: f32) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let provider = Rapier2DPhysicsProvider::new([gx, gy]);
    let mut registry = match get_physics_registry_2d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock physics registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };
    registry.providers.insert(ctx, Box::new(provider));
    0
}

/// Destroys the 2D physics provider for the given context.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_destroy(ctx: GoudContextId) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let mut registry = match get_physics_registry_2d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock physics registry".to_string(),
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

/// Sets the gravity vector for the 2D physics provider.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_gravity(ctx: GoudContextId, x: f32, y: f32) -> i32 {
    with_provider_mut(ctx, |p| {
        p.set_gravity([x, y]);
        0
    })
}

// =============================================================================
// Rigid Body Management
// =============================================================================

/// Creates a rigid body in the 2D physics world.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_type` - 0 = static, 1 = dynamic, 2 = kinematic
/// * `x`, `y` - Initial position
///
/// # Returns
///
/// A positive body handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_add_rigid_body(
    ctx: GoudContextId,
    body_type: u32,
    x: f32,
    y: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = BodyDesc {
            position: [x, y],
            body_type,
            gravity_scale: 1.0,
            ..BodyDesc::default()
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

/// Removes a rigid body from the 2D physics world.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_remove_body(ctx: GoudContextId, handle: u64) -> i32 {
    with_provider_mut(ctx, |p| {
        p.destroy_body(BodyHandle(handle));
        0
    })
}

// =============================================================================
// Collider Management
// =============================================================================

/// Attaches a collider to a rigid body.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_handle` - Handle of the body to attach to
/// * `shape_type` - 0 = circle, 1 = box, 2 = capsule
/// * `width`, `height` - Half-extents for box shapes
/// * `radius` - Radius for circle/capsule shapes
///
/// # Returns
///
/// A positive collider handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_add_collider(
    ctx: GoudContextId,
    body_handle: u64,
    shape_type: u32,
    width: f32,
    height: f32,
    radius: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = ColliderDesc {
            shape: shape_type,
            half_extents: [width, height],
            radius,
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
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

/// Steps the 2D physics simulation by `dt` seconds.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_step(ctx: GoudContextId, dt: f32) -> i32 {
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

// =============================================================================
// Forces and Impulses
// =============================================================================

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

// =============================================================================
// Raycasting
// =============================================================================

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
    with_provider(ctx, |p| {
        match p.raycast([ox, oy], [dx, dy], max_dist) {
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
        }
    })
}

// =============================================================================
// Internal Helpers
// =============================================================================

/// Acquires a read lock on the 2D physics registry and calls `f` with the
/// provider for the given context. Returns negative error code if the context
/// has no provider.
fn with_provider<F, R>(ctx: GoudContextId, f: F) -> R
where
    F: FnOnce(&dyn crate::core::providers::physics::PhysicsProvider) -> R,
    R: From<i32>,
{
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return R::from(GoudError::InvalidContext.error_code());
    }

    let registry = match get_physics_registry_2d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock physics registry".to_string(),
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

/// Acquires a write lock on the 2D physics registry and calls `f` with
/// a mutable reference to the provider for the given context.
fn with_provider_mut<F, R>(ctx: GoudContextId, f: F) -> R
where
    F: FnOnce(&mut dyn crate::core::providers::physics::PhysicsProvider) -> R,
    R: From<i32>,
{
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return R::from(GoudError::InvalidContext.error_code());
    }

    let mut registry = match get_physics_registry_2d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock physics registry".to_string(),
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
