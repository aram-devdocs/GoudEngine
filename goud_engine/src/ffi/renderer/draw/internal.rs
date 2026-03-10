use crate::core::error::GoudError;
use crate::ffi::window::WindowState;
use crate::libs::graphics::backend::{DrawOps, ShaderOps, TextureOps};

use super::super::immediate::{model_matrix, ortho_matrix, ImmediateStateData};
use super::super::texture::GoudTextureHandle;

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

    // SAFETY: gl::Viewport is always safe to call with valid dimensions.
    unsafe {
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
    }

    let backend = window_state.backend_mut();
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);
    let model = model_matrix(x, y, width, height, rotation);

    let tex_index = (texture & 0xFFFFFFFF) as u32;
    let tex_generation = ((texture >> 32) & 0xFFFFFFFF) as u32;
    let tex_handle = TextureHandle::new(tex_index, tex_generation);

    // SAFETY: vao was created by ensure_immediate_state and is valid for this context.
    unsafe {
        gl::BindVertexArray(vao);
    }

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
        _vertex_buffer,
        _index_buffer,
        vao,
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

    // SAFETY: gl::Viewport is always safe to call with valid dimensions.
    unsafe {
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
    }

    let backend = window_state.backend_mut();
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);
    let model = model_matrix(x, y, width, height, rotation);

    // SAFETY: vao was created by ensure_immediate_state and is valid for this context.
    unsafe {
        gl::BindVertexArray(vao);
    }

    backend.bind_shader(shader)?;
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_mat4(u_model, &model);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 0);
    backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?;

    Ok(())
}
