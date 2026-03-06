//! Reverse mapping from FFI error codes back to `GoudError` variants.

use super::codes::{
    GoudErrorCode, ERR_ALREADY_INITIALIZED, ERR_AUDIO_INIT_FAILED, ERR_BACKEND_NOT_SUPPORTED,
    ERR_BUFFER_CREATION_FAILED, ERR_COMPONENT_ALREADY_EXISTS, ERR_COMPONENT_NOT_FOUND,
    ERR_CONTEXT_DESTROYED, ERR_DRAW_CALL_FAILED, ERR_ENTITY_ALREADY_EXISTS, ERR_ENTITY_NOT_FOUND,
    ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH, ERR_INITIALIZATION_FAILED,
    ERR_INPUT_DEVICE_NOT_FOUND, ERR_INTERNAL_ERROR, ERR_INVALID_CONTEXT, ERR_INVALID_HANDLE,
    ERR_INVALID_INPUT_ACTION, ERR_INVALID_STATE, ERR_NOT_IMPLEMENTED, ERR_NOT_INITIALIZED,
    ERR_PHYSICS_INIT_FAILED, ERR_PLATFORM_ERROR, ERR_QUERY_FAILED, ERR_RENDER_TARGET_FAILED,
    ERR_RESOURCE_ALREADY_EXISTS, ERR_RESOURCE_INVALID_FORMAT, ERR_RESOURCE_LOAD_FAILED,
    ERR_RESOURCE_NOT_FOUND, ERR_SHADER_COMPILATION_FAILED, ERR_SHADER_LINK_FAILED,
    ERR_TEXTURE_CREATION_FAILED, ERR_WINDOW_CREATION_FAILED, SUCCESS,
};
use super::types::GoudError;

impl GoudError {
    /// Constructs a `GoudError` from an FFI error code.
    ///
    /// Returns `None` for [`SUCCESS`] (code 0) and for unknown error codes.
    /// For variants that carry a `String` payload, the returned error uses an
    /// empty string -- callers should populate the message separately if needed
    /// (e.g., via `last_error_message()`).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudError, ERR_NOT_INITIALIZED, SUCCESS};
    ///
    /// assert_eq!(
    ///     GoudError::from_error_code(ERR_NOT_INITIALIZED),
    ///     Some(GoudError::NotInitialized),
    /// );
    /// assert_eq!(GoudError::from_error_code(SUCCESS), None);
    /// ```
    pub fn from_error_code(code: GoudErrorCode) -> Option<Self> {
        match code {
            SUCCESS => None,

            // Context errors (1-99)
            ERR_NOT_INITIALIZED => Some(GoudError::NotInitialized),
            ERR_ALREADY_INITIALIZED => Some(GoudError::AlreadyInitialized),
            ERR_INVALID_CONTEXT => Some(GoudError::InvalidContext),
            ERR_CONTEXT_DESTROYED => Some(GoudError::ContextDestroyed),
            ERR_INITIALIZATION_FAILED => Some(GoudError::InitializationFailed(String::new())),

            // Resource errors (100-199)
            ERR_RESOURCE_NOT_FOUND => Some(GoudError::ResourceNotFound(String::new())),
            ERR_RESOURCE_LOAD_FAILED => Some(GoudError::ResourceLoadFailed(String::new())),
            ERR_RESOURCE_INVALID_FORMAT => Some(GoudError::ResourceInvalidFormat(String::new())),
            ERR_RESOURCE_ALREADY_EXISTS => Some(GoudError::ResourceAlreadyExists(String::new())),
            ERR_INVALID_HANDLE => Some(GoudError::InvalidHandle),
            ERR_HANDLE_EXPIRED => Some(GoudError::HandleExpired),
            ERR_HANDLE_TYPE_MISMATCH => Some(GoudError::HandleTypeMismatch),

            // Graphics errors (200-299)
            ERR_SHADER_COMPILATION_FAILED => {
                Some(GoudError::ShaderCompilationFailed(String::new()))
            }
            ERR_SHADER_LINK_FAILED => Some(GoudError::ShaderLinkFailed(String::new())),
            ERR_TEXTURE_CREATION_FAILED => Some(GoudError::TextureCreationFailed(String::new())),
            ERR_BUFFER_CREATION_FAILED => Some(GoudError::BufferCreationFailed(String::new())),
            ERR_RENDER_TARGET_FAILED => Some(GoudError::RenderTargetFailed(String::new())),
            ERR_BACKEND_NOT_SUPPORTED => Some(GoudError::BackendNotSupported(String::new())),
            ERR_DRAW_CALL_FAILED => Some(GoudError::DrawCallFailed(String::new())),

            // Entity errors (300-399)
            ERR_ENTITY_NOT_FOUND => Some(GoudError::EntityNotFound),
            ERR_ENTITY_ALREADY_EXISTS => Some(GoudError::EntityAlreadyExists),
            ERR_COMPONENT_NOT_FOUND => Some(GoudError::ComponentNotFound),
            ERR_COMPONENT_ALREADY_EXISTS => Some(GoudError::ComponentAlreadyExists),
            ERR_QUERY_FAILED => Some(GoudError::QueryFailed(String::new())),

            // Input errors (400-499)
            ERR_INPUT_DEVICE_NOT_FOUND => Some(GoudError::InputDeviceNotFound),
            ERR_INVALID_INPUT_ACTION => Some(GoudError::InvalidInputAction(String::new())),

            // System errors (500-599)
            ERR_WINDOW_CREATION_FAILED => Some(GoudError::WindowCreationFailed(String::new())),
            ERR_AUDIO_INIT_FAILED => Some(GoudError::AudioInitFailed(String::new())),
            ERR_PHYSICS_INIT_FAILED => Some(GoudError::PhysicsInitFailed(String::new())),
            ERR_PLATFORM_ERROR => Some(GoudError::PlatformError(String::new())),

            // Internal errors (900-999)
            ERR_INTERNAL_ERROR => Some(GoudError::InternalError(String::new())),
            ERR_NOT_IMPLEMENTED => Some(GoudError::NotImplemented(String::new())),
            ERR_INVALID_STATE => Some(GoudError::InvalidState(String::new())),

            _ => None,
        }
    }
}
