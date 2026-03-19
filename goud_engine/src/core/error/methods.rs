//! Implementation methods for `GoudError`.

use super::codes::*;
use super::types::GoudError;

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

            // Graphics errors (200-299)
            GoudError::ShaderCompilationFailed(_) => ERR_SHADER_COMPILATION_FAILED,
            GoudError::ShaderLinkFailed(_) => ERR_SHADER_LINK_FAILED,
            GoudError::TextureCreationFailed(_) => ERR_TEXTURE_CREATION_FAILED,
            GoudError::BufferCreationFailed(_) => ERR_BUFFER_CREATION_FAILED,
            GoudError::RenderTargetFailed(_) => ERR_RENDER_TARGET_FAILED,
            GoudError::BackendNotSupported(_) => ERR_BACKEND_NOT_SUPPORTED,
            GoudError::DrawCallFailed(_) => ERR_DRAW_CALL_FAILED,

            // Entity errors (300-399)
            GoudError::EntityNotFound => ERR_ENTITY_NOT_FOUND,
            GoudError::EntityAlreadyExists => ERR_ENTITY_ALREADY_EXISTS,
            GoudError::ComponentNotFound => ERR_COMPONENT_NOT_FOUND,
            GoudError::ComponentAlreadyExists => ERR_COMPONENT_ALREADY_EXISTS,
            GoudError::QueryFailed(_) => ERR_QUERY_FAILED,

            // Input errors (400-499)
            GoudError::InputDeviceNotFound => ERR_INPUT_DEVICE_NOT_FOUND,
            GoudError::InvalidInputAction(_) => ERR_INVALID_INPUT_ACTION,

            // System errors (500-599)
            GoudError::WindowCreationFailed(_) => ERR_WINDOW_CREATION_FAILED,
            GoudError::AudioInitFailed(_) => ERR_AUDIO_INIT_FAILED,
            GoudError::PhysicsInitFailed(_) => ERR_PHYSICS_INIT_FAILED,
            GoudError::PlatformError(_) => ERR_PLATFORM_ERROR,

            // Provider errors (600-699)
            GoudError::ProviderError { .. } => ERR_PROVIDER_OPERATION_FAILED,

            // Script errors (800-899)
            GoudError::ScriptError(_) => ERR_SCRIPT_ERROR,

            // Internal errors (900-999)
            GoudError::InternalError(_) => ERR_INTERNAL_ERROR,
            GoudError::NotImplemented(_) => ERR_NOT_IMPLEMENTED,
            GoudError::InvalidState(_) => ERR_INVALID_STATE,
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

    /// Returns the error message for errors that contain one, or a default description.
    ///
    /// For errors with associated string messages, this returns the message.
    /// For errors without messages, this returns a default description.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error = GoudError::InitializationFailed("GPU not found".to_string());
    /// assert_eq!(error.message(), "GPU not found");
    ///
    /// let error = GoudError::NotInitialized;
    /// assert_eq!(error.message(), "Engine has not been initialized");
    /// ```
    pub fn message(&self) -> &str {
        match self {
            // Context errors
            GoudError::NotInitialized => "Engine has not been initialized",
            GoudError::AlreadyInitialized => "Engine has already been initialized",
            GoudError::InvalidContext => "Invalid engine context",
            GoudError::ContextDestroyed => "Engine context has been destroyed",
            GoudError::InitializationFailed(msg) => msg,

            // Resource errors
            GoudError::ResourceNotFound(msg) => msg,
            GoudError::ResourceLoadFailed(msg) => msg,
            GoudError::ResourceInvalidFormat(msg) => msg,
            GoudError::ResourceAlreadyExists(msg) => msg,
            GoudError::InvalidHandle => "Invalid handle",
            GoudError::HandleExpired => "Handle has expired",
            GoudError::HandleTypeMismatch => "Handle type mismatch",

            // Graphics errors
            GoudError::ShaderCompilationFailed(msg) => msg,
            GoudError::ShaderLinkFailed(msg) => msg,
            GoudError::TextureCreationFailed(msg) => msg,
            GoudError::BufferCreationFailed(msg) => msg,
            GoudError::RenderTargetFailed(msg) => msg,
            GoudError::BackendNotSupported(msg) => msg,
            GoudError::DrawCallFailed(msg) => msg,

            // Entity errors
            GoudError::EntityNotFound => "Entity not found",
            GoudError::EntityAlreadyExists => "Entity already exists",
            GoudError::ComponentNotFound => "Component not found",
            GoudError::ComponentAlreadyExists => "Component already exists",
            GoudError::QueryFailed(msg) => msg,

            // Input errors
            GoudError::InputDeviceNotFound => "Input device not found",
            GoudError::InvalidInputAction(msg) => msg,

            // System errors
            GoudError::WindowCreationFailed(msg) => msg,
            GoudError::AudioInitFailed(msg) => msg,
            GoudError::PhysicsInitFailed(msg) => msg,
            GoudError::PlatformError(msg) => msg,

            // Provider errors
            GoudError::ProviderError { message, .. } => message,

            // Script errors
            GoudError::ScriptError(msg) => msg,

            // Internal errors
            GoudError::InternalError(msg) => msg,
            GoudError::NotImplemented(msg) => msg,
            GoudError::InvalidState(msg) => msg,
        }
    }
}
