//! # Context Lifecycle FFI Functions
//!
//! This module provides FFI entry points for context creation, destruction,
//! and validity checking.

use crate::context_registry::GoudContextId;
use crate::core::debugger::ContextConfig;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::GOUD_INVALID_CONTEXT_ID;
use crate::sdk::context::Context;
use std::ffi::{c_char, CStr};

/// FFI-safe debugger configuration for context creation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GoudDebuggerConfig {
    /// Enables the debugger runtime for this context.
    pub enabled: bool,
    /// Publishes local attach metadata for this route.
    pub publish_local_attach: bool,
    /// Optional null-terminated UTF-8 route label.
    pub route_label: *const c_char,
}

impl Default for GoudDebuggerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            publish_local_attach: false,
            route_label: std::ptr::null(),
        }
    }
}

/// FFI-safe context configuration.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GoudContextConfig {
    /// Debugger runtime settings.
    pub debugger: GoudDebuggerConfig,
}

unsafe fn ffi_context_config_to_sdk(
    config: &GoudContextConfig,
) -> Result<ContextConfig, GoudError> {
    let route_label = if config.debugger.route_label.is_null() {
        None
    } else {
        // SAFETY: The caller guarantees `route_label` is either null or points to a valid,
        // null-terminated UTF-8 string for the duration of this call.
        Some(
            unsafe { CStr::from_ptr(config.debugger.route_label) }
                .to_str()
                .map_err(|err| {
                    GoudError::InternalError(format!("Invalid UTF-8 route label: {err}"))
                })?
                .to_string(),
        )
    };

    Ok(ContextConfig {
        debugger: crate::core::debugger::DebuggerConfig {
            enabled: config.debugger.enabled,
            publish_local_attach: config.debugger.publish_local_attach,
            route_label,
        },
    })
}

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
    Context::create()
}

/// Creates a new engine context with the provided configuration.
///
/// # Safety
///
/// `config` must be a valid, non-null pointer to a [`GoudContextConfig`].
#[no_mangle]
pub unsafe extern "C" fn goud_context_create_with_config(
    config: *const GoudContextConfig,
) -> GoudContextId {
    if config.is_null() {
        set_last_error(GoudError::InternalError(
            "config pointer is null".to_string(),
        ));
        return GOUD_INVALID_CONTEXT_ID;
    }

    // SAFETY: `config` is validated non-null above and only read for the duration of this call.
    let config = match unsafe { ffi_context_config_to_sdk(&*config) } {
        Ok(config) => config,
        Err(err) => {
            set_last_error(err);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    Context::create_with_config(config)
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
    // Clean up provider registry before destroying the context.
    crate::ffi::providers::provider_registry_remove(context_id);
    Context::destroy(context_id)
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
    Context::is_valid(context_id)
}
