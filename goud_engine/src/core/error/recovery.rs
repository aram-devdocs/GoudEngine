//! Error recovery classification for GoudEngine error codes.
//!
//! Every error code is classified into one of three recovery classes:
//!
//! - **Recoverable**: The operation failed but can be retried or worked around.
//!   Examples: missing resource, expired handle, component not found.
//! - **Fatal**: The engine is in an unrecoverable state and must be re-initialized
//!   or the application must exit. Examples: invalid context, destroyed context.
//! - **Degraded**: A subsystem failed to initialize but the engine can continue
//!   with reduced functionality. Examples: audio init failed, physics init failed.
//!
//! Recovery hints provide human-readable guidance on how to handle each error.

use super::codes::*;

/// Classification of how an error can be recovered from.
///
/// Used to determine the appropriate response to an error at runtime.
/// SDKs can query this via FFI to make automated recovery decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RecoveryClass {
    /// The error is transient or can be resolved by the caller.
    Recoverable = 0,
    /// The engine is in an unrecoverable state; re-initialization or exit required.
    Fatal = 1,
    /// A subsystem is unavailable but the engine can continue with reduced functionality.
    Degraded = 2,
}

/// Returns the recovery classification for a given error code.
///
/// # Examples
///
/// ```
/// use goud_engine::core::error::{recovery_class, RecoveryClass, ERR_NOT_INITIALIZED, SUCCESS};
///
/// assert_eq!(recovery_class(ERR_NOT_INITIALIZED), RecoveryClass::Fatal);
/// assert_eq!(recovery_class(SUCCESS), RecoveryClass::Recoverable);
/// ```
pub const fn recovery_class(code: GoudErrorCode) -> RecoveryClass {
    match code {
        // Fatal: engine cannot continue
        ERR_NOT_INITIALIZED | ERR_INVALID_CONTEXT | ERR_CONTEXT_DESTROYED
        | ERR_INVALID_STATE | ERR_INTERNAL_ERROR => RecoveryClass::Fatal,

        // Degraded: subsystem unavailable, engine can continue
        ERR_BACKEND_NOT_SUPPORTED | ERR_AUDIO_INIT_FAILED | ERR_PHYSICS_INIT_FAILED => {
            RecoveryClass::Degraded
        }

        // Recoverable: everything else (including SUCCESS and unknown codes)
        _ => RecoveryClass::Recoverable,
    }
}

