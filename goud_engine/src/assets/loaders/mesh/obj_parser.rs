//! OBJ mesh extraction via the `tobj` crate.
//!
//! Parses Wavefront OBJ files (with optional MTL material references)
//! into [`MeshAsset`] data.

#[cfg(feature = "native")]
use crate::assets::AssetLoadError;
#[cfg(feature = "native")]
use super::asset::{MeshAsset, MeshVertex, SubMesh};

/// Parses an OBJ file from raw bytes into a [`MeshAsset`].
///
/// Each named object/group in the OBJ becomes a [`SubMesh`].
/// Material references are recorded in `SubMesh::material_index` but
/// material data itself is not loaded here.
///
/// # Errors
///
/// Returns [`AssetLoadError::DecodeFailed`] if `tobj` cannot parse the
/// input or if the file contains no mesh data.
#[cfg(feature = "native")]
pub(super) fn parse_obj(bytes: &[u8]) -> Result<MeshAsset, AssetLoadError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj_buf(&mut cursor, &load_options, |_mtl_path| {
        // We do not load MTL files here; material loading is a separate concern.
        Err(tobj::LoadError::GenericFailure)
    })
    .map_err(|e| AssetLoadError::decode_failed(format!("OBJ parse error: {e}")))?;

    if models.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "OBJ file contains no mesh data",
        ));
    }

    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sub_meshes: Vec<SubMesh> = Vec::new();

    for model in &models {
        let mesh = &model.mesh;
        let base_vertex = vertices.len() as u32;
        let start_index = indices.len() as u32;

        let position_count = mesh.positions.len() / 3;
        let has_normals = !mesh.normals.is_empty();
        let has_uvs = !mesh.texcoords.is_empty();

        for i in 0..position_count {
            let position = [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ];
            let normal = if has_normals {
                [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]
            } else {
                [0.0, 0.0, 1.0]
            };
            let uv = if has_uvs {
                [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
            } else {
                [0.0, 0.0]
            };

            vertices.push(MeshVertex {
                position,
                normal,
                uv,
            });
        }

        for &idx in &mesh.indices {
            indices.push(base_vertex + idx);
        }

        let index_count = mesh.indices.len() as u32;
        sub_meshes.push(SubMesh {
            name: model.name.clone(),
            start_index,
            index_count,
            material_index: mesh.material_id.map(|id| id as u32),
        });
    }

    if vertices.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "OBJ file contains no vertex data",
        ));
    }

    Ok(MeshAsset {
        vertices,
        indices,
        sub_meshes,
    })
}
