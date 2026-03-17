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
