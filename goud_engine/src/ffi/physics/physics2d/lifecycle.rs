use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::providers::types::PhysicsBackend2D;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;
use crate::libs::providers::impls::simple_physics::SimplePhysicsProvider;

use super::super::get_physics_registry_2d;
use super::super::physics2d_state::clear_context;

const PHYSICS_BACKEND_DEFAULT: u32 = PhysicsBackend2D::Default as u32;

/// Creates a 2D physics provider for the given context with initial gravity.
///
/// Uses `PhysicsBackend2D::Default` backend selection.
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
    goud_physics_create_with_backend(ctx, gx, gy, PHYSICS_BACKEND_DEFAULT)
}

/// Creates a 2D physics provider for the given context with explicit backend.
///
/// Backend values:
/// - `0`: Default (same as `goud_physics_create` delegation)
/// - `1`: Rapier2D
/// - `2`: SimplePhysicsProvider
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
pub extern "C" fn goud_physics_create_with_backend(
    ctx: GoudContextId,
    gx: f32,
    gy: f32,
    backend: u32,
) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let backend_value = match PhysicsBackend2D::from_u32(backend) {
        Some(backend) => backend,
        None => {
            set_last_error(GoudError::InvalidState(
                "invalid physics backend".to_string(),
            ));
            return GoudError::InvalidState(String::new()).error_code();
        }
    };
    let provider: Box<dyn crate::core::providers::physics::PhysicsProvider> = match backend_value {
        PhysicsBackend2D::Default | PhysicsBackend2D::Rapier => {
            Box::new(Rapier2DPhysicsProvider::new([gx, gy]))
        }
        PhysicsBackend2D::Simple => Box::new(SimplePhysicsProvider::new([gx, gy])),
    };

    let mut registry = match get_physics_registry_2d().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock physics registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };
    registry.providers.insert(ctx, provider);
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
    clear_context(ctx);
    0
}

/// Sets the gravity vector for the 2D physics provider.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_set_gravity(ctx: GoudContextId, x: f32, y: f32) -> i32 {
    super::super::physics2d_common::with_provider_mut(ctx, |p| {
        p.set_gravity([x, y]);
        0
    })
}
