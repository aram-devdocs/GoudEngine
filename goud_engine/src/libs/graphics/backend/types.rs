//! GPU Resource Types
//!
//! This module defines type-safe handles for GPU resources like buffers,
//! textures, and shaders. These types are backend-agnostic and provide
//! a safe, typed interface for working with GPU objects.

#![allow(dead_code)] // Will be used in Step 5.1.3+ when implementing buffer/texture/shader operations

use crate::core::handle::Handle;

// ============================================================================
// Buffer Types
// ============================================================================

/// Marker type for buffer handles
#[derive(Debug)]
pub struct BufferMarker;

/// Type-safe handle to a GPU buffer.
///
/// Buffers store arrays of data (vertices, indices, uniforms) on the GPU.
pub type BufferHandle = Handle<BufferMarker>;

/// Type of buffer based on its usage.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferType {
    /// Vertex buffer (vertex attributes)
    Vertex = 0,
    /// Index buffer (element indices)
    Index = 1,
    /// Uniform buffer (shader constants)
    Uniform = 2,
}

/// Buffer usage hint for optimization.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BufferUsage {
    /// Data will be set once and used many times
    #[default]
    Static = 0,
    /// Data will be changed frequently and used many times
    Dynamic = 1,
    /// Data will be set once and used a few times
    Stream = 2,
}

// ============================================================================
// Texture Types
// ============================================================================

/// Marker type for texture handles
#[derive(Debug)]
pub struct TextureMarker;

/// Type-safe handle to a GPU texture.
pub type TextureHandle = Handle<TextureMarker>;

/// Texture format describing pixel layout and data type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    /// 8-bit red channel
    R8 = 0,
    /// 8-bit red-green channels
    RG8 = 1,
    /// 8-bit RGB channels
    RGB8 = 2,
    /// 8-bit RGBA channels (most common)
    RGBA8 = 3,
    /// 16-bit floating point RGBA
    RGBA16F = 4,
    /// 32-bit floating point RGBA
    RGBA32F = 5,
    /// Depth component (24-bit typical)
    Depth = 6,
    /// Depth + stencil combined
    DepthStencil = 7,
}

/// Texture filtering mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureFilter {
    /// Nearest-neighbor filtering (sharp, pixelated)
    Nearest = 0,
    /// Linear interpolation (smooth, blurred)
    #[default]
    Linear = 1,
}

/// Texture wrapping mode for coordinates outside [0, 1].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureWrap {
    /// Repeat the texture
    #[default]
    Repeat = 0,
    /// Mirror the texture on repeat
    MirroredRepeat = 1,
    /// Clamp to edge color
    ClampToEdge = 2,
    /// Clamp to border color
    ClampToBorder = 3,
}

// ============================================================================
// Shader Types
// ============================================================================

/// Marker type for shader program handles
#[derive(Debug)]
pub struct ShaderMarker;

/// Type-safe handle to a compiled shader program.
pub type ShaderHandle = Handle<ShaderMarker>;

/// Shader stage type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    /// Vertex shader (per-vertex processing)
    Vertex = 0,
    /// Fragment/Pixel shader (per-fragment processing)
    Fragment = 1,
    /// Geometry shader (optional, per-primitive processing)
    Geometry = 2,
    /// Compute shader (general-purpose GPU compute)
    Compute = 3,
}

// ============================================================================
// Vertex Layout Types
// ============================================================================

/// Vertex attribute data type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttributeType {
    /// Single-component float (f32)
    Float = 0,
    /// 2-component float vector (f32, f32)
    Float2 = 1,
    /// 3-component float vector (f32, f32, f32)
    Float3 = 2,
    /// 4-component float vector (f32, f32, f32, f32)
    Float4 = 3,
    /// Single-component signed integer (i32)
    Int = 4,
    /// 2-component signed integer vector (i32, i32)
    Int2 = 5,
    /// 3-component signed integer vector (i32, i32, i32)
    Int3 = 6,
    /// 4-component signed integer vector (i32, i32, i32, i32)
    Int4 = 7,
    /// Single-component unsigned integer (u32)
    UInt = 8,
    /// 2-component unsigned integer vector (u32, u32)
    UInt2 = 9,
    /// 3-component unsigned integer vector (u32, u32, u32)
    UInt3 = 10,
    /// 4-component unsigned integer vector (u32, u32, u32, u32)
    UInt4 = 11,
}

