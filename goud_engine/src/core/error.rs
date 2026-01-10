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

            // Internal errors
            GoudError::InternalError(msg) => msg,
            GoudError::NotImplemented(msg) => msg,
            GoudError::InvalidState(msg) => msg,
        }
    }
}

// =============================================================================
// Standard Trait Implementations
// =============================================================================

impl std::fmt::Display for GoudError {
    /// Formats the error for user-friendly display.
    ///
    /// Format: `"[GOUD-{code}] {category}: {message}"`
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error = GoudError::NotInitialized;
    /// let display = format!("{}", error);
    /// assert!(display.contains("[GOUD-1]"));
    /// assert!(display.contains("Context"));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[GOUD-{}] {}: {}",
            self.error_code(),
            self.category(),
            self.message()
        )
    }
}

impl std::error::Error for GoudError {
    /// Returns the source of this error, if any.
    ///
    /// Currently, `GoudError` does not wrap other errors, so this always returns `None`.
    /// Future versions may add error chaining for wrapped errors.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// =============================================================================
// Conversion Implementations
// =============================================================================

impl From<std::io::Error> for GoudError {
    /// Converts an I/O error into a `GoudError`.
    ///
    /// The I/O error is mapped to an appropriate `GoudError` variant based on its kind:
    /// - `NotFound` -> `ResourceNotFound`
    /// - `PermissionDenied` -> `ResourceLoadFailed`
    /// - Other -> `ResourceLoadFailed` with the error message
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    /// use std::io;
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let goud_error: GoudError = io_error.into();
    /// assert!(matches!(goud_error, GoudError::ResourceNotFound(_)));
    /// ```
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => GoudError::ResourceNotFound(error.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                GoudError::ResourceLoadFailed(format!("Permission denied: {error}"))
            }
            _ => GoudError::ResourceLoadFailed(error.to_string()),
        }
    }
}

impl From<String> for GoudError {
    /// Converts a string into a `GoudError::InternalError`.
    ///
    /// This is a convenience conversion for creating internal errors from strings.
    /// Use more specific error variants when the error category is known.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error: GoudError = "something went wrong".to_string().into();
    /// assert!(matches!(error, GoudError::InternalError(_)));
    /// ```
    fn from(msg: String) -> Self {
        GoudError::InternalError(msg)
    }
}

impl From<&str> for GoudError {
    /// Converts a string slice into a `GoudError::InternalError`.
    ///
    /// This is a convenience conversion for creating internal errors from string literals.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error: GoudError = "something went wrong".into();
    /// assert!(matches!(error, GoudError::InternalError(_)));
    /// ```
    fn from(msg: &str) -> Self {
        GoudError::InternalError(msg.to_string())
    }
}

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
// FFI Error Bridge
// =============================================================================
//
// This section provides the infrastructure for passing errors across the FFI
// boundary. It uses thread-local storage to store the last error, which can
// then be retrieved by language bindings (C#, Python, etc.) after a function
// call returns an error code.
//
// ## Usage Pattern
//
// 1. Rust function encounters an error
// 2. Rust function calls `set_last_error(error)`
// 3. Rust function returns error code via `GoudFFIResult`
// 4. Language binding checks if `success` is false
// 5. Language binding calls `goud_last_error_code()` and `goud_last_error_message()`
// 6. Language binding calls `take_last_error()` to clear the error
//
// ## Thread Safety
//
// Each thread has its own error storage. Errors do not cross thread boundaries.
// This matches the behavior of `errno` in C and is safe for multi-threaded use.

use std::cell::RefCell;

thread_local! {
    /// Thread-local storage for the last error.
    ///
    /// Each thread has its own error storage, ensuring that errors from one
    /// thread do not affect another. This is critical for thread-safe FFI.
    static LAST_ERROR: RefCell<Option<GoudError>> = const { RefCell::new(None) };
}

/// Sets the last error for the current thread.
///
/// This function stores the error in thread-local storage where it can be
/// retrieved by `last_error_code()` and `last_error_message()`.
///
/// # Thread Safety
///
/// The error is stored in thread-local storage and will not affect other threads.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, last_error_code, ERR_NOT_INITIALIZED};
///
/// set_last_error(GoudError::NotInitialized);
/// assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);
/// ```
pub fn set_last_error(error: GoudError) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
}

/// Takes the last error from the current thread, clearing it.
///
/// This function removes the error from thread-local storage and returns it.
/// Subsequent calls will return `None` until a new error is set.
///
/// # Thread Safety
///
/// Only affects the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, take_last_error};
///
/// set_last_error(GoudError::NotInitialized);
/// let error = take_last_error();
/// assert!(error.is_some());
/// assert!(take_last_error().is_none()); // Cleared after take
/// ```
pub fn take_last_error() -> Option<GoudError> {
    LAST_ERROR.with(|e| e.borrow_mut().take())
}

/// Gets the last error from the current thread without clearing it.
///
/// This function clones the error from thread-local storage. Use `take_last_error()`
/// if you want to clear the error after retrieval.
///
/// # Thread Safety
///
/// Only accesses the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, get_last_error};
///
/// set_last_error(GoudError::NotInitialized);
/// let error1 = get_last_error();
/// let error2 = get_last_error();
/// assert_eq!(error1, error2); // Error not cleared
/// ```
pub fn get_last_error() -> Option<GoudError> {
    LAST_ERROR.with(|e| e.borrow().clone())
}

/// Returns the error code of the last error for the current thread.
///
/// Returns `SUCCESS` (0) if no error is set. This does not clear the error.
///
/// # Thread Safety
///
/// Only accesses the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{set_last_error, last_error_code, clear_last_error, GoudError, SUCCESS, ERR_NOT_INITIALIZED};
///
/// clear_last_error();
/// assert_eq!(last_error_code(), SUCCESS);
///
/// set_last_error(GoudError::NotInitialized);
/// assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);
/// ```
pub fn last_error_code() -> GoudErrorCode {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|err| err.error_code())
            .unwrap_or(SUCCESS)
    })
}

/// Returns the error message of the last error for the current thread.
///
/// Returns `None` if no error is set. This does not clear the error.
/// The returned string is a copy, safe to use across FFI.
///
/// # Thread Safety
///
/// Only accesses the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, last_error_message};
///
/// set_last_error(GoudError::InitializationFailed("GPU not found".to_string()));
/// let msg = last_error_message();
/// assert_eq!(msg, Some("GPU not found".to_string()));
/// ```
pub fn last_error_message() -> Option<String> {
    LAST_ERROR.with(|e| e.borrow().as_ref().map(|err| err.message().to_string()))
}

