//! Texture integration bridge for the 3D renderer.

/// Trait for texture manager integration.
///
/// This is a legacy bridge: texture binding still happens through the caller's
/// GL-backed implementation. A future refactor should route textures through
/// [`crate::libs::graphics::backend::RenderBackend`] for full backend portability.
pub trait TextureManagerTrait {
    /// Bind a texture to a slot
    fn bind_texture(&self, texture_id: u32, slot: u32);
}
