//! glTF/GLB [`ModelProvider`] implementation.
//!
//! Wraps the existing [`parse_gltf`](super::super::gltf_parser::parse_gltf)
//! parser behind the provider trait.  Extracts skeleton (skin) data and all
//! animations from the glTF document.

use crate::assets::loaders::animation::keyframe::{AnimationChannel, EasingFunction, Keyframe};
use crate::assets::loaders::animation::KeyframeAnimation;
use crate::assets::{AssetLoadError, LoadContext};

use super::super::provider::{BoneData, ModelData, ModelProvider, SkeletonData};

/// glTF 2.0 model provider (JSON and binary GLB variants).
#[derive(Debug, Clone, Copy, Default)]
pub struct GltfProvider;

impl ModelProvider for GltfProvider {
    fn name(&self) -> &str {
        "glTF"
    }

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }

    fn load(&self, bytes: &[u8], _context: &mut LoadContext) -> Result<ModelData, AssetLoadError> {
        use crate::core::types::{MeshAsset, MeshBounds, MeshVertex, SubMesh};

        let gltf = gltf::Gltf::from_slice(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("GLTF parse error: {e}")))?;

        let buffers = load_gltf_buffers(&gltf)?;

        // Read mesh data directly from the first mesh's primitives.
        // This avoids the scene-node flattening path which can reorder vertices.
        let gltf_mesh = gltf
            .meshes()
            .next()
            .ok_or_else(|| AssetLoadError::decode_failed("GLTF contains no meshes"))?;

        let mut vertices: Vec<MeshVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut sub_meshes: Vec<SubMesh> = Vec::new();
        let mut mesh_bounds = MeshBounds::default();
        let mut has_bounds = false;

        for primitive in gltf_mesh.primitives() {
            let reader =
                primitive.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| {
                    AssetLoadError::decode_failed("GLTF primitive missing POSITION")
                })?
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| compute_face_normals(&positions));

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let base_vertex = vertices.len() as u32;
            let start_index = indices.len() as u32;

            for i in 0..positions.len() {
                vertices.push(MeshVertex {
                    position: positions[i],
                    normal: if i < normals.len() {
                        normals[i]
                    } else {
                        [0.0, 0.0, 1.0]
                    },
                    uv: if i < uvs.len() {
                        uvs[i]
                    } else {
                        [0.0, 0.0]
                    },
                });
            }

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
            if has_bounds {
                mesh_bounds = mesh_bounds.union(bounds);
            } else {
                mesh_bounds = bounds;
                has_bounds = true;
            }

            let index_count = indices.len() as u32 - start_index;
            sub_meshes.push(SubMesh {
                name: gltf_mesh
                    .name()
                    .unwrap_or("primitive")
                    .to_string(),
                start_index,
                index_count,
                material_index: primitive.material().index().map(|i| i as u32),
                material: extract_primitive_material(&primitive),
                bounds,
            });
        }

        if vertices.is_empty() {
            return Err(AssetLoadError::decode_failed("GLTF mesh has no vertices"));
        }

        let mesh = MeshAsset {
            vertices,
            indices,
            sub_meshes,
            bounds: mesh_bounds,
        };

        let skeleton = extract_skeleton(&gltf, &buffers, mesh.vertices.len());
        let animations = extract_all_animations(&gltf, &buffers);

        Ok(ModelData {
            mesh,
            skeleton,
            animations,
        })
    }
}

/// Extract material properties from a glTF primitive's material.
fn extract_primitive_material(
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

/// Compute per-face normals from triangle positions when NORMAL is absent.
fn compute_face_normals(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
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

/// Load buffer data from glTF (supports embedded GLB and data URIs).
fn load_gltf_buffers(gltf: &gltf::Gltf) -> Result<Vec<Vec<u8>>, AssetLoadError> {
    use crate::assets::loaders::gltf_utils::decode_data_uri;

    let mut buffers = Vec::new();
    for buffer in gltf.buffers() {
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => gltf
                .blob
                .as_deref()
                .ok_or_else(|| AssetLoadError::decode_failed("GLB binary chunk missing"))?
                .to_vec(),
            gltf::buffer::Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    decode_data_uri(uri)?
                } else {
                    return Err(AssetLoadError::decode_failed(format!(
                        "External buffer URI not supported in model provider: {uri}"
                    )));
                }
            }
        };
        buffers.push(data);
    }
    Ok(buffers)
}

