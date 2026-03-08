//! Frame management sub-trait for `RenderBackend`.

use crate::libs::error::GoudResult;

/// Frame lifecycle operations.
///
/// Called once per frame to bracket all rendering work.
/// Backends may use these hooks for command buffer recording (Vulkan/Metal)
/// or simply as no-ops (OpenGL).
pub trait FrameOps {
    /// Begins a new frame. Called once per frame before any rendering.
    ///
    /// This may perform backend-specific setup like resetting state or
    /// beginning command recording (Vulkan, Metal).
    fn begin_frame(&mut self) -> GoudResult<()>;

    /// Ends the current frame. Called once per frame after all rendering.
    ///
    /// This may submit command buffers or perform cleanup.
    fn end_frame(&mut self) -> GoudResult<()>;
}
