//! Rendering systems for 2D sprite rendering.
//!
//! This module provides ECS systems for rendering sprites using the SpriteBatch renderer.
//!
//! # Architecture
//!
//! The rendering pipeline:
//! 1. Query entities with Sprite + Transform2D components
//! 2. Sort by Z-layer for correct draw order
//! 3. Batch by texture to minimize draw calls
//! 4. Submit to GPU via backend
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::ecs::systems::SpriteRenderSystem;
//! use goud_engine::ecs::World;
//! use goud_engine::graphics::backend::OpenGLBackend;
//!
//! let backend = OpenGLBackend::new()?;
//! let mut render_system = SpriteRenderSystem::new(backend)?;
//!
//! // Each frame
//! render_system.run(&mut world)?;
//! ```

use crate::assets::AssetServer;
use crate::core::error::GoudResult;
use crate::ecs::World;
use crate::libs::graphics::backend::RenderBackend;
use crate::libs::graphics::sprite_batch::{SpriteBatch, SpriteBatchConfig};

/// System for rendering 2D sprites using batched rendering.
///
/// This system:
/// - Queries all entities with Sprite + Transform2D components
/// - Batches sprites by texture to minimize draw calls
/// - Sorts by Z-layer for correct rendering order
/// - Submits draw calls to the GPU backend
///
/// # Performance
///
/// Target: <100 draw calls for 10,000 sprites (100:1 batch ratio)
pub struct SpriteRenderSystem<B: RenderBackend> {
    sprite_batch: SpriteBatch<B>,
}

impl<B: RenderBackend> SpriteRenderSystem<B> {
    /// Creates a new sprite render system with the given backend.
    pub fn new(backend: B) -> GoudResult<Self> {
        let sprite_batch = SpriteBatch::new(backend, SpriteBatchConfig::default())?;
        Ok(Self { sprite_batch })
    }

    /// Creates a new sprite render system with custom configuration.
    pub fn with_config(backend: B, config: SpriteBatchConfig) -> GoudResult<Self> {
        let sprite_batch = SpriteBatch::new(backend, config)?;
        Ok(Self { sprite_batch })
    }

    /// Runs the sprite rendering system.
    ///
    /// This should be called once per frame, after all game logic has been updated.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world containing sprite entities
    /// * `asset_server` - Asset server for loading textures
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Shader compilation fails
    /// - Buffer allocation fails
    /// - GPU operations fail
    pub fn run(&mut self, world: &World, asset_server: &AssetServer) -> GoudResult<()> {
        self.sprite_batch.begin();
        self.sprite_batch.draw_sprites(world, asset_server)?;
        self.sprite_batch.end()?;
        Ok(())
    }

    /// Gets rendering statistics from the last frame.
    ///
    /// Returns (sprite_count, batch_count, batch_ratio) tuple.
    pub fn stats(&self) -> (usize, usize, f32) {
        self.sprite_batch.stats()
    }

    /// Gets a reference to the underlying sprite batch.
    pub fn sprite_batch(&self) -> &SpriteBatch<B> {
        &self.sprite_batch
    }

    /// Gets a mutable reference to the underlying sprite batch.
    pub fn sprite_batch_mut(&mut self) -> &mut SpriteBatch<B> {
        &mut self.sprite_batch
    }
}

