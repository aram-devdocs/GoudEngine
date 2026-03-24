//! Helper functions for glTF mesh/image extraction.

use crate::assets::{AssetLoadError, LoadContext};

pub(super) fn process_embedded_images(
    gltf: &gltf::Gltf,
    buffers: &[Vec<u8>],
    context: &mut LoadContext,
) -> Result<std::collections::HashMap<usize, String>, AssetLoadError> {
    use crate::assets::loaders::gltf_utils::decode_data_uri;
    let mut image_paths = std::collections::HashMap::new();

    for (image_idx, image) in gltf.images().enumerate() {
        // Process all image sources.
        match image.source() {
            // Image data from a buffer view (embedded in GLB).
            gltf::image::Source::View { view, mime_type } => {
                let offset = view.offset();
                let len = view.length();
                if let Some(buffer_data) = buffers.get(view.buffer().index()) {
                    if offset + len <= buffer_data.len() {
                        let image_bytes = buffer_data[offset..offset + len].to_vec();
                        let ext = if mime_type.contains("png") {
                            "png"
                        } else if mime_type.contains("jpeg") {
                            "jpg"
                        } else {
                            "img"
                        };
                        let base_name = context.path().stem().unwrap_or("image").to_string();
                        let dir = context.path().directory().unwrap_or("").to_string();
                        let asset_path = if dir.is_empty() {
                            format!("{}__embedded_image_{}.{}", base_name, image_idx, ext)
                        } else {
                            format!(
                                "{}/{}__embedded_image_{}.{}",
                                dir, base_name, image_idx, ext
                            )
                        };
                        context.add_embedded_asset(&asset_path, image_bytes);
                        image_paths.insert(image_idx, asset_path);
                    }
                }
            }
            // Image from a URI (data: or external file).
            gltf::image::Source::Uri { uri, mime_type } => {
                if uri.starts_with("data:") {
                    // Data URI — decode and emit as embedded asset.
                    let data = decode_data_uri(uri)?;
                    let ext = if mime_type
                        .as_ref()
                        .map(|m| m.contains("png"))
                        .unwrap_or(false)
                        || uri.contains("png")
                    {
                        "png"
                    } else if mime_type
                        .as_ref()
                        .map(|m| m.contains("jpeg"))
                        .unwrap_or(false)
                        || uri.contains("jpeg")
                    {
                        "jpg"
                    } else {
                        "img"
                    };
                    let base_name = context.path().stem().unwrap_or("image").to_string();
                    let dir = context.path().directory().unwrap_or("").to_string();
                    let asset_path = if dir.is_empty() {
                        format!("{}__embedded_image_{}.{}", base_name, image_idx, ext)
                    } else {
                        format!(
                            "{}/{}__embedded_image_{}.{}",
                            dir, base_name, image_idx, ext
                        )
                    };
                    context.add_embedded_asset(&asset_path, data);
                    image_paths.insert(image_idx, asset_path);
                } else {
                    // External file URI — resolve relative to asset directory.
                    let base = context.path().directory().unwrap_or("");
                    let resolved = if base.is_empty() {
                        uri.to_string()
                    } else {
                        format!("{}/{}", base, uri)
                    };
                    image_paths.insert(image_idx, resolved);
                }
            }
        }
    }

    Ok(image_paths)
}

