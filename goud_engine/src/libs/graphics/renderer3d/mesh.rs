//! Vertex data generation and GPU buffer upload helpers for the 3D renderer.
//!
//! All functions in this module are pure math (no direct GPU calls) except
//! `upload_buffer`, `create_grid_mesh`, and `create_axis_mesh`, which delegate
//! to the [`RenderBackend`] for buffer allocation.

use crate::libs::graphics::backend::{types::BufferType, types::BufferUsage, BufferHandle};
use crate::libs::graphics::backend::{
    RenderBackend, VertexAttribute, VertexAttributeType, VertexLayout,
};

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

// ============================================================================
// Vertex data generation
// ============================================================================

pub(super) fn generate_grid_vertices(size: f32, divisions: u32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let step = size / divisions as f32;
    let half = size / 2.0;

    let xz_color = [0.3, 0.3, 0.3];
    let xy_color = [0.2, 0.25, 0.3];
    let yz_color = [0.3, 0.2, 0.25];

    for i in 0..=divisions {
        let pos = -half + step * i as f32;

        vertices.extend_from_slice(&[pos, 0.0, -half]);
        vertices.extend_from_slice(&xz_color);
        vertices.extend_from_slice(&[pos, 0.0, half]);
        vertices.extend_from_slice(&xz_color);

        vertices.extend_from_slice(&[-half, 0.0, pos]);
        vertices.extend_from_slice(&xz_color);
        vertices.extend_from_slice(&[half, 0.0, pos]);
        vertices.extend_from_slice(&xz_color);
    }

    for i in 0..=divisions {
        let pos = -half + step * i as f32;

        vertices.extend_from_slice(&[pos, -half, 0.0]);
        vertices.extend_from_slice(&xy_color);
        vertices.extend_from_slice(&[pos, half, 0.0]);
        vertices.extend_from_slice(&xy_color);

        vertices.extend_from_slice(&[-half, pos, 0.0]);
        vertices.extend_from_slice(&xy_color);
        vertices.extend_from_slice(&[half, pos, 0.0]);
        vertices.extend_from_slice(&xy_color);
    }

    for i in 0..=divisions {
        let pos = -half + step * i as f32;

        vertices.extend_from_slice(&[0.0, pos, -half]);
        vertices.extend_from_slice(&yz_color);
        vertices.extend_from_slice(&[0.0, pos, half]);
        vertices.extend_from_slice(&yz_color);

        vertices.extend_from_slice(&[0.0, -half, pos]);
        vertices.extend_from_slice(&yz_color);
        vertices.extend_from_slice(&[0.0, half, pos]);
        vertices.extend_from_slice(&yz_color);
    }

    vertices
}

#[rustfmt::skip]
pub(super) fn generate_axis_vertices(size: f32) -> Vec<f32> {
    vec![
        // X axis (red)
        0.0, 0.0, 0.0, 1.0, 0.2, 0.2,  size, 0.0, 0.0, 1.0, 0.2, 0.2,
        0.0, 0.0, 0.0, 0.5, 0.1, 0.1, -size, 0.0, 0.0, 0.5, 0.1, 0.1,
        // Y axis (green)
        0.0, 0.0, 0.0, 0.2, 1.0, 0.2,  0.0, size, 0.0, 0.2, 1.0, 0.2,
        0.0, 0.0, 0.0, 0.1, 0.5, 0.1,  0.0,-size, 0.0, 0.1, 0.5, 0.1,
        // Z axis (blue)
        0.0, 0.0, 0.0, 0.2, 0.2, 1.0,  0.0, 0.0, size, 0.2, 0.2, 1.0,
        0.0, 0.0, 0.0, 0.1, 0.1, 0.5,  0.0, 0.0,-size, 0.1, 0.1, 0.5,
        // Origin marker (small cross)
        -0.2, 0.0, 0.0, 1.0, 1.0, 1.0,  0.2, 0.0, 0.0, 1.0, 1.0, 1.0,
         0.0,-0.2, 0.0, 1.0, 1.0, 1.0,  0.0, 0.2, 0.0, 1.0, 1.0, 1.0,
         0.0, 0.0,-0.2, 1.0, 1.0, 1.0,  0.0, 0.0, 0.2, 1.0, 1.0, 1.0,
    ]
}

