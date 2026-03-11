//! Native UI render system that executes UI render commands.
//!
//! The system keeps UI rendering on the engine side by:
//! - Translating UI text commands into direct text pipeline requests.
//! - Resolving UI font families through the standard UI fallback chain.
//! - Rendering UI quads through backend-safe abstractions.

use std::collections::HashMap;

use self::bytes::{index_slice_as_bytes, vertex_slice_as_bytes};
use self::resources::{resolve_font, resolve_texture_asset};
use crate::assets::loaders::FontAsset;
use crate::assets::{AssetHandle, AssetServer};
use crate::core::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureFilter,
    TextureFormat as BackendTextureFormat, TextureHandle, TextureWrap,
};
use crate::libs::graphics::backend::RenderBackend;
use crate::rendering::sprite_batch::types::SpriteVertex;
use crate::rendering::text::{DirectTextDrawRequest, TextRenderStats, TextRenderSystem};
use crate::ui::UiRenderCommand;

mod bytes;
mod resources;

/// Per-frame UI rendering statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiRenderStats {
    /// Number of solid quad commands submitted.
    pub quad_commands: usize,
    /// Number of textured quad commands submitted.
    pub textured_quad_commands: usize,
    /// Number of text commands submitted.
    pub text_commands: usize,
    /// Draw calls used for quad rendering.
    pub quad_draw_calls: usize,
    /// Total indices submitted for quad rendering.
    pub quad_index_count: usize,
    /// Total glyphs submitted by the text renderer.
    pub text_glyph_count: usize,
    /// Text draw calls issued by the text renderer.
    pub text_draw_calls: usize,
}

#[derive(Debug, Clone, Copy)]
struct UiQuadBatch {
    gpu_texture: TextureHandle,
    index_start: u32,
    index_count: u32,
}

/// Native runtime render system for UI command streams.
pub struct UiRenderSystem {
    text_render_system: TextRenderSystem,
    font_cache: HashMap<String, AssetHandle<FontAsset>>,
    texture_cache: HashMap<String, TextureHandle>,
    quad_vertices: Vec<SpriteVertex>,
    quad_indices: Vec<u32>,
    quad_batches: Vec<UiQuadBatch>,
    quad_vertex_buffer: Option<BufferHandle>,
    quad_index_buffer: Option<BufferHandle>,
    quad_shader: Option<ShaderHandle>,
    white_texture: Option<TextureHandle>,
    stats: UiRenderStats,
}

impl UiRenderSystem {
    const UI_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_tex_coord;
layout (location = 2) in vec4 a_color;

uniform vec2 u_viewport;

out vec2 v_tex_coord;
out vec4 v_color;

void main() {
    vec2 ndc;
    ndc.x = (a_position.x / u_viewport.x) * 2.0 - 1.0;
    ndc.y = 1.0 - (a_position.y / u_viewport.y) * 2.0;
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_tex_coord = a_tex_coord;
    v_color = a_color;
}
"#;

    const UI_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_tex_coord;
in vec4 v_color;

uniform sampler2D u_texture;
out vec4 FragColor;

void main() {
    FragColor = texture(u_texture, v_tex_coord) * v_color;
}
"#;

    /// Creates a new UI render system.
    pub fn new() -> Self {
        Self {
            text_render_system: TextRenderSystem::new(),
            font_cache: HashMap::new(),
            texture_cache: HashMap::new(),
            quad_vertices: Vec::with_capacity(128),
            quad_indices: Vec::with_capacity(192),
            quad_batches: Vec::with_capacity(16),
            quad_vertex_buffer: None,
            quad_index_buffer: None,
            quad_shader: None,
            white_texture: None,
            stats: UiRenderStats::default(),
        }
    }

