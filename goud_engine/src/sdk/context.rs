//! # SDK Context Lifecycle API
//!
//! Provides static functions for engine context creation, destruction,
//! and validation. These are annotated with `#[goud_api]` to auto-generate
//! FFI wrappers that replace hand-written functions in `ffi/context.rs`.
//!
//! Context functions are static (no `self` receiver) because the context
//! must be created before a GoudGame instance exists.

use crate::core::error::{set_last_error, GoudError};
use crate::core::context_registry::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// Zero-sized type that hosts context lifecycle functions.
///
/// All methods are static (no `self` receiver) and generate FFI wrappers
/// via the `#[goud_api]` proc-macro.
pub struct Context;

#[goud_engine_macros::goud_api(module = "context")]
impl Context {
    /// Creates a new engine context.
    ///
    /// Returns a unique context ID on success, or `GOUD_INVALID_CONTEXT_ID`
    /// on failure.
    pub fn create() -> GoudContextId {
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

    /// Destroys an engine context and frees all associated resources.
    ///
    /// Returns `true` on success, `false` on error.
    pub fn destroy(context_id: GoudContextId) -> bool {
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

    /// Checks if a context ID is valid (created and not yet destroyed).
    pub fn is_valid(context_id: GoudContextId) -> bool {
        if context_id == GOUD_INVALID_CONTEXT_ID {
            return false;
        }

        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => return false,
        };

        registry.is_valid(context_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_create_destroy() {
        let id = Context::create();
        assert!(!id.is_invalid());
        assert!(Context::is_valid(id));

        assert!(Context::destroy(id));
        assert!(!Context::is_valid(id));
    }

    #[test]
    fn test_context_destroy_invalid() {
        assert!(!Context::destroy(GOUD_INVALID_CONTEXT_ID));
    }

    #[test]
    fn test_context_is_valid_invalid() {
        assert!(!Context::is_valid(GOUD_INVALID_CONTEXT_ID));
    }
}