/// Clears the last error for the current thread.
///
/// After calling this, `last_error_code()` will return `SUCCESS` and
/// `last_error_message()` will return `None`.
///
/// # Thread Safety
///
/// Only affects the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, clear_last_error, last_error_code, SUCCESS};
///
/// set_last_error(GoudError::NotInitialized);
/// clear_last_error();
/// assert_eq!(last_error_code(), SUCCESS);
/// ```
pub fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

// =============================================================================
// FFI Result Type
// =============================================================================

/// FFI-safe result type for returning success/failure status across the FFI boundary.
///
/// This struct is designed to be passed by value across FFI and provides both
/// a boolean success flag and the error code for detailed error handling.
///
/// # Memory Layout
///
/// Uses `#[repr(C)]` for predictable memory layout across language boundaries.
/// The struct is 8 bytes (4 bytes for code, 4 bytes for success with padding).
///
/// # Usage
///
/// ```
/// use goud_engine::core::error::{GoudFFIResult, GoudError, SUCCESS};
///
/// // Success case
/// let result = GoudFFIResult::success();
/// assert!(result.success);
/// assert_eq!(result.code, SUCCESS);
///
/// // Error case
/// let result = GoudFFIResult::from_error(GoudError::NotInitialized);
/// assert!(!result.success);
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoudFFIResult {
    /// The error code. `SUCCESS` (0) on success, error code on failure.
    pub code: GoudErrorCode,
    /// True if the operation succeeded, false otherwise.
    pub success: bool,
}

impl GoudFFIResult {
    /// Creates a successful result.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, SUCCESS};
    ///
    /// let result = GoudFFIResult::success();
    /// assert!(result.success);
    /// assert_eq!(result.code, SUCCESS);
    /// ```
    #[inline]
    pub const fn success() -> Self {
        Self {
            code: SUCCESS,
            success: true,
        }
    }

    /// Creates a result from an error code.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, ERR_NOT_INITIALIZED};
    ///
    /// let result = GoudFFIResult::from_code(ERR_NOT_INITIALIZED);
    /// assert!(!result.success);
    /// assert_eq!(result.code, ERR_NOT_INITIALIZED);
    /// ```
    #[inline]
    pub const fn from_code(code: GoudErrorCode) -> Self {
        Self {
            code,
            success: code == SUCCESS,
        }
    }

    /// Creates a result from a `GoudError`.
    ///
    /// This also sets the thread-local last error for message retrieval.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, GoudError, ERR_NOT_INITIALIZED, last_error_message};
    ///
    /// let result = GoudFFIResult::from_error(GoudError::NotInitialized);
    /// assert!(!result.success);
    /// assert_eq!(result.code, ERR_NOT_INITIALIZED);
    /// ```
    #[inline]
    pub fn from_error(error: GoudError) -> Self {
        let code = error.error_code();
        set_last_error(error);
        Self {
            code,
            success: false,
        }
    }

    /// Creates a result from a `GoudResult<T>`.
    ///
    /// On success, clears any previous error. On error, sets the last error.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, GoudResult, GoudError, SUCCESS, ERR_NOT_INITIALIZED};
    ///
    /// let ok_result: GoudResult<i32> = Ok(42);
    /// let ffi_result = GoudFFIResult::from_result(ok_result);
    /// assert!(ffi_result.success);
    ///
    /// let err_result: GoudResult<i32> = Err(GoudError::NotInitialized);
    /// let ffi_result = GoudFFIResult::from_result(err_result);
    /// assert!(!ffi_result.success);
    /// assert_eq!(ffi_result.code, ERR_NOT_INITIALIZED);
    /// ```
    #[inline]
    pub fn from_result<T>(result: GoudResult<T>) -> Self {
        match result {
            Ok(_) => {
                clear_last_error();
                Self::success()
            }
            Err(error) => Self::from_error(error),
        }
    }

    /// Returns true if the result indicates success.
    #[inline]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the result indicates failure.
    #[inline]
    pub const fn is_error(&self) -> bool {
        !self.success
    }
}

impl Default for GoudFFIResult {
    /// Default is success.
    fn default() -> Self {
        Self::success()
    }
}

impl From<GoudError> for GoudFFIResult {
    fn from(error: GoudError) -> Self {
        Self::from_error(error)
    }
}

