//! Type conversion helpers between engine types and OpenGL constants.

use super::super::{
    types::{
        BufferType, BufferUsage, TextureFilter, TextureFormat, TextureWrap, VertexAttributeType,
    },
    BlendFactor,
};
use crate::libs::graphics::backend::types::PrimitiveTopology;

/// Converts `BufferType` to OpenGL buffer target constant.
pub(super) fn buffer_type_to_gl_target(buffer_type: BufferType) -> u32 {
    match buffer_type {
        BufferType::Vertex => gl::ARRAY_BUFFER,
        BufferType::Index => gl::ELEMENT_ARRAY_BUFFER,
        BufferType::Uniform => gl::UNIFORM_BUFFER,
    }
}

/// Converts `BufferUsage` to OpenGL usage hint constant.
pub(super) fn buffer_usage_to_gl_usage(usage: BufferUsage) -> u32 {
    match usage {
        BufferUsage::Static => gl::STATIC_DRAW,
        BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
        BufferUsage::Stream => gl::STREAM_DRAW,
    }
}

/// Converts `BlendFactor` to OpenGL blend factor constant.
pub(super) fn blend_factor_to_gl(factor: BlendFactor) -> u32 {
    match factor {
        BlendFactor::Zero => gl::ZERO,
        BlendFactor::One => gl::ONE,
        BlendFactor::SrcColor => gl::SRC_COLOR,
        BlendFactor::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
        BlendFactor::DstColor => gl::DST_COLOR,
        BlendFactor::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlpha => gl::SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha => gl::DST_ALPHA,
        BlendFactor::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
        BlendFactor::ConstantColor => gl::CONSTANT_COLOR,
        BlendFactor::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
        BlendFactor::ConstantAlpha => gl::CONSTANT_ALPHA,
        BlendFactor::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
    }
}

/// Converts `TextureFormat` to OpenGL internal format, pixel format, and pixel type.
///
/// Returns `(internal_format, format, type)` for use with `glTexImage2D`.
pub(super) fn texture_format_to_gl(format: TextureFormat) -> (u32, u32, u32) {
    match format {
        TextureFormat::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
        TextureFormat::RG8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
        TextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
        TextureFormat::RGBA8 | TextureFormat::RGBA8Linear => {
            (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
        }
        TextureFormat::RGBA16F => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT),
        TextureFormat::RGBA32F => (gl::RGBA32F, gl::RGBA, gl::FLOAT),
        TextureFormat::Depth => (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_INT),
        TextureFormat::DepthStencil => (
            gl::DEPTH24_STENCIL8,
            gl::DEPTH_STENCIL,
            gl::UNSIGNED_INT_24_8,
        ),
        // GL extension constants for compressed texture formats
        TextureFormat::BC1 => (0x83F0, gl::RGB, gl::UNSIGNED_BYTE), // GL_COMPRESSED_RGB_S3TC_DXT1_EXT
        TextureFormat::BC3 => (0x83F3, gl::RGBA, gl::UNSIGNED_BYTE), // GL_COMPRESSED_RGBA_S3TC_DXT5_EXT
        TextureFormat::BC5 => (0x8DBD, gl::RG, gl::UNSIGNED_BYTE),   // GL_COMPRESSED_RG_RGTC2
        TextureFormat::BC7 => (0x8E8C, gl::RGBA, gl::UNSIGNED_BYTE), // GL_COMPRESSED_RGBA_BPTC_UNORM
    }
}

/// Converts `TextureFilter` to OpenGL filter constant.
pub(super) fn texture_filter_to_gl(filter: TextureFilter) -> u32 {
    match filter {
        TextureFilter::Nearest => gl::NEAREST,
        TextureFilter::Linear => gl::LINEAR,
    }
}

/// Converts `TextureWrap` to OpenGL wrap constant.
pub(super) fn texture_wrap_to_gl(wrap: TextureWrap) -> u32 {
    match wrap {
        TextureWrap::Repeat => gl::REPEAT,
        TextureWrap::MirroredRepeat => gl::MIRRORED_REPEAT,
        TextureWrap::ClampToEdge => gl::CLAMP_TO_EDGE,
        TextureWrap::ClampToBorder => gl::CLAMP_TO_BORDER,
    }
}

/// Returns the number of bytes per pixel for a given texture format.
pub(super) fn bytes_per_pixel(format: TextureFormat) -> usize {
    match format {
        TextureFormat::R8 => 1,
        TextureFormat::RG8 => 2,
        TextureFormat::RGB8 => 3,
        TextureFormat::RGBA8 | TextureFormat::RGBA8Linear => 4,
        TextureFormat::RGBA16F => 8,      // 4 channels × 2 bytes
        TextureFormat::RGBA32F => 16,     // 4 channels × 4 bytes
        TextureFormat::Depth => 4,        // 24-bit or 32-bit typically
        TextureFormat::DepthStencil => 4, // 24 + 8 bits = 32 bits
        // Block-compressed formats: bytes per block, not per pixel.
        // These should not be used with per-pixel upload paths.
        TextureFormat::BC1 => 8,  // 8 bytes per 4x4 block
        TextureFormat::BC3 => 16, // 16 bytes per 4x4 block
        TextureFormat::BC5 => 16, // 16 bytes per 4x4 block
        TextureFormat::BC7 => 16, // 16 bytes per 4x4 block
    }
}

/// Converts `PrimitiveTopology` to OpenGL primitive mode constant.
pub(super) fn topology_to_gl(topology: PrimitiveTopology) -> u32 {
    match topology {
        PrimitiveTopology::Points => gl::POINTS,
        PrimitiveTopology::Lines => gl::LINES,
        PrimitiveTopology::LineStrip => gl::LINE_STRIP,
        PrimitiveTopology::Triangles => gl::TRIANGLES,
        PrimitiveTopology::TriangleStrip => gl::TRIANGLE_STRIP,
        PrimitiveTopology::TriangleFan => gl::TRIANGLE_FAN,
    }
}

/// Converts `VertexAttributeType` to OpenGL type constant.
pub(super) fn attribute_type_to_gl_type(attr_type: VertexAttributeType) -> u32 {
    match attr_type {
        VertexAttributeType::Float
        | VertexAttributeType::Float2
        | VertexAttributeType::Float3
        | VertexAttributeType::Float4 => gl::FLOAT,
        VertexAttributeType::Int
        | VertexAttributeType::Int2
        | VertexAttributeType::Int3
        | VertexAttributeType::Int4 => gl::INT,
        VertexAttributeType::UInt
        | VertexAttributeType::UInt2
        | VertexAttributeType::UInt3
        | VertexAttributeType::UInt4 => gl::UNSIGNED_INT,
    }
}
