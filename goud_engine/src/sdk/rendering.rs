//! # SDK Rendering API
//!
//! Provides methods on [`GoudGame`] for 2D rendering operations
//! including frame management, immediate-mode sprite/quad drawing, and render
//! state control.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop platform with OpenGL).

pub(crate) mod immediate;

use super::GoudGame;
use crate::core::error::{GoudError, GoudResult};

pub use immediate::ImmediateRenderState;

// Re-export rendering types for SDK users
pub use crate::rendering::sprite_batch::SpriteBatchConfig;

// Re-export 3D types (they live in rendering_3d but users expect them here)
pub use crate::libs::graphics::renderer3d::{
    FogConfig, GridConfig, GridRenderMode, Light, LightType, PrimitiveCreateInfo, PrimitiveType,
    SkyboxConfig,
};

// =============================================================================
// 2D Rendering -- ECS-based SpriteBatch (not FFI-generated)
// =============================================================================

impl GoudGame {
    /// Begins a 2D rendering pass.
    ///
    /// Call this before drawing sprites. Must be paired with
    /// [`end_2d_render`](Self::end_2d_render).
    pub fn begin_2d_render(&mut self) -> GoudResult<()> {
        match &mut self.sprite_batch {
            Some(batch) => {
                batch.begin();
                Ok(())
            }
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Ends the 2D rendering pass and submits batched draw calls to the GPU.
    pub fn end_2d_render(&mut self) -> GoudResult<()> {
        match &mut self.sprite_batch {
            Some(batch) => batch.end(),
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Draws all entities with Sprite + Transform2D components.
    pub fn draw_sprites(&mut self) -> GoudResult<()> {
        let asset_server = self
            .asset_server
            .as_ref()
            .ok_or(GoudError::NotInitialized)?;
        let default = self.scene_manager.default_scene();
        let world = self
            .scene_manager
            .get_scene(default)
            .ok_or(GoudError::NotInitialized)?;
        match &mut self.sprite_batch {
            Some(batch) => batch.draw_sprites(world, asset_server),
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Returns 2D rendering statistics: `(sprite_count, batch_count, batch_ratio)`.
    #[inline]
    pub fn render_2d_stats(&self) -> (usize, usize, f32) {
        match &self.sprite_batch {
            Some(batch) => batch.stats(),
            None => (0, 0, 0.0),
        }
    }

    /// Returns `true` if a 2D renderer (SpriteBatch) is initialized.
    #[inline]
    pub fn has_2d_renderer(&self) -> bool {
        self.sprite_batch.is_some()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::GameConfig;

    #[test]
    fn test_begin_2d_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(game.begin_2d_render().is_err());
    }

    #[test]
    fn test_end_2d_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(game.end_2d_render().is_err());
    }

    #[test]
    fn test_draw_sprites_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(game.draw_sprites().is_err());
    }

    #[test]
    fn test_render_2d_stats_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert_eq!(game.render_2d_stats(), (0, 0, 0.0));
    }

    #[test]
    fn test_has_2d_renderer_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.has_2d_renderer());
    }

    #[test]
    fn test_draw_sprite_headless_returns_false() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.draw_sprite(0, 0.0, 0.0, 10.0, 10.0, 0.0, 1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_draw_quad_headless_returns_false() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.draw_quad(0.0, 0.0, 10.0, 10.0, 1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_begin_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.begin_render());
    }

    #[test]
    fn test_end_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.end_render());
    }

    #[test]
    fn test_ortho_matrix_identity_like() {
        let m = immediate::ortho_matrix(0.0, 2.0, 0.0, 2.0);
        assert!((m[0] - 1.0).abs() < 0.001);
        assert!((m[5] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_model_matrix_no_rotation() {
        let m = immediate::model_matrix(10.0, 20.0, 5.0, 5.0, 0.0);
        assert!((m[12] - 10.0).abs() < 0.001);
        assert!((m[13] - 20.0).abs() < 0.001);
    }
}
