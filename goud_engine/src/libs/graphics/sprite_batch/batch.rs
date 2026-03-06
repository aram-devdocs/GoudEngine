//! Core `SpriteBatch` struct and frame lifecycle implementation.

use super::config::SpriteBatchConfig;
use super::types::{SpriteBatchEntry, SpriteInstance, SpriteVertex};
use crate::assets::{loaders::TextureAsset, AssetHandle, AssetServer};
use crate::core::error::GoudResult;
use crate::core::math::{Rect, Vec2};
use crate::ecs::components::{Sprite, Transform2D};
use crate::ecs::query::Query;
use crate::ecs::{Entity, World};
use crate::libs::graphics::backend::types::{BufferHandle, ShaderHandle, TextureHandle};
use crate::libs::graphics::backend::RenderBackend;
use std::collections::HashMap;

/// High-performance sprite batch renderer.
///
/// The SpriteBatch collects sprites, sorts them by texture and Z-layer,
/// then renders them in efficient batches to minimize draw calls.
///
/// # Lifecycle
///
/// ```rust,ignore
/// batch.begin();           // Start a new frame
/// batch.draw_sprites(&world, &asset_server);  // Gather and batch sprites
/// batch.end();             // Flush remaining batches
/// ```
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
pub struct SpriteBatch<B: RenderBackend> {
    /// Graphics backend
    pub(super) backend: B,
    /// Configuration
    pub(super) config: SpriteBatchConfig,
    /// Vertex buffer handle
    pub(super) vertex_buffer: Option<BufferHandle>,
    /// Index buffer handle (shared quad indices)
    pub(super) index_buffer: Option<BufferHandle>,
    /// Current vertex buffer capacity (number of sprites)
    pub(super) vertex_capacity: usize,
    /// Sprite shader program
    pub(super) shader: Option<ShaderHandle>,
    /// Collected sprite instances this frame
    pub(super) sprites: Vec<SpriteInstance>,
    /// CPU-side vertex buffer
    pub(super) vertices: Vec<SpriteVertex>,
    /// Prepared batches for rendering
    pub(super) batches: Vec<SpriteBatchEntry>,
    /// Texture handle cache (AssetHandle -> GPU TextureHandle)
    pub(super) texture_cache: HashMap<AssetHandle<TextureAsset>, TextureHandle>,
    /// Current frame number (for debugging)
    pub(super) frame_count: u64,
}

#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
impl<B: RenderBackend> SpriteBatch<B> {
    // =========================================================================
    // Lifecycle
    // =========================================================================

    /// Creates a new sprite batch renderer.
    pub fn new(backend: B, config: SpriteBatchConfig) -> GoudResult<Self> {
        Ok(Self {
            backend,
            config,
            vertex_buffer: None,
            index_buffer: None,
            vertex_capacity: 0,
            shader: None,
            sprites: Vec::with_capacity(config.initial_capacity),
            vertices: Vec::with_capacity(config.initial_capacity * 4),
            batches: Vec::with_capacity(128),
            texture_cache: HashMap::new(),
            frame_count: 0,
        })
    }

    /// Begins a new frame of sprite rendering.
    ///
    /// This clears the sprite collection from the previous frame.
    pub fn begin(&mut self) {
        self.sprites.clear();
        self.vertices.clear();
        self.batches.clear();
        self.frame_count += 1;
    }

    /// Ends the current frame and flushes remaining batches.
    pub fn end(&mut self) -> GoudResult<()> {
        // Flush is automatic in draw_sprites, but we could add
        // final state cleanup here if needed
        Ok(())
    }

    // =========================================================================
    // Drawing
    // =========================================================================

    /// Draws all sprites from the world.
    ///
    /// This performs the gather-sort-batch-render pipeline:
    /// 1. Queries all entities with Sprite + Transform2D
    /// 2. Sorts by Z-layer and texture
    /// 3. Generates batches
    /// 4. Submits draw calls
    pub fn draw_sprites(&mut self, world: &World, asset_server: &AssetServer) -> GoudResult<()> {
        self.gather_sprites(world)?;

        if self.config.enable_z_sorting {
            self.sort_sprites();
        }

        self.generate_batches(asset_server)?;
        self.render_batches()?;

        Ok(())
    }

    // =========================================================================
    // Internal: Gather
    // =========================================================================

    /// Gathers all sprites from the world into the internal sprite list.
    pub(super) fn gather_sprites(&mut self, world: &World) -> GoudResult<()> {
        self.sprites.clear();

        let query: Query<(Entity, &Sprite, &Transform2D)> = Query::new(world);

        for (entity, sprite, transform) in query.iter(world) {
            let matrix = transform.matrix();

            let size = if let Some(custom_size) = sprite.custom_size {
                custom_size
            } else if let Some(ref source_rect) = sprite.source_rect {
                Vec2::new(source_rect.width, source_rect.height)
            } else {
                // Default to texture size (will be obtained from AssetServer once integrated)
                Vec2::new(64.0, 64.0)
            };

            // Use transform's Y position as Z-layer for 2D sorting
            let z_layer = transform.position.y;

            let instance = SpriteInstance::from_components(entity, sprite, matrix, z_layer, size);
            self.sprites.push(instance);
        }

        Ok(())
    }

    // =========================================================================
    // Internal: Sort
    // =========================================================================

