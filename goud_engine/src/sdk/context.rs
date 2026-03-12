//! # SDK Context Lifecycle API
//!
//! Provides static functions for engine context creation, destruction,
//! and validation. These are annotated with `#[goud_api]` to auto-generate
//! FFI wrappers that replace hand-written functions in `ffi/context.rs`.
//!
//! Context functions are static (no `self` receiver) because the context
//! must be created before a GoudGame instance exists.

use crate::context_registry::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
pub use crate::core::debugger::ContextConfig;
use crate::core::debugger::RuntimeSurfaceKind;
use crate::core::error::{set_last_error, GoudError};

/// Zero-sized type that hosts context lifecycle functions.
///
/// All methods are static (no `self` receiver) and generate FFI wrappers
/// via the `#[goud_api]` proc-macro.
pub struct Context;

// NOTE: FFI wrappers are hand-written in ffi/context.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl Context {
    /// Creates a new engine context.
    ///
    /// Returns a unique context ID on success, or `GOUD_INVALID_CONTEXT_ID`
    /// on failure.
    pub fn create() -> GoudContextId {
        Self::create_with_config(ContextConfig::default())
    }

    /// Creates a new engine context with the provided configuration.
    pub fn create_with_config(config: ContextConfig) -> GoudContextId {
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        match registry.create_with_config(config, RuntimeSurfaceKind::HeadlessContext) {
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
    use crate::core::debugger::{
        active_route_count, current_manifest, reset_for_tests, test_lock, DebuggerConfig,
    };

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

    #[test]
    fn test_debugger_context_create_with_config_registers_and_cleans_up_route() {
        let _guard = test_lock();
        reset_for_tests();

        let id = Context::create_with_config(ContextConfig {
            debugger: DebuggerConfig {
                enabled: true,
                publish_local_attach: true,
                route_label: Some("headless".to_string()),
            },
        });

        assert!(!id.is_invalid());
        assert_eq!(active_route_count(), 1);
        let manifest = current_manifest().expect("manifest should be published");
        assert_eq!(manifest.routes.len(), 1);
        assert_eq!(manifest.routes[0].label.as_deref(), Some("headless"));

        assert!(Context::destroy(id));
        assert_eq!(active_route_count(), 0);
        assert!(current_manifest().is_none());
    }

    #[test]
    fn test_debugger_context_create_keeps_disabled_shorthand_path() {
        let _guard = test_lock();
        reset_for_tests();

        let id = Context::create();
        assert!(!id.is_invalid());
        assert_eq!(active_route_count(), 0);
        assert!(current_manifest().is_none());
        assert!(Context::destroy(id));
    }

    #[test]
    fn test_debugger_context_create_with_config_creates_distinct_routes() {
        let _guard = test_lock();
        reset_for_tests();

        let config = ContextConfig {
            debugger: DebuggerConfig {
                enabled: true,
                publish_local_attach: true,
                route_label: None,
            },
        };
        let id_a = Context::create_with_config(config.clone());
        let id_b = Context::create_with_config(config);

        let manifest = current_manifest().expect("manifest should exist");
        assert_eq!(manifest.routes.len(), 2);
        assert_ne!(
            manifest.routes[0].route_id.context_id,
            manifest.routes[1].route_id.context_id
        );

        assert!(Context::destroy(id_a));
        assert_eq!(active_route_count(), 1);
        assert!(Context::destroy(id_b));
        assert_eq!(active_route_count(), 0);
    }
}
