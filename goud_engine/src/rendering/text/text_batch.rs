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
use super::shader;
use super::{layout_shaped_text, shape_text, TextDirection};
pub use crate::rendering::text::text_batch_requests::DirectTextDrawRequest;

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
    ///
    /// Unlike `atlas_cache` (TrueType), bitmap atlases do not yet support
    /// hot-reload invalidation. When bitmap font hot-reload is implemented,
    /// add `invalidate_font` / `process_reloads` methods mirroring
    /// [`GlyphAtlasCache`].  For now, use [`clear_bitmap_atlas_cache`](Self::clear_bitmap_atlas_cache)
    /// to manually purge entries.
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

            self.draw_component_text(text, transform, asset_server, backend)?;
        }

        Ok(())
    }

    fn draw_component_text(
        &mut self,
        text: &Text,
        transform: &Transform2D,
        asset_server: &AssetServer,
        backend: &mut dyn RenderBackend,
    ) -> Result<(), String> {
        let config = TextLayoutConfig {
            max_width: text.max_width,
            line_spacing: text.line_spacing,
            alignment: text.alignment,
        };

        let (layout, gpu_texture) = if let Some(bitmap_handle) = text.bitmap_font_handle.as_ref() {
            self.resolve_bitmap_font(
                &text.content,
                text.font_size,
                &config,
                bitmap_handle,
                asset_server,
                backend,
            )?
        } else {
            match self.resolve_truetype_font(
                &text.content,
                text.font_size,
                &config,
                &text.font_handle,
                asset_server,
                backend,
            )? {
                Some(result) => result,
                None => return Ok(()),
            }
        };

        if layout.glyphs.is_empty() {
            return Ok(());
        }

        self.append_glyph_batch(&layout, text.color, transform, gpu_texture);
        Ok(())
    }

    /// Resolves a bitmap font, builds/caches its atlas, and returns layout
    /// and GPU texture handle.
    fn resolve_bitmap_font(
        &mut self,
        content: &str,
        font_size: f32,
        config: &TextLayoutConfig,
        bitmap_handle: &crate::assets::AssetHandle<BitmapFontAsset>,
        asset_server: &AssetServer,
        backend: &mut dyn RenderBackend,
    ) -> Result<(super::layout::TextLayoutResult, TextureHandle), String> {
        let bitmap_font = asset_server
            .get::<BitmapFontAsset>(bitmap_handle)
            .ok_or_else(|| format!("bitmap font asset not found for handle {:?}", bitmap_handle))?;

        let cache_key = bitmap_handle.index();
        let atlas = self.bitmap_atlas_cache.entry(cache_key).or_insert_with(|| {
            let w = bitmap_font.scale_w;
            let h = bitmap_font.scale_h;
            let data = bitmap_font.texture_data.clone().unwrap_or_else(|| {
                log::warn!(
                    "Bitmap font texture data not loaded for font at cache key {cache_key}; \
                     rendering will be transparent"
                );
                vec![0u8; (w * h * 4) as usize]
            });
            BitmapGlyphAtlas::new(bitmap_font, w, h, data)
        });

        let kerning = if bitmap_font.kernings.is_empty() {
            None
        } else {
            Some(&bitmap_font.kernings)
        };

        let layout = layout_text(content, atlas, font_size, config, kerning);
        let tex = atlas.ensure_gpu_texture(backend)?;
        Ok((layout, tex))
    }

    /// Resolves a TrueType font, builds/caches its atlas, and returns layout
    /// and GPU texture handle. Returns `None` if the font handle is invalid.
    pub(crate) fn resolve_truetype_font(
        &mut self,
        content: &str,
        font_size: f32,
        config: &TextLayoutConfig,
        font_handle: &crate::assets::AssetHandle<FontAsset>,
        asset_server: &AssetServer,
        backend: &mut dyn RenderBackend,
    ) -> Result<Option<(super::layout::TextLayoutResult, TextureHandle)>, String> {
        if !font_handle.is_valid() {
            return Ok(None);
        }

        let font_asset = asset_server
            .get::<FontAsset>(font_handle)
            .ok_or_else(|| format!("font asset not found for handle {:?}", font_handle))?;

        let parsed_font = font_asset.parse()?;
        let atlas = self
            .atlas_cache
            .get_or_create_mut(font_asset, *font_handle, font_size)?;

        let shaped = shape_text(content, font_asset.data(), font_size, TextDirection::Auto)?;
        atlas.ensure_glyph_indices(&parsed_font, shaped.glyph_indices())?;

        let layout = layout_shaped_text(&shaped, atlas, font_size, config);
        let tex = atlas.ensure_gpu_texture(backend)?;
        Ok(Some((layout, tex)))
    }

    /// Emits glyph quads for a layout and appends or merges a draw batch.
    pub(crate) fn append_glyph_batch(
        &mut self,
        layout: &super::layout::TextLayoutResult,
        color: Color,
        transform: &Transform2D,
        gpu_texture: TextureHandle,
    ) {
        let batch_index_start = self.indices.len() as u32;
        let matrix = transform.matrix();

        for glyph in &layout.glyphs {
            self.emit_glyph_quad(
                glyph.x,
                glyph.y,
                glyph.size_x,
                glyph.size_y,
                &glyph.uv_rect,
                color,
                &matrix,
            );
        }

        let batch_index_count = self.indices.len() as u32 - batch_index_start;
        self.stats.glyph_count += layout.glyphs.len();

        // Merge with the previous batch if the texture matches.
        if let Some(last) = self.batches.last_mut() {
            if last.gpu_texture == gpu_texture {
                last.index_count += batch_index_count;
                return;
            }
        }

        self.batches.push(TextDrawBatch {
            gpu_texture,
            index_start: batch_index_start,
            index_count: batch_index_count,
        });
    }

    /// Uploads geometry to the GPU and issues draw calls.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU buffer creation or draw operations fail.
    pub fn end(
        &mut self,
        backend: &mut dyn RenderBackend,
        viewport: (u32, u32),
    ) -> Result<(), String> {
        if self.batches.is_empty() {
            return Ok(());
        }

        self.upload_buffers(backend)?;
        let shader = shader::ensure_shader(&mut self.shader, backend)?;

        backend
            .bind_shader(shader)
            .map_err(|e| format!("text shader bind failed: {e}"))?;
        backend.enable_blending();

        if let Some(location) = backend.get_uniform_location(shader, "u_texture") {
            backend.set_uniform_int(location, 0);
        }
        if let Some(location) = backend.get_uniform_location(shader, "u_viewport") {
            backend.set_uniform_vec2(location, viewport.0.max(1) as f32, viewport.1.max(1) as f32);
        }

        // Keep the caller's current VAO binding.
        // `GoudGame::draw_text` binds the immediate VAO before entering this path,
        // and forcing a backend default VAO here can invalidate per-VAO element/index state.

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

        #[cfg(feature = "native")]
        {
            let mut bound_vao = 0i32;
            let mut bound_vbo = 0i32;
            let mut bound_ibo = 0i32;
            let mut bound_program = 0i32;
            // SAFETY: These OpenGL state queries are read-only and valid with an active context.
            unsafe {
                gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut bound_vao);
                gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut bound_vbo);
                gl::GetIntegerv(gl::ELEMENT_ARRAY_BUFFER_BINDING, &mut bound_ibo);
                gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut bound_program);
            }
            if bound_vao == 0 || bound_vbo == 0 || bound_ibo == 0 {
                return Err(format!(
                    "text draw state incomplete: vao={bound_vao} vbo={bound_vbo} ibo={bound_ibo}"
                ));
            }
            // SAFETY: Querying object validity is safe with an active context.
            let vao_valid = unsafe { gl::IsVertexArray(bound_vao as u32) == gl::TRUE };
            if !vao_valid || bound_program == 0 {
                return Err(format!(
                    "text draw state invalid: vao={bound_vao} vao_valid={vao_valid} program={bound_program}"
                ));
            }
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
                    (index_start as usize) * std::mem::size_of::<u32>(),
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

    /// Removes all cached bitmap font atlases.
    ///
    /// This is the manual invalidation path for bitmap fonts. Unlike the
    /// TrueType [`GlyphAtlasCache`] which supports per-font invalidation
    /// and hot-reload, bitmap atlases currently require a full cache clear.
    pub fn clear_bitmap_atlas_cache(&mut self) {
        self.bitmap_atlas_cache.clear();
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
#[path = "text_batch_tests.rs"]
mod tests;
