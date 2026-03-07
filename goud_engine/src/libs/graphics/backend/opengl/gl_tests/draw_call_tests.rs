//! Tests for OpenGL draw call operations (require GL context).

use crate::libs::error::GoudError;
use crate::libs::graphics::backend::opengl::backend::OpenGLBackend;
use crate::libs::graphics::backend::types::{
    BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout,
};
use crate::libs::graphics::backend::RenderBackend;

const VERTEX_SRC: &str = r#"
    #version 330 core
    layout(location = 0) in vec3 position;
    void main() { gl_Position = vec4(position, 1.0); }
"#;

const FRAGMENT_SRC: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() { FragColor = vec4(1.0); }
"#;

fn make_tri_layout() -> VertexLayout {
    VertexLayout::new(12).with_attribute(VertexAttribute::new(
        0,
        VertexAttributeType::Float3,
        0,
        false,
    ))
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_arrays_without_shader_fails() {
    let mut backend = OpenGLBackend::new().unwrap();

    let vertices: [f32; 9] = [0.0, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, -0.5, 0.0];
    let buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();
    backend.bind_buffer(buffer).unwrap();

    let result = backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GoudError::InvalidState(_)));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_arrays_without_vertex_buffer_fails() {
    let mut backend = OpenGLBackend::new().unwrap();

    let shader = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();
    backend.bind_shader(shader).unwrap();

    let result = backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GoudError::InvalidState(_)));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_arrays_basic() {
    let mut backend = OpenGLBackend::new().unwrap();

    let vertices: [f32; 9] = [0.0, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, -0.5, 0.0];
    let buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    let shader = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    backend.bind_shader(shader).unwrap();
    backend.bind_buffer(buffer).unwrap();
    backend.set_vertex_attributes(&make_tri_layout());

    let result = backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3);
    assert!(result.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_indexed_without_index_buffer_fails() {
    let mut backend = OpenGLBackend::new().unwrap();

    let vertices: [f32; 12] = [
        -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0, -0.5, 0.5, 0.0,
    ];
    let buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    let shader = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    backend.bind_shader(shader).unwrap();
    backend.bind_buffer(buffer).unwrap();
    backend.set_vertex_attributes(&make_tri_layout());

    let result = backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GoudError::InvalidState(_)));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_indexed_basic() {
    let mut backend = OpenGLBackend::new().unwrap();

    let vertices: [f32; 12] = [
        -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0, -0.5, 0.5, 0.0,
    ];
    let vertex_buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
    let index_buffer = backend
        .create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            bytemuck::cast_slice(&indices),
        )
        .unwrap();

    let shader = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    backend.bind_shader(shader).unwrap();
    backend.bind_buffer(vertex_buffer).unwrap();
    backend.set_vertex_attributes(&make_tri_layout());
    backend.bind_buffer(index_buffer).unwrap();

    let result = backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0);
    assert!(result.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_indexed_u16() {
    let mut backend = OpenGLBackend::new().unwrap();

    let vertices: [f32; 12] = [
        -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0, -0.5, 0.5, 0.0,
    ];
    let vertex_buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    let index_buffer = backend
        .create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            bytemuck::cast_slice(&indices),
        )
        .unwrap();

    let shader = backend.create_shader(VERTEX_SRC, FRAGMENT_SRC).unwrap();

    backend.bind_shader(shader).unwrap();
    backend.bind_buffer(vertex_buffer).unwrap();
    backend.set_vertex_attributes(&make_tri_layout());
    backend.bind_buffer(index_buffer).unwrap();

    let result = backend.draw_indexed_u16(PrimitiveTopology::Triangles, 6, 0);
    assert!(result.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_arrays_instanced() {
    let mut backend = OpenGLBackend::new().unwrap();

    if !backend.info().capabilities.supports_instancing {
        return;
    }

    let vertices: [f32; 9] = [0.0, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, -0.5, 0.0];
    let buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    let vertex_src = r#"
        #version 330 core
        layout(location = 0) in vec3 position;
        void main() {
            vec3 offset = vec3(gl_InstanceID * 0.5, 0.0, 0.0);
            gl_Position = vec4(position + offset, 1.0);
        }
    "#;

    let shader = backend.create_shader(vertex_src, FRAGMENT_SRC).unwrap();

    backend.bind_shader(shader).unwrap();
    backend.bind_buffer(buffer).unwrap();
    backend.set_vertex_attributes(&make_tri_layout());

    let result = backend.draw_arrays_instanced(PrimitiveTopology::Triangles, 0, 3, 10);
    assert!(result.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_draw_indexed_instanced() {
    let mut backend = OpenGLBackend::new().unwrap();

    if !backend.info().capabilities.supports_instancing {
        return;
    }

    let vertices: [f32; 12] = [
        -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0, -0.5, 0.5, 0.0,
    ];
    let vertex_buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
    let index_buffer = backend
        .create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            bytemuck::cast_slice(&indices),
        )
        .unwrap();

    let vertex_src = r#"
        #version 330 core
        layout(location = 0) in vec3 position;
        void main() {
            vec3 offset = vec3(gl_InstanceID * 0.3, 0.0, 0.0);
            gl_Position = vec4(position + offset, 1.0);
        }
    "#;

    let shader = backend.create_shader(vertex_src, FRAGMENT_SRC).unwrap();

    backend.bind_shader(shader).unwrap();
    backend.bind_buffer(vertex_buffer).unwrap();
    backend.set_vertex_attributes(&make_tri_layout());
    backend.bind_buffer(index_buffer).unwrap();

    let result = backend.draw_indexed_instanced(PrimitiveTopology::Triangles, 6, 0, 5);
    assert!(result.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_set_vertex_attributes_multiple() {
    let mut backend = OpenGLBackend::new().unwrap();

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vertex {
        position: [f32; 3],
        color: [f32; 4],
    }

    unsafe impl bytemuck::Pod for Vertex {}
    unsafe impl bytemuck::Zeroable for Vertex {}

    let vertices = [
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let buffer = backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices),
        )
        .unwrap();

    backend.bind_buffer(buffer).unwrap();

    // 3 floats + 4 floats = 28 bytes stride
    let layout = VertexLayout::new(28)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float3,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float4,
            12,
            false,
        ));

    // Should not panic
    backend.set_vertex_attributes(&layout);
}
