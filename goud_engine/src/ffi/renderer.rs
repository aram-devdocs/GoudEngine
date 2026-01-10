//! # FFI Renderer Module
//!
//! This module provides C-compatible functions for rendering operations.
//! It integrates with the window FFI to provide basic 2D rendering capabilities.
//!
//! ## Design
//!
//! The renderer FFI provides two modes of operation:
//!
//! 1. **Immediate mode**: Draw individual sprites/quads with explicit parameters
//! 2. **ECS mode**: Automatically render all entities with Sprite + Transform2D components
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // In game loop
//! while (!goud_window_should_close(contextId)) {
//!     float deltaTime = goud_window_poll_events(contextId);
//!     
//!     // Clear screen
//!     goud_window_clear(contextId, 0.1f, 0.1f, 0.2f, 1.0f);
//!     
//!     // Begin rendering
//!     goud_renderer_begin(contextId);
//!     
//!     // Draw sprites (immediate mode)
//!     goud_renderer_draw_quad(contextId, x, y, width, height, r, g, b, a);
//!     
//!     // Or draw all ECS sprites
//!     goud_renderer_draw_ecs_sprites(contextId);
//!     
//!     // End rendering
//!     goud_renderer_end(contextId);
//!     
//!     goud_window_swap_buffers(contextId);
//! }
//! ```

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::RenderBackend;

// ============================================================================
// Renderer State
// ============================================================================

// Tracks whether we're currently in a rendering frame.
thread_local! {
    static RENDER_ACTIVE: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Begins a new rendering frame.
///
/// This must be called before any drawing operations and before `goud_renderer_end`.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_begin(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Mark rendering as active
    RENDER_ACTIVE.with(|cell| {
        *cell.borrow_mut() = true;
    });

    // Begin frame on the backend and set viewport to framebuffer size
    with_window_state(context_id, |state| {
        if let Err(e) = state.backend_mut().begin_frame() {
            set_last_error(e);
            return false;
        }

        // Set viewport to framebuffer size (handles HiDPI/Retina displays)
        let (fb_width, fb_height) = state.get_framebuffer_size();
        unsafe {
            gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
        }

        true
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        false
    })
}

/// Ends the current rendering frame.
///
/// This must be called after all drawing operations and before `goud_window_swap_buffers`.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_end(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Mark rendering as inactive
    RENDER_ACTIVE.with(|cell| {
        *cell.borrow_mut() = false;
    });

    // End frame on the backend
    with_window_state(context_id, |state| {
        if let Err(e) = state.backend_mut().end_frame() {
            set_last_error(e);
            return false;
        }
        true
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        false
    })
}

/// Sets the viewport for rendering.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `x` - Viewport X position
/// * `y` - Viewport Y position
/// * `width` - Viewport width
/// * `height` - Viewport height
#[no_mangle]
pub extern "C" fn goud_renderer_set_viewport(
    context_id: GoudContextId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().set_viewport(x, y, width, height);
    });
}

/// Enables alpha blending for transparent sprites.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_enable_blending(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().enable_blending();
    });
}

/// Disables alpha blending.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_disable_blending(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().disable_blending();
    });
}

/// Enables depth testing.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_enable_depth_test(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().enable_depth_test();
    });
}

/// Disables depth testing.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_disable_depth_test(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().disable_depth_test();
    });
}

/// Clears the depth buffer.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_clear_depth(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().clear_depth();
    });
}

// ============================================================================
// Texture Operations
// ============================================================================

/// Opaque texture handle for FFI.
pub type GoudTextureHandle = u64;

/// Invalid texture handle constant.
pub const GOUD_INVALID_TEXTURE: GoudTextureHandle = u64::MAX;

