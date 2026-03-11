//! # SDK Rendering API
//!
//! Provides methods on [`GoudGame`] for 2D rendering operations
//! including frame management, immediate-mode sprite/quad drawing, and render
//! state control.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop platform with OpenGL).

use super::GoudGame;
use crate::assets::loaders::FontAsset;
use crate::core::error::{GoudError, GoudResult};
use crate::core::math::{Color, Vec2};
use crate::core::types::TextAlignment;
use crate::libs::graphics::backend::{
    BlendFactor, BufferOps, ClearOps, DrawOps, FrameOps, ShaderOps, StateOps, TextureOps,
};
use crate::rendering::text::{DirectTextDrawRequest, TextBatch};

// Re-export rendering types for SDK users
pub use crate::rendering::sprite_batch::SpriteBatchConfig;

// Re-export 3D types (they live in rendering_3d but users expect them here)
pub use crate::libs::graphics::renderer3d::{
    FogConfig, GridConfig, GridRenderMode, Light, LightType, PrimitiveCreateInfo, PrimitiveType,
    SkyboxConfig,
};

// =============================================================================
// Immediate-Mode Render State
// =============================================================================

/// GPU resources for immediate-mode sprite and quad rendering.
///
/// Created when the OpenGL backend is initialized and stored in GoudGame.
/// Contains the compiled shader program, vertex/index buffers, VAO, and
/// cached uniform locations needed by `draw_sprite` and `draw_quad`.
pub struct ImmediateRenderState {
    /// Shader program for sprite rendering
    pub(crate) shader: crate::libs::graphics::backend::types::ShaderHandle,
    /// Vertex buffer for quad rendering.
    pub(crate) vertex_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Index buffer for quad rendering.
    pub(crate) index_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Vertex Array Object (required for macOS Core Profile)
    pub(crate) vao: u32,
    /// Cached uniform locations
    pub(crate) u_projection: i32,
    pub(crate) u_model: i32,
    pub(crate) u_color: i32,
    pub(crate) u_use_texture: i32,
    pub(crate) u_texture: i32,
    pub(crate) u_uv_offset: i32,
    pub(crate) u_uv_scale: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ImmediateQuadVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

// SAFETY: ImmediateQuadVertex is plain-old vertex data.
unsafe impl bytemuck::Pod for ImmediateQuadVertex {}
unsafe impl bytemuck::Zeroable for ImmediateQuadVertex {}

pub(crate) fn create_immediate_render_state(
    backend: &mut crate::libs::graphics::backend::opengl::OpenGLBackend,
) -> GoudResult<ImmediateRenderState> {
    use crate::libs::graphics::backend::types::{BufferType, BufferUsage};

    let shader = backend.create_shader(SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER)?;

    let u_projection = backend
        .get_uniform_location(shader, "u_projection")
        .unwrap_or(-1);
    let u_model = backend
        .get_uniform_location(shader, "u_model")
        .unwrap_or(-1);
    let u_color = backend
        .get_uniform_location(shader, "u_color")
        .unwrap_or(-1);
    let u_use_texture = backend
        .get_uniform_location(shader, "u_use_texture")
        .unwrap_or(-1);
    let u_texture = backend
        .get_uniform_location(shader, "u_texture")
        .unwrap_or(-1);
    let u_uv_offset = backend
        .get_uniform_location(shader, "u_uv_offset")
        .unwrap_or(-1);
    let u_uv_scale = backend
        .get_uniform_location(shader, "u_uv_scale")
        .unwrap_or(-1);

    let mut vao = 0u32;
    // SAFETY: An OpenGL context is current for the backend lifetime. The stack
    // variable is valid output storage and the generated VAO is immediately bound.
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let vertices: [ImmediateQuadVertex; 4] = [
        ImmediateQuadVertex {
            position: [-0.5, -0.5],
            tex_coords: [0.0, 0.0],
        },
        ImmediateQuadVertex {
            position: [0.5, -0.5],
            tex_coords: [1.0, 0.0],
        },
        ImmediateQuadVertex {
            position: [0.5, 0.5],
            tex_coords: [1.0, 1.0],
        },
        ImmediateQuadVertex {
            position: [-0.5, 0.5],
            tex_coords: [0.0, 1.0],
        },
    ];
    let vertex_buffer = backend.create_buffer(
        BufferType::Vertex,
        BufferUsage::Static,
        bytemuck::cast_slice(&vertices),
    )?;
    backend.bind_buffer(vertex_buffer)?;

    configure_immediate_vertex_layout();

    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
    let index_buffer = backend.create_buffer(
        BufferType::Index,
        BufferUsage::Static,
        bytemuck::cast_slice(&indices),
    )?;
    backend.bind_buffer(index_buffer)?;

    // SAFETY: Binding VAO 0 unbinds the current vertex array in OpenGL.
    unsafe {
        gl::BindVertexArray(0);
    }

    Ok(ImmediateRenderState {
        shader,
        vertex_buffer,
        index_buffer,
        vao,
        u_projection,
        u_model,
        u_color,
        u_use_texture,
        u_texture,
        u_uv_offset,
        u_uv_scale,
    })
}

fn configure_immediate_vertex_layout() {
    // SAFETY: The immediate-mode VAO and vertex buffer are already bound by the caller,
    // and this layout matches `ImmediateQuadVertex`.
    unsafe {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 16, std::ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 16, 8 as *const std::ffi::c_void);
        gl::DisableVertexAttribArray(2);
        gl::VertexAttrib4f(2, 1.0, 1.0, 1.0, 1.0);
    }
}

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
// Renderer FFI (single annotated impl block for all goud_renderer_* functions)
// =============================================================================

