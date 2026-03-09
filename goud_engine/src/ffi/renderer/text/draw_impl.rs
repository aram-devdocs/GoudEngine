use crate::core::error::GoudError;
use crate::core::handle::Handle;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::WindowState;
use crate::libs::graphics::backend::types::{PrimitiveTopology, TextureHandle};
use crate::libs::graphics::backend::{DrawOps, ShaderOps, TextureOps};
use crate::rendering::text::{
    layout_shaped_text, shape_text, GlyphAtlas, TextDirection, TextLayoutConfig,
};

use super::super::immediate::{model_matrix, ortho_matrix, ImmediateStateData, IMMEDIATE_STATE};
use super::{FontMarker, GoudFontHandle, FONT_STATES};

pub(super) fn draw_text_internal(
    window_state: &mut WindowState,
    context_id: GoudContextId,
    state_data: ImmediateStateData,
    font_handle: GoudFontHandle,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    config: TextLayoutConfig,
    direction: TextDirection,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), GoudError> {
    let context_key = (context_id.index(), context_id.generation());
    let typed_handle = Handle::<FontMarker>::from_u64(font_handle);
    if !typed_handle.is_valid() {
        return Err(GoudError::InvalidHandle);
    }

    FONT_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let state = states
            .get_mut(&context_key)
            .ok_or(GoudError::InvalidHandle)?;
        let loaded_font = state
            .fonts
            .get_mut(typed_handle)
            .ok_or(GoudError::InvalidHandle)?;

        let shaped = shape_text(text, &loaded_font.font_bytes, font_size, direction)
            .map_err(GoudError::ResourceInvalidFormat)?;

        let size_key = font_size.round().max(1.0) as u32;
        if !loaded_font.atlases.contains_key(&size_key) {
            let new_atlas = GlyphAtlas::generate(&loaded_font.font, font_size)
                .map_err(GoudError::ResourceInvalidFormat)?;
            loaded_font.atlases.insert(size_key, new_atlas);
        }

        let atlas = loaded_font
            .atlases
            .get_mut(&size_key)
            .expect("atlas inserted above");

        atlas
            .ensure_glyph_indices(&loaded_font.font, shaped.glyph_indices())
            .map_err(GoudError::ResourceInvalidFormat)?;
        let layout = layout_shaped_text(&shaped, atlas, font_size, &config);
        if layout.glyphs.is_empty() {
            return Ok(());
        }

        let texture = atlas
            .ensure_gpu_texture(window_state.backend_mut())
            .map_err(GoudError::TextureCreationFailed)?;
        draw_layout(window_state, state_data, texture, &layout, x, y, r, g, b, a)
    })
}

fn draw_layout(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    texture: TextureHandle,
    layout: &crate::rendering::text::TextLayoutResult,
    x: f32,
    y: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), GoudError> {
    let (
        shader,
        _vertex_buffer,
        _index_buffer,
        vao,
        u_projection,
        u_model,
        u_color,
        u_use_texture,
        u_texture,
        u_uv_offset,
        u_uv_scale,
    ) = state_data;

    let (fb_width, fb_height) = window_state.get_framebuffer_size();
    let (win_width, win_height) = window_state.get_size();

    // SAFETY: gl::Viewport is safe with integer dimensions from the active context.
    unsafe {
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
    }

    let backend = window_state.backend_mut();
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);

    // SAFETY: vao is created and owned by ensure_immediate_state for this context.
    unsafe {
        gl::BindVertexArray(vao);
    }

    backend.bind_shader(shader)?;
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 1);
    backend.set_uniform_int(u_texture, 0);
    backend.bind_texture(texture, 0)?;

    for glyph in &layout.glyphs {
        if glyph.size_x <= 0.0 || glyph.size_y <= 0.0 {
            continue;
        }

        let center_x = x + glyph.x + glyph.size_x * 0.5;
        let center_y = y + glyph.y + glyph.size_y * 0.5;
        let model = model_matrix(center_x, center_y, glyph.size_x, glyph.size_y, 0.0);
        backend.set_uniform_mat4(u_model, &model);
        backend.set_uniform_vec2(u_uv_offset, glyph.uv_rect.u_min, glyph.uv_rect.v_min);
        backend.set_uniform_vec2(
            u_uv_scale,
            glyph.uv_rect.u_max - glyph.uv_rect.u_min,
            glyph.uv_rect.v_max - glyph.uv_rect.v_min,
        );
        backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?;
    }

    Ok(())
}

pub(super) fn extract_state(context_id: GoudContextId) -> Option<ImmediateStateData> {
    let context_key = (context_id.index(), context_id.generation());
    IMMEDIATE_STATE.with(|cell| {
        let states = cell.borrow();
        states.get(&context_key).map(|s| {
            (
                s.shader,
                s.vertex_buffer,
                s.index_buffer,
                s.vao,
                s.u_projection,
                s.u_model,
                s.u_color,
                s.u_use_texture,
                s.u_texture,
                s.u_uv_offset,
                s.u_uv_scale,
            )
        })
    })
}