/// Loads a texture from an image file.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `path` - Path to the image file (null-terminated C string)
///
/// # Returns
///
/// A texture handle on success, or `GOUD_INVALID_TEXTURE` on error.
///
/// # Safety
///
/// The `path` pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_texture_load(
    context_id: GoudContextId,
    path: *const std::os::raw::c_char,
) -> GoudTextureHandle {
    use std::ffi::CStr;

    if context_id == GOUD_INVALID_CONTEXT_ID || path.is_null() {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_TEXTURE;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Invalid UTF-8 in path".to_string(),
            ));
            return GOUD_INVALID_TEXTURE;
        }
    };

    // Load image data
    let img = match image::open(path_str) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Failed to load image '{path_str}': {e}"
            )));
            return GOUD_INVALID_TEXTURE;
        }
    };

    let width = img.width();
    let height = img.height();
    let data = img.into_raw();

    // Create GPU texture
    let result = with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};

        match state.backend_mut().create_texture(
            width,
            height,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &data,
        ) {
            Ok(handle) => {
                // Pack index and generation into a u64 handle
                ((handle.generation() as u64) << 32) | (handle.index() as u64)
            }
            Err(e) => {
                set_last_error(e);
                GOUD_INVALID_TEXTURE
            }
        }
    });

    result.unwrap_or(GOUD_INVALID_TEXTURE)
}

/// Destroys a texture and releases its GPU resources.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `texture` - The texture handle to destroy
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_texture_destroy(
    context_id: GoudContextId,
    texture: GoudTextureHandle,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || texture == GOUD_INVALID_TEXTURE {
        return false;
    }

    with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::TextureHandle;

        // Unpack index and generation from the u64 handle
        let index = (texture & 0xFFFFFFFF) as u32;
        let generation = ((texture >> 32) & 0xFFFFFFFF) as u32;
        let handle = TextureHandle::new(index, generation);

        if state.backend_mut().destroy_texture(handle) {
            true
        } else {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    })
    .unwrap_or(false)
}

// ============================================================================
// Shader Operations (placeholder for future expansion)
// ============================================================================

/// Opaque shader handle for FFI.
pub type GoudShaderHandle = u64;

/// Invalid shader handle constant.
pub const GOUD_INVALID_SHADER: GoudShaderHandle = u64::MAX;

// ============================================================================
// Buffer Operations (placeholder for future expansion)
// ============================================================================

/// Opaque buffer handle for FFI.
pub type GoudBufferHandle = u64;

/// Invalid buffer handle constant.
pub const GOUD_INVALID_BUFFER: GoudBufferHandle = u64::MAX;

// ============================================================================
// Rendering Statistics
// ============================================================================

/// FFI-safe rendering statistics.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoudRenderStats {
    /// Number of draw calls this frame
    pub draw_calls: u32,
    /// Number of triangles rendered
    pub triangles: u32,
    /// Number of texture binds
    pub texture_binds: u32,
    /// Number of shader binds  
    pub shader_binds: u32,
}

/// Gets rendering statistics for the current frame.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `out_stats` - Pointer to store the statistics
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `out_stats` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_get_stats(
    context_id: GoudContextId,
    out_stats: *mut GoudRenderStats,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || out_stats.is_null() {
        return false;
    }

    // For now, return empty stats (will be populated when we have a proper stats tracking)
    *out_stats = GoudRenderStats::default();
    true
}

// ============================================================================
// Immediate-Mode Rendering
// ============================================================================