// NOTE: FFI wrappers are hand-written in ffi/renderer.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl GoudGame {
    /// Begins a new rendering frame. Call before any drawing operations.
    pub fn begin_render(&mut self) -> bool {
        let backend = match self.render_backend.as_mut() {
            Some(b) => b,
            None => return false,
        };
        if backend.begin_frame().is_err() {
            return false;
        }
        let (fb_w, fb_h) = self.get_framebuffer_size();
        // SAFETY: OpenGL viewport call is safe when a context is current.
        unsafe {
            gl::Viewport(0, 0, fb_w as i32, fb_h as i32);
        }
        true
    }

    /// Ends the current rendering frame.
    pub fn end_render(&mut self) -> bool {
        match self.render_backend.as_mut() {
            Some(b) => b.end_frame().is_ok(),
            None => false,
        }
    }

    /// Sets the viewport rectangle for rendering.
    pub fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        if let Some(backend) = self.render_backend.as_mut() {
            backend.set_viewport(x, y, width, height);
        }
    }

    /// Enables alpha blending for transparent sprites.
    pub fn enable_blending(&mut self) {
        if let Some(backend) = self.render_backend.as_mut() {
            backend.enable_blending();
        }
    }

    /// Disables alpha blending.
    pub fn disable_blending(&mut self) {
        if let Some(backend) = self.render_backend.as_mut() {
            backend.disable_blending();
        }
    }

    /// Enables depth testing.
    pub fn enable_depth_test(&mut self) {
        if let Some(backend) = self.render_backend.as_mut() {
            backend.enable_depth_test();
        }
    }

    /// Disables depth testing.
    pub fn disable_depth_test(&mut self) {
        if let Some(backend) = self.render_backend.as_mut() {
            backend.disable_depth_test();
        }
    }

    /// Clears the depth buffer.
    pub fn clear_depth(&mut self) {
        if let Some(backend) = self.render_backend.as_mut() {
            backend.clear_depth();
        }
    }

    /// Draws a textured sprite at the given position (immediate mode).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite(
        &mut self,
        texture: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        self.draw_sprite_rect(
            texture, x, y, width, height, rotation, 0.0, 0.0, 1.0, 1.0, r, g, b, a,
        )
    }

    /// Draws a textured sprite with a source rectangle for sprite sheet animation.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite_rect(
        &mut self,
        texture: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        src_x: f32,
        src_y: f32,
        src_w: f32,
        src_h: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        use crate::libs::graphics::backend::types::{PrimitiveTopology, TextureHandle};

        if !self.ensure_immediate_state() {
            return false;
        }
        let state = match &self.immediate_state {
            Some(s) => s,
            None => return false,
        };

        let (fb_w, fb_h) = self.get_framebuffer_size();
        let (win_w, win_h) = self.get_window_size();

        // Cache values from immediate state before borrowing backend
        let (shader, vao, u_proj, u_model, u_color, u_use_tex, u_tex, u_uv_off, u_uv_sc) = (
            state.shader,
            state.vao,
            state.u_projection,
            state.u_model,
            state.u_color,
            state.u_use_texture,
            state.u_texture,
            state.u_uv_offset,
            state.u_uv_scale,
        );
        let vertex_buffer = state.vertex_buffer;
        let index_buffer = state.index_buffer;

        let backend = match self.render_backend.as_mut() {
            Some(b) => b,
            None => return false,
        };

        backend.enable_blending();
        backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        if backend.bind_buffer(vertex_buffer).is_err() || backend.bind_buffer(index_buffer).is_err()
        {
            return false;
        }

        // SAFETY: OpenGL calls require a current context.
        unsafe {
            gl::Viewport(0, 0, fb_w as i32, fb_h as i32);
            gl::BindVertexArray(vao);
        }
        configure_immediate_vertex_layout();

        let projection = ortho_matrix(0.0, win_w as f32, win_h as f32, 0.0);
        let model = model_matrix(x, y, width, height, rotation);

        let tex_index = (texture & 0xFFFF_FFFF) as u32;
        let tex_gen = ((texture >> 32) & 0xFFFF_FFFF) as u32;
        let tex_handle = TextureHandle::new(tex_index, tex_gen);

        if backend.bind_shader(shader).is_err() {
            return false;
        }
        backend.set_uniform_mat4(u_proj, &projection);
        backend.set_uniform_mat4(u_model, &model);
        backend.set_uniform_vec4(u_color, r, g, b, a);
        backend.set_uniform_int(u_use_tex, 1);
        backend.set_uniform_int(u_tex, 0);
        backend.set_uniform_vec2(u_uv_off, src_x, src_y);
        backend.set_uniform_vec2(u_uv_sc, src_w, src_h);

        if backend.bind_texture(tex_handle, 0).is_err() {
            return false;
        }
        backend
            .draw_indexed(PrimitiveTopology::Triangles, 6, 0)
            .is_ok()
    }

    /// Draws a colored quad (no texture) at the given position.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_quad(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        use crate::libs::graphics::backend::types::PrimitiveTopology;

        if !self.ensure_immediate_state() {
            return false;
        }
        let state = match &self.immediate_state {
            Some(s) => s,
            None => return false,
        };

        let (fb_w, fb_h) = self.get_framebuffer_size();
        let (win_w, win_h) = self.get_window_size();

        let (shader, vao, u_proj, u_model, u_color, u_use_tex) = (
            state.shader,
            state.vao,
            state.u_projection,
            state.u_model,
            state.u_color,
            state.u_use_texture,
        );
        let vertex_buffer = state.vertex_buffer;
        let index_buffer = state.index_buffer;

        let backend = match self.render_backend.as_mut() {
            Some(b) => b,
            None => return false,
        };

        backend.enable_blending();
        backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        if backend.bind_buffer(vertex_buffer).is_err() || backend.bind_buffer(index_buffer).is_err()
        {
            return false;
        }

        // SAFETY: OpenGL calls require a current context.
        unsafe {
            gl::Viewport(0, 0, fb_w as i32, fb_h as i32);
            gl::BindVertexArray(vao);
        }
        configure_immediate_vertex_layout();

        let projection = ortho_matrix(0.0, win_w as f32, win_h as f32, 0.0);
        let model = model_matrix(x, y, width, height, 0.0);

        if backend.bind_shader(shader).is_err() {
            return false;
        }
        backend.set_uniform_mat4(u_proj, &projection);
        backend.set_uniform_mat4(u_model, &model);
        backend.set_uniform_vec4(u_color, r, g, b, a);
        backend.set_uniform_int(u_use_tex, 0);

        backend
            .draw_indexed(PrimitiveTopology::Triangles, 6, 0)
            .is_ok()
    }

    /// Draws UTF-8 text using a native font asset path.
    ///
    /// `max_width <= 0.0` disables wrapping. Alignment is currently left-only.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &mut self,
        font_path: &str,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        max_width: f32,
        line_spacing: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        if text.is_empty() || font_size <= 0.0 || line_spacing <= 0.0 {
            return false;
        }

        let viewport = self.get_window_size();
        if !self.ensure_immediate_state() {
            return false;
        }
        let vao = match &self.immediate_state {
            Some(state) => state.vao,
            None => return false,
        };
        let asset_server = match self.asset_server.as_mut() {
            Some(server) => server,
            None => return false,
        };
        let backend = match self.render_backend.as_mut() {
            Some(backend) => backend,
            None => return false,
        };

        crate::rendering::ensure_ui_asset_loaders(asset_server);
        let font_handle = asset_server.load::<FontAsset>(font_path);
        if !asset_server.is_loaded(&font_handle) {
            return false;
        }

        let request = DirectTextDrawRequest {
            content: text.to_string(),
            position: Vec2::new(x, y),
            font_handle,
            font_size,
            color: Color::new(r, g, b, a),
            alignment: TextAlignment::Left,
            max_width: (max_width > 0.0).then_some(max_width),
            line_spacing,
        };

        let batch = self.text_batch.get_or_insert_with(TextBatch::new);
        batch.begin();
        backend.disable_depth_test();
        backend.enable_blending();
        backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        // SAFETY: The native OpenGL context is current for this frame and `vao`
        // is owned by the live immediate render state initialized above.
        unsafe {
            gl::BindVertexArray(vao);
        }
        batch
            .draw_text_requests(&[request], asset_server, backend)
            .is_ok()
            && batch.end(backend, viewport).is_ok()
    }

    /// Gets rendering statistics for the current frame.
    ///
    /// Writes default statistics to the provided out-pointer.
    /// Returns `true` on success, `false` if the pointer is null.
    ///
    /// # Safety
    ///
    /// `out_stats` must be a valid, aligned, writable pointer to a
    /// `GoudRenderStats` value, or null (in which case this returns `false`).
    pub unsafe fn get_render_stats(
        &self,
        out_stats: *mut crate::ffi::renderer::GoudRenderStats,
    ) -> bool {
        if out_stats.is_null() {
            return false;
        }
        // SAFETY: Caller guarantees out_stats is a valid pointer.
        unsafe {
            *out_stats = crate::ffi::renderer::GoudRenderStats::default();
        }
        true
    }
}