impl VertexAttributeType {
    /// Returns the size of this attribute type in bytes.
    pub const fn size_bytes(&self) -> usize {
        match self {
            VertexAttributeType::Float => 4,
            VertexAttributeType::Float2 => 8,
            VertexAttributeType::Float3 => 12,
            VertexAttributeType::Float4 => 16,
            VertexAttributeType::Int => 4,
            VertexAttributeType::Int2 => 8,
            VertexAttributeType::Int3 => 12,
            VertexAttributeType::Int4 => 16,
            VertexAttributeType::UInt => 4,
            VertexAttributeType::UInt2 => 8,
            VertexAttributeType::UInt3 => 12,
            VertexAttributeType::UInt4 => 16,
        }
    }

    /// Returns the component count for this attribute type.
    pub const fn component_count(&self) -> u32 {
        match self {
            VertexAttributeType::Float | VertexAttributeType::Int | VertexAttributeType::UInt => 1,
            VertexAttributeType::Float2
            | VertexAttributeType::Int2
            | VertexAttributeType::UInt2 => 2,
            VertexAttributeType::Float3
            | VertexAttributeType::Int3
            | VertexAttributeType::UInt3 => 3,
            VertexAttributeType::Float4
            | VertexAttributeType::Int4
            | VertexAttributeType::UInt4 => 4,
        }
    }
}

/// Description of a single vertex attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    /// Attribute location/index in the shader
    pub location: u32,
    /// Attribute data type
    pub attribute_type: VertexAttributeType,
    /// Byte offset from start of vertex
    pub offset: u32,
    /// Whether to normalize integer types to [0, 1] or [-1, 1]
    pub normalized: bool,
}

impl VertexAttribute {
    /// Creates a new vertex attribute.
    pub const fn new(
        location: u32,
        attribute_type: VertexAttributeType,
        offset: u32,
        normalized: bool,
    ) -> Self {
        Self {
            location,
            attribute_type,
            offset,
            normalized,
        }
    }
}

/// Layout describing the structure of vertex data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexLayout {
    /// Size of a single vertex in bytes
    pub stride: u32,
    /// List of attributes in this vertex
    pub attributes: Vec<VertexAttribute>,
}

impl VertexLayout {
    /// Creates a new vertex layout.
    pub fn new(stride: u32) -> Self {
        Self {
            stride,
            attributes: Vec::new(),
        }
    }

    /// Adds an attribute to this layout.
    pub fn with_attribute(mut self, attribute: VertexAttribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    /// Calculates the total size of all attributes.
    pub fn total_attribute_size(&self) -> usize {
        self.attributes
            .iter()
            .map(|a| a.attribute_type.size_bytes())
            .sum()
    }
}

// ============================================================================
// Draw Command Types
// ============================================================================

/// Primitive topology for draw calls.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimitiveTopology {
    /// Individual points
    Points = 0,
    /// Individual lines (2 vertices per line)
    Lines = 1,
    /// Connected line strip
    LineStrip = 2,
    /// Individual triangles (3 vertices per triangle)
    #[default]
    Triangles = 3,
    /// Connected triangle strip
    TriangleStrip = 4,
    /// Triangle fan
    TriangleFan = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_handle_valid() {
        let handle = BufferHandle::new(0, 1);
        assert!(handle.is_valid());
        assert_eq!(handle.index(), 0);
        assert_eq!(handle.generation(), 1);
    }

    #[test]
    fn test_buffer_handle_invalid() {
        let handle = BufferHandle::INVALID;
        assert!(!handle.is_valid());
    }

    #[test]
    fn test_buffer_type_repr() {
        assert_eq!(BufferType::Vertex as u8, 0);
        assert_eq!(BufferType::Index as u8, 1);
        assert_eq!(BufferType::Uniform as u8, 2);
    }

    #[test]
    fn test_buffer_usage_default() {
        assert_eq!(BufferUsage::default(), BufferUsage::Static);
    }

