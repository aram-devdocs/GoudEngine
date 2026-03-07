//! Error context propagation for GoudEngine.
//!
//! Provides subsystem and operation metadata that can be attached to errors
//! stored in thread-local storage, queryable across FFI.

/// Metadata describing where an error originated.
///
/// Stores the subsystem (e.g., graphics, ECS) and specific operation
/// (e.g., "shader_compile", "entity_spawn") that produced an error.
/// Designed for FFI consumption alongside `GoudError`.
#[derive(Clone, Debug, Default)]
pub struct GoudErrorContext {
    /// The engine subsystem that produced the error (e.g., "graphics").
    pub subsystem: &'static str,
    /// The specific operation that failed (e.g., "shader_compile").
    pub operation: &'static str,
}

impl GoudErrorContext {
    /// Creates a new error context with the given subsystem and operation.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudErrorContext;
    /// use goud_engine::core::error::context::subsystems;
    ///
    /// let ctx = GoudErrorContext::new(subsystems::GRAPHICS, "shader_compile");
    /// assert_eq!(ctx.subsystem, "graphics");
    /// assert_eq!(ctx.operation, "shader_compile");
    /// ```
    pub fn new(subsystem: &'static str, operation: &'static str) -> Self {
        Self {
            subsystem,
            operation,
        }
    }
}

/// Well-known subsystem constants for error context.
pub mod subsystems {
    /// Graphics/rendering subsystem.
    pub const GRAPHICS: &str = "graphics";
    /// Entity Component System subsystem.
    pub const ECS: &str = "ecs";
    /// Audio subsystem.
    pub const AUDIO: &str = "audio";
    /// Platform/windowing subsystem.
    pub const PLATFORM: &str = "platform";
    /// Resource/asset management subsystem.
    pub const RESOURCE: &str = "resource";
    /// Provider subsystem.
    pub const PROVIDER: &str = "provider";
    /// Input handling subsystem.
    pub const INPUT: &str = "input";
    /// Internal/unexpected errors.
    pub const INTERNAL: &str = "internal";
}
