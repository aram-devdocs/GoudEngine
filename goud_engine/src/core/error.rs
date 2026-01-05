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
}
