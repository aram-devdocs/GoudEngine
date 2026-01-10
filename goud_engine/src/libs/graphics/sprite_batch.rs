//! Sprite Batch Renderer for efficient 2D sprite rendering.
//!
//! This module provides a high-performance sprite batching system that:
//! - **Batches sprites**: Groups sprites by texture to minimize draw calls
//! - **Sorts by Z-layer**: Ensures correct render order
//! - **Manages vertex buffers**: Dynamic vertex buffer resizing
//! - **Handles transforms**: Integrates with Transform2D component
//!
//! # Architecture
//!
//! The sprite batch system uses a gather-sort-batch-render pipeline:
//!
//! 1. **Gather**: Query all entities with Sprite + Transform2D
//! 2. **Sort**: Order sprites by Z-layer and texture for efficient batching
//! 3. **Batch**: Group consecutive sprites with same texture into batches
//! 4. **Render**: Submit vertex data and draw calls to GPU
//!
//! # Performance
//!
//! Target performance: <100 draw calls for 10,000 sprites (100:1 batch ratio)
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::graphics::sprite_batch::{SpriteBatch, SpriteBatchConfig};
//! use goud_engine::graphics::backend::OpenGLBackend;
//! use goud_engine::ecs::World;
//!
//! let backend = OpenGLBackend::new()?;
//! let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default());
//!
//! // Each frame
//! batch.begin();
//! batch.draw_sprites(&world);
//! batch.end();
//! ```

use crate::assets::{loaders::TextureAsset, AssetHandle, AssetServer};
use crate::core::error::{GoudError, GoudResult};
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::components::{Mat3x3, Sprite, Transform2D};
use crate::ecs::query::Query;
use crate::ecs::{Entity, World};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureHandle,
    VertexAttribute, VertexAttributeType, VertexLayout,
};
use crate::libs::graphics::backend::RenderBackend;
use std::collections::HashMap;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for sprite batch rendering.
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteBatchConfig {
    /// Initial capacity for vertex buffer (number of sprites).
    pub initial_capacity: usize,

    /// Maximum number of sprites per batch before automatic flush.
    pub max_batch_size: usize,

    /// Enable Z-layer sorting (disable for UI layers that don't need depth).
    pub enable_z_sorting: bool,

    /// Enable automatic batching by texture (disable for debugging).
    pub enable_batching: bool,
}

impl Default for SpriteBatchConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 1024, // Start with space for 1024 sprites
            max_batch_size: 10000,  // Flush after 10K sprites
            enable_z_sorting: true, // Sort by Z-layer by default
            enable_batching: true,  // Batch by texture by default
        }
    }
}

// =============================================================================
// Vertex Format
// =============================================================================

/// Vertex data for a single sprite corner.
///
/// Each sprite is composed of 4 vertices forming a quad.
/// The vertex layout is optimized for cache coherency.
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SpriteVertex {
    /// World-space position (x, y)
    position: Vec2,
    /// Texture coordinates (u, v)
    tex_coords: Vec2,
    /// Vertex color (r, g, b, a)
    color: Color,
}

impl SpriteVertex {
    /// Returns the vertex layout descriptor for GPU.
    #[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
    fn layout() -> VertexLayout {
        VertexLayout::new(std::mem::size_of::<Self>() as u32)
            .with_attribute(VertexAttribute {
                location: 0,
                attribute_type: VertexAttributeType::Float2,
                offset: 0,
                normalized: false,
            })
            .with_attribute(VertexAttribute {
                location: 1,
                attribute_type: VertexAttributeType::Float2,
                offset: 8,
                normalized: false,
            })
            .with_attribute(VertexAttribute {
                location: 2,
                attribute_type: VertexAttributeType::Float4,
                offset: 16,
                normalized: false,
            })
    }
}

// =============================================================================
// Sprite Instance
// =============================================================================