    #[test]
    fn test_texture_handle_valid() {
        let handle = TextureHandle::new(5, 2);
        assert!(handle.is_valid());
        assert_eq!(handle.index(), 5);
        assert_eq!(handle.generation(), 2);
    }

    #[test]
    fn test_texture_format_variants() {
        assert_eq!(TextureFormat::RGBA8 as u8, 3);
        assert_eq!(TextureFormat::Depth as u8, 6);
    }

    #[test]
    fn test_texture_filter_default() {
        assert_eq!(TextureFilter::default(), TextureFilter::Linear);
    }

    #[test]
    fn test_texture_wrap_default() {
        assert_eq!(TextureWrap::default(), TextureWrap::Repeat);
    }

    #[test]
    fn test_shader_handle_equality() {
        let h1 = ShaderHandle::new(1, 1);
        let h2 = ShaderHandle::new(1, 1);
        let h3 = ShaderHandle::new(1, 2);
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_vertex_attribute_type_size() {
        assert_eq!(VertexAttributeType::Float.size_bytes(), 4);
        assert_eq!(VertexAttributeType::Float2.size_bytes(), 8);
        assert_eq!(VertexAttributeType::Float3.size_bytes(), 12);
        assert_eq!(VertexAttributeType::Float4.size_bytes(), 16);
        assert_eq!(VertexAttributeType::Int.size_bytes(), 4);
        assert_eq!(VertexAttributeType::UInt4.size_bytes(), 16);
    }

    #[test]
    fn test_vertex_attribute_type_component_count() {
        assert_eq!(VertexAttributeType::Float.component_count(), 1);
        assert_eq!(VertexAttributeType::Float2.component_count(), 2);
        assert_eq!(VertexAttributeType::Float3.component_count(), 3);
        assert_eq!(VertexAttributeType::Float4.component_count(), 4);
    }

    #[test]
    fn test_vertex_attribute_new() {
        let attr = VertexAttribute::new(0, VertexAttributeType::Float3, 0, false);
        assert_eq!(attr.location, 0);
        assert_eq!(attr.attribute_type, VertexAttributeType::Float3);
        assert_eq!(attr.offset, 0);
        assert!(!attr.normalized);
    }

    #[test]
    fn test_vertex_layout_new() {
        let layout = VertexLayout::new(24);
        assert_eq!(layout.stride, 24);
        assert_eq!(layout.attributes.len(), 0);
    }

    #[test]
    fn test_vertex_layout_with_attributes() {
        let layout = VertexLayout::new(24)
            .with_attribute(VertexAttribute::new(
                0,
                VertexAttributeType::Float3,
                0,
                false,
            ))
            .with_attribute(VertexAttribute::new(
                1,
                VertexAttributeType::Float2,
                12,
                false,
            ));

        assert_eq!(layout.attributes.len(), 2);
        assert_eq!(layout.attributes[0].location, 0);
        assert_eq!(layout.attributes[1].location, 1);
    }

    #[test]
    fn test_vertex_layout_total_size() {
        let layout = VertexLayout::new(20)
            .with_attribute(VertexAttribute::new(
                0,
                VertexAttributeType::Float3,
                0,
                false,
            ))
            .with_attribute(VertexAttribute::new(
                1,
                VertexAttributeType::Float2,
                12,
                false,
            ));

        // Float3 (12 bytes) + Float2 (8 bytes) = 20 bytes
        assert_eq!(layout.total_attribute_size(), 20);
    }

    #[test]
    fn test_primitive_topology_default() {
        assert_eq!(PrimitiveTopology::default(), PrimitiveTopology::Triangles);
    }

    #[test]
    fn test_primitive_topology_variants() {
        assert_eq!(PrimitiveTopology::Points as u8, 0);
        assert_eq!(PrimitiveTopology::Triangles as u8, 3);
        assert_eq!(PrimitiveTopology::TriangleFan as u8, 5);
    }

    #[test]
    fn test_handle_types_are_copy() {
        let b1 = BufferHandle::new(1, 1);
        let b2 = b1;
        assert_eq!(b1, b2);

        let t1 = TextureHandle::new(2, 3);
        let t2 = t1;
        assert_eq!(t1, t2);

        let s1 = ShaderHandle::new(4, 5);
        let s2 = s1;
        assert_eq!(s1, s2);
    }
}
