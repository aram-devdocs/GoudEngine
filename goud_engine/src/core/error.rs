//! Error handling infrastructure for GoudEngine.
//!
//! This module provides FFI-compatible error codes and error types that work
//! consistently across Rust and all language bindings (C#, Python, etc.).
//!
//! # Error Code Ranges
//!
//! Error codes are organized into ranges by category:
//!
//! | Range     | Category   | Description                          |
//! |-----------|------------|--------------------------------------|
//! | 0         | Success    | Operation completed successfully     |
//! | 1-99      | Context    | Initialization and context errors    |
//! | 100-199   | Resource   | Asset and resource management errors |
//! | 200-299   | Graphics   | Rendering and GPU errors             |
//! | 300-399   | Entity     | ECS entity and component errors      |
//! | 400-499   | Input      | Input handling errors                |
//! | 500-599   | System     | Platform and system errors           |
//! | 900-999   | Internal   | Unexpected internal errors           |
//!
//! # FFI Compatibility
//!
//! Error codes use `i32` for maximum C ABI compatibility. Negative values
//! are reserved for future use (e.g., platform-specific errors).

/// FFI-compatible error code type.
///
/// Uses `i32` for C ABI compatibility across all platforms.
/// Negative values are reserved for future platform-specific errors.
pub type GoudErrorCode = i32;

// =============================================================================
// Error Code Constants
// =============================================================================

/// Operation completed successfully.
pub const SUCCESS: GoudErrorCode = 0;

// -----------------------------------------------------------------------------
// Context Errors (1-99): Engine initialization and context management
// -----------------------------------------------------------------------------

/// Base code for context/initialization errors.
pub const CONTEXT_ERROR_BASE: GoudErrorCode = 1;

/// Engine has not been initialized.
pub const ERR_NOT_INITIALIZED: GoudErrorCode = 1;

/// Engine has already been initialized.
pub const ERR_ALREADY_INITIALIZED: GoudErrorCode = 2;

/// Invalid engine context.
pub const ERR_INVALID_CONTEXT: GoudErrorCode = 3;

/// Engine context has been destroyed.
pub const ERR_CONTEXT_DESTROYED: GoudErrorCode = 4;

/// Engine initialization failed (generic).
/// Specific error message available via `last_error_message()`.
pub const ERR_INITIALIZATION_FAILED: GoudErrorCode = 10;

// -----------------------------------------------------------------------------
// Resource Errors (100-199): Asset loading and resource management
// -----------------------------------------------------------------------------

/// Base code for resource/asset errors.
pub const RESOURCE_ERROR_BASE: GoudErrorCode = 100;

/// Requested resource was not found.
pub const ERR_RESOURCE_NOT_FOUND: GoudErrorCode = 100;

/// Failed to load resource from source.
pub const ERR_RESOURCE_LOAD_FAILED: GoudErrorCode = 101;

/// Resource format is invalid or unsupported.
pub const ERR_RESOURCE_INVALID_FORMAT: GoudErrorCode = 102;

/// Resource with this identifier already exists.
pub const ERR_RESOURCE_ALREADY_EXISTS: GoudErrorCode = 103;

/// Handle is invalid (null or malformed).
pub const ERR_INVALID_HANDLE: GoudErrorCode = 110;

/// Handle refers to a resource that has been deallocated.
pub const ERR_HANDLE_EXPIRED: GoudErrorCode = 111;

/// Handle type does not match expected resource type.
pub const ERR_HANDLE_TYPE_MISMATCH: GoudErrorCode = 112;

// -----------------------------------------------------------------------------
// Graphics Errors (200-299): Rendering and GPU operations
// -----------------------------------------------------------------------------

/// Base code for graphics/rendering errors.
pub const GRAPHICS_ERROR_BASE: GoudErrorCode = 200;

/// Shader compilation failed.
pub const ERR_SHADER_COMPILATION_FAILED: GoudErrorCode = 200;

/// Shader program linking failed.
pub const ERR_SHADER_LINK_FAILED: GoudErrorCode = 201;

/// Texture creation failed.
pub const ERR_TEXTURE_CREATION_FAILED: GoudErrorCode = 210;