/// Recursively process a node and its children, extracting mesh data and applying transformations.
#[allow(clippy::too_many_arguments)]
pub(super) fn process_node(
    node: &gltf::Node,
    buffers: &[Vec<u8>],
    image_paths: &std::collections::HashMap<usize, String>,
    vertices: &mut Vec<crate::core::types::MeshVertex>,
    indices: &mut Vec<u32>,
    sub_meshes: &mut Vec<crate::core::types::SubMesh>,
    mesh_bounds: &mut crate::core::types::MeshBounds,
    has_bounds: &mut bool,
    all_bone_indices: &mut Vec<[u32; 4]>,
    all_bone_weights: &mut Vec<[f32; 4]>,
) {
    use crate::core::types::{MeshBounds, MeshVertex, SubMesh};

    // If this node has a mesh, process it.
    if let Some(mesh) = node.mesh() {
        let node_name = if let Some(name) = node.name() {
            name.to_string()
        } else {
            format!("node_{}", node.index())
        };
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));

            let positions: Vec<[f32; 3]> = match reader.read_positions() {
                Some(pos) => pos.collect(),
                None => {
                    log::warn!("GLTF primitive in node '{}' missing POSITION", node_name);
                    continue;
                }
            };

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| compute_face_normals(&positions));

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            // Read JOINTS_0 / WEIGHTS_0 from the same reader/primitive.
            let joints: Option<Vec<[u16; 4]>> =
                reader.read_joints(0).map(|j| j.into_u16().collect());
            let weights: Option<Vec<[f32; 4]>> =
                reader.read_weights(0).map(|w| w.into_f32().collect());

            // Apply node transform to positions.
            let (translation, _rotation, scale) = node.transform().decomposed();

            let base_vertex = vertices.len() as u32;
            let start_index = indices.len() as u32;

            for i in 0..positions.len() {
                let mut pos = positions[i];
                // Apply translation and scale from the node transform.
                pos[0] = pos[0] * scale[0] + translation[0];
                pos[1] = pos[1] * scale[1] + translation[1];
                pos[2] = pos[2] * scale[2] + translation[2];

                vertices.push(MeshVertex {
                    position: pos,
                    normal: if i < normals.len() {
                        normals[i]
                    } else {
                        [0.0, 0.0, 1.0]
                    },
                    uv: if i < uvs.len() { uvs[i] } else { [0.0, 0.0] },
                });

                // Bone data for this vertex (defaults to zero if absent).
                let bi = joints
                    .as_ref()
                    .and_then(|j| j.get(i))
                    .map(|j| [j[0] as u32, j[1] as u32, j[2] as u32, j[3] as u32])
                    .unwrap_or([0; 4]);
                let bw = weights
                    .as_ref()
                    .and_then(|w| w.get(i))
                    .copied()
                    .unwrap_or([0.0; 4]);
                all_bone_indices.push(bi);
                all_bone_weights.push(bw);
            }

            // Re-create reader for index access (positions reader was consumed).
            let reader = primitive.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));
            if let Some(idx_reader) = reader.read_indices() {
                for idx in idx_reader.into_u32() {
                    indices.push(base_vertex + idx);
                }
            } else {
                // Unindexed: sequential vertex order.
                for i in 0..positions.len() as u32 {
                    indices.push(base_vertex + i);
                }
            }

            let bounds = MeshBounds::from_positions(&positions);
            if *has_bounds {
                *mesh_bounds = mesh_bounds.union(bounds);
            } else {
                *mesh_bounds = bounds;
                *has_bounds = true;
            }

            let index_count = indices.len() as u32 - start_index;
            let mesh_name = mesh.name().unwrap_or("mesh");
            let primitive_idx = primitive.index();

            // Construct submesh name: node_name.mesh_name.primitive_index
            let submesh_name = format!("{}.{}.primitive_{}", node_name, mesh_name, primitive_idx);

            let mut material = extract_primitive_material(&primitive);
            // If material has a base color texture, check if it's an embedded image.
            if let Some(ref mut mat) = material {
                if let Some(bc_tex) = primitive
                    .material()
                    .pbr_metallic_roughness()
                    .base_color_texture()
                {
                    let image_idx = bc_tex.texture().source().index();
                    if let Some(path) = image_paths.get(&image_idx) {
                        mat.base_color_texture_path = Some(path.clone());
                    }
                }
            }

            sub_meshes.push(SubMesh {
                name: submesh_name,
                start_index,
                index_count,
                material_index: primitive.material().index().map(|i| i as u32),
                material,
                bounds,
            });
        }
    }

    // Process children recursively.
    for child in node.children() {
        process_node(
            &child,
            buffers,
            image_paths,
            vertices,
            indices,
            sub_meshes,
            mesh_bounds,
            has_bounds,
            all_bone_indices,
            all_bone_weights,
        );
    }
}
pub(super) fn extract_primitive_material(
    primitive: &gltf::Primitive,
) -> Option<crate::core::types::MeshMaterial> {
    use crate::core::types::MeshMaterial;
    let mat = primitive.material();
    let pbr = mat.pbr_metallic_roughness();
    let bc = pbr.base_color_factor();
    Some(MeshMaterial {
        name: mat.name().map(|s| s.to_string()),
        base_color_factor: bc,
        base_color_texture_path: None,
        normal_texture_path: None,
        metallic_roughness_texture_path: None,
        emissive_texture_path: None,
        emissive_factor: [0.0, 0.0, 0.0],
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        alpha_cutoff: None,
        double_sided: mat.double_sided(),
    })
}
pub(super) fn compute_face_normals(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32, 0.0, 1.0]; positions.len()];
    for tri in 0..(positions.len() / 3) {
        let i0 = tri * 3;
        let (v0, v1, v2) = (positions[i0], positions[i0 + 1], positions[i0 + 2]);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let n = if len > 1e-8 {
            [nx / len, ny / len, nz / len]
        } else {
            [0.0, 0.0, 1.0]
        };
        normals[i0] = n;
        normals[i0 + 1] = n;
        normals[i0 + 2] = n;
    }
    normals
}