    /// Executes a UI command stream against the current frame backend.
    pub fn run(
        &mut self,
        commands: &[UiRenderCommand],
        asset_server: &mut AssetServer,
        backend: &mut dyn RenderBackend,
        viewport: (u32, u32),
    ) -> GoudResult<()> {
        self.stats = UiRenderStats::default();
        ensure_ui_asset_loaders(asset_server);
        self.begin_quad_frame();
        let solid_texture = self.ensure_white_texture(backend)?;

        let mut text_requests = Vec::new();

        for command in commands {
            match command {
                UiRenderCommand::Quad(quad) => {
                    self.stats.quad_commands += 1;
                    self.emit_ui_quad(quad.rect, quad.color, solid_texture);
                }
                UiRenderCommand::TexturedQuad(textured) => {
                    self.stats.textured_quad_commands += 1;
                    if let Some(texture) = resolve_texture_asset(
                        asset_server,
                        backend,
                        &mut self.texture_cache,
                        &textured.texture_path,
                    ) {
                        self.emit_ui_quad(textured.rect, textured.tint, texture);
                    } else {
                        self.emit_ui_quad(textured.rect, textured.tint, solid_texture);
                    }
                }
                UiRenderCommand::Text(text) => {
                    self.stats.text_commands += 1;
                    if text.text.is_empty() {
                        continue;
                    }

                    if let Some(font_handle) =
                        resolve_font(asset_server, &mut self.font_cache, &text.font_family)
                    {
                        text_requests.push(DirectTextDrawRequest {
                            content: text.text.clone(),
                            position: crate::core::math::Vec2::new(
                                text.position[0],
                                text.position[1],
                            ),
                            font_handle,
                            font_size: text.font_size,
                            color: text.color,
                            alignment: crate::core::types::TextAlignment::Left,
                            max_width: None,
                            line_spacing: 1.0,
                        });
                    }
                }
            }
        }

        if !self.quad_batches.is_empty() {
            self.end_quad_frame(backend, viewport)?;
            self.stats.quad_draw_calls = self.quad_batches.len();
            self.stats.quad_index_count = self.quad_indices.len();
        }

        if !text_requests.is_empty() {
            self.text_render_system
                .run_requests(&text_requests, asset_server, backend, viewport)
                .map_err(GoudError::InvalidState)?;
            let text_stats: TextRenderStats = self.text_render_system.stats();
            self.stats.text_glyph_count = text_stats.glyph_count;
            self.stats.text_draw_calls = text_stats.draw_calls;
        }

        Ok(())
    }

    /// Returns stats from the latest `run` call.
    pub fn stats(&self) -> UiRenderStats {
        self.stats
    }

    fn begin_quad_frame(&mut self) {
        self.quad_vertices.clear();
        self.quad_indices.clear();
        self.quad_batches.clear();
    }

    fn emit_ui_quad(
        &mut self,
        rect: crate::core::math::Rect,
        color: crate::core::math::Color,
        gpu_texture: TextureHandle,
    ) {
        let base = self.quad_vertices.len() as u32;
        let positions = [
            crate::core::math::Vec2::new(rect.x, rect.y),
            crate::core::math::Vec2::new(rect.x + rect.width, rect.y),
            crate::core::math::Vec2::new(rect.x + rect.width, rect.y + rect.height),
            crate::core::math::Vec2::new(rect.x, rect.y + rect.height),
        ];
        let uvs = [
            crate::core::math::Vec2::new(0.0, 0.0),
            crate::core::math::Vec2::new(1.0, 0.0),
            crate::core::math::Vec2::new(1.0, 1.0),
            crate::core::math::Vec2::new(0.0, 1.0),
        ];

        for i in 0..4 {
            self.quad_vertices.push(SpriteVertex {
                position: positions[i],
                tex_coords: uvs[i],
                color,
            });
        }

        self.quad_indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);

        let index_start = self.quad_indices.len() as u32 - 6;
        if let Some(last) = self.quad_batches.last_mut() {
            if last.gpu_texture == gpu_texture {
                last.index_count += 6;
                return;
            }
        }