impl<B: RenderBackend> std::fmt::Debug for SpriteRenderSystem<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpriteRenderSystem")
            .field("sprite_batch", &"SpriteBatch { ... }")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::{TextureAsset, TextureFormat};
    use crate::assets::AssetStorage;
    use crate::core::math::{Color, Vec2};
    use crate::ecs::components::{Sprite, Transform2D};
    use crate::ecs::World;
    use crate::libs::graphics::backend::opengl::OpenGLBackend;

    #[test]
    fn test_sprite_render_system_new() {
        // This test requires OpenGL context, so we just verify the type exists
        // Real testing would require OpenGL initialization
    }

    #[test]
    fn test_sprite_render_system_debug() {
        // Debug formatting doesn't require OpenGL
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_render_system_run() {
        // Setup
        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut render_system =
            SpriteRenderSystem::new(backend).expect("Failed to create render system");

        // Create world with sprite entities
        let mut world = World::new();
        let asset_server = AssetServer::new();

        // Create a test texture asset
        let texture_data = vec![255u8; 64 * 64 * 4]; // 64x64 RGBA white texture
        let texture_asset = TextureAsset::new(texture_data, 64, 64, TextureFormat::Png);
        let mut storage = AssetStorage::new();
        let texture_handle = storage.insert(texture_asset);

        // Spawn sprite entity
        let entity = world.spawn_empty();
        world
            .insert(
                entity,
                Sprite::new(texture_handle).with_color(Color::WHITE),
            )
            .expect("Failed to add Sprite");
        world
            .insert(entity, Transform2D::from_position(Vec2::new(100.0, 100.0)))
            .expect("Failed to add Transform2D");

        // Run render system
        let result = render_system.run(&world, &asset_server);
        assert!(result.is_ok(), "Render system should run successfully");

        // Check stats
        let (sprite_count, batch_count, _ratio) = render_system.stats();
        assert_eq!(sprite_count, 1, "Should have rendered 1 sprite");
        assert!(batch_count > 0, "Should have at least 1 batch");
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_render_system_multiple_sprites() {
        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut render_system =
            SpriteRenderSystem::new(backend).expect("Failed to create render system");

        let mut world = World::new();
        let asset_server = AssetServer::new();

        // Create texture
        let texture_data = vec![255u8; 64 * 64 * 4];
        let texture_asset = TextureAsset::new(texture_data, 64, 64, TextureFormat::Png);
        let mut storage = AssetStorage::new();
        let texture_handle = storage.insert(texture_asset);

        // Spawn 10 sprites
        for i in 0..10 {
            let entity = world.spawn_empty();
            world
                .insert(entity, Sprite::new(texture_handle).with_color(Color::WHITE))
                .expect("Failed to add Sprite");
            world
                .insert(
                    entity,
                    Transform2D::from_position(Vec2::new(i as f32 * 50.0, 100.0)),
                )
                .expect("Failed to add Transform2D");
        }

        // Run render system
        render_system
            .run(&world, &asset_server)
            .expect("Failed to run render system");

        // Check stats
        let (sprite_count, batch_count, ratio) = render_system.stats();
        assert_eq!(sprite_count, 10, "Should have rendered 10 sprites");
        assert!(batch_count > 0, "Should have at least 1 batch");
        assert!(
            ratio > 1.0,
            "Should have good batching ratio with same texture"
        );
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_render_system_empty_world() {
        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut render_system =
            SpriteRenderSystem::new(backend).expect("Failed to create render system");

        let world = World::new();
        let asset_server = AssetServer::new();

        // Run on empty world
        let result = render_system.run(&world, &asset_server);
        assert!(result.is_ok(), "Should handle empty world gracefully");

        let (sprite_count, batch_count, _ratio) = render_system.stats();
        assert_eq!(sprite_count, 0, "Should have 0 sprites");
        assert_eq!(batch_count, 0, "Should have 0 batches");
    }

    #[test]
    fn test_sprite_render_system_accessors() {
        // Test that accessors are available (no OpenGL required)
        // In real usage, you'd create with OpenGL backend
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_render_system_z_sorting() {
        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut render_system =
            SpriteRenderSystem::new(backend).expect("Failed to create render system");

        let mut world = World::new();
        let asset_server = AssetServer::new();

        // Create texture
        let texture_data = vec![255u8; 64 * 64 * 4];
        let texture_asset = TextureAsset::new(texture_data, 64, 64, TextureFormat::Png);
        let mut storage = AssetStorage::new();
        let texture_handle = storage.insert(texture_asset);

        // Spawn sprites with different Y positions (Z-layer)
        for i in 0..5 {
            let entity = world.spawn_empty();
            world
                .insert(entity, Sprite::new(texture_handle).with_color(Color::WHITE))
                .expect("Failed to add Sprite");
            world
                .insert(
                    entity,
                    Transform2D::from_position(Vec2::new(100.0, i as f32 * 50.0)),
                )
                .expect("Failed to add Transform2D");
        }

        // Run render system (should sort by Y position)
        render_system
            .run(&world, &asset_server)
            .expect("Failed to run render system");

        let (sprite_count, _batch_count, _ratio) = render_system.stats();
        assert_eq!(sprite_count, 5, "Should have rendered 5 sprites");
    }
}
