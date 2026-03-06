//! # Draw Command FFI
//!
//! Immediate-mode draw calls: sprites, sprite sheet rects, and colored quads.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::RenderBackend;

use super::immediate::{
    ensure_immediate_state, model_matrix, ortho_matrix, ImmediateStateData, IMMEDIATE_STATE,
};
use super::texture::{GoudTextureHandle, GOUD_INVALID_TEXTURE};

// ============================================================================
// Public FFI Draw Functions
// ============================================================================

/// Draws a textured sprite at the given position.
///
/// This is an immediate-mode draw call — the sprite is rendered immediately
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
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if texture == GOUD_INVALID_TEXTURE {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    // Ensure immediate-mode resources are initialized
    if let Err(e) = ensure_immediate_state(context_id) {
        set_last_error(e);
        return false;
    }

    let state_data = match extract_state(context_id) {
        Some(data) => data,
        None => {
            set_last_error(GoudError::InvalidContext);
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
/// * `src_x` - Source rectangle X offset in normalized UV coordinates (0.0–1.0)
/// * `src_y` - Source rectangle Y offset in normalized UV coordinates (0.0–1.0)
/// * `src_w` - Source rectangle width in normalized UV coordinates (0.0–1.0)
/// * `src_h` - Source rectangle height in normalized UV coordinates (0.0–1.0)
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
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if texture == GOUD_INVALID_TEXTURE {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    // Ensure immediate-mode resources are initialized
    if let Err(e) = ensure_immediate_state(context_id) {
        set_last_error(e);
        return false;
    }

    let state_data = match extract_state(context_id) {
        Some(data) => data,
        None => {
            set_last_error(GoudError::InvalidContext);
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
/// This is an immediate-mode draw call — the quad is rendered immediately
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
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Ensure immediate-mode resources are initialized
    if let Err(e) = ensure_immediate_state(context_id) {
        set_last_error(e);
        return false;
    }

    let state_data = match extract_state(context_id) {
        Some(data) => data,
        None => {
            set_last_error(GoudError::InvalidContext);
            return false;
        }
    };

    let result = with_window_state(context_id, |window_state| {
        draw_quad_internal(window_state, state_data, x, y, width, height, r, g, b, a)
    });

    map_draw_result(result)
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Extracts state data from thread-local storage for a given context.
fn extract_state(context_id: GoudContextId) -> Option<ImmediateStateData> {
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

/// Maps the draw result (Option<Result<(), GoudError>>) to a bool FFI return value.
fn map_draw_result(result: Option<Result<(), GoudError>>) -> bool {
    match result {
        Some(Ok(())) => true,
        Some(Err(e)) => {
            set_last_error(e);
            false
        }
        None => {
            set_last_error(GoudError::InvalidContext);
            false
        }
    }
}

/// Internal function to draw a sprite (delegates to `draw_sprite_rect_internal` with full UV).
fn draw_sprite_internal(
    window_state: &mut crate::ffi::window::WindowState,
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
    // Draw with default UV (full texture: offset 0,0 scale 1,1)
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
        1.0, // Default UV: use full texture
        r,
        g,
        b,
        a,
    )
}

/// Internal function to draw a sprite with source rectangle.
fn draw_sprite_rect_internal(
    window_state: &mut crate::ffi::window::WindowState,
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

    // Get framebuffer size for viewport (handles HiDPI/Retina displays)
    let (fb_width, fb_height) = window_state.get_framebuffer_size();

    // Get logical window size for projection matrix
    let (win_width, win_height) = window_state.get_size();

    // Set viewport to framebuffer size (required for HiDPI)
    unsafe {
        // SAFETY: gl::Viewport is always safe to call with valid dimensions
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
    }

    let backend = window_state.backend_mut();

    // Create orthographic projection using logical window coordinates
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);

    // Create model matrix
    let model = model_matrix(x, y, width, height, rotation);

    // Unpack texture handle
    let tex_index = (texture & 0xFFFFFFFF) as u32;
    let tex_generation = ((texture >> 32) & 0xFFFFFFFF) as u32;
    let tex_handle = TextureHandle::new(tex_index, tex_generation);

    // Bind VAO (includes vertex buffer, index buffer, and vertex attributes)
    unsafe {
        // SAFETY: vao was created by ensure_immediate_state and is valid for this context
        gl::BindVertexArray(vao);
    }

    // Bind shader
    backend.bind_shader(shader)?;

    // Set uniforms
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_mat4(u_model, &model);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 1); // true
    backend.set_uniform_int(u_texture, 0); // texture unit 0

    // Set UV transform uniforms for sprite sheet support
    backend.set_uniform_vec2(u_uv_offset, uv_x, uv_y);
    backend.set_uniform_vec2(u_uv_scale, uv_w, uv_h);

    // Bind texture
    backend.bind_texture(tex_handle, 0)?;

    // Draw
    backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?;

    Ok(())
}

/// Internal function to draw a colored quad (no texture).
fn draw_quad_internal(
    window_state: &mut crate::ffi::window::WindowState,
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

    // Get framebuffer size for viewport (handles HiDPI/Retina displays)
    let (fb_width, fb_height) = window_state.get_framebuffer_size();

    // Get logical window size for projection matrix
    let (win_width, win_height) = window_state.get_size();

    // Set viewport to framebuffer size (required for HiDPI)
    unsafe {
        // SAFETY: gl::Viewport is always safe to call with valid dimensions
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
    }

    let backend = window_state.backend_mut();

    // Create orthographic projection (screen coordinates)
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);

    // Create model matrix (no rotation for simple quads)
    let model = model_matrix(x, y, width, height, 0.0);

    // Bind VAO (includes vertex buffer, index buffer, and vertex attributes)
    unsafe {
        // SAFETY: vao was created by ensure_immediate_state and is valid for this context
        gl::BindVertexArray(vao);
    }

    // Bind shader
    backend.bind_shader(shader)?;

    // Set uniforms
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_mat4(u_model, &model);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 0); // false - no texture

    // Draw
    backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?;

    Ok(())
}
