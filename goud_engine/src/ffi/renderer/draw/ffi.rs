use crate::core::error::set_last_error;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::with_window_state;

use super::super::texture::GoudTextureHandle;
use super::helpers::{map_draw_result, prepare_draw_state, prepare_textured_draw_state};
use super::internal::{draw_quad_internal, draw_sprite_internal, draw_sprite_rect_internal};

/// Draws a textured sprite at the given position.
///
/// This is an immediate-mode draw call - the sprite is rendered immediately
/// and not retained between frames.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `texture` - Texture handle from `goud_texture_load`
/// * `x` - X position (center of sprite)
/// * `y` - Y position (center of sprite)
/// * `width` - Width of the sprite
/// * `height` - Height of the sprite
/// * `rotation` - Rotation in radians
/// * `r`, `g`, `b`, `a` - Color tint (1.0 for no tint)
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_draw_sprite(
    context_id: GoudContextId,
    texture: GoudTextureHandle,
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
    let state_data = match prepare_textured_draw_state(context_id, texture) {
        Ok(state_data) => state_data,
        Err(error) => {
            set_last_error(error);
            return false;
        }
    };

    let result = with_window_state(context_id, |window_state| {
        draw_sprite_internal(
            window_state,
            state_data,
            texture,
            x,
            y,
            width,
            height,
            rotation,
            r,
            g,
            b,
            a,
        )
    });

    map_draw_result(result)
}

/// Draws a textured sprite with a source rectangle for sprite sheet animation.
///
/// This is an immediate-mode draw call that supports sprite sheets by allowing
/// you to specify which portion of the texture to render.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `texture` - Texture handle from `goud_texture_load`
/// * `x` - X position (center of sprite)
/// * `y` - Y position (center of sprite)
/// * `width` - Width of the sprite on screen
/// * `height` - Height of the sprite on screen
/// * `rotation` - Rotation in radians
/// * `src_x` - Source rectangle X offset in normalized UV coordinates (0.0-1.0)
/// * `src_y` - Source rectangle Y offset in normalized UV coordinates (0.0-1.0)
/// * `src_w` - Source rectangle width in normalized UV coordinates (0.0-1.0)
/// * `src_h` - Source rectangle height in normalized UV coordinates (0.0-1.0)
/// * `r`, `g`, `b`, `a` - Color tint (1.0 for no tint)
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Example
///
/// For a 128x128 sprite sheet with 32x32 frames (4x4 grid):
/// - Frame at row 0, col 0: src_x=0.0, src_y=0.0, src_w=0.25, src_h=0.25
/// - Frame at row 0, col 1: src_x=0.25, src_y=0.0, src_w=0.25, src_h=0.25
/// - Frame at row 1, col 0: src_x=0.0, src_y=0.25, src_w=0.25, src_h=0.25
#[no_mangle]
pub extern "C" fn goud_renderer_draw_sprite_rect(
    context_id: GoudContextId,
    texture: GoudTextureHandle,
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
    let state_data = match prepare_textured_draw_state(context_id, texture) {
        Ok(state_data) => state_data,
        Err(error) => {
            set_last_error(error);
            return false;
        }
    };

    let result = with_window_state(context_id, |window_state| {
        draw_sprite_rect_internal(
            window_state,
            state_data,
            texture,
            x,
            y,
            width,
            height,
            rotation,
            src_x,
            src_y,
            src_w,
            src_h,
            r,
            g,
            b,
            a,
        )
    });

    map_draw_result(result)
}

/// Draws a colored quad (no texture) at the given position.
///
/// This is an immediate-mode draw call - the quad is rendered immediately
/// and not retained between frames.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `x` - X position (center of quad)
/// * `y` - Y position (center of quad)
/// * `width` - Width of the quad
/// * `height` - Height of the quad
/// * `r`, `g`, `b`, `a` - Color of the quad
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_draw_quad(
    context_id: GoudContextId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> bool {
    let state_data = match prepare_draw_state(context_id) {
        Ok(state_data) => state_data,
        Err(error) => {
            set_last_error(error);
            return false;
        }
    };

    let result = with_window_state(context_id, |window_state| {
        draw_quad_internal(window_state, state_data, x, y, width, height, r, g, b, a)
    });

    map_draw_result(result)
}