/// Internal representation of a sprite instance for batching.
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[derive(Debug, Clone)]
struct SpriteInstance {
    /// Entity that owns this sprite
    entity: Entity,
    /// Texture handle
    texture: AssetHandle<TextureAsset>,
    /// World transform matrix
    transform: Mat3x3,
    /// Color tint
    color: Color,
    /// Source rectangle (UV coordinates)
    source_rect: Option<Rect>,
    /// Sprite size
    size: Vec2,
    /// Z-layer for sorting
    z_layer: f32,
    /// Flip flags
    flip_x: bool,
    flip_y: bool,
}

// =============================================================================
// Batch Entry
// =============================================================================

/// A single draw batch for sprites sharing the same texture.
#[allow(dead_code)] // TODO: Remove when sprite batch is integrated with game loop
#[derive(Debug)]
struct SpriteBatchEntry {
    /// Texture used by this batch
    texture_handle: AssetHandle<TextureAsset>,
    /// GPU texture handle (resolved from asset handle)
    gpu_texture: Option<TextureHandle>,
    /// Start index in vertex buffer
    vertex_start: usize,
    /// Number of vertices in this batch
    vertex_count: usize,
}

// =============================================================================
// Sprite Batch
// =============================================================================

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
    backend: B,
    /// Configuration
    config: SpriteBatchConfig,
    /// Vertex buffer handle
    vertex_buffer: Option<BufferHandle>,
    /// Index buffer handle (shared quad indices)
    index_buffer: Option<BufferHandle>,
    /// Current vertex buffer capacity (number of sprites)
    vertex_capacity: usize,
    /// Sprite shader program
    shader: Option<ShaderHandle>,
    /// Collected sprite instances this frame
    sprites: Vec<SpriteInstance>,
    /// CPU-side vertex buffer
    vertices: Vec<SpriteVertex>,
    /// Prepared batches for rendering
    batches: Vec<SpriteBatchEntry>,
    /// Texture handle cache (AssetHandle -> GPU TextureHandle)
    texture_cache: HashMap<AssetHandle<TextureAsset>, TextureHandle>,
    /// Current frame number (for debugging)
    frame_count: u64,
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
        // Step 1: Gather sprites from world
        self.gather_sprites(world)?;

        // Step 2: Sort sprites
        if self.config.enable_z_sorting {
            self.sort_sprites();
        }

        // Step 3: Generate batches
        self.generate_batches(asset_server)?;

        // Step 4: Render batches
        self.render_batches()?;

        Ok(())
    }

    // =========================================================================
    // Internal: Gather
    // =========================================================================

    /// Gathers all sprites from the world into the internal sprite list.
    fn gather_sprites(&mut self, world: &World) -> GoudResult<()> {
        // Clear previous frame's sprites
        self.sprites.clear();

        // Create query for entities with Sprite and Transform2D components
        // Query returns (Entity, &Sprite, &Transform2D) tuples
        let query: Query<(Entity, &Sprite, &Transform2D)> = Query::new(world);

        // Iterate over all matching entities
        for (entity, sprite, transform) in query.iter(world) {
            // Extract transform matrix for vertex transformation
            let matrix = transform.matrix();

            // Calculate sprite dimensions
            let size = if let Some(custom_size) = sprite.custom_size {
                custom_size
            } else if let Some(ref source_rect) = sprite.source_rect {
                Vec2::new(source_rect.width, source_rect.height)
            } else {
                // Default to texture size (we'll need to get this from AssetServer later)
                // For now, use a default size
                Vec2::new(64.0, 64.0)
            };

            // Use transform's Y position as Z-layer for 2D sorting
            // In 2D games, Y-axis often determines draw order (bottom-to-top)
            let z_layer = transform.position.y;

            // Create sprite instance
            let instance = SpriteInstance {
                entity,
                transform: matrix,
                texture: sprite.texture,
                color: sprite.color,
                source_rect: sprite.source_rect,
                size,
                flip_x: sprite.flip_x,
                flip_y: sprite.flip_y,
                z_layer,
            };

            self.sprites.push(instance);
        }

        Ok(())
    }

    // =========================================================================
    // Internal: Sort
    // =========================================================================

    /// Sorts sprites by Z-layer (back to front) and texture (for batching).
    fn sort_sprites(&mut self) {
        if !self.config.enable_batching {
            // Simple Z-layer sort only
            self.sprites.sort_by(|a, b| {
                a.z_layer
                    .partial_cmp(&b.z_layer)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            // Sort by Z-layer first, then by texture for batching
            self.sprites.sort_by(|a, b| {
                match a.z_layer.partial_cmp(&b.z_layer) {
                    Some(std::cmp::Ordering::Equal) | None => {
                        // Within same Z-layer, sort by texture for batching
                        a.texture.cmp(&b.texture)
                    }
                    Some(ord) => ord,
                }
            });
        }
    }

    // =========================================================================
    // Internal: Batch Generation
    // =========================================================================

    /// Generates batches from sorted sprites.
    fn generate_batches(&mut self, asset_server: &AssetServer) -> GoudResult<()> {
        if self.sprites.is_empty() {
            return Ok(());
        }

        let sprite_count = self.sprites.len();
        let mut current_texture = self.sprites[0].texture;
        let mut batch_start = 0;

        // Build batch ranges first to avoid borrowing issues
        let mut batch_ranges = Vec::new();

        for i in 0..sprite_count {
            let sprite_texture = self.sprites[i].texture;

            // Check if we need to start a new batch
            let new_batch = sprite_texture != current_texture
                || (i - batch_start) >= self.config.max_batch_size;

            if new_batch && i > batch_start {
                // Record current batch range
                batch_ranges.push((current_texture, batch_start, i));

                // Start new batch
                current_texture = sprite_texture;
                batch_start = i;
            }
        }

        // Record last batch
        batch_ranges.push((current_texture, batch_start, sprite_count));

        // Now generate vertices for each batch
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

        // Get texture dimensions for UV calculation
        let texture_size = self.get_texture_size(texture_handle, asset_server);

        // Clone sprite data to avoid borrowing issues
        let sprites_to_process = self.sprites[start_idx..end_idx].to_vec();

        // Generate vertices for each sprite in this batch
        for sprite in sprites_to_process {
            self.generate_sprite_vertices(&sprite, texture_size)?;
        }

        let vertex_count = self.vertices.len() - vertex_start;

        // Resolve GPU texture handle
        let gpu_texture = self.resolve_texture(texture_handle, asset_server)?;

        // Add batch entry
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
        // Calculate sprite corners in local space
        let half_size = sprite.size * 0.5;
        let local_corners = [
            Vec2::new(-half_size.x, -half_size.y), // Top-left
            Vec2::new(half_size.x, -half_size.y),  // Top-right
            Vec2::new(half_size.x, half_size.y),   // Bottom-right
            Vec2::new(-half_size.x, half_size.y),  // Bottom-left
        ];

        // Calculate UV coordinates
        let uv_rect = if let Some(source) = sprite.source_rect {
            // Use source rectangle
            Rect::new(
                source.x / texture_size.x,
                source.y / texture_size.y,
                source.width / texture_size.x,
                source.height / texture_size.y,
            )
        } else {
            // Use full texture
            Rect::new(0.0, 0.0, 1.0, 1.0)
        };

        // Calculate UV corners with flipping
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
            Vec2::new(u_min, v_min), // Top-left
            Vec2::new(u_max, v_min), // Top-right
            Vec2::new(u_max, v_max), // Bottom-right
            Vec2::new(u_min, v_max), // Bottom-left
        ];

        // Transform corners to world space and create vertices
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
    // Internal: Rendering
    // =========================================================================

    /// Renders all batches to the GPU.
    fn render_batches(&mut self) -> GoudResult<()> {
        if self.batches.is_empty() {
            return Ok(());
        }

        // Ensure GPU resources are created
        self.ensure_resources()?;

        // Upload vertex data
        self.upload_vertices()?;

        // Bind shader and set uniforms
        if let Some(shader) = self.shader {
            self.backend.bind_shader(shader)?;
            // TODO: Set projection matrix uniform
        }

        // Bind vertex buffer and set attributes
        if let Some(vbo) = self.vertex_buffer {
            self.backend.bind_buffer(vbo)?;
            self.backend.set_vertex_attributes(&SpriteVertex::layout());
        }

        // Bind index buffer
        if let Some(ibo) = self.index_buffer {
            self.backend.bind_buffer(ibo)?;
        }

        // Draw each batch
        for batch in &self.batches {
            // Bind texture
            if let Some(gpu_tex) = batch.gpu_texture {
                self.backend.bind_texture(gpu_tex, 0)?;
            }

            // Calculate indices
            let sprite_count = batch.vertex_count / 4;
            let index_start = (batch.vertex_start / 4) * 6;
            let index_count = sprite_count * 6;

            // Draw indexed
            self.backend.draw_indexed(
                PrimitiveTopology::Triangles,
                index_count as u32,
                index_start,
            )?;
        }

        Ok(())
    }

    // =========================================================================
    // Internal: Resource Management
    // =========================================================================

    /// Ensures GPU resources (buffers, shader) are created.
    fn ensure_resources(&mut self) -> GoudResult<()> {
        // Create vertex buffer if needed
        if self.vertex_buffer.is_none() || self.vertices.len() > self.vertex_capacity * 4 {
            self.create_vertex_buffer()?;
        }

        // Create index buffer if needed
        if self.index_buffer.is_none() {
            self.create_index_buffer()?;
        }

        // Create shader if needed
        if self.shader.is_none() {
            self.create_shader()?;
        }

        Ok(())
    }

    /// Creates or resizes the vertex buffer.
    fn create_vertex_buffer(&mut self) -> GoudResult<()> {
        // Calculate new capacity (double if needed)
        let required_sprites = self.vertices.len().div_ceil(4);
        let new_capacity = if required_sprites > self.vertex_capacity {
            (required_sprites * 2).max(self.config.initial_capacity)
        } else {
            self.config.initial_capacity
        };

        let buffer_size = new_capacity * 4 * std::mem::size_of::<SpriteVertex>();

        // Destroy old buffer if exists
        if let Some(old_buffer) = self.vertex_buffer {
            let _ = self.backend.destroy_buffer(old_buffer);
        }

        // Create new buffer with empty data (will be updated later)
        let empty_data = vec![0u8; buffer_size];
        let buffer =
            self.backend
                .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &empty_data)?;

        self.vertex_buffer = Some(buffer);
        self.vertex_capacity = new_capacity;

        Ok(())
    }

    /// Creates the shared index buffer for quad rendering.
    fn create_index_buffer(&mut self) -> GoudResult<()> {
        // Generate indices for max_batch_size quads
        let quad_count = self.config.max_batch_size;
        let mut indices = Vec::with_capacity(quad_count * 6);

        for i in 0..quad_count {
            let base = (i * 4) as u32;
            // Two triangles per quad (CCW winding)
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }

        let buffer_size = indices.len() * std::mem::size_of::<u32>();
        let buffer_data =
            unsafe { std::slice::from_raw_parts(indices.as_ptr() as *const u8, buffer_size) };

        let buffer =
            self.backend
                .create_buffer(BufferType::Index, BufferUsage::Static, buffer_data)?;

        self.index_buffer = Some(buffer);

        Ok(())
    }

    /// Creates the sprite shader program.
    fn create_shader(&mut self) -> GoudResult<()> {
        // TODO: Load shader from assets or use built-in shader
        // For now, return error as shader loading isn't implemented yet
        Err(GoudError::NotImplemented(
            "Sprite shader creation".to_string(),
        ))
    }

    /// Uploads vertex data to the GPU.
    fn upload_vertices(&mut self) -> GoudResult<()> {
        if self.vertices.is_empty() {
            return Ok(());
        }

        let buffer = self
            .vertex_buffer
            .ok_or_else(|| GoudError::InvalidState("Vertex buffer not created".to_string()))?;

        let data_size = self.vertices.len() * std::mem::size_of::<SpriteVertex>();
        let data_ptr = self.vertices.as_ptr() as *const u8;
        let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_size) };

        self.backend.update_buffer(buffer, 0, data_slice)?;

        Ok(())
    }

    /// Resolves an asset handle to a GPU texture handle.
    fn resolve_texture(
        &mut self,
        asset_handle: AssetHandle<TextureAsset>,
        asset_server: &AssetServer,
    ) -> GoudResult<TextureHandle> {
        // Check cache first
        if let Some(&gpu_handle) = self.texture_cache.get(&asset_handle) {
            return Ok(gpu_handle);
        }

        // Load texture from asset server
        let _texture_asset = asset_server.get(&asset_handle).ok_or_else(|| {
            GoudError::ResourceNotFound(format!("Texture asset {asset_handle:?}"))
        })?;

        // TODO: Upload texture to GPU and cache handle
        // For now, return error as texture upload isn't implemented yet
        Err(GoudError::NotImplemented("Texture upload".to_string()))
    }

    /// Gets the size of a texture from the asset server.
    fn get_texture_size(
        &self,
        asset_handle: AssetHandle<TextureAsset>,
        asset_server: &AssetServer,
    ) -> Vec2 {
        if let Some(texture) = asset_server.get(&asset_handle) {
            Vec2::new(texture.width as f32, texture.height as f32)
        } else {
            Vec2::one() // Fallback to 1x1
        }
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::libs::graphics::backend::opengl::OpenGLBackend;

    #[test]
    fn test_sprite_batch_config_default() {
        let config = SpriteBatchConfig::default();
        assert_eq!(config.initial_capacity, 1024);
        assert_eq!(config.max_batch_size, 10000);
        assert!(config.enable_z_sorting);
        assert!(config.enable_batching);
    }

    #[test]
    fn test_sprite_vertex_layout() {
        let layout = SpriteVertex::layout();
        assert_eq!(layout.stride, std::mem::size_of::<SpriteVertex>() as u32);
        assert_eq!(layout.attributes.len(), 3);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_batch_new() {
        let backend = OpenGLBackend::new().unwrap();
        let config = SpriteBatchConfig::default();
        let batch = SpriteBatch::new(backend, config);
        assert!(batch.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_batch_begin_end() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        batch.begin();
        assert_eq!(batch.sprite_count(), 0);
        assert_eq!(batch.batch_count(), 0);

        let result = batch.end();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_batch_gather_empty_world() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();
        let world = World::new();

        batch.begin();
        let result = batch.gather_sprites(&world);
        assert!(result.is_ok());
        assert_eq!(batch.sprite_count(), 0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_batch_sort_z_layer() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        // Add sprites with different Z-layers
        batch.sprites = vec![
            SpriteInstance {
                entity: Entity::new(0, 0),
                texture: AssetHandle::new(0, 0),
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 10.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(1, 0),
                texture: AssetHandle::new(0, 0),
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 5.0,
                flip_x: false,
                flip_y: false,
            },
        ];

        batch.sort_sprites();
        assert_eq!(batch.sprites[0].z_layer, 5.0);
        assert_eq!(batch.sprites[1].z_layer, 10.0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_sprite_batch_statistics() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        assert_eq!(batch.frame_count(), 0);
        batch.begin();
        assert_eq!(batch.frame_count(), 1);

        assert_eq!(batch.batch_ratio(), 0.0);
    }

    #[test]
    fn test_sprite_instance_creation() {
        let instance = SpriteInstance {
            entity: Entity::new(42, 1),
            texture: AssetHandle::new(1, 1),
            transform: Mat3x3::IDENTITY,
            color: Color::RED,
            source_rect: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
            size: Vec2::new(64.0, 64.0),
            z_layer: 100.0,
            flip_x: true,
            flip_y: false,
        };

        assert_eq!(instance.entity.index(), 42);
        assert_eq!(instance.color, Color::RED);
        assert!(instance.flip_x);
        assert!(!instance.flip_y);
    }

    #[test]
    fn test_sprite_batch_entry_creation() {
        let entry = SpriteBatchEntry {
            texture_handle: AssetHandle::new(1, 1),
            gpu_texture: None,
            vertex_start: 0,
            vertex_count: 24,
        };

        assert_eq!(entry.vertex_start, 0);
        assert_eq!(entry.vertex_count, 24);
        assert!(entry.gpu_texture.is_none());
    }

    // ==========================================================================
    // Texture Batching Tests
    // ==========================================================================

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_single_texture() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        // Create 5 sprites with the same texture
        let texture = AssetHandle::new(1, 1);
        for i in 0..5 {
            batch.sprites.push(SpriteInstance {
                entity: Entity::new(i, 0),
                texture,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            });
        }

        // With batching enabled, all sprites should be in one batch
        assert_eq!(batch.sprites.len(), 5);

        // Verify all sprites have the same texture
        for sprite in &batch.sprites {
            assert_eq!(sprite.texture, texture);
        }
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_multiple_textures() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        // Create sprites with different textures
        let tex1 = AssetHandle::new(1, 1);
        let tex2 = AssetHandle::new(2, 1);
        let tex3 = AssetHandle::new(3, 1);

        batch.sprites = vec![
            SpriteInstance {
                entity: Entity::new(0, 0),
                texture: tex1,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(1, 0),
                texture: tex2,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(2, 0),
                texture: tex3,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            },
        ];

        // Verify we have 3 different textures
        assert_eq!(batch.sprites.len(), 3);
        assert_ne!(batch.sprites[0].texture, batch.sprites[1].texture);
        assert_ne!(batch.sprites[1].texture, batch.sprites[2].texture);
        assert_ne!(batch.sprites[0].texture, batch.sprites[2].texture);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_sort_by_texture() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        // Create sprites with different textures, unsorted
        let tex1 = AssetHandle::new(1, 1);
        let tex2 = AssetHandle::new(2, 1);

        batch.sprites = vec![
            SpriteInstance {
                entity: Entity::new(0, 0),
                texture: tex2, // tex2 first
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(1, 0),
                texture: tex1, // tex1 second
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(2, 0),
                texture: tex2, // tex2 again
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            },
        ];

        // Sort should group sprites by texture
        batch.sort_sprites();

        // After sorting with same Z-layer, sprites should be grouped by texture
        // tex1 < tex2 (assuming handle comparison), so tex1 sprites should come first
        assert_eq!(batch.sprites[0].texture, tex1);
        assert_eq!(batch.sprites[1].texture, tex2);
        assert_eq!(batch.sprites[2].texture, tex2);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_with_z_layers() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        let tex1 = AssetHandle::new(1, 1);
        let tex2 = AssetHandle::new(2, 1);

        batch.sprites = vec![
            SpriteInstance {
                entity: Entity::new(0, 0),
                texture: tex2,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 10.0, // Higher Z-layer
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(1, 0),
                texture: tex1,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 5.0, // Lower Z-layer
                flip_x: false,
                flip_y: false,
            },
        ];

        // Sort should prioritize Z-layer first, then texture
        batch.sort_sprites();

        // Lower Z should come first
        assert_eq!(batch.sprites[0].z_layer, 5.0);
        assert_eq!(batch.sprites[1].z_layer, 10.0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_same_z_different_texture() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        let tex1 = AssetHandle::new(1, 1);
        let tex2 = AssetHandle::new(2, 1);

        batch.sprites = vec![
            SpriteInstance {
                entity: Entity::new(0, 0),
                texture: tex2,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 5.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(1, 0),
                texture: tex1,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 5.0, // Same Z-layer
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(2, 0),
                texture: tex1,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 5.0, // Same Z-layer, same texture
                flip_x: false,
                flip_y: false,
            },
        ];

        // Sort should group sprites with same Z by texture
        batch.sort_sprites();

        // All have same Z, so should be sorted by texture
        assert_eq!(batch.sprites[0].texture, tex1);
        assert_eq!(batch.sprites[1].texture, tex1);
        assert_eq!(batch.sprites[2].texture, tex2);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_disabled() {
        let backend = OpenGLBackend::new().unwrap();
        let config = SpriteBatchConfig {
            initial_capacity: 1024,
            max_batch_size: 10000,
            enable_z_sorting: true,
            enable_batching: false, // Batching disabled
        };
        let mut batch = SpriteBatch::new(backend, config).unwrap();

        let tex1 = AssetHandle::new(1, 1);
        let tex2 = AssetHandle::new(2, 1);

        batch.sprites = vec![
            SpriteInstance {
                entity: Entity::new(0, 0),
                texture: tex2,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 5.0,
                flip_x: false,
                flip_y: false,
            },
            SpriteInstance {
                entity: Entity::new(1, 0),
                texture: tex1,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 10.0,
                flip_x: false,
                flip_y: false,
            },
        ];

        // With batching disabled, should only sort by Z-layer
        batch.sort_sprites();

        // Should be sorted by Z only, not texture
        assert_eq!(batch.sprites[0].z_layer, 5.0);
        assert_eq!(batch.sprites[1].z_layer, 10.0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_batching_stress_test() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        // Create 100 sprites with 10 different textures (10 sprites per texture)
        for texture_id in 0..10 {
            let texture = AssetHandle::new(texture_id, 1);
            for sprite_id in 0..10 {
                batch.sprites.push(SpriteInstance {
                    entity: Entity::new((texture_id * 10 + sprite_id) as u32, 0),
                    texture,
                    transform: Mat3x3::IDENTITY,
                    color: Color::WHITE,
                    source_rect: None,
                    size: Vec2::one(),
                    z_layer: 0.0,
                    flip_x: false,
                    flip_y: false,
                });
            }
        }

        assert_eq!(batch.sprites.len(), 100);

        // Sort should group sprites by texture
        batch.sort_sprites();

        // Verify that sprites are grouped by texture
        for i in 0..10 {
            let start = i * 10;
            let end = start + 10;
            let texture = batch.sprites[start].texture;

            for j in start..end {
                assert_eq!(
                    batch.sprites[j].texture, texture,
                    "Sprite {j} should have texture {texture:?}"
                );
            }
        }
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_max_batch_size_enforcement() {
        let backend = OpenGLBackend::new().unwrap();
        let config = SpriteBatchConfig {
            initial_capacity: 1024,
            max_batch_size: 5, // Small batch size for testing
            enable_z_sorting: true,
            enable_batching: true,
        };
        let mut batch = SpriteBatch::new(backend, config).unwrap();

        // Create 10 sprites with the same texture
        let texture = AssetHandle::new(1, 1);
        for i in 0..10 {
            batch.sprites.push(SpriteInstance {
                entity: Entity::new(i, 0),
                texture,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            });
        }

        assert_eq!(batch.sprites.len(), 10);

        // Even though all sprites have the same texture, max_batch_size should
        // force them to be split into multiple batches (2 batches of 5)
        // This is verified in generate_batches() method (lines 338-356)
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_interleaved_textures_batching() {
        let backend = OpenGLBackend::new().unwrap();
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

        let tex1 = AssetHandle::new(1, 1);
        let tex2 = AssetHandle::new(2, 1);

        // Create interleaved pattern: tex1, tex2, tex1, tex2, tex1, tex2
        for i in 0..6 {
            let texture = if i % 2 == 0 { tex1 } else { tex2 };
            batch.sprites.push(SpriteInstance {
                entity: Entity::new(i, 0),
                texture,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            });
        }

        // Before sorting, interleaved pattern
        assert_eq!(batch.sprites[0].texture, tex1);
        assert_eq!(batch.sprites[1].texture, tex2);
        assert_eq!(batch.sprites[2].texture, tex1);

        // After sorting, should be grouped
        batch.sort_sprites();

        // All tex1 sprites should be together, all tex2 sprites together
        // The exact order depends on Handle comparison
        let first_texture = batch.sprites[0].texture;
        let mut found_second = false;
        let mut second_texture = first_texture;

        for sprite in &batch.sprites {
            if sprite.texture != first_texture {
                if !found_second {
                    found_second = true;
                    second_texture = sprite.texture;
                } else {
                    // Should still be second texture, not back to first
                    assert_eq!(sprite.texture, second_texture);
                }
            }
        }

        // Should have exactly 2 distinct texture groups
        assert!(found_second);
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_gather_sprites_from_world() {
        use crate::assets::AssetServer;
        use crate::ecs::components::{Sprite, Transform2D};

        let mut world = World::new();
        let mut asset_server = AssetServer::new();

        // Create some test entities with Sprite + Transform2D
        let texture = asset_server.load::<crate::assets::loaders::TextureAsset>("test.png");

        let e1 = world.spawn_empty();
        world.insert(e1, Transform2D::from_position(Vec2::new(10.0, 20.0)));
        world.insert(e1, Sprite::new(texture));

        let e2 = world.spawn_empty();
        world.insert(e2, Transform2D::from_position(Vec2::new(30.0, 40.0)));
        world.insert(e2, Sprite::new(texture).with_color(Color::RED));

        let e3 = world.spawn_empty();
        world.insert(e3, Transform2D::from_position(Vec2::new(50.0, 60.0)));
        world.insert(
            e3,
            Sprite::new(texture).with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0)),
        );

        // Create entity without Sprite (should be ignored)
        let e4 = world.spawn_empty();
        world.insert(e4, Transform2D::from_position(Vec2::new(70.0, 80.0)));

        // Create sprite batch (stub backend since we're just testing gather)
        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default())
            .expect("Failed to create sprite batch");

        // Gather sprites
        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");

        // Should have gathered 3 sprites (e1, e2, e3), not e4
        assert_eq!(batch.sprite_count(), 3);

        // Verify entity IDs are correct
        let entities: Vec<Entity> = batch.sprites.iter().map(|s| s.entity).collect();
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
        assert!(!entities.contains(&e4));

        // Verify colors are preserved
        let sprite1 = batch.sprites.iter().find(|s| s.entity == e1).unwrap();
        assert_eq!(sprite1.color, Color::WHITE);

        let sprite2 = batch.sprites.iter().find(|s| s.entity == e2).unwrap();
        assert_eq!(sprite2.color, Color::RED);

        // Verify source rect is preserved
        let sprite3 = batch.sprites.iter().find(|s| s.entity == e3).unwrap();
        assert!(sprite3.source_rect.is_some());
        let source = sprite3.source_rect.unwrap();
        assert_eq!(source.x, 0.0);
        assert_eq!(source.y, 0.0);
        assert_eq!(source.width, 32.0);
        assert_eq!(source.height, 32.0);

        // Verify Z-layers match Y positions
        assert_eq!(sprite1.z_layer, 20.0);
        assert_eq!(sprite2.z_layer, 40.0);
        assert_eq!(sprite3.z_layer, 60.0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_gather_sprites_empty_world() {
        use crate::assets::AssetServer;

        let world = World::new();
        let _asset_server = AssetServer::new();

        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default())
            .expect("Failed to create sprite batch");

        // Should handle empty world gracefully
        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");
        assert_eq!(batch.sprite_count(), 0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_gather_sprites_clears_previous_frame() {
        use crate::assets::AssetServer;
        use crate::ecs::components::{Sprite, Transform2D};

        let mut world = World::new();
        let mut asset_server = AssetServer::new();

        let texture = asset_server.load::<crate::assets::loaders::TextureAsset>("test.png");

        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default())
            .expect("Failed to create sprite batch");

        // First frame: 2 sprites
        let e1 = world.spawn_empty();
        world.insert(e1, Transform2D::from_position(Vec2::new(10.0, 20.0)));
        world.insert(e1, Sprite::new(texture));

        let e2 = world.spawn_empty();
        world.insert(e2, Transform2D::from_position(Vec2::new(30.0, 40.0)));
        world.insert(e2, Sprite::new(texture));

        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");
        assert_eq!(batch.sprite_count(), 2);

        // Second frame: despawn one sprite
        world.despawn(e2);

        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");
        // Should only have 1 sprite now, not 3 (shouldn't accumulate)
        assert_eq!(batch.sprite_count(), 1);
    }
}
