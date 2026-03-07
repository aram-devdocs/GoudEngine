//! FFI-compatible error code constants organized by category.
//!
//! Error codes use `i32` for maximum C ABI compatibility. Negative values
//! are reserved for future use (e.g., platform-specific errors).
//!
//! # Error Code Ranges
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
//! | 600-699   | Provider   | Provider subsystem errors            |
//! | 900-999   | Internal   | Unexpected internal errors           |

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
/// Recovery: call the initialization function first.
pub const ERR_NOT_INITIALIZED: GoudErrorCode = 1;

/// Engine has already been initialized.
/// Recovery: shut down the engine before re-initializing.
pub const ERR_ALREADY_INITIALIZED: GoudErrorCode = 2;

/// Invalid engine context.
/// Recovery: ensure the context was properly created and not corrupted.
pub const ERR_INVALID_CONTEXT: GoudErrorCode = 3;

/// Engine context has been destroyed.
/// Recovery: re-initialize the engine to obtain a new context.
pub const ERR_CONTEXT_DESTROYED: GoudErrorCode = 4;

/// Engine initialization failed (generic).
/// Specific error message available via `last_error_message()`.
/// Recovery: check the error message for details and verify dependencies.
pub const ERR_INITIALIZATION_FAILED: GoudErrorCode = 10;

// -----------------------------------------------------------------------------
// Resource Errors (100-199): Asset loading and resource management
// -----------------------------------------------------------------------------

/// Base code for resource/asset errors.
pub const RESOURCE_ERROR_BASE: GoudErrorCode = 100;

/// Requested resource was not found.
/// Recovery: verify the file path and check the working directory.
pub const ERR_RESOURCE_NOT_FOUND: GoudErrorCode = 100;

/// Failed to load resource from source.
/// Recovery: check file permissions and ensure the file is not locked.
pub const ERR_RESOURCE_LOAD_FAILED: GoudErrorCode = 101;

/// Resource format is invalid or unsupported.
/// Recovery: verify the file is not corrupted and uses a supported format.
pub const ERR_RESOURCE_INVALID_FORMAT: GoudErrorCode = 102;

/// Resource with this identifier already exists.
/// Recovery: use a unique identifier or remove the existing resource first.
pub const ERR_RESOURCE_ALREADY_EXISTS: GoudErrorCode = 103;

/// Handle is invalid (null or malformed).
/// Recovery: ensure the handle was obtained from a valid creation call.
pub const ERR_INVALID_HANDLE: GoudErrorCode = 110;

/// Handle refers to a resource that has been deallocated.
/// Recovery: re-create the resource to get a new handle.
pub const ERR_HANDLE_EXPIRED: GoudErrorCode = 111;

/// Handle type does not match expected resource type.
/// Recovery: pass the correct handle type for the operation.
pub const ERR_HANDLE_TYPE_MISMATCH: GoudErrorCode = 112;

// -----------------------------------------------------------------------------
// Graphics Errors (200-299): Rendering and GPU operations
// -----------------------------------------------------------------------------

/// Base code for graphics/rendering errors.
pub const GRAPHICS_ERROR_BASE: GoudErrorCode = 200;

/// Shader compilation failed.
/// Recovery: review shader source; the error message contains GPU compiler output.
pub const ERR_SHADER_COMPILATION_FAILED: GoudErrorCode = 200;

/// Shader program linking failed.
/// Recovery: verify shader stage inputs/outputs match and uniforms are declared.
pub const ERR_SHADER_LINK_FAILED: GoudErrorCode = 201;

/// Texture creation failed.
/// Recovery: check texture dimensions and format; reduce size or free GPU resources.
pub const ERR_TEXTURE_CREATION_FAILED: GoudErrorCode = 210;

/// Buffer creation failed.
/// Recovery: reduce buffer size or free unused GPU buffers.
pub const ERR_BUFFER_CREATION_FAILED: GoudErrorCode = 211;

/// Render target creation failed.
/// Recovery: verify attachment formats and dimensions are consistent.
pub const ERR_RENDER_TARGET_FAILED: GoudErrorCode = 220;

/// Graphics backend not supported on this platform.
/// Recovery: update GPU drivers or select a different supported backend.
pub const ERR_BACKEND_NOT_SUPPORTED: GoudErrorCode = 230;

/// Draw call failed.
/// Recovery: verify buffer bindings and shader state; try updating GPU drivers.
pub const ERR_DRAW_CALL_FAILED: GoudErrorCode = 240;