impl<T> From<GoudResult<T>> for GoudFFIResult {
    fn from(result: GoudResult<T>) -> Self {
        Self::from_result(result)
    }
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
                    "Error {error:?} should be in Context category"
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
                    "Context error {error:?} has code {code} which is outside range 1-99"
                );
            }
        }

        #[test]
        fn test_goud_error_derives() {
            // Test Debug
            let error = GoudError::NotInitialized;
            let debug_str = format!("{error:?}");
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
                    "Error {error:?} should be in Resource category"
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
                    "Resource error {error:?} has code {code} which is outside range 100-199"
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
            let debug_str = format!("{error:?}");
            assert!(debug_str.contains("ResourceNotFound"));
            assert!(debug_str.contains("test.png"));

            let handle_error = GoudError::InvalidHandle;
            let debug_str = format!("{handle_error:?}");
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

    // =========================================================================
    // GoudError Graphics Variant Tests
    // =========================================================================

    mod graphics_errors {
        use super::*;

        #[test]
        fn test_shader_compilation_failed_error_code() {
            let error = GoudError::ShaderCompilationFailed(
                "ERROR: 0:15: 'vec3' : undeclared identifier".to_string(),
            );
            assert_eq!(error.error_code(), ERR_SHADER_COMPILATION_FAILED);
            assert_eq!(error.error_code(), 200);
        }

        #[test]
        fn test_shader_link_failed_error_code() {
            let error = GoudError::ShaderLinkFailed(
                "ERROR: Varying variable 'vTexCoord' not written".to_string(),
            );
            assert_eq!(error.error_code(), ERR_SHADER_LINK_FAILED);
            assert_eq!(error.error_code(), 201);
        }

        #[test]
        fn test_texture_creation_failed_error_code() {
            let error =
                GoudError::TextureCreationFailed("GL_OUT_OF_MEMORY: 4096x4096 RGBA8".to_string());
            assert_eq!(error.error_code(), ERR_TEXTURE_CREATION_FAILED);
            assert_eq!(error.error_code(), 210);
        }

        #[test]
        fn test_buffer_creation_failed_error_code() {
            let error = GoudError::BufferCreationFailed(
                "Failed to allocate 256MB vertex buffer".to_string(),
            );
            assert_eq!(error.error_code(), ERR_BUFFER_CREATION_FAILED);
            assert_eq!(error.error_code(), 211);
        }

        #[test]
        fn test_render_target_failed_error_code() {
            let error =
                GoudError::RenderTargetFailed("GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT".to_string());
            assert_eq!(error.error_code(), ERR_RENDER_TARGET_FAILED);
            assert_eq!(error.error_code(), 220);
        }

        #[test]
        fn test_backend_not_supported_error_code() {
            let error =
                GoudError::BackendNotSupported("Vulkan 1.2 required, found 1.0".to_string());
            assert_eq!(error.error_code(), ERR_BACKEND_NOT_SUPPORTED);
            assert_eq!(error.error_code(), 230);
        }

        #[test]
        fn test_draw_call_failed_error_code() {
            let error = GoudError::DrawCallFailed(
                "glDrawElements failed: GL_INVALID_OPERATION".to_string(),
            );
            assert_eq!(error.error_code(), ERR_DRAW_CALL_FAILED);
            assert_eq!(error.error_code(), 240);
        }

        #[test]
        fn test_all_graphics_errors_in_graphics_category() {
            let errors: Vec<GoudError> = vec![
                GoudError::ShaderCompilationFailed("test".to_string()),
                GoudError::ShaderLinkFailed("test".to_string()),
                GoudError::TextureCreationFailed("test".to_string()),
                GoudError::BufferCreationFailed("test".to_string()),
                GoudError::RenderTargetFailed("test".to_string()),
                GoudError::BackendNotSupported("test".to_string()),
                GoudError::DrawCallFailed("test".to_string()),
            ];

            for error in errors {
                assert_eq!(
                    error.category(),
                    "Graphics",
                    "Error {error:?} should be in Graphics category"
                );
            }
        }

        #[test]
        fn test_graphics_error_codes_in_valid_range() {
            let errors: Vec<GoudError> = vec![
                GoudError::ShaderCompilationFailed("test".to_string()),
                GoudError::ShaderLinkFailed("test".to_string()),
                GoudError::TextureCreationFailed("test".to_string()),
                GoudError::BufferCreationFailed("test".to_string()),
                GoudError::RenderTargetFailed("test".to_string()),
                GoudError::BackendNotSupported("test".to_string()),
                GoudError::DrawCallFailed("test".to_string()),
            ];

            for error in errors {
                let code = error.error_code();
                assert!(
                    code >= 200 && code < 300,
                    "Graphics error {error:?} has code {code} which is outside range 200-299"
                );
            }
        }

        #[test]
        fn test_graphics_errors_preserve_message() {
            // Test ShaderCompilationFailed with line number info
            let shader_err = "ERROR: 0:42: 'sampler2D' : syntax error";
            if let GoudError::ShaderCompilationFailed(msg) =
                GoudError::ShaderCompilationFailed(shader_err.to_string())
            {
                assert_eq!(msg, shader_err);
                assert!(msg.contains("42")); // Verify line number preserved
            } else {
                panic!("Expected ShaderCompilationFailed variant");
            }

            // Test BackendNotSupported with version info
            let backend_err = "Metal not available on Linux; available: OpenGL 4.5, Vulkan 1.2";
            if let GoudError::BackendNotSupported(msg) =
                GoudError::BackendNotSupported(backend_err.to_string())
            {
                assert_eq!(msg, backend_err);
            } else {
                panic!("Expected BackendNotSupported variant");
            }
        }

        #[test]
        fn test_graphics_error_equality() {
            // Same variant, same message
            let err1 = GoudError::ShaderCompilationFailed("error line 10".to_string());
            let err2 = GoudError::ShaderCompilationFailed("error line 10".to_string());
            assert_eq!(err1, err2);

            // Same variant, different message
            let err3 = GoudError::ShaderCompilationFailed("error line 20".to_string());
            assert_ne!(err1, err3);

            // Different variants with similar messages
            let err4 = GoudError::ShaderLinkFailed("error line 10".to_string());
            assert_ne!(err1, err4);

            // Different graphics error types
            assert_ne!(
                GoudError::TextureCreationFailed("fail".to_string()),
                GoudError::BufferCreationFailed("fail".to_string())
            );
        }

        #[test]
        fn test_graphics_error_debug_format() {
            let error = GoudError::ShaderCompilationFailed("syntax error at line 5".to_string());
            let debug_str = format!("{error:?}");
            assert!(debug_str.contains("ShaderCompilationFailed"));
            assert!(debug_str.contains("syntax error at line 5"));

            let error2 = GoudError::DrawCallFailed("invalid state".to_string());
            let debug_str2 = format!("{error2:?}");
            assert!(debug_str2.contains("DrawCallFailed"));
            assert!(debug_str2.contains("invalid state"));
        }

        #[test]
        fn test_graphics_error_codes_are_distinct() {
            // Verify all graphics error codes are unique
            let codes = vec![
                ERR_SHADER_COMPILATION_FAILED,
                ERR_SHADER_LINK_FAILED,
                ERR_TEXTURE_CREATION_FAILED,
                ERR_BUFFER_CREATION_FAILED,
                ERR_RENDER_TARGET_FAILED,
                ERR_BACKEND_NOT_SUPPORTED,
                ERR_DRAW_CALL_FAILED,
            ];

            // Check all codes are distinct
            for (i, code1) in codes.iter().enumerate() {
                for (j, code2) in codes.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            code1, code2,
                            "Error codes at index {i} and {j} should be different"
                        );
                    }
                }
            }
        }

        #[test]
        fn test_graphics_error_code_gaps_for_future_expansion() {
            // Verify there are gaps between error codes for future additions
            // Shader errors: 200-209
            assert!(ERR_SHADER_COMPILATION_FAILED == 200);
            assert!(ERR_SHADER_LINK_FAILED == 201);

            // Texture/buffer errors: 210-219
            assert!(ERR_TEXTURE_CREATION_FAILED == 210);
            assert!(ERR_BUFFER_CREATION_FAILED == 211);

            // Render target errors: 220-229
            assert!(ERR_RENDER_TARGET_FAILED == 220);

            // Backend errors: 230-239
            assert!(ERR_BACKEND_NOT_SUPPORTED == 230);

            // Draw call errors: 240-249
            assert!(ERR_DRAW_CALL_FAILED == 240);
        }
    }

    // =========================================================================
    // GoudError Entity Variant Tests
    // =========================================================================

    mod entity_errors {
        use super::*;

        #[test]
        fn test_entity_not_found_error_code() {
            let error = GoudError::EntityNotFound;
            assert_eq!(error.error_code(), ERR_ENTITY_NOT_FOUND);
            assert_eq!(error.error_code(), 300);
        }

        #[test]
        fn test_entity_already_exists_error_code() {
            let error = GoudError::EntityAlreadyExists;
            assert_eq!(error.error_code(), ERR_ENTITY_ALREADY_EXISTS);
            assert_eq!(error.error_code(), 301);
        }

        #[test]
        fn test_component_not_found_error_code() {
            let error = GoudError::ComponentNotFound;
            assert_eq!(error.error_code(), ERR_COMPONENT_NOT_FOUND);
            assert_eq!(error.error_code(), 310);
        }

        #[test]
        fn test_component_already_exists_error_code() {
            let error = GoudError::ComponentAlreadyExists;
            assert_eq!(error.error_code(), ERR_COMPONENT_ALREADY_EXISTS);
            assert_eq!(error.error_code(), 311);
        }

        #[test]
        fn test_query_failed_error_code() {
            let error = GoudError::QueryFailed("conflicting access on Position".to_string());
            assert_eq!(error.error_code(), ERR_QUERY_FAILED);
            assert_eq!(error.error_code(), 320);
        }

        #[test]
        fn test_all_entity_errors_in_entity_category() {
            let errors: Vec<GoudError> = vec![
                GoudError::EntityNotFound,
                GoudError::EntityAlreadyExists,
                GoudError::ComponentNotFound,
                GoudError::ComponentAlreadyExists,
                GoudError::QueryFailed("test".to_string()),
            ];

            for error in errors {
                assert_eq!(
                    error.category(),
                    "Entity",
                    "Error {error:?} should be in Entity category"
                );
            }
        }

        #[test]
        fn test_entity_error_codes_in_valid_range() {
            let errors: Vec<GoudError> = vec![
                GoudError::EntityNotFound,
                GoudError::EntityAlreadyExists,
                GoudError::ComponentNotFound,
                GoudError::ComponentAlreadyExists,
                GoudError::QueryFailed("test".to_string()),
            ];

            for error in errors {
                let code = error.error_code();
                assert!(
                    code >= 300 && code < 400,
                    "Entity error {error:?} has code {code} which is outside range 300-399"
                );
            }
        }

        #[test]
        fn test_query_failed_preserves_message() {
            let query_err = "Conflicting access: &mut Position and &Position on same entity";
            if let GoudError::QueryFailed(msg) = GoudError::QueryFailed(query_err.to_string()) {
                assert_eq!(msg, query_err);
            } else {
                panic!("Expected QueryFailed variant");
            }
        }

        #[test]
        fn test_entity_error_equality() {
            // Unit variants are equal
            assert_eq!(GoudError::EntityNotFound, GoudError::EntityNotFound);
            assert_eq!(GoudError::ComponentNotFound, GoudError::ComponentNotFound);

            // Different unit variants are not equal
            assert_ne!(GoudError::EntityNotFound, GoudError::EntityAlreadyExists);
            assert_ne!(
                GoudError::ComponentNotFound,
                GoudError::ComponentAlreadyExists
            );

            // QueryFailed with same message
            let err1 = GoudError::QueryFailed("error".to_string());
            let err2 = GoudError::QueryFailed("error".to_string());
            assert_eq!(err1, err2);

            // QueryFailed with different message
            let err3 = GoudError::QueryFailed("different".to_string());
            assert_ne!(err1, err3);
        }

        #[test]
        fn test_entity_error_debug_format() {
            let error = GoudError::EntityNotFound;
            let debug_str = format!("{error:?}");
            assert!(debug_str.contains("EntityNotFound"));

            let error2 = GoudError::QueryFailed("access conflict".to_string());
            let debug_str2 = format!("{error2:?}");
            assert!(debug_str2.contains("QueryFailed"));
            assert!(debug_str2.contains("access conflict"));
        }

        #[test]
        fn test_entity_error_codes_are_distinct() {
            let codes = vec![
                ERR_ENTITY_NOT_FOUND,
                ERR_ENTITY_ALREADY_EXISTS,
                ERR_COMPONENT_NOT_FOUND,
                ERR_COMPONENT_ALREADY_EXISTS,
                ERR_QUERY_FAILED,
            ];

            for (i, code1) in codes.iter().enumerate() {
                for (j, code2) in codes.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            code1, code2,
                            "Error codes at index {i} and {j} should be different"
                        );
                    }
                }
            }
        }

        #[test]
        fn test_entity_error_code_gaps_for_future_expansion() {
            // Entity errors: 300-309
            assert!(ERR_ENTITY_NOT_FOUND == 300);
            assert!(ERR_ENTITY_ALREADY_EXISTS == 301);

            // Component errors: 310-319
            assert!(ERR_COMPONENT_NOT_FOUND == 310);
            assert!(ERR_COMPONENT_ALREADY_EXISTS == 311);

            // Query errors: 320-329
            assert!(ERR_QUERY_FAILED == 320);
        }
    }

    // =========================================================================
    // GoudError System Variant Tests
    // =========================================================================

    mod system_errors {
        use super::*;

        #[test]
        fn test_window_creation_failed_error_code() {
            let error = GoudError::WindowCreationFailed("No display server found".to_string());
            assert_eq!(error.error_code(), ERR_WINDOW_CREATION_FAILED);
            assert_eq!(error.error_code(), 500);
        }

        #[test]
        fn test_audio_init_failed_error_code() {
            let error = GoudError::AudioInitFailed("No audio devices found".to_string());
            assert_eq!(error.error_code(), ERR_AUDIO_INIT_FAILED);
            assert_eq!(error.error_code(), 510);
        }

        #[test]
        fn test_physics_init_failed_error_code() {
            let error = GoudError::PhysicsInitFailed("Invalid gravity configuration".to_string());
            assert_eq!(error.error_code(), ERR_PHYSICS_INIT_FAILED);
            assert_eq!(error.error_code(), 520);
        }

        #[test]
        fn test_platform_error_error_code() {
            let error =
                GoudError::PlatformError("macOS: Failed to acquire Metal device".to_string());
            assert_eq!(error.error_code(), ERR_PLATFORM_ERROR);
            assert_eq!(error.error_code(), 530);
        }

        #[test]
        fn test_all_system_errors_in_system_category() {
            let errors: Vec<GoudError> = vec![
                GoudError::WindowCreationFailed("test".to_string()),
                GoudError::AudioInitFailed("test".to_string()),
                GoudError::PhysicsInitFailed("test".to_string()),
                GoudError::PlatformError("test".to_string()),
            ];

            for error in errors {
                assert_eq!(
                    error.category(),
                    "System",
                    "Error {error:?} should be in System category"
                );
            }
        }

        #[test]
        fn test_system_error_codes_in_valid_range() {
            let errors: Vec<GoudError> = vec![
                GoudError::WindowCreationFailed("test".to_string()),
                GoudError::AudioInitFailed("test".to_string()),
                GoudError::PhysicsInitFailed("test".to_string()),
                GoudError::PlatformError("test".to_string()),
            ];

            for error in errors {
                let code = error.error_code();
                assert!(
                    code >= 500 && code < 600,
                    "System error {error:?} has code {code} which is outside range 500-599"
                );
            }
        }

        #[test]
        fn test_system_errors_preserve_message() {
            // Test WindowCreationFailed
            let window_err = "Failed to create GLFW window: 800x600";
            if let GoudError::WindowCreationFailed(msg) =
                GoudError::WindowCreationFailed(window_err.to_string())
            {
                assert_eq!(msg, window_err);
            } else {
                panic!("Expected WindowCreationFailed variant");
            }

            // Test AudioInitFailed
            let audio_err = "ALSA: Unable to open default audio device";
            if let GoudError::AudioInitFailed(msg) =
                GoudError::AudioInitFailed(audio_err.to_string())
            {
                assert_eq!(msg, audio_err);
            } else {
                panic!("Expected AudioInitFailed variant");
            }

            // Test PhysicsInitFailed
            let physics_err = "Box2D: Invalid world bounds";
            if let GoudError::PhysicsInitFailed(msg) =
                GoudError::PhysicsInitFailed(physics_err.to_string())
            {
                assert_eq!(msg, physics_err);
            } else {
                panic!("Expected PhysicsInitFailed variant");
            }

            // Test PlatformError
            let platform_err = "Linux: X11 display connection failed";
            if let GoudError::PlatformError(msg) =
                GoudError::PlatformError(platform_err.to_string())
            {
                assert_eq!(msg, platform_err);
            } else {
                panic!("Expected PlatformError variant");
            }
        }

        #[test]
        fn test_system_error_equality() {
            // Same variant, same message
            let err1 = GoudError::WindowCreationFailed("error".to_string());
            let err2 = GoudError::WindowCreationFailed("error".to_string());
            assert_eq!(err1, err2);

            // Same variant, different message
            let err3 = GoudError::WindowCreationFailed("different".to_string());
            assert_ne!(err1, err3);

            // Different variants
            let err4 = GoudError::AudioInitFailed("error".to_string());
            assert_ne!(err1, err4);
        }

        #[test]
        fn test_system_error_debug_format() {
            let error = GoudError::WindowCreationFailed("GLFW error 65543".to_string());
            let debug_str = format!("{error:?}");
            assert!(debug_str.contains("WindowCreationFailed"));
            assert!(debug_str.contains("GLFW error 65543"));

            let error2 = GoudError::PlatformError("Win32 error".to_string());
            let debug_str2 = format!("{error2:?}");
            assert!(debug_str2.contains("PlatformError"));
            assert!(debug_str2.contains("Win32 error"));
        }

        #[test]
        fn test_system_error_codes_are_distinct() {
            let codes = vec![
                ERR_WINDOW_CREATION_FAILED,
                ERR_AUDIO_INIT_FAILED,
                ERR_PHYSICS_INIT_FAILED,
                ERR_PLATFORM_ERROR,
            ];

            for (i, code1) in codes.iter().enumerate() {
                for (j, code2) in codes.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            code1, code2,
                            "Error codes at index {i} and {j} should be different"
                        );
                    }
                }
            }
        }

        #[test]
        fn test_system_error_code_gaps_for_future_expansion() {
            // Window errors: 500-509
            assert!(ERR_WINDOW_CREATION_FAILED == 500);

            // Audio errors: 510-519
            assert!(ERR_AUDIO_INIT_FAILED == 510);

            // Physics errors: 520-529
            assert!(ERR_PHYSICS_INIT_FAILED == 520);

            // Platform errors: 530-539
            assert!(ERR_PLATFORM_ERROR == 530);
        }
    }

    // =========================================================================
    // GoudError Internal Variant Tests
    // =========================================================================

    mod internal_errors {
        use super::*;

        #[test]
        fn test_internal_error_error_code() {
            let error =
                GoudError::InternalError("Unexpected null pointer in render queue".to_string());
            assert_eq!(error.error_code(), ERR_INTERNAL_ERROR);
            assert_eq!(error.error_code(), 900);
        }

        #[test]
        fn test_not_implemented_error_code() {
            let error = GoudError::NotImplemented("Vulkan backend".to_string());
            assert_eq!(error.error_code(), ERR_NOT_IMPLEMENTED);
            assert_eq!(error.error_code(), 901);
        }

        #[test]
        fn test_invalid_state_error_code() {
            let error = GoudError::InvalidState("Renderer called after shutdown".to_string());
            assert_eq!(error.error_code(), ERR_INVALID_STATE);
            assert_eq!(error.error_code(), 902);
        }

        #[test]
        fn test_all_internal_errors_in_internal_category() {
            let errors: Vec<GoudError> = vec![
                GoudError::InternalError("test".to_string()),
                GoudError::NotImplemented("test".to_string()),
                GoudError::InvalidState("test".to_string()),
            ];

            for error in errors {
                assert_eq!(
                    error.category(),
                    "Internal",
                    "Error {error:?} should be in Internal category"
                );
            }
        }

        #[test]
        fn test_internal_error_codes_in_valid_range() {
            let errors: Vec<GoudError> = vec![
                GoudError::InternalError("test".to_string()),
                GoudError::NotImplemented("test".to_string()),
                GoudError::InvalidState("test".to_string()),
            ];

            for error in errors {
                let code = error.error_code();
                assert!(
                    code >= 900 && code < 1000,
                    "Internal error {error:?} has code {code} which is outside range 900-999"
                );
            }
        }

        #[test]
        fn test_internal_errors_preserve_message() {
            // Test InternalError
            let internal_err = "FATAL: Inconsistent component storage state";
            if let GoudError::InternalError(msg) =
                GoudError::InternalError(internal_err.to_string())
            {
                assert_eq!(msg, internal_err);
            } else {
                panic!("Expected InternalError variant");
            }

            // Test NotImplemented
            let not_impl_err = "Feature 'ray tracing' is not yet implemented";
            if let GoudError::NotImplemented(msg) =
                GoudError::NotImplemented(not_impl_err.to_string())
            {
                assert_eq!(msg, not_impl_err);
            } else {
                panic!("Expected NotImplemented variant");
            }

            // Test InvalidState
            let invalid_state_err = "Cannot add components while iterating";
            if let GoudError::InvalidState(msg) =
                GoudError::InvalidState(invalid_state_err.to_string())
            {
                assert_eq!(msg, invalid_state_err);
            } else {
                panic!("Expected InvalidState variant");
            }
        }

        #[test]
        fn test_internal_error_equality() {
            // Same variant, same message
            let err1 = GoudError::InternalError("bug".to_string());
            let err2 = GoudError::InternalError("bug".to_string());
            assert_eq!(err1, err2);

            // Same variant, different message
            let err3 = GoudError::InternalError("different bug".to_string());
            assert_ne!(err1, err3);

            // Different variants
            let err4 = GoudError::NotImplemented("bug".to_string());
            assert_ne!(err1, err4);

            let err5 = GoudError::InvalidState("bug".to_string());
            assert_ne!(err1, err5);
            assert_ne!(err4, err5);
        }

        #[test]
        fn test_internal_error_debug_format() {
            let error = GoudError::InternalError("assertion failed".to_string());
            let debug_str = format!("{error:?}");
            assert!(debug_str.contains("InternalError"));
            assert!(debug_str.contains("assertion failed"));

            let error2 = GoudError::NotImplemented("3D audio".to_string());
            let debug_str2 = format!("{error2:?}");
            assert!(debug_str2.contains("NotImplemented"));
            assert!(debug_str2.contains("3D audio"));

            let error3 = GoudError::InvalidState("already running".to_string());
            let debug_str3 = format!("{error3:?}");
            assert!(debug_str3.contains("InvalidState"));
            assert!(debug_str3.contains("already running"));
        }

        #[test]
        fn test_internal_error_codes_are_distinct() {
            let codes = vec![ERR_INTERNAL_ERROR, ERR_NOT_IMPLEMENTED, ERR_INVALID_STATE];

            for (i, code1) in codes.iter().enumerate() {
                for (j, code2) in codes.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            code1, code2,
                            "Error codes at index {i} and {j} should be different"
                        );
                    }
                }
            }
        }

        #[test]
        fn test_internal_error_code_ordering() {
            // Internal errors should be consecutive starting at 900
            assert_eq!(ERR_INTERNAL_ERROR, 900);
            assert_eq!(ERR_NOT_IMPLEMENTED, 901);
            assert_eq!(ERR_INVALID_STATE, 902);
        }
    }

    // =========================================================================
    // Trait and Conversion Tests
    // =========================================================================

    mod traits {
        use super::*;
        use std::error::Error;

        #[test]
        fn test_display_format_context_errors() {
            let error = GoudError::NotInitialized;
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-1] Context: Engine has not been initialized");

            let error = GoudError::InitializationFailed("GPU not found".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-10] Context: GPU not found");
        }

        #[test]
        fn test_display_format_resource_errors() {
            let error = GoudError::ResourceNotFound("textures/player.png".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-100] Resource: textures/player.png");

            let error = GoudError::InvalidHandle;
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-110] Resource: Invalid handle");
        }

        #[test]
        fn test_display_format_graphics_errors() {
            let error = GoudError::ShaderCompilationFailed("syntax error at line 42".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-200] Graphics: syntax error at line 42");
        }

        #[test]
        fn test_display_format_entity_errors() {
            let error = GoudError::EntityNotFound;
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-300] Entity: Entity not found");

            let error = GoudError::QueryFailed("conflicting access".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-320] Entity: conflicting access");
        }

        #[test]
        fn test_display_format_system_errors() {
            let error = GoudError::WindowCreationFailed("no display".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-500] System: no display");
        }

        #[test]
        fn test_display_format_internal_errors() {
            let error = GoudError::InternalError("unexpected state".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-900] Internal: unexpected state");

            let error = GoudError::NotImplemented("feature X".to_string());
            let display = format!("{error}");
            assert_eq!(display, "[GOUD-901] Internal: feature X");
        }

        #[test]
        fn test_error_trait_implementation() {
            let error = GoudError::NotInitialized;

            // Verify Error trait is implemented
            let error_ref: &dyn Error = &error;
            assert!(error_ref.source().is_none());

            // Verify Display works through Error trait
            let display = format!("{error_ref}");
            assert!(display.contains("GOUD-1"));
        }

        #[test]
        fn test_message_method() {
            // Unit variants return default messages
            assert_eq!(
                GoudError::NotInitialized.message(),
                "Engine has not been initialized"
            );
            assert_eq!(GoudError::InvalidHandle.message(), "Invalid handle");
            assert_eq!(GoudError::EntityNotFound.message(), "Entity not found");

            // Variants with messages return the message
            let error = GoudError::InitializationFailed("custom message".to_string());
            assert_eq!(error.message(), "custom message");

            let error = GoudError::ResourceNotFound("path/to/file".to_string());
            assert_eq!(error.message(), "path/to/file");
        }

        #[test]
        fn test_from_io_error_not_found() {
            let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
            let goud_error: GoudError = io_error.into();

            assert!(matches!(goud_error, GoudError::ResourceNotFound(_)));
            assert_eq!(goud_error.error_code(), ERR_RESOURCE_NOT_FOUND);
        }

        #[test]
        fn test_from_io_error_permission_denied() {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
            let goud_error: GoudError = io_error.into();

            assert!(matches!(goud_error, GoudError::ResourceLoadFailed(_)));
            assert_eq!(goud_error.error_code(), ERR_RESOURCE_LOAD_FAILED);
            assert!(goud_error.message().contains("Permission denied"));
        }

        #[test]
        fn test_from_io_error_other() {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "network error");
            let goud_error: GoudError = io_error.into();

            assert!(matches!(goud_error, GoudError::ResourceLoadFailed(_)));
            assert_eq!(goud_error.error_code(), ERR_RESOURCE_LOAD_FAILED);
        }

        #[test]
        fn test_from_string() {
            let msg = "something went wrong".to_string();
            let error: GoudError = msg.into();

            assert!(matches!(error, GoudError::InternalError(_)));
            assert_eq!(error.message(), "something went wrong");
            assert_eq!(error.error_code(), ERR_INTERNAL_ERROR);
        }

        #[test]
        fn test_from_str() {
            let error: GoudError = "oops".into();

            assert!(matches!(error, GoudError::InternalError(_)));
            assert_eq!(error.message(), "oops");
            assert_eq!(error.error_code(), ERR_INTERNAL_ERROR);
        }

        #[test]
        fn test_goud_result_ok() {
            fn might_fail() -> GoudResult<i32> {
                Ok(42)
            }

            let result = might_fail();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }

        #[test]
        fn test_goud_result_err() {
            fn always_fails() -> GoudResult<i32> {
                Err(GoudError::NotInitialized)
            }

            let result = always_fails();
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error, GoudError::NotInitialized);
        }

        #[test]
        fn test_goud_result_with_question_mark() {
            fn inner() -> GoudResult<i32> {
                Err(GoudError::ResourceNotFound("missing.txt".to_string()))
            }

            fn outer() -> GoudResult<i32> {
                let value = inner()?;
                Ok(value + 1)
            }

            let result = outer();
            assert!(result.is_err());
            if let Err(GoudError::ResourceNotFound(msg)) = result {
                assert_eq!(msg, "missing.txt");
            } else {
                panic!("Expected ResourceNotFound error");
            }
        }

        #[test]
        fn test_error_can_be_boxed() {
            // Verify GoudError can be used as Box<dyn Error>
            let error: Box<dyn Error> = Box::new(GoudError::NotInitialized);
            let display = format!("{error}");
            assert!(display.contains("GOUD-1"));
        }

        #[test]
        fn test_display_versus_debug() {
            let error = GoudError::InitializationFailed("test message".to_string());

            let display = format!("{error}");
            let debug = format!("{error:?}");

            // Display is user-friendly with format
            assert!(display.contains("[GOUD-10]"));
            assert!(display.contains("Context"));
            assert!(display.contains("test message"));

            // Debug is Rust-style
            assert!(debug.contains("InitializationFailed"));
            assert!(debug.contains("test message"));

            // They should be different
            assert_ne!(display, debug);
        }
    }

    // =========================================================================
    // FFI Error Bridge Tests
    // =========================================================================

    mod ffi {
        use super::*;
        use std::sync::{Arc, Barrier};
        use std::thread;

        /// Helper to ensure clean state for each test
        fn with_clean_error_state<F, R>(f: F) -> R
        where
            F: FnOnce() -> R,
        {
            clear_last_error();
            let result = f();
            clear_last_error();
            result
        }

        #[test]
        fn test_set_and_get_last_error() {
            with_clean_error_state(|| {
                set_last_error(GoudError::NotInitialized);
                let error = get_last_error();
                assert!(error.is_some());
                assert_eq!(error.unwrap(), GoudError::NotInitialized);
            });
        }

        #[test]
        fn test_get_does_not_clear_error() {
            with_clean_error_state(|| {
                set_last_error(GoudError::NotInitialized);

                // Call get_last_error multiple times
                let error1 = get_last_error();
                let error2 = get_last_error();
                let error3 = get_last_error();

                // All should return the same error
                assert_eq!(error1, error2);
                assert_eq!(error2, error3);
                assert!(error1.is_some());
            });
        }

        #[test]
        fn test_take_clears_error() {
            with_clean_error_state(|| {
                set_last_error(GoudError::NotInitialized);

                // First take should return the error
                let error1 = take_last_error();
                assert!(error1.is_some());
                assert_eq!(error1.unwrap(), GoudError::NotInitialized);

                // Subsequent takes should return None
                let error2 = take_last_error();
                assert!(error2.is_none());

                let error3 = take_last_error();
                assert!(error3.is_none());
            });
        }

        #[test]
        fn test_last_error_code_no_error() {
            with_clean_error_state(|| {
                // No error set, should return SUCCESS
                assert_eq!(last_error_code(), SUCCESS);
            });
        }

        #[test]
        fn test_last_error_code_with_error() {
            with_clean_error_state(|| {
                set_last_error(GoudError::NotInitialized);
                assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

                set_last_error(GoudError::ResourceNotFound("test".to_string()));
                assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);

                set_last_error(GoudError::ShaderCompilationFailed("error".to_string()));
                assert_eq!(last_error_code(), ERR_SHADER_COMPILATION_FAILED);
            });
        }

        #[test]
        fn test_last_error_message_no_error() {
            with_clean_error_state(|| {
                // No error set, should return None
                assert!(last_error_message().is_none());
            });
        }

        #[test]
        fn test_last_error_message_with_error() {
            with_clean_error_state(|| {
                // Error without custom message
                set_last_error(GoudError::NotInitialized);
                let msg = last_error_message();
                assert!(msg.is_some());
                assert_eq!(msg.unwrap(), "Engine has not been initialized");

                // Error with custom message
                set_last_error(GoudError::InitializationFailed("GPU not found".to_string()));
                let msg = last_error_message();
                assert!(msg.is_some());
                assert_eq!(msg.unwrap(), "GPU not found");
            });
        }

        #[test]
        fn test_clear_last_error() {
            with_clean_error_state(|| {
                set_last_error(GoudError::NotInitialized);
                assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

                clear_last_error();

                assert_eq!(last_error_code(), SUCCESS);
                assert!(last_error_message().is_none());
                assert!(get_last_error().is_none());
            });
        }

        #[test]
        fn test_overwrite_error() {
            with_clean_error_state(|| {
                // Set first error
                set_last_error(GoudError::NotInitialized);
                assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

                // Overwrite with second error
                set_last_error(GoudError::ResourceNotFound("file.txt".to_string()));
                assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);

                // First error should be gone
                let error = take_last_error();
                assert!(matches!(error, Some(GoudError::ResourceNotFound(_))));
            });
        }

        #[test]
        fn test_thread_isolation() {
            // This test verifies that errors set in one thread don't affect other threads
            let barrier = Arc::new(Barrier::new(2));
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                // Clear any existing error in this thread
                clear_last_error();

                // Wait for main thread to set its error
                barrier_clone.wait();

                // This thread should have no error (thread-local storage)
                assert_eq!(
                    last_error_code(),
                    SUCCESS,
                    "Thread should have no error from main thread"
                );

                // Set a different error in this thread
                set_last_error(GoudError::ResourceNotFound("thread_file.txt".to_string()));
                assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);

                // Wait for main thread to check its error
                barrier_clone.wait();
            });

            with_clean_error_state(|| {
                // Set error in main thread
                set_last_error(GoudError::NotInitialized);
                assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

                // Signal spawned thread to check
                barrier.wait();

                // Wait for spawned thread to set its error
                barrier.wait();

                // Main thread should still have its original error (unaffected by spawned thread)
                assert_eq!(
                    last_error_code(),
                    ERR_NOT_INITIALIZED,
                    "Main thread error should not be affected by spawned thread"
                );
            });

            handle.join().unwrap();
        }

        #[test]
        fn test_multiple_threads_independent_errors() {
            use std::sync::atomic::{AtomicUsize, Ordering};

            const THREAD_COUNT: usize = 4;
            let success_count = Arc::new(AtomicUsize::new(0));

            let handles: Vec<_> = (0..THREAD_COUNT)
                .map(|i| {
                    let success_count = Arc::clone(&success_count);
                    thread::spawn(move || {
                        clear_last_error();

                        // Each thread sets a different error
                        let error = match i {
                            0 => GoudError::NotInitialized,
                            1 => GoudError::AlreadyInitialized,
                            2 => GoudError::InvalidContext,
                            _ => GoudError::ContextDestroyed,
                        };
                        let expected_code = error.error_code();
                        set_last_error(error);

                        // Verify own error
                        if last_error_code() == expected_code {
                            success_count.fetch_add(1, Ordering::SeqCst);
                        }

                        clear_last_error();
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            assert_eq!(
                success_count.load(Ordering::SeqCst),
                THREAD_COUNT,
                "All threads should have set and verified their own errors"
            );
        }

        // =====================================================================
        // GoudFFIResult Tests
        // =====================================================================

        #[test]
        fn test_ffi_result_success() {
            let result = GoudFFIResult::success();
            assert!(result.success);
            assert!(result.is_success());
            assert!(!result.is_error());
            assert_eq!(result.code, SUCCESS);
        }

        #[test]
        fn test_ffi_result_from_code_success() {
            let result = GoudFFIResult::from_code(SUCCESS);
            assert!(result.success);
            assert_eq!(result.code, SUCCESS);
        }

        #[test]
        fn test_ffi_result_from_code_error() {
            let result = GoudFFIResult::from_code(ERR_NOT_INITIALIZED);
            assert!(!result.success);
            assert!(result.is_error());
            assert!(!result.is_success());
            assert_eq!(result.code, ERR_NOT_INITIALIZED);
        }

        #[test]
        fn test_ffi_result_from_error() {
            with_clean_error_state(|| {
                let result = GoudFFIResult::from_error(GoudError::NotInitialized);
                assert!(!result.success);
                assert_eq!(result.code, ERR_NOT_INITIALIZED);

                // Should have set last error
                assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);
            });
        }

        #[test]
        fn test_ffi_result_from_error_with_message() {
            with_clean_error_state(|| {
                let error = GoudError::InitializationFailed("Custom error message".to_string());
                let result = GoudFFIResult::from_error(error);

                assert!(!result.success);
                assert_eq!(result.code, ERR_INITIALIZATION_FAILED);

                // Message should be retrievable
                let msg = last_error_message();
                assert_eq!(msg, Some("Custom error message".to_string()));
            });
        }

        #[test]
        fn test_ffi_result_from_result_ok() {
            with_clean_error_state(|| {
                // Set an error first to verify it gets cleared
                set_last_error(GoudError::NotInitialized);

                let result: GoudResult<i32> = Ok(42);
                let ffi_result = GoudFFIResult::from_result(result);

                assert!(ffi_result.success);
                assert_eq!(ffi_result.code, SUCCESS);

                // Error should be cleared
                assert_eq!(last_error_code(), SUCCESS);
            });
        }

        #[test]
        fn test_ffi_result_from_result_err() {
            with_clean_error_state(|| {
                let result: GoudResult<i32> =
                    Err(GoudError::ResourceNotFound("test.png".to_string()));
                let ffi_result = GoudFFIResult::from_result(result);

                assert!(!ffi_result.success);
                assert_eq!(ffi_result.code, ERR_RESOURCE_NOT_FOUND);

                // Error should be set
                assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);
                assert_eq!(last_error_message(), Some("test.png".to_string()));
            });
        }

        #[test]
        fn test_ffi_result_default() {
            let result = GoudFFIResult::default();
            assert!(result.success);
            assert_eq!(result.code, SUCCESS);
        }

        #[test]
        fn test_ffi_result_from_goud_error() {
            with_clean_error_state(|| {
                let ffi_result: GoudFFIResult = GoudError::EntityNotFound.into();
                assert!(!ffi_result.success);
                assert_eq!(ffi_result.code, ERR_ENTITY_NOT_FOUND);
            });
        }

        #[test]
        fn test_ffi_result_from_goud_result() {
            with_clean_error_state(|| {
                let ok: GoudResult<String> = Ok("hello".to_string());
                let ffi_result: GoudFFIResult = ok.into();
                assert!(ffi_result.success);

                let err: GoudResult<String> = Err(GoudError::InvalidHandle);
                let ffi_result: GoudFFIResult = err.into();
                assert!(!ffi_result.success);
                assert_eq!(ffi_result.code, ERR_INVALID_HANDLE);
            });
        }

        #[test]
        fn test_ffi_result_derive_traits() {
            // Test Clone
            let result1 = GoudFFIResult::from_code(ERR_NOT_INITIALIZED);
            let result2 = result1;
            assert_eq!(result1, result2);

            // Test Copy (implicit in the above)

            // Test Debug
            let debug_str = format!("{result1:?}");
            assert!(debug_str.contains("GoudFFIResult"));
            assert!(debug_str.contains("code"));
            assert!(debug_str.contains("success"));

            // Test PartialEq and Eq
            assert_eq!(GoudFFIResult::success(), GoudFFIResult::success());
            assert_ne!(
                GoudFFIResult::success(),
                GoudFFIResult::from_code(ERR_NOT_INITIALIZED)
            );
        }

        #[test]
        fn test_ffi_result_repr_c() {
            // Verify size and alignment are reasonable for FFI
            use std::mem::{align_of, size_of};

            // GoudFFIResult should have predictable size
            // i32 (4 bytes) + bool (1 byte) + padding = typically 8 bytes on most platforms
            let size = size_of::<GoudFFIResult>();
            assert!(size >= 5, "GoudFFIResult should be at least 5 bytes");
            assert!(size <= 8, "GoudFFIResult should be at most 8 bytes");

            // Alignment should be at least 4 bytes (for i32)
            let align = align_of::<GoudFFIResult>();
            assert!(
                align >= 4,
                "GoudFFIResult should have at least 4-byte alignment"
            );
        }

        #[test]
        fn test_ffi_workflow_simulation() {
            // Simulate a typical FFI workflow:
            // 1. Rust function is called
            // 2. Function returns error result
            // 3. C# retrieves error code and message
            // 4. C# clears the error

            with_clean_error_state(|| {
                // Simulate Rust function that fails
                fn rust_ffi_function() -> GoudFFIResult {
                    let result: GoudResult<()> = Err(GoudError::ShaderCompilationFailed(
                        "ERROR: 0:15: 'vec3' : undeclared identifier".to_string(),
                    ));
                    GoudFFIResult::from_result(result)
                }

                // Call the function
                let ffi_result = rust_ffi_function();

                // C# side would check result
                assert!(!ffi_result.success);
                assert_eq!(ffi_result.code, ERR_SHADER_COMPILATION_FAILED);

                // C# side would get detailed error info
                let code = last_error_code();
                let message = last_error_message();

                assert_eq!(code, ERR_SHADER_COMPILATION_FAILED);
                assert!(message.is_some());
                assert!(message.unwrap().contains("vec3"));

                // C# side would clear the error
                clear_last_error();

                // Verify clean state
                assert_eq!(last_error_code(), SUCCESS);
                assert!(last_error_message().is_none());
            });
        }

        #[test]
        fn test_ffi_success_workflow_simulation() {
            // Simulate a successful FFI call

            with_clean_error_state(|| {
                // Simulate Rust function that succeeds
                fn rust_ffi_function() -> GoudFFIResult {
                    let result: GoudResult<()> = Ok(());
                    GoudFFIResult::from_result(result)
                }

                // Call the function
                let ffi_result = rust_ffi_function();

                // C# side checks result
                assert!(ffi_result.success);
                assert_eq!(ffi_result.code, SUCCESS);

                // No error should be set
                assert_eq!(last_error_code(), SUCCESS);
                assert!(last_error_message().is_none());
            });
        }
    }
}
