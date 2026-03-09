use crate::core::error::GoudError;
use crate::core::handle::Handle;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::WindowState;
use crate::libs::graphics::backend::types::TextureHandle;
use crate::rendering::text::{
    layout_shaped_text, shape_text, GlyphAtlas, TextDirection, TextLayoutConfig,
};

use super::super::draw::draw_sprite_rect_internal;
use super::super::immediate::{ImmediateStateData, IMMEDIATE_STATE};
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
    let texture_handle = texture.to_u64();

    for glyph in &layout.glyphs {
        if glyph.size_x <= 0.0 || glyph.size_y <= 0.0 {
            continue;
        }

        let center_x = x + glyph.x + glyph.size_x * 0.5;
        let center_y = y + glyph.y + glyph.size_y * 0.5;
        draw_sprite_rect_internal(
            window_state,
            state_data,
            texture_handle,
            center_x,
            center_y,
            glyph.size_x,
            glyph.size_y,
            0.0,
            glyph.uv_rect.u_min,
            glyph.uv_rect.v_min,
            glyph.uv_rect.u_max - glyph.uv_rect.u_min,
            glyph.uv_rect.v_max - glyph.uv_rect.v_min,
            r,
            g,
            b,
            a,
        )?;
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
