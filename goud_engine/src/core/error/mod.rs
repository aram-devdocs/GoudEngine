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

mod codes;
mod conversions;
mod ffi_bridge;
mod reverse_mapping;
mod types;

// Re-export everything so external code sees the same flat API as before.

pub use codes::{
    error_category, is_error, is_success, GoudErrorCode, CONTEXT_ERROR_BASE, ENTITY_ERROR_BASE,
    ERR_ALREADY_INITIALIZED, ERR_AUDIO_INIT_FAILED, ERR_BACKEND_NOT_SUPPORTED,
    ERR_BUFFER_CREATION_FAILED, ERR_COMPONENT_ALREADY_EXISTS, ERR_COMPONENT_NOT_FOUND,
    ERR_CONTEXT_DESTROYED, ERR_DRAW_CALL_FAILED, ERR_ENTITY_ALREADY_EXISTS, ERR_ENTITY_NOT_FOUND,
    ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH, ERR_INITIALIZATION_FAILED,
    ERR_INPUT_DEVICE_NOT_FOUND, ERR_INTERNAL_ERROR, ERR_INVALID_CONTEXT, ERR_INVALID_HANDLE,
    ERR_INVALID_INPUT_ACTION, ERR_INVALID_STATE, ERR_NOT_IMPLEMENTED, ERR_NOT_INITIALIZED,
    ERR_PHYSICS_INIT_FAILED, ERR_PLATFORM_ERROR, ERR_QUERY_FAILED, ERR_RENDER_TARGET_FAILED,
    ERR_RESOURCE_ALREADY_EXISTS, ERR_RESOURCE_INVALID_FORMAT, ERR_RESOURCE_LOAD_FAILED,
    ERR_RESOURCE_NOT_FOUND, ERR_SHADER_COMPILATION_FAILED, ERR_SHADER_LINK_FAILED,
    ERR_TEXTURE_CREATION_FAILED, ERR_WINDOW_CREATION_FAILED, GRAPHICS_ERROR_BASE, INPUT_ERROR_BASE,
    INTERNAL_ERROR_BASE, RESOURCE_ERROR_BASE, SUCCESS, SYSTEM_ERROR_BASE,
};

pub use ffi_bridge::{
    clear_last_error, get_last_error, last_error_code, last_error_message, set_last_error,
    take_last_error, GoudFFIResult,
};

pub use types::GoudError;

// =============================================================================
// Result Type Alias
// =============================================================================

/// A specialized `Result` type for GoudEngine operations.
///
/// This type alias provides a convenient way to work with results that may
/// contain a `GoudError`. It's the standard return type for fallible operations
/// throughout the engine.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudResult, GoudError};
///
/// fn load_texture(path: &str) -> GoudResult<u32> {
///     if path.is_empty() {
///         return Err(GoudError::ResourceNotFound("Empty path".to_string()));
///     }
///     Ok(42) // texture id
/// }
///
/// match load_texture("player.png") {
///     Ok(id) => println!("Loaded texture with id: {}", id),
///     Err(e) => println!("Failed to load: {}", e),
/// }
/// ```
pub type GoudResult<T> = Result<T, GoudError>;

#[cfg(test)]
mod tests {
    mod codes_tests;
    mod context_errors;
    mod entity_errors;
    mod ffi_tests;
    mod graphics_errors;
    mod internal_errors;
    mod resource_errors;
    mod round_trip;
    mod system_errors;
    mod traits;
}
