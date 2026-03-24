//! Vertex data generation and GPU buffer upload helpers for the 3D renderer.
//!
//! All functions in this module are pure math (no direct GPU calls) except
//! `upload_buffer`, `create_grid_mesh`, and `create_axis_mesh`, which delegate
//! to the [`RenderBackend`] for buffer allocation.

use crate::libs::graphics::backend::{types::BufferType, types::BufferUsage, BufferHandle};
use crate::libs::graphics::backend::{
    RenderBackend, VertexAttribute, VertexAttributeType, VertexLayout,
};
use crate::libs::graphics::renderer3d::types::InstanceTransform;
use cgmath::Matrix4;

// ============================================================================
// Vertex layouts
// ============================================================================

/// Build the vertex layout for 3D objects: pos (3f) + normal (3f) + texcoord (2f) = 32 bytes
pub(super) fn object_vertex_layout() -> VertexLayout {
    VertexLayout::new(32)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float3,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float3,
            12,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            2,
            VertexAttributeType::Float2,
            24,
            false,
        ))
}

/// Build the vertex layout for grid/axis lines: pos (3f) + color (3f) = 24 bytes
pub(super) fn grid_vertex_layout() -> VertexLayout {
    VertexLayout::new(24)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float3,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float3,
            12,
            false,
        ))
}

/// Build the per-instance layout: model matrix (4 vec4 columns) + color (vec4).
pub(super) fn instance_vertex_layout() -> VertexLayout {
    VertexLayout::new(80)
        .with_attribute(VertexAttribute::new(
            3,
            VertexAttributeType::Float4,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            4,
            VertexAttributeType::Float4,
            16,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            5,
            VertexAttributeType::Float4,
            32,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            6,
            VertexAttributeType::Float4,
            48,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            7,
            VertexAttributeType::Float4,
            64,
            false,
        ))
}

/// Build the vertex layout for skinned meshes:
/// pos (3f) + normal (3f) + uv (2f) + bone_ids (4f) + bone_weights (4f) = 64 bytes
pub(super) fn skinned_vertex_layout() -> VertexLayout {
    VertexLayout::new(64)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float3,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float3,
            12,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            2,
            VertexAttributeType::Float2,
            24,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            3,
            VertexAttributeType::Float4,
            32,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            4,
            VertexAttributeType::Float4,
            48,
            false,
        ))
}

/// Build the per-instance layout for instanced skinned rendering:
/// model_0..model_3 (4 x vec4) + bone_offset (1f) + _pad (3f) + color (vec4) = 88 bytes.
///
/// Bone offset is at location 9 (1 float), followed by 3 floats of padding
/// so that the color vec4 at location 10 starts 16-byte aligned.
pub(super) fn instanced_skinned_instance_layout() -> VertexLayout {
    // 4 * 16 (model cols) + 4 (bone_offset) + 12 (pad) + 16 (color) = 96 bytes
    VertexLayout::new(96)
        .with_attribute(VertexAttribute::new(
            5,
            VertexAttributeType::Float4,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            6,
            VertexAttributeType::Float4,
            16,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            7,
            VertexAttributeType::Float4,
            32,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            8,
            VertexAttributeType::Float4,
            48,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            9,
            VertexAttributeType::Float,
            64,
            false,
        ))
        // 3 floats padding at offset 68 (not declared as attribute)
        .with_attribute(VertexAttribute::new(
            10,
            VertexAttributeType::Float4,
            80,
            false,
        ))
}

/// Build the fullscreen post-process layout: clip pos (2f) + uv (2f).
pub(super) fn postprocess_vertex_layout() -> VertexLayout {
    VertexLayout::new(16)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float2,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float2,
            8,
            false,
        ))
}

// ============================================================================
// Buffer upload helpers
// ============================================================================

/// Upload a slice of `f32` vertex data to the GPU and return a buffer handle.
pub(super) fn upload_buffer(
    backend: &mut dyn RenderBackend,
    vertices: &[f32],
) -> Result<BufferHandle, String> {
    backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(vertices),
        )
        .map_err(|e| format!("Buffer creation failed: {e}"))
}

/// Upload a slice of instance data to the GPU and return a buffer handle.
pub(super) fn upload_instance_buffer(
    backend: &mut dyn RenderBackend,
    instances: &[InstanceTransform],
) -> Result<BufferHandle, String> {
    backend
        .create_buffer(
            BufferType::Vertex,
            BufferUsage::Dynamic,
            bytemuck::cast_slice(pack_instance_data(instances).as_slice()),
        )
        .map_err(|e| format!("Instance buffer creation failed: {e}"))
}

/// Updates an existing GPU instance buffer with the provided transforms.
pub(super) fn update_instance_buffer(
    backend: &mut dyn RenderBackend,
    buffer: BufferHandle,
    instances: &[InstanceTransform],
) -> Result<(), String> {
    let packed = pack_instance_data(instances);
    backend
        .update_buffer(buffer, 0, bytemuck::cast_slice(packed.as_slice()))
        .map_err(|e| format!("Instance buffer update failed: {e}"))
}

/// Generate and upload the grid mesh, returning `(handle, vertex_count)`.
pub(super) fn create_grid_mesh(
    backend: &mut dyn RenderBackend,
    size: f32,
    divisions: u32,
) -> Result<(BufferHandle, i32), String> {
    let vertices = generate_grid_vertices(size, divisions);
    let count = (vertices.len() / 6) as i32;
    let handle = upload_buffer(backend, &vertices)?;
    Ok((handle, count))
}

/// Generate and upload the axis-indicator mesh, returning `(handle, vertex_count)`.
pub(super) fn create_axis_mesh(
    backend: &mut dyn RenderBackend,
    size: f32,
) -> Result<(BufferHandle, i32), String> {
    let vertices = generate_axis_vertices(size);
    let count = (vertices.len() / 6) as i32;
    let handle = upload_buffer(backend, &vertices)?;
    Ok((handle, count))
}

pub(super) fn create_postprocess_quad() -> Vec<f32> {
    vec![
        -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0,
        1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 1.0,
    ]
}

// Vertex generation lives in an included file to keep this module focused on layouts and uploads.
include!("mesh_geometry.in");