/// Extract skeleton data from the first skin in the glTF document.
fn extract_skeleton(
    gltf: &gltf::Gltf,
    buffers: &[Vec<u8>],
    vertex_count: usize,
) -> Option<SkeletonData> {
    let skin = gltf.skins().next()?;
    let joints: Vec<gltf::Node> = skin.joints().collect();

    // Build a map from node index to joint index.
    let joint_index_map: std::collections::HashMap<usize, usize> = joints
        .iter()
        .enumerate()
        .map(|(i, node)| (node.index(), i))
        .collect();

    // Build a parent map: node_index -> parent_node_index.
    // The gltf crate does not expose parent directly, so we walk the tree.
    let node_parent_map = build_node_parent_map(gltf);

    // Read inverse bind matrices.
    let reader = skin.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));
    let inverse_bind_matrices: Vec<[[f32; 4]; 4]> = reader
        .read_inverse_bind_matrices()
        .map(|ibm| ibm.collect())
        .unwrap_or_default();

    let identity_ibm: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let mut bones = Vec::with_capacity(joints.len());
    for (i, joint_node) in joints.iter().enumerate() {
        // Find the parent joint index by walking up the node tree.
        let parent_index = node_parent_map
            .get(&joint_node.index())
            .and_then(|&parent_node_idx| joint_index_map.get(&parent_node_idx).copied())
            .map(|pi| pi as i32)
            .unwrap_or(-1);

        let ibm_col_major = inverse_bind_matrices
            .get(i)
            .copied()
            .unwrap_or(identity_ibm);

        // glTF inverse-bind matrices are already column-major (mat[col][row]).
        // Flatten without transposing.
        let ibm = flatten_mat4(&ibm_col_major);

        bones.push(BoneData {
            name: joint_node.name().unwrap_or("unnamed_bone").to_string(),
            parent_index,
            inverse_bind_matrix: ibm,
        });
    }

    // Extract per-vertex bone indices and weights from the first mesh primitive
    // that has JOINTS_0 and WEIGHTS_0 attributes.
    let mut bone_indices = vec![[0u32; 4]; vertex_count];
    let mut bone_weights = vec![[0.0f32; 4]; vertex_count];

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));

            if let (Some(joints_reader), Some(weights_reader)) =
                (reader.read_joints(0), reader.read_weights(0))
            {
                let joints_data: Vec<[u16; 4]> = joints_reader.into_u16().collect();
                let weights_data: Vec<[f32; 4]> = weights_reader.into_f32().collect();

                let count = joints_data.len().min(vertex_count);
                for i in 0..count {
                    bone_indices[i] = [
                        joints_data[i][0] as u32,
                        joints_data[i][1] as u32,
                        joints_data[i][2] as u32,
                        joints_data[i][3] as u32,
                    ];
                    bone_weights[i] = weights_data[i];
                }

                // Use data from the first primitive that has skin attributes.
                break;
            }
        }
    }

    Some(SkeletonData {
        bones,
        bone_indices,
        bone_weights,
    })
}