#[rustfmt::skip]
pub(super) fn generate_cube_vertices(width: f32, height: f32, depth: f32) -> Vec<f32> {
    let w = width / 2.0;
    let h = height / 2.0;
    let d = depth / 2.0;

    vec![
        // Front face (z+)
        -w,-h, d, 0.0, 0.0, 1.0, 0.0, 0.0,  w,-h, d, 0.0, 0.0, 1.0, 1.0, 0.0,
         w, h, d, 0.0, 0.0, 1.0, 1.0, 1.0,   w, h, d, 0.0, 0.0, 1.0, 1.0, 1.0,
        -w, h, d, 0.0, 0.0, 1.0, 0.0, 1.0,  -w,-h, d, 0.0, 0.0, 1.0, 0.0, 0.0,
        // Back face (z-)
         w,-h,-d, 0.0, 0.0,-1.0, 0.0, 0.0,  -w,-h,-d, 0.0, 0.0,-1.0, 1.0, 0.0,
        -w, h,-d, 0.0, 0.0,-1.0, 1.0, 1.0,  -w, h,-d, 0.0, 0.0,-1.0, 1.0, 1.0,
         w, h,-d, 0.0, 0.0,-1.0, 0.0, 1.0,   w,-h,-d, 0.0, 0.0,-1.0, 0.0, 0.0,
        // Left face (x-)
        -w,-h,-d,-1.0, 0.0, 0.0, 0.0, 0.0,  -w,-h, d,-1.0, 0.0, 0.0, 1.0, 0.0,
        -w, h, d,-1.0, 0.0, 0.0, 1.0, 1.0,  -w, h, d,-1.0, 0.0, 0.0, 1.0, 1.0,
        -w, h,-d,-1.0, 0.0, 0.0, 0.0, 1.0,  -w,-h,-d,-1.0, 0.0, 0.0, 0.0, 0.0,
        // Right face (x+)
         w,-h, d, 1.0, 0.0, 0.0, 0.0, 0.0,   w,-h,-d, 1.0, 0.0, 0.0, 1.0, 0.0,
         w, h,-d, 1.0, 0.0, 0.0, 1.0, 1.0,   w, h,-d, 1.0, 0.0, 0.0, 1.0, 1.0,
         w, h, d, 1.0, 0.0, 0.0, 0.0, 1.0,   w,-h, d, 1.0, 0.0, 0.0, 0.0, 0.0,
        // Bottom face (y-)
        -w,-h,-d, 0.0,-1.0, 0.0, 0.0, 0.0,   w,-h,-d, 0.0,-1.0, 0.0, 1.0, 0.0,
         w,-h, d, 0.0,-1.0, 0.0, 1.0, 1.0,   w,-h, d, 0.0,-1.0, 0.0, 1.0, 1.0,
        -w,-h, d, 0.0,-1.0, 0.0, 0.0, 1.0,  -w,-h,-d, 0.0,-1.0, 0.0, 0.0, 0.0,
        // Top face (y+)
        -w, h, d, 0.0, 1.0, 0.0, 0.0, 0.0,   w, h, d, 0.0, 1.0, 0.0, 1.0, 0.0,
         w, h,-d, 0.0, 1.0, 0.0, 1.0, 1.0,   w, h,-d, 0.0, 1.0, 0.0, 1.0, 1.0,
        -w, h,-d, 0.0, 1.0, 0.0, 0.0, 1.0,  -w, h, d, 0.0, 1.0, 0.0, 0.0, 0.0,
    ]
}

#[rustfmt::skip]
pub(super) fn generate_plane_vertices(width: f32, depth: f32) -> Vec<f32> {
    let w = width / 2.0;
    let d = depth / 2.0;

    vec![
        // Top face
        -w, 0.0, d, 0.0, 1.0, 0.0, 0.0, 1.0,   w, 0.0, d, 0.0, 1.0, 0.0, 1.0, 1.0,
         w, 0.0,-d, 0.0, 1.0, 0.0, 1.0, 0.0,    w, 0.0,-d, 0.0, 1.0, 0.0, 1.0, 0.0,
        -w, 0.0,-d, 0.0, 1.0, 0.0, 0.0, 0.0,   -w, 0.0, d, 0.0, 1.0, 0.0, 0.0, 1.0,
        // Bottom face (double-sided)
        -w, 0.0,-d, 0.0,-1.0, 0.0, 0.0, 0.0,    w, 0.0,-d, 0.0,-1.0, 0.0, 1.0, 0.0,
         w, 0.0, d, 0.0,-1.0, 0.0, 1.0, 1.0,    w, 0.0, d, 0.0,-1.0, 0.0, 1.0, 1.0,
        -w, 0.0, d, 0.0,-1.0, 0.0, 0.0, 1.0,   -w, 0.0,-d, 0.0,-1.0, 0.0, 0.0, 0.0,
    ]
}