    /// Sorts sprites by Z-layer (back to front) and texture (for batching).
    pub(super) fn sort_sprites(&mut self) {
        if !self.config.enable_batching {
            self.sprites.sort_by(|a, b| {
                a.z_layer
                    .partial_cmp(&b.z_layer)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            self.sprites
                .sort_by(|a, b| match a.z_layer.partial_cmp(&b.z_layer) {
                    Some(std::cmp::Ordering::Equal) | None => a.texture.cmp(&b.texture),
                    Some(ord) => ord,
                });
        }
    }

    // =========================================================================
    // Internal: Batch Generation
    // =========================================================================

    /// Generates batches from sorted sprites.
    pub(super) fn generate_batches(&mut self, asset_server: &AssetServer) -> GoudResult<()> {
        if self.sprites.is_empty() {
            return Ok(());
        }

        let sprite_count = self.sprites.len();
        let mut current_texture = self.sprites[0].texture;
        let mut batch_start = 0;
        let mut batch_ranges = Vec::new();

        for i in 0..sprite_count {
            let sprite_texture = self.sprites[i].texture;

            let new_batch = sprite_texture != current_texture
                || (i - batch_start) >= self.config.max_batch_size;

            if new_batch && i > batch_start {
                batch_ranges.push((current_texture, batch_start, i));
                current_texture = sprite_texture;
                batch_start = i;
            }
        }

        batch_ranges.push((current_texture, batch_start, sprite_count));

        for (texture_handle, start_idx, end_idx) in batch_ranges {
            self.finalize_batch(texture_handle, start_idx, end_idx, asset_server)?;
        }

        Ok(())
    }

    /// Finalizes a batch by generating vertices.
    fn finalize_batch(
        &mut self,
        texture_handle: AssetHandle<TextureAsset>,
        start_idx: usize,
        end_idx: usize,
        asset_server: &AssetServer,
    ) -> GoudResult<()> {
        let vertex_start = self.vertices.len();
        let texture_size = self.get_texture_size(texture_handle, asset_server);
        let sprites_to_process = self.sprites[start_idx..end_idx].to_vec();

        for sprite in sprites_to_process {
            self.generate_sprite_vertices(&sprite, texture_size)?;
        }

        let vertex_count = self.vertices.len() - vertex_start;
        let gpu_texture = self.resolve_texture(texture_handle, asset_server)?;

        self.batches.push(SpriteBatchEntry {
            texture_handle,
            gpu_texture: Some(gpu_texture),
            vertex_start,
            vertex_count,
        });

        Ok(())
    }

    /// Generates 4 vertices for a single sprite quad.
    fn generate_sprite_vertices(
        &mut self,
        sprite: &SpriteInstance,
        texture_size: Vec2,
    ) -> GoudResult<()> {
        let half_size = sprite.size * 0.5;
        let local_corners = [
            Vec2::new(-half_size.x, -half_size.y), // Top-left
            Vec2::new(half_size.x, -half_size.y),  // Top-right
            Vec2::new(half_size.x, half_size.y),   // Bottom-right
            Vec2::new(-half_size.x, half_size.y),  // Bottom-left
        ];

        let uv_rect = if let Some(source) = sprite.source_rect {
            Rect::new(
                source.x / texture_size.x,
                source.y / texture_size.y,
                source.width / texture_size.x,
                source.height / texture_size.y,
            )
        } else {
            Rect::new(0.0, 0.0, 1.0, 1.0)
        };

        let u_min = if sprite.flip_x {
            uv_rect.x + uv_rect.width
        } else {
            uv_rect.x
        };
        let u_max = if sprite.flip_x {
            uv_rect.x
        } else {
            uv_rect.x + uv_rect.width
        };
        let v_min = if sprite.flip_y {
            uv_rect.y + uv_rect.height
        } else {
            uv_rect.y
        };
        let v_max = if sprite.flip_y {
            uv_rect.y
        } else {
            uv_rect.y + uv_rect.height
        };

        let uv_corners = [
            Vec2::new(u_min, v_min),
            Vec2::new(u_max, v_min),
            Vec2::new(u_max, v_max),
            Vec2::new(u_min, v_max),
        ];

        for i in 0..4 {
            let world_pos = sprite.transform.transform_point(local_corners[i]);
            self.vertices.push(SpriteVertex {
                position: world_pos,
                tex_coords: uv_corners[i],
                color: sprite.color,
            });
        }

        Ok(())
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Returns the number of sprites rendered this frame.
    pub fn sprite_count(&self) -> usize {
        self.sprites.len()
    }

    /// Returns the number of batches rendered this frame.
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Returns the current frame number.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the batch ratio (sprites per draw call).
    pub fn batch_ratio(&self) -> f32 {
        if self.batches.is_empty() {
            0.0
        } else {
            self.sprites.len() as f32 / self.batches.len() as f32
        }
    }

    /// Returns rendering statistics as (sprite_count, batch_count, batch_ratio) tuple.
    ///
    /// This is a convenience method that combines `sprite_count()`, `batch_count()`,
    /// and `batch_ratio()` for easy performance monitoring.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (sprites, batches, ratio) = batch.stats();
    /// println!("Rendered {} sprites in {} batches ({}:1 ratio)", sprites, batches, ratio);
    /// ```
    pub fn stats(&self) -> (usize, usize, f32) {
        (self.sprite_count(), self.batch_count(), self.batch_ratio())
    }
}
