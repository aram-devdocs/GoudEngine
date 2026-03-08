//! Core `TextBatch` struct for batched text rendering.
//!
//! Collects text entities from the ECS world, lays out glyphs using the
//! text layout engine, and renders them as textured quads grouped by
//! atlas texture to minimise draw calls.

use crate::assets::{loaders::BitmapFontAsset, loaders::FontAsset, AssetServer};
use crate::core::math::{Color, Vec2};
use crate::ecs::components::{Text, Transform2D};
use crate::ecs::query::Query;
use crate::ecs::{Entity, World};
use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureHandle,
};
use crate::rendering::sprite_batch::types::SpriteVertex;

use super::atlas_cache::GlyphAtlasCache;
use super::bitmap_atlas::BitmapGlyphAtlas;
use super::glyph_atlas::UvRect;
use super::layout::{layout_text, TextLayoutConfig};

/// A single draw batch for glyphs sharing the same atlas texture.
#[derive(Debug)]
pub struct TextDrawBatch {
    /// GPU texture handle for this batch's atlas.
    pub gpu_texture: TextureHandle,
    /// Starting index in the shared index buffer.
    pub index_start: u32,
    /// Number of indices to draw.
    pub index_count: u32,
}

/// Rendering statistics for a single text frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextRenderStats {
    /// Total number of glyphs submitted.
    pub glyph_count: usize,
    /// Number of draw calls issued.
    pub draw_calls: usize,
}

/// High-performance text batch renderer.
///
/// Follows the same begin/draw/end lifecycle as [`SpriteBatch`](crate::rendering::sprite_batch::SpriteBatch):
///
/// ```rust,ignore
/// batch.begin();
/// batch.draw_text(&world, &asset_server, &mut backend)?;
/// batch.end(&mut backend)?;
/// ```
pub struct TextBatch {
    /// Cached TrueType glyph atlases keyed by (font_handle, size).
    atlas_cache: GlyphAtlasCache,
    /// Cached bitmap font atlases keyed by asset handle index.
    bitmap_atlas_cache: std::collections::HashMap<u32, BitmapGlyphAtlas>,
    /// CPU-side vertex buffer (4 vertices per glyph quad).
    vertices: Vec<SpriteVertex>,
    /// CPU-side index buffer (6 indices per glyph quad).
    indices: Vec<u32>,
    /// Draw batches grouped by atlas texture.
    batches: Vec<TextDrawBatch>,
    /// GPU vertex buffer handle.
    vertex_buffer: Option<BufferHandle>,
    /// GPU index buffer handle.
    index_buffer: Option<BufferHandle>,
    /// Text shader program.
    shader: Option<ShaderHandle>,
    /// Per-frame statistics.
    stats: TextRenderStats,
}

