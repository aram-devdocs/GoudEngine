use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

use super::{super::get_physics_registry_3d, with_provider_mut};

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
