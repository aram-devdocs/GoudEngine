//! # Context Lifecycle FFI Functions
//!
//! This module provides FFI entry points for context creation, destruction,
//! and validity checking.

use crate::context_registry::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::core::error::{set_last_error, GoudError};

/// Creates a new engine context.
///
/// Returns a unique context ID that must be passed to all subsequent FFI calls.
/// On failure, returns `GOUD_INVALID_CONTEXT_ID` and sets the last error.
///
/// # Thread Safety
///
/// This function is thread-safe and can be called from any thread.
/// However, the returned context must be used from a single thread.
///
/// # Error Codes
///
/// - `INTERNAL_ERROR_BASE + 0` (InternalError) - Failed to lock registry or
///   create context
#[no_mangle]
pub extern "C" fn goud_context_create() -> GoudContextId {
    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    match registry.create() {
        Ok(id) => id,
        Err(err) => {
            set_last_error(err);
            GOUD_INVALID_CONTEXT_ID
        }
    }
}

/// Destroys an engine context, freeing all associated resources.
///
/// After calling this, the context ID becomes invalid and should not be used.
/// All entities, components, and resources in the context are destroyed.
///
/// # Arguments
///
/// * `context_id` - The context to destroy
///
/// # Returns
///
/// `true` if the context was successfully destroyed, `false` on error.
///
/// # Thread Safety
///
/// This function is thread-safe and can be called from any thread.
#[no_mangle]
pub extern "C" fn goud_context_destroy(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return false;
        }
    };

    match registry.destroy(context_id) {
        Ok(()) => true,
        Err(err) => {
            set_last_error(err);
            false
        }
    }
}

/// Checks if a context ID is valid.
///
/// A context is valid if it was created and has not been destroyed.
///
/// # Arguments
///
/// * `context_id` - The context ID to check
///
/// # Returns
///
/// `true` if the context is valid, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_context_is_valid(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return false;
        }
    };

    registry.is_valid(context_id)
}
