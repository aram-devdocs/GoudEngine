//! ECS rendering system for text.
//!
//! Wraps a [`TextBatch`] in a system interface that mirrors
//! [`SpriteRenderSystem`](crate::rendering::SpriteRenderSystem).

use crate::assets::AssetServer;
use crate::ecs::World;
use crate::libs::graphics::backend::render_backend::RenderBackend;

use super::text_batch::{TextBatch, TextRenderStats};

/// System for rendering text entities using batched rendering.
///
/// Each frame the system:
/// 1. Queries entities with `Text` + `Transform2D` components.
/// 2. Lays out glyphs via the text layout engine.
/// 3. Batches glyph quads by atlas texture.
/// 4. Submits draw calls to the GPU backend.
pub struct TextRenderSystem {
    text_batch: TextBatch,
}

impl TextRenderSystem {
    /// Creates a new text render system.
    pub fn new() -> Self {
        Self {
            text_batch: TextBatch::new(),
        }
    }

    /// Runs the text rendering pipeline for a single frame.
    ///
    /// # Errors
    ///
    /// Returns an error if font loading, atlas generation, or GPU
    /// operations fail.
    pub fn run(
        &mut self,
        world: &World,
        asset_server: &AssetServer,
        backend: &mut dyn RenderBackend,
    ) -> Result<(), String> {
        self.text_batch.begin();
        self.text_batch.draw_text(world, asset_server, backend)?;
        self.text_batch.end(backend)?;
        Ok(())
    }

    /// Returns rendering statistics from the last frame.
    pub fn stats(&self) -> TextRenderStats {
        self.text_batch.stats()
    }

    /// Returns a reference to the underlying text batch.
    pub fn text_batch(&self) -> &TextBatch {
        &self.text_batch
    }
}

impl Default for TextRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TextRenderSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextRenderSystem")
            .field("text_batch", &"TextBatch { ... }")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_render_system_new() {
        let system = TextRenderSystem::new();
        let stats = system.stats();
        assert_eq!(stats.glyph_count, 0);
        assert_eq!(stats.draw_calls, 0);
    }

    #[test]
    fn test_text_render_system_default() {
        let system = TextRenderSystem::default();
        assert_eq!(system.stats().glyph_count, 0);
    }

    #[test]
    fn test_text_render_system_debug() {
        let system = TextRenderSystem::new();
        let debug = format!("{:?}", system);
        assert!(debug.contains("TextRenderSystem"));
    }
}
