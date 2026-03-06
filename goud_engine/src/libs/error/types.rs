//! The main `GoudError` enum and its methods.
//!
//! All error variants map to FFI-compatible error codes defined in [`super::codes`].

use super::codes::{
    error_category, GoudErrorCode, ERR_ALREADY_INITIALIZED, ERR_AUDIO_INIT_FAILED,
    ERR_BACKEND_NOT_SUPPORTED, ERR_BUFFER_CREATION_FAILED, ERR_COMPONENT_ALREADY_EXISTS,
    ERR_COMPONENT_NOT_FOUND, ERR_CONTEXT_DESTROYED, ERR_DRAW_CALL_FAILED,
    ERR_ENTITY_ALREADY_EXISTS, ERR_ENTITY_NOT_FOUND, ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH,
    ERR_INITIALIZATION_FAILED, ERR_INTERNAL_ERROR, ERR_INVALID_CONTEXT, ERR_INVALID_HANDLE,
    ERR_INVALID_STATE, ERR_NOT_IMPLEMENTED, ERR_NOT_INITIALIZED, ERR_PHYSICS_INIT_FAILED,
    ERR_PLATFORM_ERROR, ERR_PROVIDER_OPERATION_FAILED, ERR_QUERY_FAILED,
    ERR_RENDER_TARGET_FAILED, ERR_RESOURCE_ALREADY_EXISTS, ERR_RESOURCE_INVALID_FORMAT,
    ERR_RESOURCE_LOAD_FAILED, ERR_RESOURCE_NOT_FOUND, ERR_SHADER_COMPILATION_FAILED,
    ERR_SHADER_LINK_FAILED, ERR_TEXTURE_CREATION_FAILED, ERR_WINDOW_CREATION_FAILED,
};

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
/// - **Resource**: Asset and resource management (codes 100-199)
/// - **Graphics**: Rendering and GPU errors (codes 200-299)
/// - **Entity**: ECS entity and component errors (codes 300-399)
/// - **Input**: Input handling errors (codes 400-499)
/// - **System**: Platform and system errors (codes 500-599)
/// - **Provider**: Provider subsystem errors (codes 600-699)
/// - **Internal**: Unexpected internal errors (codes 900-999)
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

    // -------------------------------------------------------------------------
    // Graphics Errors (codes 200-299)
    // -------------------------------------------------------------------------
    /// Shader compilation failed.
    ///
    /// This error occurs when a vertex, fragment, or compute shader fails to compile.
    /// The string contains the shader compiler error message, which typically includes
    /// line numbers and error descriptions from the GPU driver.
    ShaderCompilationFailed(String),

    /// Shader program linking failed.
    ///
    /// This error occurs when compiled shaders fail to link into a program.
    /// Common causes include mismatched inputs/outputs between shader stages,
    /// missing uniforms, or exceeding GPU resource limits.
    /// The string contains the linker error message.
    ShaderLinkFailed(String),

    /// Texture creation failed.
    ///
    /// This error occurs when the GPU fails to allocate or create a texture.
    /// Common causes include insufficient GPU memory, unsupported texture format,
    /// or dimensions exceeding GPU limits.
    /// The string contains details about the failure.
    TextureCreationFailed(String),

    /// Buffer creation failed.
    ///
    /// This error occurs when the GPU fails to allocate a buffer (vertex, index,
    /// uniform, or storage buffer). Common causes include insufficient GPU memory
    /// or exceeding buffer size limits.
    /// The string contains details about the failure.
    BufferCreationFailed(String),

    /// Render target creation failed.
    ///
    /// This error occurs when creating a framebuffer or render target fails.
    /// Common causes include unsupported attachment formats, mismatched
    /// attachment dimensions, or exceeding attachment limits.
    /// The string contains details about the failure.
    RenderTargetFailed(String),

    /// Graphics backend not supported on this platform.
    ///
    /// This error occurs when the requested graphics API (OpenGL, Vulkan, Metal, etc.)
    /// is not available on the current platform or the installed GPU driver
    /// does not meet minimum version requirements.
    /// The string contains the requested backend and available alternatives.
    BackendNotSupported(String),

    /// Draw call failed.
    ///
    /// This error occurs when a draw call fails during rendering.
    /// This is typically a serious error indicating GPU state corruption,
    /// invalid buffer bindings, or driver issues.
    /// The string contains details about the failed draw call.
    DrawCallFailed(String),

    // -------------------------------------------------------------------------
    // Entity Errors (codes 300-399)
    // -------------------------------------------------------------------------
    /// Entity was not found.
    ///
    /// This error occurs when attempting to access an entity that does not exist
    /// in the world, either because it was never created or has been despawned.
    EntityNotFound,

    /// Entity already exists.
    ///
    /// This error occurs when attempting to create an entity with an ID that
    /// is already in use. This typically indicates a logic error in entity
    /// management.
    EntityAlreadyExists,

    /// Component was not found on entity.
    ///
    /// This error occurs when querying or accessing a component type that
    /// the specified entity does not have attached.
    ComponentNotFound,

    /// Component already exists on entity.
    ///
    /// This error occurs when attempting to add a component type that the
    /// entity already has. Use replace or update methods instead.
    ComponentAlreadyExists,

    /// Query execution failed.
    ///
    /// This error occurs when an ECS query fails to execute. Common causes
    /// include conflicting access patterns (mutable + immutable on same component)
    /// or invalid query parameters.
    /// The string contains details about the query failure.
    QueryFailed(String),

    // -------------------------------------------------------------------------
    // System Errors (codes 500-599)
    // -------------------------------------------------------------------------
    /// Window creation failed.
    ///
    /// This error occurs when the platform window system fails to create
    /// a window. Common causes include missing display server, invalid
    /// window parameters, or resource exhaustion.
    /// The string contains details about the failure.
    WindowCreationFailed(String),

    /// Audio system initialization failed.
    ///
    /// This error occurs when the audio subsystem fails to initialize.
    /// Common causes include missing audio devices, driver issues,
    /// or unsupported audio formats.
    /// The string contains details about the failure.
    AudioInitFailed(String),

    /// Physics system initialization failed.
    ///
    /// This error occurs when the physics engine fails to initialize.
    /// Common causes include invalid physics configuration or
    /// incompatible settings.
    /// The string contains details about the failure.
    PhysicsInitFailed(String),

    /// Generic platform error.
    ///
    /// This error occurs for platform-specific failures that don't fit
    /// into other categories. The string contains platform-specific
    /// error details.
    PlatformError(String),

    // -------------------------------------------------------------------------
    // Provider Errors (codes 600-699)
    // -------------------------------------------------------------------------
    /// Provider subsystem error.
    ///
    /// This error occurs when a provider operation fails. The `subsystem`
    /// field identifies which provider category (e.g., "render", "physics",
    /// "audio") encountered the error, enabling FFI error code routing.
    ProviderError {
        /// The provider subsystem that encountered the error.
        subsystem: &'static str,
        /// Details about the failure.
        message: String,
    },

    // -------------------------------------------------------------------------
    // Internal Errors (codes 900-999)
    // -------------------------------------------------------------------------
    /// Internal engine error.
    ///
    /// This error indicates an unexpected internal state or failure within
    /// the engine. These errors typically indicate bugs in the engine itself
    /// and should be reported.
    /// The string contains details about the internal failure.
    InternalError(String),

    /// Feature not yet implemented.
    ///
    /// This error occurs when attempting to use a feature that has not
    /// been implemented yet. This is primarily used during development
    /// to mark incomplete functionality.
    /// The string contains details about the unimplemented feature.
    NotImplemented(String),

    /// Invalid engine state.
    ///
    /// This error occurs when the engine is in an invalid or unexpected state.
    /// This typically indicates a bug in state management or an invalid
    /// sequence of operations.
    /// The string contains details about the invalid state.
    InvalidState(String),
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

            // System errors (500-599)
            GoudError::WindowCreationFailed(_) => ERR_WINDOW_CREATION_FAILED,
            GoudError::AudioInitFailed(_) => ERR_AUDIO_INIT_FAILED,
            GoudError::PhysicsInitFailed(_) => ERR_PHYSICS_INIT_FAILED,
            GoudError::PlatformError(_) => ERR_PLATFORM_ERROR,

            // Provider errors (600-699)
            GoudError::ProviderError { .. } => ERR_PROVIDER_OPERATION_FAILED,

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

            // System errors
            GoudError::WindowCreationFailed(msg) => msg,
            GoudError::AudioInitFailed(msg) => msg,
            GoudError::PhysicsInitFailed(msg) => msg,
            GoudError::PlatformError(msg) => msg,

            // Provider errors
            GoudError::ProviderError { message, .. } => message,

            // Internal errors
            GoudError::InternalError(msg) => msg,
            GoudError::NotImplemented(msg) => msg,
            GoudError::InvalidState(msg) => msg,
        }
    }
}
