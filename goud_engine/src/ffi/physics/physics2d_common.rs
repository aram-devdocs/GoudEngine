//! Shared helpers for 2D physics FFI.
//!
//! Keeps lock/error boilerplate out of `physics2d.rs` so exported API files
//! remain focused and below size limits.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::get_physics_registry_2d;

/// Error sentinel for handle-returning functions.
pub(super) const INVALID_HANDLE: i64 = -1;

/// Acquires a read lock on the 2D physics registry and calls `f` with the
/// provider for the given context. Returns a negative error code if the context
/// has no provider.
pub(super) fn with_provider<F, R>(ctx: GoudContextId, f: F) -> R
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
pub(super) fn with_provider_mut<F, R>(ctx: GoudContextId, f: F) -> R
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
