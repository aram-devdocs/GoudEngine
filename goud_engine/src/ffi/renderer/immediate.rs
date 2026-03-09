//! # Immediate-Mode Rendering State
//!
//! Thread-local state management, shader sources, geometry setup, and math
//! helpers for immediate-mode quad/sprite rendering.

use crate::core::error::GoudError;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::{BufferOps, ShaderOps};

// ============================================================================
// Immediate-Mode State
// ============================================================================

// Thread-local storage for immediate-mode rendering resources per context.
// We use (index, generation) as a key to avoid needing access to private fields.
thread_local! {
    pub(super) static IMMEDIATE_STATE: std::cell::RefCell<std::collections::HashMap<(u32, u32), ImmediateRenderState>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// State for immediate-mode rendering.
pub(super) struct ImmediateRenderState {
    /// Shader program for sprite rendering
    pub shader: crate::libs::graphics::backend::types::ShaderHandle,
    /// Vertex buffer for quad rendering
    pub vertex_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Index buffer for quad rendering (shared)
    pub index_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Vertex Array Object (required for macOS Core Profile)
    pub vao: u32,
    /// Uniform locations (cached)
    pub u_projection: i32,
    pub u_model: i32,
    pub u_color: i32,
    pub u_use_texture: i32,
    pub u_texture: i32,
    /// UV transform uniforms for sprite sheets
    pub u_uv_offset: i32,
    pub u_uv_scale: i32,
}

/// Vertex data for immediate-mode quad rendering.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct QuadVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

// SAFETY: QuadVertex is a plain data type with no padding issues
unsafe impl bytemuck::Pod for QuadVertex {}
unsafe impl bytemuck::Zeroable for QuadVertex {}

/// State data tuple type for immediate rendering.
pub(super) type ImmediateStateData = (
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

// ============================================================================
// Shader Sources
// ============================================================================

/// Sprite shader vertex source (GLSL 330 Core).
/// Supports UV transformation for sprite sheet animation.
pub(super) const SPRITE_VERTEX_SHADER: &str = r#"
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
pub(super) const SPRITE_FRAGMENT_SHADER: &str = r#"
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

// ============================================================================
// Initialization
// ============================================================================

/// Initializes immediate-mode rendering resources for a context.
pub(super) fn ensure_immediate_state(context_id: GoudContextId) -> Result<(), GoudError> {
    use crate::libs::graphics::backend::types::{BufferType, BufferUsage};

    if context_id == GOUD_INVALID_CONTEXT_ID {
        return Err(GoudError::InvalidContext);
    }

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
            // SAFETY: gl::GenVertexArrays writes to a valid stack variable; BindVertexArray
            // takes the returned VAO name which is guaranteed valid by OpenGL.
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
            // SAFETY: VAO is bound above, vertex buffer is bound, and attribute locations
            // 0 and 1 are valid for the shader created in this function.
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
            // SAFETY: Passing 0 to gl::BindVertexArray unbinds the current VAO, which is
            // always valid per the OpenGL specification.
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

/// Binds the immediate-mode VAO for draw calls.
pub(super) fn bind_immediate_vao(vao: u32) {
    // SAFETY: `vao` is created and owned by the immediate renderer state for the
    // active GL context, so binding it is valid for subsequent draw calls.
    unsafe {
        gl::BindVertexArray(vao);
    }
}

// ============================================================================
// Math Helpers
// ============================================================================

/// Creates an orthographic projection matrix.
pub(super) fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
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
pub(super) fn model_matrix(x: f32, y: f32, width: f32, height: f32, rotation: f32) -> [f32; 16] {
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
