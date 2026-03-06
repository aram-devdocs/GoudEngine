//! Tests for type conversion helpers (no GL context required).

use crate::libs::graphics::backend::opengl::conversions::{
    attribute_type_to_gl_type, blend_factor_to_gl, buffer_type_to_gl_target,
    buffer_usage_to_gl_usage, bytes_per_pixel, texture_filter_to_gl, texture_format_to_gl,
    texture_wrap_to_gl, topology_to_gl,
};
use crate::libs::graphics::backend::types::{
    BufferType, BufferUsage, PrimitiveTopology, TextureFilter, TextureFormat, TextureWrap,
    VertexAttributeType,
};
use crate::libs::graphics::backend::BlendFactor;

#[test]
fn test_buffer_type_to_gl_target() {
    assert_eq!(
        buffer_type_to_gl_target(BufferType::Vertex),
        gl::ARRAY_BUFFER
    );
    assert_eq!(
        buffer_type_to_gl_target(BufferType::Index),
        gl::ELEMENT_ARRAY_BUFFER
    );
    assert_eq!(
        buffer_type_to_gl_target(BufferType::Uniform),
        gl::UNIFORM_BUFFER
    );
}

#[test]
fn test_buffer_usage_to_gl_usage() {
    assert_eq!(
        buffer_usage_to_gl_usage(BufferUsage::Static),
        gl::STATIC_DRAW
    );
    assert_eq!(
        buffer_usage_to_gl_usage(BufferUsage::Dynamic),
        gl::DYNAMIC_DRAW
    );
    assert_eq!(
        buffer_usage_to_gl_usage(BufferUsage::Stream),
        gl::STREAM_DRAW
    );
}

#[test]
fn test_blend_factor_to_gl() {
    assert_eq!(blend_factor_to_gl(BlendFactor::Zero), gl::ZERO);
    assert_eq!(blend_factor_to_gl(BlendFactor::One), gl::ONE);
    assert_eq!(blend_factor_to_gl(BlendFactor::SrcAlpha), gl::SRC_ALPHA);
    assert_eq!(
        blend_factor_to_gl(BlendFactor::OneMinusSrcAlpha),
        gl::ONE_MINUS_SRC_ALPHA
    );
}

#[test]
fn test_texture_format_to_gl() {
    let (internal, format, type_) = texture_format_to_gl(TextureFormat::RGBA8);
    assert_eq!(internal, gl::RGBA8);
    assert_eq!(format, gl::RGBA);
    assert_eq!(type_, gl::UNSIGNED_BYTE);

    let (internal, format, type_) = texture_format_to_gl(TextureFormat::RGB8);
    assert_eq!(internal, gl::RGB8);
    assert_eq!(format, gl::RGB);
    assert_eq!(type_, gl::UNSIGNED_BYTE);
}

#[test]
fn test_texture_filter_to_gl() {
    assert_eq!(texture_filter_to_gl(TextureFilter::Nearest), gl::NEAREST);
    assert_eq!(texture_filter_to_gl(TextureFilter::Linear), gl::LINEAR);
}

#[test]
fn test_texture_wrap_to_gl() {
    assert_eq!(texture_wrap_to_gl(TextureWrap::Repeat), gl::REPEAT);
    assert_eq!(
        texture_wrap_to_gl(TextureWrap::MirroredRepeat),
        gl::MIRRORED_REPEAT
    );
    assert_eq!(
        texture_wrap_to_gl(TextureWrap::ClampToEdge),
        gl::CLAMP_TO_EDGE
    );
    assert_eq!(
        texture_wrap_to_gl(TextureWrap::ClampToBorder),
        gl::CLAMP_TO_BORDER
    );
}

#[test]
fn test_bytes_per_pixel() {
    assert_eq!(bytes_per_pixel(TextureFormat::R8), 1);
    assert_eq!(bytes_per_pixel(TextureFormat::RG8), 2);
    assert_eq!(bytes_per_pixel(TextureFormat::RGB8), 3);
    assert_eq!(bytes_per_pixel(TextureFormat::RGBA8), 4);
    assert_eq!(bytes_per_pixel(TextureFormat::RGBA16F), 8);
    assert_eq!(bytes_per_pixel(TextureFormat::RGBA32F), 16);
}

#[test]
fn test_topology_to_gl_conversion() {
    assert_eq!(topology_to_gl(PrimitiveTopology::Points), gl::POINTS);
    assert_eq!(topology_to_gl(PrimitiveTopology::Lines), gl::LINES);
    assert_eq!(topology_to_gl(PrimitiveTopology::LineStrip), gl::LINE_STRIP);
    assert_eq!(topology_to_gl(PrimitiveTopology::Triangles), gl::TRIANGLES);
    assert_eq!(
        topology_to_gl(PrimitiveTopology::TriangleStrip),
        gl::TRIANGLE_STRIP
    );
    assert_eq!(
        topology_to_gl(PrimitiveTopology::TriangleFan),
        gl::TRIANGLE_FAN
    );
}

#[test]
fn test_attribute_type_to_gl_conversion() {
    // Float types
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Float),
        gl::FLOAT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Float2),
        gl::FLOAT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Float3),
        gl::FLOAT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Float4),
        gl::FLOAT
    );

    // Int types
    assert_eq!(attribute_type_to_gl_type(VertexAttributeType::Int), gl::INT);
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Int2),
        gl::INT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Int3),
        gl::INT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::Int4),
        gl::INT
    );

    // UInt types
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::UInt),
        gl::UNSIGNED_INT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::UInt2),
        gl::UNSIGNED_INT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::UInt3),
        gl::UNSIGNED_INT
    );
    assert_eq!(
        attribute_type_to_gl_type(VertexAttributeType::UInt4),
        gl::UNSIGNED_INT
    );
}

#[test]
fn test_opengl_backend_implements_send_sync() {
    use crate::libs::graphics::backend::opengl::backend::OpenGLBackend;
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OpenGLBackend>();
}
