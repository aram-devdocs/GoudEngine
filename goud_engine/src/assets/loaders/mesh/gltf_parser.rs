//! GLTF/GLB mesh extraction.
//!
//! Parses GLTF 2.0 (JSON + external buffers) and GLB (binary container)
//! files into [`MeshAsset`] data using the `gltf` crate.

#[cfg(feature = "native")]
use super::asset::{MeshAsset, MeshBounds, MeshVertex, SubMesh};
#[cfg(feature = "native")]
use crate::assets::{AssetLoadError, LoadContext};
#[cfg(feature = "native")]
use cgmath::{Matrix4, SquareMatrix};

#[cfg(feature = "native")]
mod support;
#[cfg(feature = "native")]
use support::{
    collect_buffer_data, collect_image_assets, extract_material, node_transform_matrix,
    primitive_name, transform_normal, transform_position,
};

/// Parses a GLTF or GLB file from raw bytes into a [`MeshAsset`].
///
/// All primitives referenced by the active scene are flattened into a single
/// vertex/index buffer. Each primitive becomes a [`SubMesh`] entry.
#[cfg(feature = "native")]
pub(super) fn parse_gltf(
    bytes: &[u8],
    context: &mut LoadContext,
) -> Result<MeshAsset, AssetLoadError> {
    let gltf::Gltf { document, mut blob } = gltf::Gltf::from_slice(bytes)
        .map_err(|e| AssetLoadError::decode_failed(format!("GLTF parse error: {e}")))?;

    let buffers = collect_buffer_data(&document, &mut blob, context)?;
    let image_paths = collect_image_assets(&document, &buffers, context)?;

    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sub_meshes: Vec<SubMesh> = Vec::new();
    let mut mesh_bounds = MeshBounds::default();
    let mut has_mesh_bounds = false;
    let mut flattened_any = false;

    if let Some(scene) = document.default_scene() {
        for node in scene.nodes() {
            flatten_node(
                node,
                Matrix4::identity(),
                &buffers,
                &image_paths,
                &mut vertices,
                &mut indices,
                &mut sub_meshes,
                &mut mesh_bounds,
                &mut has_mesh_bounds,
                &mut flattened_any,
            )?;
        }
    } else {
        for scene in document.scenes() {
            for node in scene.nodes() {
                flatten_node(
                    node,
                    Matrix4::identity(),
                    &buffers,
                    &image_paths,
                    &mut vertices,
                    &mut indices,
                    &mut sub_meshes,
                    &mut mesh_bounds,
                    &mut has_mesh_bounds,
                    &mut flattened_any,
                )?;
            }
        }
    }

    if !flattened_any {
        for mesh in document.meshes() {
            append_mesh_primitives(
                None,
                mesh,
                Matrix4::identity(),
                &buffers,
                &image_paths,
                &mut vertices,
                &mut indices,
                &mut sub_meshes,
                &mut mesh_bounds,
                &mut has_mesh_bounds,
            )?;
        }
    }

    if vertices.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "GLTF file contains no mesh data",
        ));
    }

    Ok(MeshAsset {
        vertices,
        indices,
        sub_meshes,
        bounds: mesh_bounds,
    })
}

#[cfg(feature = "native")]
#[allow(clippy::too_many_arguments)]
fn flatten_node(
    node: gltf::Node,
    parent_transform: Matrix4<f32>,
    buffers: &[Vec<u8>],
    image_paths: &[Option<String>],
    vertices: &mut Vec<MeshVertex>,
    indices: &mut Vec<u32>,
    sub_meshes: &mut Vec<SubMesh>,
    mesh_bounds: &mut MeshBounds,
    has_mesh_bounds: &mut bool,
    flattened_any: &mut bool,
) -> Result<(), AssetLoadError> {
    let world_transform = parent_transform * node_transform_matrix(node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        append_mesh_primitives(
            Some(node.clone()),
            mesh,
            world_transform,
            buffers,
            image_paths,
            vertices,
            indices,
            sub_meshes,
            mesh_bounds,
            has_mesh_bounds,
        )?;
        *flattened_any = true;
    }

    for child in node.children() {
        flatten_node(
            child,
            world_transform,
            buffers,
            image_paths,
            vertices,
            indices,
            sub_meshes,
            mesh_bounds,
            has_mesh_bounds,
            flattened_any,
        )?;
    }

    Ok(())
}

