//! `RenderBackend` trait definition, composed from focused sub-traits.
//!
//! Each sub-trait covers a specific area of GPU operations:
//! - [`FrameOps`] -- frame lifecycle (begin/end)
//! - [`ClearOps`] -- clear color and buffer clearing
//! - [`StateOps`] -- viewport, depth, blending, culling state
//! - [`BufferOps`] -- GPU buffer create/update/destroy/bind
//! - [`TextureOps`] -- GPU texture create/update/destroy/bind
//! - [`ShaderOps`] -- shader compile/link/destroy/bind and uniforms
//! - [`DrawOps`] -- vertex attribute setup and draw calls
//!
//! The composed [`RenderBackend`] supertrait requires all sub-traits
//! plus `Send + Sync`, providing the full rendering API contract.

mod buffer_ops;
mod clear_ops;
mod draw_ops;
mod frame_ops;
mod render_target_ops;
mod shader_ops;
mod state_ops;
mod texture_ops;

pub use buffer_ops::BufferOps;
pub use clear_ops::ClearOps;
pub use draw_ops::DrawOps;
pub use frame_ops::FrameOps;
pub use render_target_ops::RenderTargetOps;
pub use shader_ops::ShaderOps;
pub use state_ops::StateOps;
pub use texture_ops::TextureOps;

use super::capabilities::{BackendCapabilities, BackendInfo};

/// Main render backend trait abstracting graphics operations.
///
/// This trait provides a platform-agnostic interface for rendering operations,
/// allowing the engine to support multiple graphics APIs without changing
/// higher-level rendering code.
///
/// Composed from focused sub-traits:
/// [`FrameOps`], [`ClearOps`], [`StateOps`], [`BufferOps`],
/// [`TextureOps`], [`RenderTargetOps`], [`ShaderOps`], [`DrawOps`].
///
/// # Safety
///
/// Implementations must ensure:
/// - All GPU handles remain valid for their lifetime
/// - Operations on destroyed handles return errors gracefully
/// - Thread safety is maintained per API requirements
///
/// # Object Safety
///
/// This trait is intentionally NOT object-safe to allow for:
/// - Associated types for handle wrappers
/// - Generic methods for efficient implementations
/// - Zero-cost abstractions where possible
pub trait RenderBackend:
    FrameOps
    + ClearOps
    + StateOps
    + BufferOps
    + TextureOps
    + RenderTargetOps
    + ShaderOps
    + DrawOps
    + Send
    + Sync
{
    /// Returns information about this backend implementation.
    fn info(&self) -> &BackendInfo;

    /// Rebinds the backend's default vertex array state when the underlying API
    /// requires one for vertex attribute setup and indexed draws.
    fn bind_default_vertex_array(&mut self) {}

    /// Validates backend-specific state required for native text drawing.
    ///
    /// Backends that do not need extra validation, including headless test
    /// backends, can keep the default no-op implementation.
    fn validate_text_draw_state(&self) -> Result<(), String> {
        Ok(())
    }

    /// Reads the default framebuffer into RGBA8 bytes.
    ///
    /// Returned pixels are expected to be in row-major order with a top-left
    /// origin. Backends that do not support readback may keep the default
    /// implementation.
    fn read_default_framebuffer_rgba8(
        &mut self,
        _width: u32,
        _height: u32,
    ) -> Result<Vec<u8>, String> {
        Err("default framebuffer readback is not supported by this backend".to_string())
    }

    /// Returns the capabilities of this backend.
    fn capabilities(&self) -> &BackendCapabilities {
        &self.info().capabilities
    }
}

/// Marker trait bridging the "render provider" naming to the existing
/// [`RenderBackend`] trait. Any type implementing `RenderBackend` is
/// automatically a `RenderProvider`.
pub trait RenderProvider: RenderBackend {}
impl<T: RenderBackend> RenderProvider for T {}

#[cfg(test)]
mod tests {
    /// Compile-time verification that RenderProvider is automatically
    /// satisfied by any RenderBackend implementor.
    fn _assert_render_provider_blanket<T: super::RenderBackend>() {
        fn _requires_provider<U: super::RenderProvider>() {}
        _requires_provider::<T>();
    }

    /// When the `legacy-glfw-opengl` feature is enabled, verify at compile
    /// time that `OpenGLBackend` satisfies `RenderBackend` (and therefore
    /// `RenderProvider` via the blanket impl).
    #[cfg(feature = "legacy-glfw-opengl")]
    #[test]
    fn opengl_backend_implements_render_backend() {
        fn _assert_impl<T: super::RenderBackend + super::RenderProvider>() {}
        _assert_impl::<crate::libs::graphics::backend::opengl::OpenGLBackend>();
    }
}