pub(super) fn generate_sphere_vertices(radius: f32, segments: u32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let segment_count = segments.max(8);

    for i in 0..segment_count {
        let lat0 = std::f32::consts::PI * (-0.5 + (i as f32) / segment_count as f32);
        let lat1 = std::f32::consts::PI * (-0.5 + ((i + 1) as f32) / segment_count as f32);

        for j in 0..segment_count {
            let lng0 = 2.0 * std::f32::consts::PI * (j as f32) / segment_count as f32;
            let lng1 = 2.0 * std::f32::consts::PI * ((j + 1) as f32) / segment_count as f32;

            let x0 = radius * lat0.cos() * lng0.cos();
            let y0 = radius * lat0.sin();
            let z0 = radius * lat0.cos() * lng0.sin();
            let x1 = radius * lat0.cos() * lng1.cos();
            let y1 = radius * lat0.sin();
            let z1 = radius * lat0.cos() * lng1.sin();
            let x2 = radius * lat1.cos() * lng1.cos();
            let y2 = radius * lat1.sin();
            let z2 = radius * lat1.cos() * lng1.sin();
            let x3 = radius * lat1.cos() * lng0.cos();
            let y3 = radius * lat1.sin();
            let z3 = radius * lat1.cos() * lng0.sin();

            let u0 = j as f32 / segment_count as f32;
            let u1 = (j + 1) as f32 / segment_count as f32;
            let v0 = i as f32 / segment_count as f32;
            let v1 = (i + 1) as f32 / segment_count as f32;

            vertices.extend_from_slice(&[
                x0,
                y0,
                z0,
                x0 / radius,
                y0 / radius,
                z0 / radius,
                u0,
                v0,
                x1,
                y1,
                z1,
                x1 / radius,
                y1 / radius,
                z1 / radius,
                u1,
                v0,
                x2,
                y2,
                z2,
                x2 / radius,
                y2 / radius,
                z2 / radius,
                u1,
                v1,
                x0,
                y0,
                z0,
                x0 / radius,
                y0 / radius,
                z0 / radius,
                u0,
                v0,
                x2,
                y2,
                z2,
                x2 / radius,
                y2 / radius,
                z2 / radius,
                u1,
                v1,
                x3,
                y3,
                z3,
                x3 / radius,
                y3 / radius,
                z3 / radius,
                u0,
                v1,
            ]);
        }
    }

    vertices
}

pub(super) fn generate_cylinder_vertices(radius: f32, height: f32, segments: u32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let segment_count = segments.max(8);
    let h = height / 2.0;

    for i in 0..segment_count {
        let a0 = 2.0 * std::f32::consts::PI * (i as f32) / segment_count as f32;
        let a1 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / segment_count as f32;

        let x0 = radius * a0.cos();
        let z0 = radius * a0.sin();
        let x1 = radius * a1.cos();
        let z1 = radius * a1.sin();

        let u0 = i as f32 / segment_count as f32;
        let u1 = (i + 1) as f32 / segment_count as f32;

        // Side faces (two triangles)
        vertices.extend_from_slice(&[
            x0,
            -h,
            z0,
            x0 / radius,
            0.0,
            z0 / radius,
            u0,
            0.0,
            x1,
            -h,
            z1,
            x1 / radius,
            0.0,
            z1 / radius,
            u1,
            0.0,
            x1,
            h,
            z1,
            x1 / radius,
            0.0,
            z1 / radius,
            u1,
            1.0,
            x0,
            -h,
            z0,
            x0 / radius,
            0.0,
            z0 / radius,
            u0,
            0.0,
            x1,
            h,
            z1,
            x1 / radius,
            0.0,
            z1 / radius,
            u1,
            1.0,
            x0,
            h,
            z0,
            x0 / radius,
            0.0,
            z0 / radius,
            u0,
            1.0,
        ]);

        // Top cap
        vertices.extend_from_slice(&[
            0.0,
            h,
            0.0,
            0.0,
            1.0,
            0.0,
            0.5,
            0.5,
            x1,
            h,
            z1,
            0.0,
            1.0,
            0.0,
            0.5 + 0.5 * a1.cos(),
            0.5 + 0.5 * a1.sin(),
            x0,
            h,
            z0,
            0.0,
            1.0,
            0.0,
            0.5 + 0.5 * a0.cos(),
            0.5 + 0.5 * a0.sin(),
        ]);

        // Bottom cap
        vertices.extend_from_slice(&[
            0.0,
            -h,
            0.0,
            0.0,
            -1.0,
            0.0,
            0.5,
            0.5,
            x0,
            -h,
            z0,
            0.0,
            -1.0,
            0.0,
            0.5 + 0.5 * a0.cos(),
            0.5 + 0.5 * a0.sin(),
            x1,
            -h,
            z1,
            0.0,
            -1.0,
            0.0,
            0.5 + 0.5 * a1.cos(),
            0.5 + 0.5 * a1.sin(),
        ]);
    }

    vertices
}
