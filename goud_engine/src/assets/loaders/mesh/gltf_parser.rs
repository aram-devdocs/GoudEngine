//! GLTF/GLB mesh extraction.
//!
//! Parses GLTF 2.0 (JSON + external buffers) and GLB (binary container)
//! files into [`MeshAsset`] data using the `gltf` crate.

#[cfg(feature = "native")]
use super::asset::{MeshAsset, MeshVertex, SubMesh};
#[cfg(feature = "native")]
use crate::assets::AssetLoadError;

/// Parses a GLTF or GLB file from raw bytes into a [`MeshAsset`].
///
/// All meshes and their primitives are merged into a single vertex/index
/// buffer. Each primitive becomes a [`SubMesh`] entry.
///
/// # Errors
///
/// Returns [`AssetLoadError::DecodeFailed`] if the bytes cannot be parsed
/// as valid GLTF/GLB or if the file contains no mesh data.
#[cfg(feature = "native")]
pub(super) fn parse_gltf(bytes: &[u8]) -> Result<MeshAsset, AssetLoadError> {
    let gltf::Gltf { document, mut blob } = gltf::Gltf::from_slice(bytes)
        .map_err(|e| AssetLoadError::decode_failed(format!("GLTF parse error: {e}")))?;

    // Collect buffer data. For GLB the first buffer is in `blob`.
    let buffers = collect_buffer_data(&document, &mut blob)?;

    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sub_meshes: Vec<SubMesh> = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader =
                primitive.reader(|buffer| buffers.get(buffer.index()).map(|d| d.as_slice()));

            let base_vertex = vertices.len() as u32;
            let start_index = indices.len() as u32;

            // Positions (required)
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| {
                    AssetLoadError::decode_failed("GLTF primitive missing POSITION attribute")
                })?
                .collect();

            // Normals (optional -- default to [0, 0, 1])
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()]);

            // Tex coords (optional -- default to [0, 0])
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            for i in 0..positions.len() {
                vertices.push(MeshVertex {
                    position: positions[i],
                    normal: normals[i],
                    uv: uvs[i],
                });
            }

            // Indices (optional -- generate sequential if missing)
            if let Some(idx_reader) = reader.read_indices() {
                for idx in idx_reader.into_u32() {
                    indices.push(base_vertex + idx);
                }
            } else {
                for i in 0..positions.len() as u32 {
                    indices.push(base_vertex + i);
                }
            }

            let index_count = indices.len() as u32 - start_index;
            let name = mesh
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("mesh_{}", mesh.index()));

            sub_meshes.push(SubMesh {
                name,
                start_index,
                index_count,
                material_index: primitive.material().index().map(|i| i as u32),
            });
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
    })
}

/// Collects buffer data from GLTF document.
///
/// For GLB files the first buffer comes from `blob`. For plain GLTF
/// with external buffers, only embedded (data-URI) buffers are supported.
#[cfg(feature = "native")]
fn collect_buffer_data(
    document: &gltf::Document,
    blob: &mut Option<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, AssetLoadError> {
    use crate::assets::loaders::gltf_utils::decode_data_uri;

    let mut buffers = Vec::new();
    for buffer in document.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                let data = blob.take().ok_or_else(|| {
                    AssetLoadError::decode_failed("GLB missing binary blob for buffer")
                })?;
                buffers.push(data);
            }
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    buffers.push(decode_data_uri(uri)?);
                } else {
                    return Err(AssetLoadError::decode_failed(format!(
                        "External GLTF buffer URIs are not supported: {uri}"
                    )));
                }
            }
        }
    }
    Ok(buffers)
}
