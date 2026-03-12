use crate::core::error::GoudError;
use crate::core::handle::Handle;
use crate::core::math::{Color, Vec2};
use crate::ecs::components::Transform2D;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::WindowState;
use crate::libs::graphics::backend::types::TextureHandle;
use crate::rendering::text::{
    layout_shaped_text, shape_text, GlyphAtlas, TextBatch, TextDirection, TextLayoutConfig,
};

use super::{FontMarker, GoudFontHandle, FONT_STATES};

pub(super) fn draw_text_internal(
    window_state: &mut WindowState,
    context_id: GoudContextId,
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
        draw_layout(
            &mut state.text_batch,
            window_state,
            texture,
            &layout,
            x,
            y,
            r,
            g,
            b,
            a,
        )
    })
}

fn draw_layout(
    text_batch: &mut TextBatch,
    window_state: &mut WindowState,
    texture: TextureHandle,
    layout: &crate::rendering::text::TextLayoutResult,
    x: f32,
    y: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), GoudError> {
    let transform = Transform2D::from_position(Vec2::new(x, y));
    let color = Color::new(r, g, b, a);
    let viewport =
        select_text_viewport(window_state.get_size(), window_state.get_framebuffer_size());
    text_batch
        .draw_prepared_layout_frame(
            window_state.backend_mut(),
            viewport,
            layout,
            color,
            &transform,
            texture,
        )
        .map_err(GoudError::DrawCallFailed)
}

fn select_text_viewport(logical_size: (u32, u32), _framebuffer_size: (u32, u32)) -> (u32, u32) {
    logical_size
}

#[cfg(test)]
mod tests {
    use super::select_text_viewport;

    #[test]
    fn ffi_text_viewport_uses_logical_window_size() {
        let logical = (1280, 720);
        let framebuffer = (2560, 1440);
        assert_eq!(select_text_viewport(logical, framebuffer), logical);
    }
}