/// Buffer creation failed.
pub const ERR_BUFFER_CREATION_FAILED: GoudErrorCode = 211;

/// Render target creation failed.
pub const ERR_RENDER_TARGET_FAILED: GoudErrorCode = 220;

/// Graphics backend not supported on this platform.
pub const ERR_BACKEND_NOT_SUPPORTED: GoudErrorCode = 230;

/// Draw call failed.
pub const ERR_DRAW_CALL_FAILED: GoudErrorCode = 240;

// -----------------------------------------------------------------------------
// Entity Errors (300-399): ECS entity and component operations
// -----------------------------------------------------------------------------

/// Base code for ECS entity errors.
pub const ENTITY_ERROR_BASE: GoudErrorCode = 300;

/// Entity was not found.
pub const ERR_ENTITY_NOT_FOUND: GoudErrorCode = 300;

/// Entity already exists.
pub const ERR_ENTITY_ALREADY_EXISTS: GoudErrorCode = 301;

/// Component was not found on entity.
pub const ERR_COMPONENT_NOT_FOUND: GoudErrorCode = 310;

/// Component already exists on entity.
pub const ERR_COMPONENT_ALREADY_EXISTS: GoudErrorCode = 311;

/// Query execution failed.
pub const ERR_QUERY_FAILED: GoudErrorCode = 320;

// -----------------------------------------------------------------------------
// Input Errors (400-499): Input handling
// -----------------------------------------------------------------------------

/// Base code for input handling errors.
pub const INPUT_ERROR_BASE: GoudErrorCode = 400;

/// Input device not found or disconnected.
pub const ERR_INPUT_DEVICE_NOT_FOUND: GoudErrorCode = 400;

/// Invalid input action name.
pub const ERR_INVALID_INPUT_ACTION: GoudErrorCode = 401;

// -----------------------------------------------------------------------------
// System Errors (500-599): Platform and system operations
// -----------------------------------------------------------------------------

/// Base code for system/platform errors.
pub const SYSTEM_ERROR_BASE: GoudErrorCode = 500;

/// Window creation failed.
pub const ERR_WINDOW_CREATION_FAILED: GoudErrorCode = 500;

/// Audio system initialization failed.
pub const ERR_AUDIO_INIT_FAILED: GoudErrorCode = 510;

/// Physics system initialization failed.
pub const ERR_PHYSICS_INIT_FAILED: GoudErrorCode = 520;

/// Generic platform error.
pub const ERR_PLATFORM_ERROR: GoudErrorCode = 530;

// -----------------------------------------------------------------------------
// Internal Errors (900-999): Unexpected internal errors
// -----------------------------------------------------------------------------

/// Base code for internal/unexpected errors.
pub const INTERNAL_ERROR_BASE: GoudErrorCode = 900;

/// Internal engine error (unexpected state).
pub const ERR_INTERNAL_ERROR: GoudErrorCode = 900;

/// Feature not yet implemented.
pub const ERR_NOT_IMPLEMENTED: GoudErrorCode = 901;

/// Invalid engine state.
pub const ERR_INVALID_STATE: GoudErrorCode = 902;

// =============================================================================
// GoudError Enum
// =============================================================================