// -----------------------------------------------------------------------------
// Entity Errors (300-399): ECS entity and component operations
// -----------------------------------------------------------------------------

/// Base code for ECS entity errors.
pub const ENTITY_ERROR_BASE: GoudErrorCode = 300;

/// Entity was not found.
/// Recovery: verify the entity ID is valid and has not been despawned.
pub const ERR_ENTITY_NOT_FOUND: GoudErrorCode = 300;

/// Entity already exists.
/// Recovery: use a different entity ID or remove the existing entity first.
pub const ERR_ENTITY_ALREADY_EXISTS: GoudErrorCode = 301;

/// Component was not found on entity.
/// Recovery: attach the component before accessing it, or check with a has-component query.
pub const ERR_COMPONENT_NOT_FOUND: GoudErrorCode = 310;

/// Component already exists on entity.
/// Recovery: use replace/update instead of add, or remove the existing component first.
pub const ERR_COMPONENT_ALREADY_EXISTS: GoudErrorCode = 311;

/// Query execution failed.
/// Recovery: check for conflicting mutable/immutable access on the same component.
pub const ERR_QUERY_FAILED: GoudErrorCode = 320;

// -----------------------------------------------------------------------------
// Input Errors (400-499): Input handling
// -----------------------------------------------------------------------------

/// Base code for input handling errors.
pub const INPUT_ERROR_BASE: GoudErrorCode = 400;

/// Input device not found or disconnected.
/// Recovery: verify the input device is connected and recognized by the OS.
pub const ERR_INPUT_DEVICE_NOT_FOUND: GoudErrorCode = 400;

/// Invalid input action name.
/// Recovery: check the action name matches a registered input action.
pub const ERR_INVALID_INPUT_ACTION: GoudErrorCode = 401;

// -----------------------------------------------------------------------------
// System Errors (500-599): Platform and system operations
// -----------------------------------------------------------------------------

/// Base code for system/platform errors.
pub const SYSTEM_ERROR_BASE: GoudErrorCode = 500;

/// Window creation failed.
/// Recovery: verify display server is running and window parameters are valid.
pub const ERR_WINDOW_CREATION_FAILED: GoudErrorCode = 500;

/// Audio system initialization failed.
/// Recovery: check that an audio output device is available.
pub const ERR_AUDIO_INIT_FAILED: GoudErrorCode = 510;

/// Physics system initialization failed.
/// Recovery: review physics configuration for invalid values.
pub const ERR_PHYSICS_INIT_FAILED: GoudErrorCode = 520;

/// Generic platform error.
/// Recovery: check the error message for platform-specific details.
pub const ERR_PLATFORM_ERROR: GoudErrorCode = 530;

// -----------------------------------------------------------------------------
// Provider Errors (600-699): Provider subsystem errors
// -----------------------------------------------------------------------------

/// Base code for provider errors.
pub const PROVIDER_ERROR_BASE: GoudErrorCode = 600;

/// Provider initialization failed.
pub const ERR_PROVIDER_INIT_FAILED: GoudErrorCode = 600;

/// Provider was not found or not registered.
pub const ERR_PROVIDER_NOT_FOUND: GoudErrorCode = 601;

/// Provider operation failed.
pub const ERR_PROVIDER_OPERATION_FAILED: GoudErrorCode = 602;

// Provider subsystem ranges:
// 600-609: Render provider errors
// 610-619: Physics provider errors
// 620-629: Audio provider errors
// 630-639: Window provider errors
// 640-649: Input provider errors
// 700-709: Reserved for future use (network provider uses generic ProviderError)

// -----------------------------------------------------------------------------
// Internal Errors (900-999): Unexpected internal errors
// -----------------------------------------------------------------------------

/// Base code for internal/unexpected errors.
pub const INTERNAL_ERROR_BASE: GoudErrorCode = 900;

/// Internal engine error (unexpected state).
/// Recovery: report the error with full details; this is likely an engine bug.
pub const ERR_INTERNAL_ERROR: GoudErrorCode = 900;

/// Feature not yet implemented.
/// Recovery: use an alternative approach or wait for the feature to be implemented.
pub const ERR_NOT_IMPLEMENTED: GoudErrorCode = 901;

/// Invalid engine state.
/// Recovery: check the sequence of API calls; the engine may need re-initialization.
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
        600..=699 => "Provider",
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