/// Extract all animations from the glTF document.
///
/// Channel target properties are named using the **joint index** (0..N)
/// rather than the glTF node index so that `compute_bone_matrices` can
/// look them up by bone index directly.
fn extract_all_animations(gltf: &gltf::Gltf, buffers: &[Vec<u8>]) -> Vec<KeyframeAnimation> {
    // Build a mapping from glTF node index to skeleton joint index.
    // This ensures animation channels reference the same indices used by
    // `compute_bone_matrices` (which iterates 0..bone_count).
    let joint_map: std::collections::HashMap<usize, usize> = gltf
        .skins()
        .next()
        .map(|skin| {
            skin.joints()
                .enumerate()
                .map(|(i, node)| (node.index(), i))
                .collect()
        })
        .unwrap_or_default();

    let mut animations = Vec::new();

    for gltf_anim in gltf.animations() {
        let name = gltf_anim.name().unwrap_or("unnamed").to_string();
        let mut channels = Vec::new();
        let mut max_time: f32 = 0.0;

        for channel in gltf_anim.channels() {
            let target = channel.target();
            let node_index = target.node().index();
            let property = match target.property() {
                gltf::animation::Property::Translation => "translation",
                gltf::animation::Property::Rotation => "rotation",
                gltf::animation::Property::Scale => "scale",
                gltf::animation::Property::MorphTargetWeights => "morph_weights",
            };

            let reader = channel.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));

            let timestamps: Vec<f32> = match reader.read_inputs() {
                Some(inputs) => inputs.collect(),
                None => continue,
            };

            let values: Vec<f32> = match reader.read_outputs() {
                Some(outputs) => match outputs {
                    gltf::animation::util::ReadOutputs::Translations(v) => {
                        v.flat_map(|t| t.into_iter()).collect()
                    }
                    gltf::animation::util::ReadOutputs::Rotations(v) => {
                        v.into_f32().flat_map(|r| r.into_iter()).collect()
                    }
                    gltf::animation::util::ReadOutputs::Scales(v) => {
                        v.flat_map(|s| s.into_iter()).collect()
                    }
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(v) => {
                        v.into_f32().collect()
                    }
                },
                None => continue,
            };

            let component_count = if timestamps.is_empty() {
                1
            } else {
                values.len() / timestamps.len()
            };

            // Skip channels that don't target a skeleton joint.
            let joint_idx = match joint_map.get(&node_index) {
                Some(&ji) => ji,
                None => continue,
            };

            for comp in 0..component_count {
                let suffix = match component_count {
                    3 => ["x", "y", "z"][comp],
                    4 => ["x", "y", "z", "w"][comp],
                    _ => "value",
                };

                let target_property = format!("node_{joint_idx}.{property}.{suffix}");

                let keyframes: Vec<Keyframe> = timestamps
                    .iter()
                    .enumerate()
                    .map(|(i, &time)| {
                        let value = values
                            .get(i * component_count + comp)
                            .copied()
                            .unwrap_or(0.0);
                        if time > max_time {
                            max_time = time;
                        }
                        Keyframe {
                            time,
                            value,
                            easing: EasingFunction::Linear,
                        }
                    })
                    .collect();

                channels.push(AnimationChannel {
                    target_property,
                    keyframes,
                });
            }
        }

        animations.push(KeyframeAnimation::new(name, max_time, channels));
    }

    animations
}

/// Build a map from child node index to parent node index by walking all scenes.
fn build_node_parent_map(gltf: &gltf::Gltf) -> std::collections::HashMap<usize, usize> {
    let mut parent_map = std::collections::HashMap::new();

    fn walk_node(node: gltf::Node, parent_map: &mut std::collections::HashMap<usize, usize>) {
        for child in node.children() {
            parent_map.insert(child.index(), node.index());
            walk_node(child, parent_map);
        }
    }

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            walk_node(node, &mut parent_map);
        }
    }

    parent_map
}

/// Flatten a `[[f32; 4]; 4]` matrix (indexed as `mat[col][row]`) into a
/// column-major `[f32; 16]`.
///
/// The glTF crate returns inverse-bind matrices in column-major
/// array-of-arrays form where `mat[col][row]`.  Flattening without
/// transposition yields the correct column-major layout:
///   `[col0row0, col0row1, col0row2, col0row3, col1row0, ...]`.
fn flatten_mat4(m: &[[f32; 4]; 4]) -> [f32; 16] {
    [
        m[0][0], m[0][1], m[0][2], m[0][3], // column 0
        m[1][0], m[1][1], m[1][2], m[1][3], // column 1
        m[2][0], m[2][1], m[2][2], m[2][3], // column 2
        m[3][0], m[3][1], m[3][2], m[3][3], // column 3
    ]
}