/// The main error type for GoudEngine.
///
/// This enum represents all possible errors that can occur within the engine.
/// Each variant maps to a specific FFI-compatible error code, enabling consistent
/// error handling across Rust and all language bindings.
///
/// # Error Categories
///
/// Errors are organized into categories:
/// - **Context**: Engine initialization and context management (codes 1-99)
/// - More categories will be added in subsequent steps
///
/// # FFI Compatibility
///
/// Use [`GoudError::error_code()`] to get the FFI-compatible error code for any error.
/// This code can be safely passed across the FFI boundary.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, ERR_NOT_INITIALIZED};
///
/// let error = GoudError::NotInitialized;
/// assert_eq!(error.error_code(), ERR_NOT_INITIALIZED);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoudError {
    // -------------------------------------------------------------------------
    // Context Errors (codes 1-99)
    // -------------------------------------------------------------------------
    /// Engine has not been initialized.
    ///
    /// This error occurs when attempting to use engine functionality before
    /// calling the initialization function.
    NotInitialized,

    /// Engine has already been initialized.
    ///
    /// This error occurs when attempting to initialize the engine more than once.
    /// The engine must be shut down before re-initialization.
    AlreadyInitialized,

    /// Invalid engine context.
    ///
    /// The provided engine context handle is invalid or corrupted.
    InvalidContext,

    /// Engine context has been destroyed.
    ///
    /// The engine context was previously valid but has since been destroyed.
    /// Operations cannot be performed on a destroyed context.
    ContextDestroyed,

    /// Engine initialization failed with a specific reason.
    ///
    /// Contains a message describing why initialization failed.
    /// Common causes include missing dependencies, invalid configuration,
    /// or platform-specific issues.
    InitializationFailed(String),

    // -------------------------------------------------------------------------
    // Resource Errors (codes 100-199)
    // -------------------------------------------------------------------------
    /// Requested resource was not found.
    ///
    /// This error occurs when attempting to access a resource (texture, shader,
    /// audio file, etc.) that does not exist at the specified path or identifier.
    /// The string contains the resource path or identifier that was not found.
    ResourceNotFound(String),

    /// Failed to load resource from source.
    ///
    /// This error occurs when a resource exists but could not be loaded due to
    /// I/O errors, permission issues, or other loading failures.
    /// The string contains details about the loading failure.
    ResourceLoadFailed(String),

    /// Resource format is invalid or unsupported.
    ///
    /// This error occurs when a resource file exists and was read successfully,
    /// but the format is invalid, corrupted, or not supported by the engine.
    /// The string contains details about the format issue.
    ResourceInvalidFormat(String),

    /// Resource with this identifier already exists.
    ///
    /// This error occurs when attempting to create or register a resource
    /// with an identifier that is already in use.
    /// The string contains the conflicting resource identifier.
    ResourceAlreadyExists(String),

    /// Handle is invalid (null or malformed).
    ///
    /// This error occurs when an operation is performed with a handle that
    /// was never valid (null, zero, or otherwise malformed).
    InvalidHandle,

    /// Handle refers to a resource that has been deallocated.
    ///
    /// This error occurs when using a handle that was previously valid but
    /// the underlying resource has been freed. This is a use-after-free attempt
    /// that was safely caught by the generational handle system.
    HandleExpired,

    /// Handle type does not match expected resource type.
    ///
    /// This error occurs when a handle is passed to a function expecting a
    /// different resource type (e.g., passing a texture handle to a shader function).
    HandleTypeMismatch,
}

impl GoudError {
    /// Returns the FFI-compatible error code for this error.
    ///
    /// This method maps each error variant to its corresponding error code constant,
    /// which can be safely passed across the FFI boundary to C#, Python, or other
    /// language bindings.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudError, ERR_NOT_INITIALIZED, ERR_INITIALIZATION_FAILED};
    ///
    /// assert_eq!(GoudError::NotInitialized.error_code(), ERR_NOT_INITIALIZED);
    /// assert_eq!(
    ///     GoudError::InitializationFailed("GPU not found".to_string()).error_code(),
    ///     ERR_INITIALIZATION_FAILED
    /// );
    /// ```
    #[inline]
    pub const fn error_code(&self) -> GoudErrorCode {
        match self {
            // Context errors (1-99)
            GoudError::NotInitialized => ERR_NOT_INITIALIZED,
            GoudError::AlreadyInitialized => ERR_ALREADY_INITIALIZED,
            GoudError::InvalidContext => ERR_INVALID_CONTEXT,
            GoudError::ContextDestroyed => ERR_CONTEXT_DESTROYED,
            GoudError::InitializationFailed(_) => ERR_INITIALIZATION_FAILED,

            // Resource errors (100-199)
            GoudError::ResourceNotFound(_) => ERR_RESOURCE_NOT_FOUND,
            GoudError::ResourceLoadFailed(_) => ERR_RESOURCE_LOAD_FAILED,
            GoudError::ResourceInvalidFormat(_) => ERR_RESOURCE_INVALID_FORMAT,
            GoudError::ResourceAlreadyExists(_) => ERR_RESOURCE_ALREADY_EXISTS,
            GoudError::InvalidHandle => ERR_INVALID_HANDLE,
            GoudError::HandleExpired => ERR_HANDLE_EXPIRED,
            GoudError::HandleTypeMismatch => ERR_HANDLE_TYPE_MISMATCH,
        }
    }