/// Returns a human-readable recovery hint for a given error code.
///
/// The hint describes what action the caller should take to resolve the error.
/// Returns an empty string for `SUCCESS`.
///
/// # Examples
///
/// ```
/// use goud_engine::core::error::{recovery_hint, ERR_RESOURCE_NOT_FOUND, SUCCESS};
///
/// assert_eq!(recovery_hint(SUCCESS), "");
/// assert!(!recovery_hint(ERR_RESOURCE_NOT_FOUND).is_empty());
/// ```
pub const fn recovery_hint(code: GoudErrorCode) -> &'static str {
    match code {
        SUCCESS => "",

        // Context errors
        ERR_NOT_INITIALIZED => "Call the initialization function first",
        ERR_ALREADY_INITIALIZED => "Shut down the engine before re-initializing",
        ERR_INVALID_CONTEXT => {
            "Ensure the context was properly created and not corrupted"
        }
        ERR_CONTEXT_DESTROYED => "Re-initialize the engine to obtain a new context",
        ERR_INITIALIZATION_FAILED => {
            "Check the error message for details and verify dependencies"
        }

        // Resource errors
        ERR_RESOURCE_NOT_FOUND => "Verify the file path and check the working directory",
        ERR_RESOURCE_LOAD_FAILED => {
            "Check file permissions and ensure the file is not locked"
        }
        ERR_RESOURCE_INVALID_FORMAT => {
            "Verify the file is not corrupted and uses a supported format"
        }
        ERR_RESOURCE_ALREADY_EXISTS => {
            "Use a unique identifier or remove the existing resource first"
        }

        // Handle errors
        ERR_INVALID_HANDLE => {
            "Ensure the handle was obtained from a valid creation call"
        }
        ERR_HANDLE_EXPIRED => "Re-create the resource to get a new handle",
        ERR_HANDLE_TYPE_MISMATCH => {
            "Pass the correct handle type for the operation"
        }

        // Graphics errors
        ERR_SHADER_COMPILATION_FAILED => {
            "Review shader source; the error message contains GPU compiler output"
        }
        ERR_SHADER_LINK_FAILED => {
            "Verify shader stage inputs/outputs match and uniforms are declared"
        }
        ERR_TEXTURE_CREATION_FAILED => {
            "Check texture dimensions and format; reduce size or free GPU resources"
        }
        ERR_BUFFER_CREATION_FAILED => {
            "Reduce buffer size or free unused GPU buffers"
        }
        ERR_RENDER_TARGET_FAILED => {
            "Verify attachment formats and dimensions are consistent"
        }
        ERR_BACKEND_NOT_SUPPORTED => {
            "Update GPU drivers or select a different supported backend"
        }
        ERR_DRAW_CALL_FAILED => {
            "Verify buffer bindings and shader state; try updating GPU drivers"
        }

        // Entity errors
        ERR_ENTITY_NOT_FOUND => {
            "Verify the entity ID is valid and has not been despawned"
        }
        ERR_ENTITY_ALREADY_EXISTS => {
            "Use a different entity ID or remove the existing entity first"
        }
        ERR_COMPONENT_NOT_FOUND => {
            "Attach the component before accessing it, or check with a has-component query"
        }
        ERR_COMPONENT_ALREADY_EXISTS => {
            "Use replace/update instead of add, or remove the existing component first"
        }
        ERR_QUERY_FAILED => {
            "Check for conflicting mutable/immutable access on the same component"
        }

        // Input errors
        ERR_INPUT_DEVICE_NOT_FOUND => {
            "Verify the input device is connected and recognized by the OS"
        }
        ERR_INVALID_INPUT_ACTION => {
            "Check the action name matches a registered input action"
        }

        // System errors
        ERR_WINDOW_CREATION_FAILED => {
            "Verify display server is running and window parameters are valid"
        }
        ERR_AUDIO_INIT_FAILED => {
            "Check that an audio output device is available"
        }
        ERR_PHYSICS_INIT_FAILED => {
            "Review physics configuration for invalid values"
        }
        ERR_PLATFORM_ERROR => {
            "Check the error message for platform-specific details"
        }

        // Provider errors
        ERR_PROVIDER_INIT_FAILED => {
            "Check provider configuration and dependencies"
        }
        ERR_PROVIDER_NOT_FOUND => "Register the provider before accessing it",
        ERR_PROVIDER_OPERATION_FAILED => {
            "Check the error message for operation-specific details"
        }

        // Internal errors
        ERR_INTERNAL_ERROR => {
            "Report the error with full details; this is likely an engine bug"
        }
        ERR_NOT_IMPLEMENTED => {
            "Use an alternative approach or wait for the feature to be implemented"
        }
        ERR_INVALID_STATE => {
            "Check the sequence of API calls; the engine may need re-initialization"
        }

        // Unknown codes
        _ => "Unknown error code; check the error message for details",
    }
}

/// Returns `true` if the error code is classified as recoverable.
///
/// Recoverable errors are transient or can be resolved by the caller
/// without restarting the engine.
pub const fn is_recoverable(code: GoudErrorCode) -> bool {
    matches!(recovery_class(code), RecoveryClass::Recoverable)
}

/// Returns `true` if the error code is classified as fatal.
///
/// Fatal errors indicate the engine is in an unrecoverable state.
/// The application should re-initialize the engine or exit.
pub const fn is_fatal(code: GoudErrorCode) -> bool {
    matches!(recovery_class(code), RecoveryClass::Fatal)
}
