use super::{create_immediate_render_state, model_matrix, ortho_matrix};
use crate::assets::loaders::FontAsset;
use crate::core::math::{Color, Vec2};
use crate::core::types::TextAlignment;
use crate::libs::graphics::backend::{
    BlendFactor, BufferOps, ClearOps, DrawOps, FrameOps, RenderBackend, ShaderOps, StateOps,
    TextureOps,
};
use crate::rendering::text::{TextBatch, TextLayoutConfig};
use crate::sdk::GoudGame;

// NOTE: FFI wrappers are hand-written in ffi/renderer.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl GoudGame {
    /// Begins a new rendering frame. Call before any drawing operations.
    pub fn begin_render(&mut self) -> bool {
        let viewport = self.render_viewport();
        let backend = match self.render_backend.as_mut() {
            Some(b) => b,
            None => return false,
        };
        if backend.begin_frame().is_err() {
            return false;
        }
        backend.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
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
        let clamped_width = width.max(1);
        let clamped_height = height.max(1);
        #[cfg(feature = "native")]
        {
            if let Some((_, viewport)) = self.bound_render_target_viewport.as_mut() {
                viewport.x = x;
                viewport.y = y;
                viewport.width = clamped_width;
                viewport.height = clamped_height;
            } else {
                self.render_viewport.x = x;
                self.render_viewport.y = y;
                self.render_viewport.width = clamped_width;
                self.render_viewport.height = clamped_height;
            }
        }
        if let Some(backend) = self.render_backend.as_mut() {
            backend.set_viewport(x, y, clamped_width, clamped_height);
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

        let viewport = self.render_viewport();

        let (shader, vertex_layout, u_proj, u_model, u_color, u_use_tex, u_tex, u_uv_off, u_uv_sc) = (
            state.shader,
            state.vertex_layout.clone(),
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
        backend.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        backend.bind_default_vertex_array();
        if backend.bind_buffer(vertex_buffer).is_err() || backend.bind_buffer(index_buffer).is_err()
        {
            return false;
        }
        backend.set_vertex_attributes(&vertex_layout);

        let projection = ortho_matrix(
            0.0,
            viewport.logical_width as f32,
            viewport.logical_height as f32,
            0.0,
        );
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

        let viewport = self.render_viewport();

        let (shader, vertex_layout, u_proj, u_model, u_color, u_use_tex) = (
            state.shader,
            state.vertex_layout.clone(),
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
        backend.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        backend.bind_default_vertex_array();
        if backend.bind_buffer(vertex_buffer).is_err() || backend.bind_buffer(index_buffer).is_err()
        {
            return false;
        }
        backend.set_vertex_attributes(&vertex_layout);

        let projection = ortho_matrix(
            0.0,
            viewport.logical_width as f32,
            viewport.logical_height as f32,
            0.0,
        );
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

        let viewport = self.render_viewport();
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

        let config = TextLayoutConfig {
            max_width: (max_width > 0.0).then_some(max_width),
            line_spacing,
            alignment: TextAlignment::Left,
        };
        let transform = crate::ecs::components::Transform2D::from_position(Vec2::new(x, y));
        let color = Color::new(r, g, b, a);

        let batch = self.text_batch.get_or_insert_with(TextBatch::new);
        backend.disable_depth_test();
        backend.enable_blending();
        backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        backend.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        let Some((layout, texture)) = (match batch.resolve_truetype_font(
            text,
            font_size,
            &config,
            &font_handle,
            asset_server,
            backend,
        ) {
            Ok(result) => result,
            Err(_) => return false,
        }) else {
            return false;
        };
        if layout.glyphs.is_empty() {
            return true;
        }
        batch
            .draw_prepared_layout_frame(
                backend,
                viewport.logical_size(),
                &layout,
                color,
                &transform,
                texture,
            )
            .is_ok()
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

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;
    use crate::sdk::game_config::GameConfig;

    #[test]
    fn set_viewport_updates_render_viewport_state() {
        let mut game = GoudGame::new(GameConfig::default()).expect("game should construct");

        game.set_viewport(12, 34, 640, 360);

        let viewport = game.render_viewport();
        assert_eq!(viewport.x, 12);
        assert_eq!(viewport.y, 34);
        assert_eq!(viewport.width, 640);
        assert_eq!(viewport.height, 360);
        assert_eq!(viewport.logical_size(), (800, 600));
    }

    #[test]
    fn set_viewport_updates_bound_render_target_override() {
        let mut game = GoudGame::new(GameConfig::default()).expect("game should construct");
        game.bound_render_target_viewport =
            Some((5, crate::rendering::RenderViewport::fullscreen((64, 32))));

        game.set_viewport(3, 4, 16, 20);

        let viewport = game.render_viewport();
        assert_eq!(viewport.x, 3);
        assert_eq!(viewport.y, 4);
        assert_eq!(viewport.width, 16);
        assert_eq!(viewport.height, 20);
        assert_eq!(viewport.logical_size(), (64, 32));
    }

    #[test]
    fn set_viewport_clamps_zero_dimensions() {
        let mut game = GoudGame::new(GameConfig::default()).expect("game should construct");

        game.set_viewport(1, 2, 0, 0);

        let viewport = game.render_viewport();
        assert_eq!(viewport.x, 1);
        assert_eq!(viewport.y, 2);
        assert_eq!(viewport.width, 1);
        assert_eq!(viewport.height, 1);
    }
}
