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
mod shader_ops;
mod state_ops;
mod texture_ops;

pub use buffer_ops::BufferOps;
pub use clear_ops::ClearOps;
pub use draw_ops::DrawOps;
pub use frame_ops::FrameOps;
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
/// [`TextureOps`], [`ShaderOps`], [`DrawOps`].
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
    FrameOps + ClearOps + StateOps + BufferOps + TextureOps + ShaderOps + DrawOps + Send + Sync
{
    /// Returns information about this backend implementation.
    fn info(&self) -> &BackendInfo;

    /// Returns the capabilities of this backend.
    fn capabilities(&self) -> &BackendCapabilities {
        &self.info().capabilities
    }
}