    /// Returns the error category as a static string.
    ///
    /// This is a convenience method that returns the category name for this error.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// assert_eq!(GoudError::NotInitialized.category(), "Context");
    /// ```
    #[inline]
    pub const fn category(&self) -> &'static str {
        error_category(self.error_code())
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Returns the category name for an error code.
///
/// # Examples
///
/// ```
/// use goud_engine::core::error::{error_category, SUCCESS, ERR_RESOURCE_NOT_FOUND};
///
/// assert_eq!(error_category(SUCCESS), "Success");
/// assert_eq!(error_category(ERR_RESOURCE_NOT_FOUND), "Resource");
/// ```
#[inline]
pub const fn error_category(code: GoudErrorCode) -> &'static str {
    match code {
        SUCCESS => "Success",
        1..=99 => "Context",
        100..=199 => "Resource",
        200..=299 => "Graphics",
        300..=399 => "Entity",
        400..=499 => "Input",
        500..=599 => "System",
        900..=999 => "Internal",
        _ => "Unknown",
    }
}

/// Returns true if the error code indicates success.
#[inline]
pub const fn is_success(code: GoudErrorCode) -> bool {
    code == SUCCESS
}

/// Returns true if the error code indicates an error.
#[inline]
pub const fn is_error(code: GoudErrorCode) -> bool {
    code != SUCCESS
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_code_is_zero() {
        assert_eq!(SUCCESS, 0);
    }

    #[test]
    fn test_error_code_ranges_are_non_overlapping() {
        // Verify base codes define distinct ranges
        assert!(CONTEXT_ERROR_BASE < RESOURCE_ERROR_BASE);
        assert!(RESOURCE_ERROR_BASE < GRAPHICS_ERROR_BASE);
        assert!(GRAPHICS_ERROR_BASE < ENTITY_ERROR_BASE);
        assert!(ENTITY_ERROR_BASE < INPUT_ERROR_BASE);
        assert!(INPUT_ERROR_BASE < SYSTEM_ERROR_BASE);
        assert!(SYSTEM_ERROR_BASE < INTERNAL_ERROR_BASE);
    }

    #[test]
    fn test_error_category_classification() {
        assert_eq!(error_category(SUCCESS), "Success");
        assert_eq!(error_category(ERR_NOT_INITIALIZED), "Context");
        assert_eq!(error_category(ERR_RESOURCE_NOT_FOUND), "Resource");
        assert_eq!(error_category(ERR_SHADER_COMPILATION_FAILED), "Graphics");
        assert_eq!(error_category(ERR_ENTITY_NOT_FOUND), "Entity");
        assert_eq!(error_category(ERR_INPUT_DEVICE_NOT_FOUND), "Input");
        assert_eq!(error_category(ERR_WINDOW_CREATION_FAILED), "System");
        assert_eq!(error_category(ERR_INTERNAL_ERROR), "Internal");
    }

    #[test]
    fn test_is_success_and_is_error() {
        assert!(is_success(SUCCESS));
        assert!(!is_error(SUCCESS));

        assert!(!is_success(ERR_NOT_INITIALIZED));
        assert!(is_error(ERR_NOT_INITIALIZED));

        assert!(!is_success(ERR_RESOURCE_NOT_FOUND));
        assert!(is_error(ERR_RESOURCE_NOT_FOUND));
    }

    #[test]
    fn test_error_codes_within_category_bounds() {
        // Context errors should be in 1-99 range
        assert!(ERR_NOT_INITIALIZED >= 1 && ERR_NOT_INITIALIZED < 100);
        assert!(ERR_INITIALIZATION_FAILED >= 1 && ERR_INITIALIZATION_FAILED < 100);

        // Resource errors should be in 100-199 range
        assert!(ERR_RESOURCE_NOT_FOUND >= 100 && ERR_RESOURCE_NOT_FOUND < 200);
        assert!(ERR_HANDLE_TYPE_MISMATCH >= 100 && ERR_HANDLE_TYPE_MISMATCH < 200);

        // Graphics errors should be in 200-299 range
        assert!(ERR_SHADER_COMPILATION_FAILED >= 200 && ERR_SHADER_COMPILATION_FAILED < 300);
        assert!(ERR_DRAW_CALL_FAILED >= 200 && ERR_DRAW_CALL_FAILED < 300);

        // Entity errors should be in 300-399 range
        assert!(ERR_ENTITY_NOT_FOUND >= 300 && ERR_ENTITY_NOT_FOUND < 400);
        assert!(ERR_QUERY_FAILED >= 300 && ERR_QUERY_FAILED < 400);

        // Input errors should be in 400-499 range
        assert!(ERR_INPUT_DEVICE_NOT_FOUND >= 400 && ERR_INPUT_DEVICE_NOT_FOUND < 500);

        // System errors should be in 500-599 range
        assert!(ERR_WINDOW_CREATION_FAILED >= 500 && ERR_WINDOW_CREATION_FAILED < 600);
        assert!(ERR_PLATFORM_ERROR >= 500 && ERR_PLATFORM_ERROR < 600);

        // Internal errors should be in 900-999 range
        assert!(ERR_INTERNAL_ERROR >= 900 && ERR_INTERNAL_ERROR < 1000);
        assert!(ERR_INVALID_STATE >= 900 && ERR_INVALID_STATE < 1000);
    }

    #[test]
    fn test_unknown_category_for_out_of_range() {
        assert_eq!(error_category(-1), "Unknown");
        assert_eq!(error_category(1000), "Unknown");
        assert_eq!(error_category(600), "Unknown");
    }

    // =========================================================================
    // GoudError Context Variant Tests
    // =========================================================================

    mod context_errors {
        use super::*;

        #[test]
        fn test_not_initialized_error_code() {
            let error = GoudError::NotInitialized;
            assert_eq!(error.error_code(), ERR_NOT_INITIALIZED);
            assert_eq!(error.error_code(), 1);
        }

        #[test]
        fn test_already_initialized_error_code() {
            let error = GoudError::AlreadyInitialized;
            assert_eq!(error.error_code(), ERR_ALREADY_INITIALIZED);
            assert_eq!(error.error_code(), 2);
        }

        #[test]
        fn test_invalid_context_error_code() {
            let error = GoudError::InvalidContext;
            assert_eq!(error.error_code(), ERR_INVALID_CONTEXT);
            assert_eq!(error.error_code(), 3);
        }

        #[test]
        fn test_context_destroyed_error_code() {
            let error = GoudError::ContextDestroyed;
            assert_eq!(error.error_code(), ERR_CONTEXT_DESTROYED);
            assert_eq!(error.error_code(), 4);
        }

        #[test]
        fn test_initialization_failed_error_code() {
            let error = GoudError::InitializationFailed("GPU not found".to_string());
            assert_eq!(error.error_code(), ERR_INITIALIZATION_FAILED);
            assert_eq!(error.error_code(), 10);

            // Different messages should have same error code
            let error2 = GoudError::InitializationFailed("Missing dependency".to_string());
            assert_eq!(error2.error_code(), ERR_INITIALIZATION_FAILED);
        }

        #[test]
        fn test_all_context_errors_in_context_category() {
            let errors = [
                GoudError::NotInitialized,
                GoudError::AlreadyInitialized,
                GoudError::InvalidContext,
                GoudError::ContextDestroyed,
                GoudError::InitializationFailed("test".to_string()),
            ];

            for error in errors {
                assert_eq!(
                    error.category(),
                    "Context",
                    "Error {:?} should be in Context category",
                    error
                );
            }
        }

        #[test]
        fn test_context_error_codes_in_valid_range() {
            let errors = [
                GoudError::NotInitialized,
                GoudError::AlreadyInitialized,
                GoudError::InvalidContext,
                GoudError::ContextDestroyed,
                GoudError::InitializationFailed("test".to_string()),
            ];

            for error in errors {
                let code = error.error_code();
                assert!(
                    code >= 1 && code < 100,
                    "Context error {:?} has code {} which is outside range 1-99",
                    error,
                    code
                );
            }
        }

        #[test]
        fn test_goud_error_derives() {
            // Test Debug
            let error = GoudError::NotInitialized;
            let debug_str = format!("{:?}", error);
            assert!(debug_str.contains("NotInitialized"));

            // Test Clone
            let cloned = error.clone();
            assert_eq!(error, cloned);

            // Test PartialEq and Eq
            assert_eq!(GoudError::NotInitialized, GoudError::NotInitialized);
            assert_ne!(GoudError::NotInitialized, GoudError::AlreadyInitialized);

            // Test equality with message content
            let err1 = GoudError::InitializationFailed("msg1".to_string());
            let err2 = GoudError::InitializationFailed("msg1".to_string());
            let err3 = GoudError::InitializationFailed("msg2".to_string());
            assert_eq!(err1, err2);
            assert_ne!(err1, err3);
        }

        #[test]
        fn test_initialization_failed_preserves_message() {
            let message = "Failed to initialize OpenGL context: version 4.5 required";
            let error = GoudError::InitializationFailed(message.to_string());

            // Verify we can pattern match and extract the message
            if let GoudError::InitializationFailed(msg) = error {
                assert_eq!(msg, message);
            } else {
                panic!("Expected InitializationFailed variant");
            }
        }
    }

    // =========================================================================
    // GoudError Resource Variant Tests
    // =========================================================================

    mod resource_errors {
        use super::*;

        #[test]
        fn test_resource_not_found_error_code() {
            let error = GoudError::ResourceNotFound("textures/player.png".to_string());
            assert_eq!(error.error_code(), ERR_RESOURCE_NOT_FOUND);
            assert_eq!(error.error_code(), 100);
        }

        #[test]
        fn test_resource_load_failed_error_code() {
            let error = GoudError::ResourceLoadFailed("I/O error reading file".to_string());
            assert_eq!(error.error_code(), ERR_RESOURCE_LOAD_FAILED);
            assert_eq!(error.error_code(), 101);
        }

        #[test]
        fn test_resource_invalid_format_error_code() {
            let error = GoudError::ResourceInvalidFormat("Invalid PNG header".to_string());
            assert_eq!(error.error_code(), ERR_RESOURCE_INVALID_FORMAT);
            assert_eq!(error.error_code(), 102);
        }

        #[test]
        fn test_resource_already_exists_error_code() {
            let error = GoudError::ResourceAlreadyExists("player_texture".to_string());
            assert_eq!(error.error_code(), ERR_RESOURCE_ALREADY_EXISTS);
            assert_eq!(error.error_code(), 103);
        }

        #[test]
        fn test_invalid_handle_error_code() {
            let error = GoudError::InvalidHandle;
            assert_eq!(error.error_code(), ERR_INVALID_HANDLE);
            assert_eq!(error.error_code(), 110);
        }

        #[test]
        fn test_handle_expired_error_code() {
            let error = GoudError::HandleExpired;
            assert_eq!(error.error_code(), ERR_HANDLE_EXPIRED);
            assert_eq!(error.error_code(), 111);
        }

        #[test]
        fn test_handle_type_mismatch_error_code() {
            let error = GoudError::HandleTypeMismatch;
            assert_eq!(error.error_code(), ERR_HANDLE_TYPE_MISMATCH);
            assert_eq!(error.error_code(), 112);
        }

        #[test]
        fn test_all_resource_errors_in_resource_category() {
            let errors: Vec<GoudError> = vec![
                GoudError::ResourceNotFound("test".to_string()),
                GoudError::ResourceLoadFailed("test".to_string()),
                GoudError::ResourceInvalidFormat("test".to_string()),
                GoudError::ResourceAlreadyExists("test".to_string()),
                GoudError::InvalidHandle,
                GoudError::HandleExpired,
                GoudError::HandleTypeMismatch,
            ];

            for error in errors {
                assert_eq!(
                    error.category(),
                    "Resource",
                    "Error {:?} should be in Resource category",
                    error
                );
            }
        }

        #[test]
        fn test_resource_error_codes_in_valid_range() {
            let errors: Vec<GoudError> = vec![
                GoudError::ResourceNotFound("test".to_string()),
                GoudError::ResourceLoadFailed("test".to_string()),
                GoudError::ResourceInvalidFormat("test".to_string()),
                GoudError::ResourceAlreadyExists("test".to_string()),
                GoudError::InvalidHandle,
                GoudError::HandleExpired,
                GoudError::HandleTypeMismatch,
            ];

            for error in errors {
                let code = error.error_code();
                assert!(
                    code >= 100 && code < 200,
                    "Resource error {:?} has code {} which is outside range 100-199",
                    error,
                    code
                );
            }
        }

        #[test]
        fn test_resource_errors_preserve_message() {
            // Test ResourceNotFound
            let path = "assets/textures/missing.png";
            if let GoudError::ResourceNotFound(msg) = GoudError::ResourceNotFound(path.to_string())
            {
                assert_eq!(msg, path);
            } else {
                panic!("Expected ResourceNotFound variant");
            }

            // Test ResourceLoadFailed
            let reason = "Permission denied";
            if let GoudError::ResourceLoadFailed(msg) =
                GoudError::ResourceLoadFailed(reason.to_string())
            {
                assert_eq!(msg, reason);
            } else {
                panic!("Expected ResourceLoadFailed variant");
            }

            // Test ResourceInvalidFormat
            let format_err = "Unsupported texture format: PVRTC";
            if let GoudError::ResourceInvalidFormat(msg) =
                GoudError::ResourceInvalidFormat(format_err.to_string())
            {
                assert_eq!(msg, format_err);
            } else {
                panic!("Expected ResourceInvalidFormat variant");
            }

            // Test ResourceAlreadyExists
            let resource_id = "main_shader";
            if let GoudError::ResourceAlreadyExists(msg) =
                GoudError::ResourceAlreadyExists(resource_id.to_string())
            {
                assert_eq!(msg, resource_id);
            } else {
                panic!("Expected ResourceAlreadyExists variant");
            }
        }

        #[test]
        fn test_resource_error_equality() {
            // Same variant, same message
            let err1 = GoudError::ResourceNotFound("file.txt".to_string());
            let err2 = GoudError::ResourceNotFound("file.txt".to_string());
            assert_eq!(err1, err2);

            // Same variant, different message
            let err3 = GoudError::ResourceNotFound("other.txt".to_string());
            assert_ne!(err1, err3);

            // Different variants
            let err4 = GoudError::ResourceLoadFailed("file.txt".to_string());
            assert_ne!(err1, err4);

            // Handle errors (no message)
            assert_eq!(GoudError::InvalidHandle, GoudError::InvalidHandle);
            assert_ne!(GoudError::InvalidHandle, GoudError::HandleExpired);
            assert_ne!(GoudError::HandleExpired, GoudError::HandleTypeMismatch);
        }

        #[test]
        fn test_resource_error_debug_format() {
            let error = GoudError::ResourceNotFound("test.png".to_string());
            let debug_str = format!("{:?}", error);
            assert!(debug_str.contains("ResourceNotFound"));
            assert!(debug_str.contains("test.png"));

            let handle_error = GoudError::InvalidHandle;
            let debug_str = format!("{:?}", handle_error);
            assert!(debug_str.contains("InvalidHandle"));
        }

        #[test]
        fn test_handle_error_codes_are_distinct() {
            // Verify handle-related error codes are properly separated
            assert_ne!(ERR_INVALID_HANDLE, ERR_HANDLE_EXPIRED);
            assert_ne!(ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH);
            assert_ne!(ERR_INVALID_HANDLE, ERR_HANDLE_TYPE_MISMATCH);

            // All should be in the 110+ range (gap from 103)
            assert!(ERR_INVALID_HANDLE >= 110);
            assert!(ERR_HANDLE_EXPIRED >= 110);
            assert!(ERR_HANDLE_TYPE_MISMATCH >= 110);
        }
    }
}