        self.quad_batches.push(UiQuadBatch {
            gpu_texture,
            index_start,
            index_count: 6,
        });
    }

    fn ensure_white_texture(
        &mut self,
        backend: &mut dyn RenderBackend,
    ) -> GoudResult<TextureHandle> {
        if let Some(handle) = self.white_texture {
            if backend.is_texture_valid(handle) {
                return Ok(handle);
            }
            self.white_texture = None;
        }

        let handle = backend
            .create_texture(
                1,
                1,
                BackendTextureFormat::RGBA8,
                TextureFilter::Nearest,
                TextureWrap::ClampToEdge,
                &[255, 255, 255, 255],
            )
            .map_err(|e| {
                GoudError::TextureCreationFailed(format!("ui white texture create failed: {e}"))
            })?;
        self.white_texture = Some(handle);
        Ok(handle)
    }

    fn ensure_quad_shader(&mut self, backend: &mut dyn RenderBackend) -> GoudResult<ShaderHandle> {
        if let Some(shader) = self.quad_shader {
            if backend.is_shader_valid(shader) {
                return Ok(shader);
            }
            self.quad_shader = None;
        }

        let shader = backend
            .create_shader(Self::UI_VERTEX_SHADER, Self::UI_FRAGMENT_SHADER)
            .map_err(|e| {
                GoudError::ShaderCompilationFailed(format!("ui shader creation failed: {e}"))
            })?;
        self.quad_shader = Some(shader);
        Ok(shader)
    }

    fn upload_quad_buffers(&mut self, backend: &mut dyn RenderBackend) -> GoudResult<()> {
        let vertex_bytes = vertex_slice_as_bytes(&self.quad_vertices);
        match self.quad_vertex_buffer {
            Some(buffer) => backend
                .update_buffer(buffer, 0, vertex_bytes)
                .map_err(|e| {
                    GoudError::BufferCreationFailed(format!("ui VBO update failed: {e}"))
                })?,
            None => {
                let buffer = backend
                    .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, vertex_bytes)
                    .map_err(|e| {
                        GoudError::BufferCreationFailed(format!("ui VBO create failed: {e}"))
                    })?;
                self.quad_vertex_buffer = Some(buffer);
            }
        }

        let index_bytes = index_slice_as_bytes(&self.quad_indices);
        match self.quad_index_buffer {
            Some(buffer) => backend.update_buffer(buffer, 0, index_bytes).map_err(|e| {
                GoudError::BufferCreationFailed(format!("ui IBO update failed: {e}"))
            })?,
            None => {
                let buffer = backend
                    .create_buffer(BufferType::Index, BufferUsage::Dynamic, index_bytes)
                    .map_err(|e| {
                        GoudError::BufferCreationFailed(format!("ui IBO create failed: {e}"))
                    })?;
                self.quad_index_buffer = Some(buffer);
            }
        }

        Ok(())
    }

    fn end_quad_frame(
        &mut self,
        backend: &mut dyn RenderBackend,
        viewport: (u32, u32),
    ) -> GoudResult<()> {
        if self.quad_batches.is_empty() {
            return Ok(());
        }

        self.upload_quad_buffers(backend)?;
        let shader = self.ensure_quad_shader(backend)?;
        backend
            .bind_shader(shader)
            .map_err(|e| GoudError::DrawCallFailed(format!("ui shader bind failed: {e}")))?;

        if let Some(location) = backend.get_uniform_location(shader, "u_texture") {
            backend.set_uniform_int(location, 0);
        }

        if let Some(location) = backend.get_uniform_location(shader, "u_viewport") {
            let safe_width = viewport.0.max(1) as f32;
            let safe_height = viewport.1.max(1) as f32;
            backend.set_uniform_vec2(location, safe_width, safe_height);
        }

        if let Some(vbo) = self.quad_vertex_buffer {
            backend
                .bind_buffer(vbo)
                .map_err(|e| GoudError::DrawCallFailed(format!("ui VBO bind failed: {e}")))?;
            backend.set_vertex_attributes(&SpriteVertex::layout());
        }

        if let Some(ibo) = self.quad_index_buffer {
            backend
                .bind_buffer(ibo)
                .map_err(|e| GoudError::DrawCallFailed(format!("ui IBO bind failed: {e}")))?;
        }

        let draw_calls: Vec<(TextureHandle, u32, u32)> = self
            .quad_batches
            .iter()
            .map(|batch| (batch.gpu_texture, batch.index_start, batch.index_count))
            .collect();

        for (gpu_texture, index_start, index_count) in draw_calls {
            backend
                .bind_texture(gpu_texture, 0)
                .map_err(|e| GoudError::DrawCallFailed(format!("ui texture bind failed: {e}")))?;
            backend
                .draw_indexed(
                    PrimitiveTopology::Triangles,
                    index_count,
                    (index_start as usize) * std::mem::size_of::<u32>(),
                )
                .map_err(|e| GoudError::DrawCallFailed(format!("ui draw_indexed failed: {e}")))?;
        }

        Ok(())
    }
}

impl Default for UiRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Debug for UiRenderSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiRenderSystem")
            .field("stats", &self.stats)
            .field("cached_fonts", &self.font_cache.len())
            .finish()
    }
}

pub(crate) use resources::ensure_ui_asset_loaders;

#[cfg(test)]
#[path = "ui_render_system_tests.rs"]
mod tests;
