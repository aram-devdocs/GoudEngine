use crate::core::error::GoudError;
use crate::ffi::window::WindowState;
use crate::libs::graphics::backend::{
    BlendFactor, BufferOps, DrawOps, RenderBackend, ShaderOps, StateOps, TextureOps,
};

use super::super::immediate::{model_matrix, ortho_matrix, ImmediateStateData};
use super::super::texture::GoudTextureHandle;

/// Converts pixel-space source rectangle coordinates to normalized UV coordinates.
///
/// When `src_w` or `src_h` is 0, the corresponding UV dimension maps to the
/// full texture extent (1.0), matching the "use full texture" convention.
pub(crate) fn pixel_to_uv(
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    tex_w: u32,
    tex_h: u32,
) -> (f32, f32, f32, f32) {
    let tw = tex_w as f32;
    let th = tex_h as f32;
    if tw == 0.0 || th == 0.0 {
        return (0.0, 0.0, 1.0, 1.0);
    }
    let uv_x = src_x / tw;
    let uv_y = src_y / th;
    let uv_w = if src_w == 0.0 { 1.0 } else { src_w / tw };
    let uv_h = if src_h == 0.0 { 1.0 } else { src_h / th };
    (uv_x, uv_y, uv_w, uv_h)
}

/// Internal function to draw a sprite (delegates to `draw_sprite_rect_internal` with full UV).
pub(crate) fn draw_sprite_internal(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
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
) -> Result<(), GoudError> {
    draw_sprite_rect_internal(
        window_state,
        state_data,
        texture,
        x,
        y,
        width,
        height,
        rotation,
        0.0,
        0.0,
        1.0,
        1.0,
        r,
        g,
        b,
        a,
    )
}

/// Internal function to draw a sprite with source rectangle.
pub(crate) fn draw_sprite_rect_internal(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    texture: GoudTextureHandle,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    uv_x: f32,
    uv_y: f32,
    uv_w: f32,
    uv_h: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), GoudError> {
    use crate::libs::graphics::backend::types::{PrimitiveTopology, TextureHandle};

    let (
        shader,
        vertex_buffer,
        index_buffer,
        vertex_layout,
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

    let backend = window_state.backend_mut();
    backend.set_viewport(0, 0, fb_width, fb_height);
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);
    let model = model_matrix(x, y, width, height, rotation);

    let tex_index = (texture & 0xFFFFFFFF) as u32;
    let tex_generation = ((texture >> 32) & 0xFFFFFFFF) as u32;
    let tex_handle = TextureHandle::new(tex_index, tex_generation);

    backend.enable_blending();
    backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    backend.bind_default_vertex_array();
    backend.bind_buffer(vertex_buffer)?;
    backend.bind_buffer(index_buffer)?;
    backend.set_vertex_attributes(&vertex_layout);

    backend.bind_shader(shader)?;
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_mat4(u_model, &model);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 1);
    backend.set_uniform_int(u_texture, 0);
    backend.set_uniform_vec2(u_uv_offset, uv_x, uv_y);
    backend.set_uniform_vec2(u_uv_scale, uv_w, uv_h);
    backend.bind_texture(tex_handle, 0)?;
    backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?;

    Ok(())
}

/// Internal function to draw a colored quad (no texture).
pub(crate) fn draw_quad_internal(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), GoudError> {
    draw_quad_rotated_internal(
        window_state,
        state_data,
        x,
        y,
        width,
        height,
        0.0,
        r,
        g,
        b,
        a,
    )
}

/// Internal function to draw a rotated colored quad (no texture).
pub(crate) fn draw_quad_rotated_internal(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), GoudError> {
    use crate::libs::graphics::backend::types::PrimitiveTopology;

    let (
        shader,
        vertex_buffer,
        index_buffer,
        vertex_layout,
        u_projection,
        u_model,
        u_color,
        u_use_texture,
        _u_texture,
        _u_uv_offset,
        _u_uv_scale,
    ) = state_data;

    let (fb_width, fb_height) = window_state.get_framebuffer_size();
    let (win_width, win_height) = window_state.get_size();

    let backend = window_state.backend_mut();
    backend.set_viewport(0, 0, fb_width, fb_height);
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);
    let model = model_matrix(x, y, width, height, rotation);

    backend.enable_blending();
    backend.set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
    backend.bind_default_vertex_array();
    backend.bind_buffer(vertex_buffer)?;
    backend.bind_buffer(index_buffer)?;
    backend.set_vertex_attributes(&vertex_layout);

    backend.bind_shader(shader)?;
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_mat4(u_model, &model);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 0);
    backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::pixel_to_uv;

    #[test]
    fn test_pixel_to_uv_conversion() {
        let (uv_x, uv_y, uv_w, uv_h) = pixel_to_uv(64.0, 32.0, 32.0, 32.0, 256, 256);
        assert!((uv_x - 0.25).abs() < f32::EPSILON);
        assert!((uv_y - 0.125).abs() < f32::EPSILON);
        assert!((uv_w - 0.125).abs() < f32::EPSILON);
        assert!((uv_h - 0.125).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pixel_to_uv_identity() {
        // Full texture should map to (0,0,1,1)
        let (uv_x, uv_y, uv_w, uv_h) = pixel_to_uv(0.0, 0.0, 128.0, 64.0, 128, 64);
        assert!((uv_x).abs() < f32::EPSILON);
        assert!((uv_y).abs() < f32::EPSILON);
        assert!((uv_w - 1.0).abs() < f32::EPSILON);
        assert!((uv_h - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pixel_to_uv_zero_src_uses_full_texture() {
        let (uv_x, uv_y, uv_w, uv_h) = pixel_to_uv(0.0, 0.0, 0.0, 0.0, 256, 256);
        assert!((uv_x).abs() < f32::EPSILON);
        assert!((uv_y).abs() < f32::EPSILON);
        assert!((uv_w - 1.0).abs() < f32::EPSILON);
        assert!((uv_h - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pixel_to_uv_zero_texture_size() {
        let (uv_x, uv_y, uv_w, uv_h) = pixel_to_uv(10.0, 10.0, 32.0, 32.0, 0, 0);
        assert!((uv_x).abs() < f32::EPSILON);
        assert!((uv_y).abs() < f32::EPSILON);
        assert!((uv_w - 1.0).abs() < f32::EPSILON);
        assert!((uv_h - 1.0).abs() < f32::EPSILON);
    }
}