// Thread-local storage for immediate-mode rendering resources per context
// We use (index, generation) as a key to avoid needing access to private fields
thread_local! {
    static IMMEDIATE_STATE: std::cell::RefCell<std::collections::HashMap<(u32, u32), ImmediateRenderState>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// State for immediate-mode rendering.
struct ImmediateRenderState {
    /// Shader program for sprite rendering
    shader: crate::libs::graphics::backend::types::ShaderHandle,
    /// Vertex buffer for quad rendering
    vertex_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Index buffer for quad rendering (shared)
    index_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Vertex Array Object (required for macOS Core Profile)
    vao: u32,
    /// Uniform locations (cached)
    u_projection: i32,
    u_model: i32,
    u_color: i32,
    u_use_texture: i32,
    u_texture: i32,
    /// UV transform uniforms for sprite sheets
    u_uv_offset: i32,
    u_uv_scale: i32,
}

/// Vertex data for immediate-mode quad rendering.
#[repr(C)]
#[derive(Clone, Copy)]
struct QuadVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

// SAFETY: QuadVertex is a plain data type with no padding issues
unsafe impl bytemuck::Pod for QuadVertex {}
unsafe impl bytemuck::Zeroable for QuadVertex {}

/// Sprite shader vertex source (GLSL 330 Core).
/// Supports UV transformation for sprite sheet animation.
const SPRITE_VERTEX_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;

uniform mat4 u_projection;
uniform mat4 u_model;
uniform vec2 u_uv_offset;  // UV offset for sprite sheets (default: 0,0)
uniform vec2 u_uv_scale;   // UV scale for sprite sheets (default: 1,1)

out vec2 v_texcoord;

void main() {
    gl_Position = u_projection * u_model * vec4(a_position, 0.0, 1.0);
    // Transform UV coordinates: apply scale then offset
    v_texcoord = a_texcoord * u_uv_scale + u_uv_offset;
}
"#;

/// Sprite shader fragment source (GLSL 330 Core).
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

/// Initializes immediate-mode rendering resources for a context.
fn ensure_immediate_state(context_id: GoudContextId) -> Result<(), GoudError> {
    use crate::libs::graphics::backend::types::{BufferType, BufferUsage};
    use crate::libs::graphics::backend::RenderBackend;

    let context_key = (context_id.index(), context_id.generation());

    // Check if already initialized
    let already_initialized = IMMEDIATE_STATE.with(|cell| cell.borrow().contains_key(&context_key));

    if already_initialized {
        return Ok(());
    }

    // Initialize resources
    let state: Result<ImmediateRenderState, GoudError> =
        with_window_state(context_id, |window_state| {
            let backend = window_state.backend_mut();

            // Create shader
            let shader = backend.create_shader(SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER)?;

            // Get uniform locations
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

            // Create VAO (required for macOS Core Profile)
            let mut vao: u32 = 0;
            unsafe {
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);
            }

            // Create quad vertices (unit quad centered at origin)
            // Position: -0.5 to 0.5, UV: 0 to 1
            let vertices: [QuadVertex; 4] = [
                QuadVertex {
                    position: [-0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                }, // Bottom-left
                QuadVertex {
                    position: [0.5, -0.5],
                    tex_coords: [1.0, 0.0],
                }, // Bottom-right
                QuadVertex {
                    position: [0.5, 0.5],
                    tex_coords: [1.0, 1.0],
                }, // Top-right
                QuadVertex {
                    position: [-0.5, 0.5],
                    tex_coords: [0.0, 1.0],
                }, // Top-left
            ];

            let vertex_buffer = backend.create_buffer(
                BufferType::Vertex,
                BufferUsage::Static,
                bytemuck::cast_slice(&vertices),
            )?;

            // Bind vertex buffer and set up vertex attributes while VAO is bound
            backend.bind_buffer(vertex_buffer)?;

            // Set up vertex attributes (position at location 0, texcoord at location 1)
            // Stride is 16 bytes (2 floats for position + 2 floats for texcoord)
            unsafe {
                // Position attribute (location 0)
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(
                    0,                // location
                    2,                // component count (vec2)
                    gl::FLOAT,        // type
                    gl::FALSE,        // normalized
                    16,               // stride (4 floats * 4 bytes)
                    std::ptr::null(), // offset 0
                );

                // TexCoord attribute (location 1)
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(
                    1,                            // location
                    2,                            // component count (vec2)
                    gl::FLOAT,                    // type
                    gl::FALSE,                    // normalized
                    16,                           // stride
                    8 as *const std::ffi::c_void, // offset 8 bytes (after position)
                );
            }

            // Create index buffer (two triangles)
            let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
            let index_buffer = backend.create_buffer(
                BufferType::Index,
                BufferUsage::Static,
                bytemuck::cast_slice(&indices),
            )?;

            // Bind index buffer to VAO
            backend.bind_buffer(index_buffer)?;

            // Unbind VAO (will be bound during draw)
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
        })
        .ok_or(GoudError::InvalidContext)?;

    let state = state?;

    // Store state
    IMMEDIATE_STATE.with(|cell| {
        cell.borrow_mut().insert(context_key, state);
    });

    Ok(())
}

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

    // Translation * Rotation * Scale
    // Scale by width/height, rotate around center, then translate
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

    // Get the immediate state and draw
    let context_key = (context_id.index(), context_id.generation());

    // Get state data we need
    let state_data = IMMEDIATE_STATE.with(|cell| {
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
    });

    let state_data = match state_data {
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

    // Get the immediate state and draw
    let context_key = (context_id.index(), context_id.generation());

    // Get state data we need
    let state_data = IMMEDIATE_STATE.with(|cell| {
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
    });

    let state_data = match state_data {
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

/// State data tuple type for immediate rendering
type ImmediateStateData = (
    crate::libs::graphics::backend::types::ShaderHandle,
    crate::libs::graphics::backend::types::BufferHandle,
    crate::libs::graphics::backend::types::BufferHandle,
    u32, // VAO
    i32, // u_projection
    i32, // u_model
    i32, // u_color
    i32, // u_use_texture
    i32, // u_texture
    i32, // u_uv_offset
    i32, // u_uv_scale
);

/// Internal function to draw a sprite.
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
    use crate::libs::graphics::backend::RenderBackend;

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
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);

        // Ensure proper OpenGL state for 2D rendering (critical after 3D rendering)
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
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
    eprintln!("[TRACE] goud_renderer_draw_quad called: x={x}, y={y}, w={width}, h={height}");
    if context_id == GOUD_INVALID_CONTEXT_ID {
        eprintln!("[ERROR] goud_renderer_draw_quad: Invalid context ID");
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Ensure immediate-mode resources are initialized
    if let Err(e) = ensure_immediate_state(context_id) {
        eprintln!("[ERROR] ensure_immediate_state failed: {e:?}");
        set_last_error(e);
        return false;
    }

    // Get the immediate state and draw
    let context_key = (context_id.index(), context_id.generation());

    // Get state data we need
    let state_data = IMMEDIATE_STATE.with(|cell| {
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
    });

    let state_data = match state_data {
        Some(data) => data,
        None => {
            eprintln!("[ERROR] IMMEDIATE_STATE lookup failed for key {context_key:?}");
            set_last_error(GoudError::InvalidContext);
            return false;
        }
    };

    let result = with_window_state(context_id, |window_state| {
        draw_quad_internal(window_state, state_data, x, y, width, height, r, g, b, a)
    });

    match result {
        Some(Ok(())) => true,
        Some(Err(e)) => {
            eprintln!("[ERROR] draw_quad_internal failed: {e:?}");
            set_last_error(e);
            false
        }
        None => {
            eprintln!("[ERROR] with_window_state returned None for context {context_key:?}");
            set_last_error(GoudError::InvalidContext);
            false
        }
    }
}

/// Internal function to draw a quad.
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
    // DEBUG: Log draw_quad calls
    eprintln!(
        "[DEBUG] draw_quad_internal: pos=({x}, {y}), size=({width}, {height}), color=({r},{g},{b},{a})"
    );
    use crate::libs::graphics::backend::types::PrimitiveTopology;
    use crate::libs::graphics::backend::RenderBackend;

    let (
        shader,
        vertex_buffer,
        index_buffer,
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
        gl::Viewport(0, 0, fb_width as i32, fb_height as i32);

        // Ensure proper OpenGL state for 2D rendering (critical after 3D rendering)
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let backend = window_state.backend_mut();

    // Create orthographic projection (screen coordinates)
    let projection = ortho_matrix(0.0, win_width as f32, win_height as f32, 0.0);

    // Create model matrix (no rotation for simple quads)
    let model = model_matrix(x, y, width, height, 0.0);

    // Bind VAO (includes vertex buffer, index buffer, and vertex attributes)
    unsafe {
        gl::BindVertexArray(vao);
    }

    // Sync backend buffer tracking with OpenGL state
    // Required after 3D rendering which uses raw GL calls that bypass backend tracking
    backend.bind_buffer(vertex_buffer)?;
    backend.bind_buffer(index_buffer)?;

    // Bind shader
    backend.bind_shader(shader)?;

    // Set uniforms
    backend.set_uniform_mat4(u_projection, &projection);
    backend.set_uniform_mat4(u_model, &model);
    backend.set_uniform_vec4(u_color, r, g, b, a);
    backend.set_uniform_int(u_use_texture, 0); // false - no texture

    // Draw
    eprintln!("[DEBUG] About to call draw_indexed...");
    let result = backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0);
    match &result {
        Ok(_) => eprintln!("[DEBUG] draw_indexed succeeded"),
        Err(ref e) => eprintln!("[DEBUG] draw_indexed FAILED: {e:?}"),
    }
    result?;

    Ok(())
}
