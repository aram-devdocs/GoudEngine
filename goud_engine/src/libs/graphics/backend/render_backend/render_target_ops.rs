//! Render-target operations sub-trait for `RenderBackend`.

use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::{RenderTargetDesc, RenderTargetHandle, TextureHandle};

/// Offscreen render-target management operations.
pub trait RenderTargetOps {
    /// Creates an offscreen render target.
    fn create_render_target(&mut self, _desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        Err(GoudError::BackendNotSupported(
            "Render targets are not supported by this backend".to_string(),
        ))
    }

    /// Destroys a render target and any attachments it owns.
    fn destroy_render_target(&mut self, _handle: RenderTargetHandle) -> bool {
        false
    }

    /// Checks whether a render-target handle is still valid.
    fn is_render_target_valid(&self, _handle: RenderTargetHandle) -> bool {
        false
    }

    /// Binds a render target for subsequent draw calls, or `None` for the
    /// default framebuffer.
    fn bind_render_target(&mut self, _handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        Err(GoudError::BackendNotSupported(
            "Render targets are not supported by this backend".to_string(),
        ))
    }

    /// Returns the color texture attached to a render target.
    fn render_target_texture(&self, _handle: RenderTargetHandle) -> Option<TextureHandle> {
        None
    }
}