impl TextBatch {
    /// Creates a new, empty text batch.
    pub fn new() -> Self {
        Self {
            atlas_cache: GlyphAtlasCache::new(),
            bitmap_atlas_cache: std::collections::HashMap::new(),
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1536),
            batches: Vec::with_capacity(16),
            vertex_buffer: None,
            index_buffer: None,
            shader: None,
            stats: TextRenderStats::default(),
        }
    }

    /// Begins a new frame, clearing per-frame state.
    pub fn begin(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
        self.stats = TextRenderStats::default();
    }

    /// Gathers text entities, lays out glyphs, and generates quad geometry.
    ///
    /// # Pipeline
    ///
    /// 1. Query `(Entity, &Text, &Transform2D)` from the world.
    /// 2. For each entity, resolve the font asset and get/create its atlas.
    /// 3. Run the layout engine to position glyphs.
    /// 4. Generate a quad (4 vertices, 6 indices) per visible glyph.
    /// 5. Group consecutive glyphs by atlas texture into batches.
    ///
    /// # Errors
    ///
    /// Returns an error if font parsing, atlas generation, or GPU texture
    /// creation fails.
    pub fn draw_text(
        &mut self,
        world: &World,
        asset_server: &AssetServer,
        backend: &mut dyn RenderBackend,
    ) -> Result<(), String> {
        // Collect entity data first to avoid borrow conflicts with atlas_cache.
        let query: Query<(Entity, &Text, &Transform2D)> = Query::new(world);
        let entities: Vec<(Text, Transform2D)> = query
            .iter(world)
            .map(|(_entity, text, transform)| (text.clone(), *transform))
            .collect();

        for (text, transform) in &entities {
            if text.content.is_empty() {
                continue;
            }

            let config = TextLayoutConfig {
                max_width: text.max_width,
                line_spacing: text.line_spacing,
                alignment: text.alignment,
            };

            // Determine the font source: bitmap font takes priority over TTF.
            let (layout, gpu_texture) =
                if let Some(bitmap_handle) = text.bitmap_font_handle.as_ref() {
                    let bitmap_font = asset_server
                        .get::<BitmapFontAsset>(bitmap_handle)
                        .ok_or_else(|| {
                            format!("bitmap font asset not found for handle {:?}", bitmap_handle)
                        })?;

                    let cache_key = bitmap_handle.index();
                    let atlas = self.bitmap_atlas_cache.entry(cache_key).or_insert_with(|| {
                        // Create a new bitmap atlas with placeholder texture
                        // data. The actual texture is loaded externally.
                        BitmapGlyphAtlas::new(bitmap_font, 256, 256, vec![0u8; 256 * 256 * 4])
                    });

                    let kerning_map = &bitmap_font.kernings;
                    let kerning = if kerning_map.is_empty() {
                        None
                    } else {
                        Some(kerning_map)
                    };

                    let layout_result =
                        layout_text(&text.content, atlas, text.font_size, &config, kerning);
                    let tex = atlas.ensure_gpu_texture(backend)?;
                    (layout_result, tex)
                } else {
                    if !text.font_handle.is_valid() {
                        continue;
                    }

                    let font_asset = asset_server
                        .get::<FontAsset>(&text.font_handle)
                        .ok_or_else(|| {
                            format!("font asset not found for handle {:?}", text.font_handle)
                        })?;

                    let atlas = self.atlas_cache.get_or_create_mut(
                        font_asset,
                        text.font_handle,
                        text.font_size,
                    )?;

                    let layout_result =
                        layout_text(&text.content, atlas, text.font_size, &config, None);
                    let tex = atlas.ensure_gpu_texture(backend)?;
                    (layout_result, tex)
                };

            if layout.glyphs.is_empty() {
                continue;
            }

            let batch_index_start = self.indices.len() as u32;
            let matrix = transform.matrix();

            for glyph in &layout.glyphs {
                self.emit_glyph_quad(
                    glyph.x,
                    glyph.y,
                    glyph.size_x,
                    glyph.size_y,
                    &glyph.uv_rect,
                    text.color,
                    &matrix,
                );
            }

            let batch_index_count = self.indices.len() as u32 - batch_index_start;

            self.stats.glyph_count += layout.glyphs.len();

            // Merge with the previous batch if the texture matches.
            if let Some(last) = self.batches.last_mut() {
                if last.gpu_texture == gpu_texture {
                    last.index_count += batch_index_count;
                    continue;
                }
            }

            self.batches.push(TextDrawBatch {
                gpu_texture,
                index_start: batch_index_start,
                index_count: batch_index_count,
            });
        }

        Ok(())
    }

    /// Uploads geometry to the GPU and issues draw calls.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU buffer creation or draw operations fail.
    pub fn end(&mut self, backend: &mut dyn RenderBackend) -> Result<(), String> {
        if self.batches.is_empty() {
            return Ok(());
        }

        self.upload_buffers(backend)?;

        if let Some(shader) = self.shader {
            backend
                .bind_shader(shader)
                .map_err(|e| format!("text shader bind failed: {e}"))?;
        }

        if let Some(vbo) = self.vertex_buffer {
            backend
                .bind_buffer(vbo)
                .map_err(|e| format!("text VBO bind failed: {e}"))?;
            backend.set_vertex_attributes(&SpriteVertex::layout());
        }

        if let Some(ibo) = self.index_buffer {
            backend
                .bind_buffer(ibo)
                .map_err(|e| format!("text IBO bind failed: {e}"))?;
        }

        // Collect draw data to avoid borrow conflict with backend.
        let draw_calls: Vec<(TextureHandle, u32, u32)> = self
            .batches
            .iter()
            .map(|b| (b.gpu_texture, b.index_start, b.index_count))
            .collect();

        for (gpu_texture, index_start, index_count) in draw_calls {
            backend
                .bind_texture(gpu_texture, 0)
                .map_err(|e| format!("text texture bind failed: {e}"))?;

            backend
                .draw_indexed(
                    PrimitiveTopology::Triangles,
                    index_count,
                    index_start as usize,
                )
                .map_err(|e| format!("text draw_indexed failed: {e}"))?;
        }

        self.stats.draw_calls = self.batches.len();

        Ok(())
    }

    /// Returns rendering statistics for the current frame.
    pub fn stats(&self) -> TextRenderStats {
        self.stats
    }

    /// Returns a reference to the underlying atlas cache.
    pub fn atlas_cache(&self) -> &GlyphAtlasCache {
        &self.atlas_cache
    }

    /// Emits a single glyph quad (4 vertices + 6 indices).
    fn emit_glyph_quad(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv: &UvRect,
        color: Color,
        matrix: &crate::ecs::components::Mat3x3,
    ) {
        let base = self.vertices.len() as u32;

        // Quad corners (top-left origin).
        let corners = [
            Vec2::new(x, y),         // top-left
            Vec2::new(x + w, y),     // top-right
            Vec2::new(x + w, y + h), // bottom-right
            Vec2::new(x, y + h),     // bottom-left
        ];

        let uvs = [
            Vec2::new(uv.u_min, uv.v_min),
            Vec2::new(uv.u_max, uv.v_min),
            Vec2::new(uv.u_max, uv.v_max),
            Vec2::new(uv.u_min, uv.v_max),
        ];

        for i in 0..4 {
            let world_pos = matrix.transform_point(corners[i]);
            self.vertices.push(SpriteVertex {
                position: world_pos,
                tex_coords: uvs[i],
                color,
            });
        }

        // Two triangles (CCW winding).
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }

    /// Uploads vertex and index data to the GPU.
    fn upload_buffers(&mut self, backend: &mut dyn RenderBackend) -> Result<(), String> {
        let vert_bytes = vertex_slice_as_bytes(&self.vertices);

        match self.vertex_buffer {
            Some(buf) => {
                backend
                    .update_buffer(buf, 0, vert_bytes)
                    .map_err(|e| format!("text VBO update failed: {e}"))?;
            }
            None => {
                let buf = backend
                    .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, vert_bytes)
                    .map_err(|e| format!("text VBO create failed: {e}"))?;
                self.vertex_buffer = Some(buf);
            }
        }

        let idx_bytes = index_slice_as_bytes(&self.indices);

        match self.index_buffer {
            Some(buf) => {
                backend
                    .update_buffer(buf, 0, idx_bytes)
                    .map_err(|e| format!("text IBO update failed: {e}"))?;
            }
            None => {
                let buf = backend
                    .create_buffer(BufferType::Index, BufferUsage::Dynamic, idx_bytes)
                    .map_err(|e| format!("text IBO create failed: {e}"))?;
                self.index_buffer = Some(buf);
            }
        }

        Ok(())
    }
}

