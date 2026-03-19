//! Core `SpriteBatch` struct and frame lifecycle implementation.

use super::config::SpriteBatchConfig;
use super::types::{SpriteBatchEntry, SpriteInstance, SpriteVertex, TextureCacheEntry};
use crate::assets::{
    loaders::{SpriteSheetAsset, TextureAsset},
    AssetHandle, AssetServer,
};
use crate::core::error::GoudResult;
use crate::core::math::{Rect, Vec2};
use crate::ecs::components::{Sprite, Transform2D};
use crate::ecs::query::Query;
use crate::ecs::{Entity, World};
use crate::libs::graphics::backend::types::{BufferHandle, ShaderHandle};
use crate::libs::graphics::backend::RenderBackend;
use crate::rendering::RenderViewport;
use std::collections::HashMap;

use super::culling::SpriteFrustum2D;

/// High-performance sprite batch renderer.
///
/// The SpriteBatch collects sprites, sorts them by texture and Z-layer,
/// then renders them in efficient batches to minimize draw calls.
///
/// # Lifecycle
///
/// ```rust,ignore
/// batch.begin();           // Start a new frame
/// batch.draw_sprites(&world, &mut asset_server);  // Gather and batch sprites
/// batch.end();             // Flush remaining batches
/// ```
pub struct SpriteBatch<B: RenderBackend> {
    /// Graphics backend
    pub backend: B,
    /// Configuration
    pub config: SpriteBatchConfig,
    /// Vertex buffer handle
    pub vertex_buffer: Option<BufferHandle>,
    /// Index buffer handle (shared quad indices)
    pub index_buffer: Option<BufferHandle>,
    /// Current vertex buffer capacity (number of sprites)
    pub vertex_capacity: usize,
    /// Sprite shader program
    pub shader: Option<ShaderHandle>,
    /// Collected sprite instances this frame
    pub sprites: Vec<SpriteInstance>,
    /// CPU-side vertex buffer
    pub vertices: Vec<SpriteVertex>,
    /// Prepared batches for rendering
    pub batches: Vec<SpriteBatchEntry>,
    /// Texture handle cache (AssetHandle -> GPU TextureHandle)
    pub texture_cache: HashMap<AssetHandle<TextureAsset>, TextureCacheEntry>,
    /// Current frame number (for debugging)
    pub frame_count: u64,
    /// Active viewport used for sprite projection.
    pub viewport: RenderViewport,
    /// Signature of the currently compiled shader/material state.
    pub shader_signature: Option<u64>,
    /// Number of sprites rejected by frustum culling this frame.
    pub culled_count: usize,
}

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
            viewport: RenderViewport::default(),
            shader_signature: None,
            culled_count: 0,
        })
    }

    /// Begins a new frame of sprite rendering.
    ///
    /// This clears the sprite collection from the previous frame.
    pub fn begin(&mut self) {
        self.sprites.clear();
        self.vertices.clear();
        self.batches.clear();
        self.culled_count = 0;
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
    pub fn draw_sprites(
        &mut self,
        world: &World,
        asset_server: &mut AssetServer,
    ) -> GoudResult<()> {
        self.gather_sprites(world, asset_server)?;

        if self.config.enable_z_sorting {
            self.sort_sprites();
        }

        self.generate_batches(asset_server)?;
        self.render_batches(asset_server)?;

        Ok(())
    }

    // =========================================================================
    // Internal: Gather
    // =========================================================================

    /// Gathers all sprites from the world into the internal sprite list.
    pub fn gather_sprites(
        &mut self,
        world: &World,
        asset_server: &mut AssetServer,
    ) -> GoudResult<()> {
        self.sprites.clear();
        self.culled_count = 0;
        let frustum = SpriteFrustum2D::from_viewport(self.viewport);

        let query: Query<(Entity, &Sprite, &Transform2D)> = Query::new(world);

        for (entity, sprite, transform) in query.iter(world) {
            let matrix = transform.matrix();
            let (texture, source_rect) = self.resolve_sprite_source(sprite, asset_server)?;

            let size = if let Some(custom_size) = sprite.custom_size {
                custom_size
            } else if let Some(source_rect) = source_rect {
                Vec2::new(source_rect.width, source_rect.height)
            } else if texture.is_valid() {
                self.get_texture_size(texture, asset_server)
            } else {
                Vec2::new(64.0, 64.0)
            };

            let instance = SpriteInstance::from_components(
                entity,
                texture,
                source_rect,
                matrix,
                sprite.z_layer,
                size,
                sprite.color,
                sprite.flip_x,
                sprite.flip_y,
            );
            if !self.config.enable_frustum_culling || self.is_sprite_visible(&instance, &frustum) {
                self.sprites.push(instance);
            } else {
                self.culled_count += 1;
            }
        }

        Ok(())
    }

    fn is_sprite_visible(&self, sprite: &SpriteInstance, frustum: &SpriteFrustum2D) -> bool {
        frustum.intersects_rect(&self.sprite_bounds(sprite))
    }

    fn sprite_bounds(&self, sprite: &SpriteInstance) -> Rect {
        let half_size = sprite.size * 0.5;
        let local_corners = [
            Vec2::new(-half_size.x, -half_size.y),
            Vec2::new(half_size.x, -half_size.y),
            Vec2::new(half_size.x, half_size.y),
            Vec2::new(-half_size.x, half_size.y),
        ];

        let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

        for corner in local_corners {
            let world = sprite.transform.transform_point(corner);
            min.x = min.x.min(world.x);
            min.y = min.y.min(world.y);
            max.x = max.x.max(world.x);
            max.y = max.y.max(world.y);
        }

        Rect::from_min_max(min, max)
    }

    fn resolve_sprite_source(
        &self,
        sprite: &Sprite,
        asset_server: &mut AssetServer,
    ) -> GoudResult<(AssetHandle<TextureAsset>, Option<Rect>)> {
        if sprite.sprite_sheet.is_valid() {
            let frame_name = sprite.sprite_frame.as_deref().ok_or_else(|| {
                crate::core::error::GoudError::InvalidState(
                    "Sprite sheet handle set without a frame name".to_string(),
                )
            })?;
            let sheet = asset_server.get(&sprite.sprite_sheet).ok_or_else(|| {
                crate::core::error::GoudError::ResourceNotFound(format!(
                    "Sprite sheet asset {:?}",
                    sprite.sprite_sheet
                ))
            })?;
            let texture_path = sheet.texture_path().to_string();
            let region_rect = sheet
                .region(frame_name)
                .ok_or_else(|| {
                    crate::core::error::GoudError::ResourceNotFound(format!(
                        "Sprite frame '{frame_name}' not found in sprite sheet"
                    ))
                })?
                .rect;
            let texture = asset_server.load::<TextureAsset>(&texture_path);
            return Ok((texture, Some(region_rect)));
        }

        if let Some(path) = sprite.sprite_sheet_path.as_deref() {
            let frame_name = sprite.sprite_frame.as_deref().ok_or_else(|| {
                crate::core::error::GoudError::InvalidState(
                    "Sprite sheet path set without a frame name".to_string(),
                )
            })?;
            let sheet_handle = asset_server.load::<SpriteSheetAsset>(path);
            let sheet = asset_server.get(&sheet_handle).ok_or_else(|| {
                crate::core::error::GoudError::ResourceNotFound(format!(
                    "Sprite sheet asset path '{path}'"
                ))
            })?;
            let texture_path = sheet.texture_path().to_string();
            let region_rect = sheet
                .region(frame_name)
                .ok_or_else(|| {
                    crate::core::error::GoudError::ResourceNotFound(format!(
                        "Sprite frame '{frame_name}' not found in sprite sheet '{path}'"
                    ))
                })?
                .rect;
            let texture = asset_server.load::<TextureAsset>(&texture_path);
            return Ok((texture, Some(region_rect)));
        }

        Ok((sprite.texture, sprite.source_rect))
    }

    // =========================================================================
    // Internal: Sort
    // =========================================================================

    /// Sorts sprites by Z-layer (back to front) and texture (for batching).
    pub fn sort_sprites(&mut self) {
        if !self.config.enable_batching {
            self.sprites.sort_by_key(|sprite| sprite.z_layer);
        } else {
            self.sprites.sort_by(|a, b| {
                a.z_layer
                    .cmp(&b.z_layer)
                    .then_with(|| a.texture.cmp(&b.texture))
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
            _texture_handle: texture_handle,
            gpu_texture: Some(gpu_texture),
            vertex_start,
            vertex_count,
        });

        Ok(())
    }

    /// Generates 4 vertices for a single sprite quad.
    pub fn generate_sprite_vertices(
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

    /// Returns how many sprites were rejected by frustum culling this frame.
    pub fn culled_count(&self) -> usize {
        self.culled_count
    }

    /// Sets the active viewport used by this batch.
    pub fn set_viewport(&mut self, viewport: RenderViewport) {
        self.viewport = viewport;
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

impl<B: RenderBackend> crate::core::providers::diagnostics::DiagnosticsSource for SpriteBatch<B> {
    fn diagnostics_key(&self) -> &str {
        "sprite_batch"
    }

    fn collect_diagnostics(&self) -> serde_json::Value {
        let (sprites, batches, ratio) = self.stats();
        serde_json::json!({
            "sprite_count": sprites,
            "batch_count": batches,
            "batch_ratio": ratio,
            "culled_count": self.culled_count(),
        })
    }
}
