use crate::core::error::GoudResult;
use crate::libs::graphics::backend::{BufferOps, ShaderOps};

/// GPU resources for immediate-mode sprite and quad rendering.
///
/// Created when the OpenGL backend is initialized and stored in GoudGame.
/// Contains the compiled shader program, vertex/index buffers, VAO, and
/// cached uniform locations needed by `draw_sprite` and `draw_quad`.
pub struct ImmediateRenderState {
    /// Shader program for sprite rendering
    pub(crate) shader: crate::libs::graphics::backend::types::ShaderHandle,
    /// Vertex buffer for quad rendering.
    pub(crate) vertex_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Index buffer for quad rendering.
    pub(crate) index_buffer: crate::libs::graphics::backend::types::BufferHandle,
    /// Vertex Array Object (required for macOS Core Profile)
    pub(crate) vao: u32,
    /// Cached uniform locations
    pub(crate) u_projection: i32,
    pub(crate) u_model: i32,
    pub(crate) u_color: i32,
    pub(crate) u_use_texture: i32,
    pub(crate) u_texture: i32,
    pub(crate) u_uv_offset: i32,
    pub(crate) u_uv_scale: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ImmediateQuadVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

// SAFETY: ImmediateQuadVertex is plain-old vertex data.
unsafe impl bytemuck::Pod for ImmediateQuadVertex {}
unsafe impl bytemuck::Zeroable for ImmediateQuadVertex {}

pub(crate) fn create_immediate_render_state(
    backend: &mut crate::libs::graphics::backend::opengl::OpenGLBackend,
) -> GoudResult<ImmediateRenderState> {
    use crate::libs::graphics::backend::types::{BufferType, BufferUsage};

    let shader = backend.create_shader(SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER)?;

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

    let mut vao = 0u32;
    // SAFETY: An OpenGL context is current for the backend lifetime. The stack
    // variable is valid output storage and the generated VAO is immediately bound.
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let vertices: [ImmediateQuadVertex; 4] = [
        ImmediateQuadVertex {
            position: [-0.5, -0.5],
            tex_coords: [0.0, 0.0],
        },
        ImmediateQuadVertex {
            position: [0.5, -0.5],
            tex_coords: [1.0, 0.0],
        },
        ImmediateQuadVertex {
            position: [0.5, 0.5],
            tex_coords: [1.0, 1.0],
        },
        ImmediateQuadVertex {
            position: [-0.5, 0.5],
            tex_coords: [0.0, 1.0],
        },
    ];
    let vertex_buffer = backend.create_buffer(
        BufferType::Vertex,
        BufferUsage::Static,
        bytemuck::cast_slice(&vertices),
    )?;
    backend.bind_buffer(vertex_buffer)?;

    configure_immediate_vertex_layout();

    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
    let index_buffer = backend.create_buffer(
        BufferType::Index,
        BufferUsage::Static,
        bytemuck::cast_slice(&indices),
    )?;
    backend.bind_buffer(index_buffer)?;

    // SAFETY: Binding VAO 0 unbinds the current vertex array in OpenGL.
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
}

pub(crate) fn configure_immediate_vertex_layout() {
    // SAFETY: The immediate-mode VAO and vertex buffer are already bound by the caller,
    // and this layout matches `ImmediateQuadVertex`.
    unsafe {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 16, std::ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 16, 8 as *const std::ffi::c_void);
        gl::DisableVertexAttribArray(2);
        gl::VertexAttrib4f(2, 1.0, 1.0, 1.0, 1.0);
    }
}

const SPRITE_VERTEX_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;

uniform mat4 u_projection;
uniform mat4 u_model;
uniform vec2 u_uv_offset;
uniform vec2 u_uv_scale;

out vec2 v_texcoord;

void main() {
    gl_Position = u_projection * u_model * vec4(a_position, 0.0, 1.0);
    v_texcoord = a_texcoord * u_uv_scale + u_uv_offset;
}
"#;

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