impl Default for TextBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TextBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextBatch")
            .field("glyph_count", &self.stats.glyph_count)
            .field("draw_calls", &self.stats.draw_calls)
            .field("batches", &self.batches.len())
            .finish()
    }
}

/// Reinterprets a `&[SpriteVertex]` as `&[u8]` for GPU upload.
fn vertex_slice_as_bytes(vertices: &[SpriteVertex]) -> &[u8] {
    // SAFETY: SpriteVertex is #[repr(C)] with no padding invariants.
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast(), std::mem::size_of_val(vertices)) }
}

/// Reinterprets a `&[u32]` as `&[u8]` for GPU upload.
fn index_slice_as_bytes(indices: &[u32]) -> &[u8] {
    // SAFETY: u32 has no alignment/validity invariants beyond its size.
    unsafe { std::slice::from_raw_parts(indices.as_ptr().cast(), std::mem::size_of_val(indices)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_batch_new_is_empty() {
        let batch = TextBatch::new();
        let stats = batch.stats();
        assert_eq!(stats.glyph_count, 0);
        assert_eq!(stats.draw_calls, 0);
    }

    #[test]
    fn test_text_batch_begin_clears_state() {
        let mut batch = TextBatch::new();
        // Manually push some data.
        batch.vertices.push(SpriteVertex {
            position: Vec2::new(0.0, 0.0),
            tex_coords: Vec2::new(0.0, 0.0),
            color: Color::WHITE,
        });
        batch.indices.push(0);
        batch.stats.glyph_count = 5;

        batch.begin();

        assert!(batch.vertices.is_empty());
        assert!(batch.indices.is_empty());
        assert!(batch.batches.is_empty());
        assert_eq!(batch.stats().glyph_count, 0);
    }

    #[test]
    fn test_text_batch_default_equals_new() {
        let a = TextBatch::new();
        let b = TextBatch::default();
        assert_eq!(a.stats().glyph_count, b.stats().glyph_count);
        assert_eq!(a.stats().draw_calls, b.stats().draw_calls);
    }

    #[test]
    fn test_text_batch_debug_format() {
        let batch = TextBatch::new();
        let debug = format!("{:?}", batch);
        assert!(debug.contains("TextBatch"));
    }

    #[test]
    fn test_emit_glyph_quad_produces_correct_geometry() {
        let mut batch = TextBatch::new();
        let identity = crate::ecs::components::Mat3x3::IDENTITY;
        let uv = UvRect {
            u_min: 0.0,
            v_min: 0.0,
            u_max: 1.0,
            v_max: 1.0,
        };

        batch.emit_glyph_quad(10.0, 20.0, 8.0, 12.0, &uv, Color::WHITE, &identity);

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.indices.len(), 6);

        // Verify positions.
        assert_eq!(batch.vertices[0].position, Vec2::new(10.0, 20.0));
        assert_eq!(batch.vertices[1].position, Vec2::new(18.0, 20.0));
        assert_eq!(batch.vertices[2].position, Vec2::new(18.0, 32.0));
        assert_eq!(batch.vertices[3].position, Vec2::new(10.0, 32.0));

        // Verify indices form two triangles.
        assert_eq!(&batch.indices[..], &[0, 1, 2, 2, 3, 0]);
    }

    #[test]
    fn test_emit_two_quads_produces_correct_indices() {
        let mut batch = TextBatch::new();
        let identity = crate::ecs::components::Mat3x3::IDENTITY;
        let uv = UvRect {
            u_min: 0.0,
            v_min: 0.0,
            u_max: 1.0,
            v_max: 1.0,
        };

        batch.emit_glyph_quad(0.0, 0.0, 8.0, 8.0, &uv, Color::WHITE, &identity);
        batch.emit_glyph_quad(10.0, 0.0, 8.0, 8.0, &uv, Color::WHITE, &identity);

        assert_eq!(batch.vertices.len(), 8);
        assert_eq!(batch.indices.len(), 12);
        // Second quad indices should be offset by 4.
        assert_eq!(&batch.indices[6..], &[4, 5, 6, 6, 7, 4]);
    }

    #[test]
    fn test_draw_text_with_world_and_null_backend_counts_glyphs() {
        use crate::assets::loaders::FontLoader;
        use crate::assets::AssetServer;
        use crate::ecs::components::Transform2D;
        use crate::ecs::World;
        use crate::libs::graphics::backend::null::NullBackend;

        // Set up a null render backend for headless testing.
        let mut backend = NullBackend::new();

        // Create an AssetServer with a FontLoader registered.
        let mut asset_server = AssetServer::new();
        asset_server.register_loader(FontLoader::default());

        // Load the test font from embedded bytes.
        let ttf_bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf");
        let font_handle = asset_server.load_from_bytes::<crate::assets::loaders::FontAsset>(
            "test_font.ttf",
            ttf_bytes,
        );
        assert!(
            asset_server.is_loaded(&font_handle),
            "font asset should be loaded"
        );

        // Create a World and spawn an entity with Text + Transform2D.
        let mut world = World::new();
        let text = crate::ecs::components::Text::new(font_handle, "Hello")
            .with_font_size(16.0);
        let transform = Transform2D::default();
        let _entity = world.spawn().insert(text).insert(transform).id();

        // Run the text batch pipeline.
        let mut batch = TextBatch::new();
        batch.begin();
        batch
            .draw_text(&world, &asset_server, &mut backend)
            .expect("draw_text should succeed with null backend");

        // "Hello" has 5 characters, so we expect 5 glyphs.
        assert_eq!(
            batch.stats().glyph_count,
            5,
            "expected 5 glyphs for 'Hello'"
        );
    }
}