impl GoudGame {
    fn ensure_immediate_state(&mut self) -> bool {
        if self.immediate_state.is_some() {
            return true;
        }

        let backend = match self.render_backend.as_mut() {
            Some(backend) => backend,
            None => return false,
        };

        match create_immediate_render_state(backend) {
            Ok(state) => {
                self.immediate_state = Some(state);
                true
            }
            Err(_) => false,
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Creates an orthographic projection matrix.
fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
    let near = -1.0f32;
    let far = 1.0f32;
    [
        2.0 / (right - left),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (top - bottom),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (far - near),
        0.0,
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        -(far + near) / (far - near),
        1.0,
    ]
}

/// Creates a model matrix for sprite transformation.
fn model_matrix(x: f32, y: f32, width: f32, height: f32, rotation: f32) -> [f32; 16] {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    [
        width * cos_r,
        width * sin_r,
        0.0,
        0.0,
        -height * sin_r,
        height * cos_r,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        x,
        y,
        0.0,
        1.0,
    ]
}

const SPRITE_VERTEX_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;

uniform mat4 u_projection;
uniform mat4 u_model;
uniform vec2 u_uv_offset;
uniform vec2 u_uv_scale;

out vec2 v_texcoord;

void main() {
    gl_Position = u_projection * u_model * vec4(a_position, 0.0, 1.0);
    v_texcoord = a_texcoord * u_uv_scale + u_uv_offset;
}
"#;

const SPRITE_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec2 v_texcoord;

uniform vec4 u_color;
uniform bool u_use_texture;
uniform sampler2D u_texture;

out vec4 FragColor;

void main() {
    if (u_use_texture) {
        FragColor = texture(u_texture, v_texcoord) * u_color;
    } else {
        FragColor = u_color;
    }
}
"#;

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
        let m = ortho_matrix(0.0, 2.0, 0.0, 2.0);
        assert!((m[0] - 1.0).abs() < 0.001);
        assert!((m[5] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_model_matrix_no_rotation() {
        let m = model_matrix(10.0, 20.0, 5.0, 5.0, 0.0);
        assert!((m[12] - 10.0).abs() < 0.001);
        assert!((m[13] - 20.0).abs() < 0.001);
    }
}