#[cfg(feature = "native")]
#[allow(clippy::too_many_arguments)]
fn append_mesh_primitives(
    node: Option<gltf::Node>,
    mesh: gltf::Mesh,
    transform: Matrix4<f32>,
    buffers: &[Vec<u8>],
    image_paths: &[Option<String>],
    vertices: &mut Vec<MeshVertex>,
    indices: &mut Vec<u32>,
    sub_meshes: &mut Vec<SubMesh>,
    mesh_bounds: &mut MeshBounds,
    has_mesh_bounds: &mut bool,
) -> Result<(), AssetLoadError> {
    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|d| d.as_slice()));

        let base_vertex = vertices.len() as u32;
        let start_index = indices.len() as u32;

        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or_else(|| {
                AssetLoadError::decode_failed("GLTF primitive missing POSITION attribute")
            })?
            .map(|position| transform_position(transform, position))
            .collect();

        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .map(|normals| {
                normals
                    .map(|normal| transform_normal(transform, normal))
                    .collect()
            })
            .unwrap_or_else(|| {
                // Compute per-face normals from triangle geometry when normals are missing
                let face_normals = compute_face_normals(&positions);
                // Apply transform to each normal
                face_normals
                    .into_iter()
                    .map(|normal| transform_normal(transform, normal))
                    .collect()
            });

        let uvs: Vec<[f32; 2]> = reader
            .read_tex_coords(0)
            .map(|coords| coords.into_f32().collect())
            .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

        for i in 0..positions.len() {
            vertices.push(MeshVertex {
                position: positions[i],
                normal: normals[i],
                uv: uvs[i],
            });
        }

        if let Some(idx_reader) = reader.read_indices() {
            for idx in idx_reader.into_u32() {
                indices.push(base_vertex + idx);
            }
        } else {
            for i in 0..positions.len() as u32 {
                indices.push(base_vertex + i);
            }
        }

        let bounds = MeshBounds::from_positions(&positions);
        if *has_mesh_bounds {
            *mesh_bounds = mesh_bounds.union(bounds);
        } else {
            *mesh_bounds = bounds;
            *has_mesh_bounds = true;
        }

        let index_count = indices.len() as u32 - start_index;
        sub_meshes.push(SubMesh {
            name: primitive_name(node.as_ref(), &mesh, &primitive),
            start_index,
            index_count,
            material_index: primitive.material().index().map(|i| i as u32),
            material: extract_material(primitive.material(), image_paths),
            bounds,
        });
    }

    Ok(())
}

#[cfg(feature = "native")]
/// Compute per-face (flat-shaded) normals from triangle vertex positions.
///
/// For each triangle (3 consecutive vertices), computes the face normal from the
/// cross product of two edge vectors and assigns it to all 3 vertices of that triangle.
/// Falls back to a default upward normal for degenerate triangles.
fn compute_face_normals(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    for tri in 0..(positions.len() / 3) {
        let i0 = tri * 3;
        let i1 = tri * 3 + 1;
        let i2 = tri * 3 + 2;

        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        // Edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product: e1 × e2
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];

        // Normalize
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let normal = if len > 1e-8 {
            [nx / len, ny / len, nz / len]
        } else {
            [0.0, 0.0, 1.0] // Degenerate triangle: use default normal
        };

        normals[i0] = normal;
        normals[i1] = normal;
        normals[i2] = normal;
    }

    normals
}
