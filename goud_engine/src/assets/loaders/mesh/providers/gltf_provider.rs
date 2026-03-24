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

    fn load(&self, bytes: &[u8], context: &mut LoadContext) -> Result<ModelData, AssetLoadError> {
        let mesh = super::super::gltf_parser::parse_gltf(bytes, context)?;

        // Parse glTF document again for skin and animation data.
        // If buffer loading fails (e.g. external URIs), we still return the mesh
        // but without skeleton/animation data.
        let gltf = gltf::Gltf::from_slice(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("GLTF parse error: {e}")))?;

        let (skeleton, animations) = match load_gltf_buffers(&gltf) {
            Ok(buffers) => {
                let skeleton = extract_skeleton(&gltf, &buffers, mesh.vertices.len());
                let animations = extract_all_animations(&gltf, &buffers);
                (skeleton, animations)
            }
            Err(e) => {
                log::warn!("Failed to load glTF buffers for skeleton/animation extraction: {e}");
                // Buffer loading failed (external URIs, etc.) — skip skin/animation extraction.
                (None, vec![])
            }
        };

        Ok(ModelData {
            mesh,
            skeleton,
            animations,
        })
    }
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

        let ibm_row_major = inverse_bind_matrices
            .get(i)
            .copied()
            .unwrap_or(identity_ibm);

        // Convert from row-major glTF format to column-major.
        let ibm = row_major_to_column_major(&ibm_row_major);

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
fn extract_all_animations(gltf: &gltf::Gltf, buffers: &[Vec<u8>]) -> Vec<KeyframeAnimation> {
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

            for comp in 0..component_count {
                let suffix = match component_count {
                    3 => ["x", "y", "z"][comp],
                    4 => ["x", "y", "z", "w"][comp],
                    _ => "value",
                };

                let target_property = format!("node_{node_index}.{property}.{suffix}");

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

/// Convert a 4x4 matrix from row-major `[[f32; 4]; 4]` to column-major `[f32; 16]`.
fn row_major_to_column_major(m: &[[f32; 4]; 4]) -> [f32; 16] {
    [
        m[0][0], m[1][0], m[2][0], m[3][0], // column 0
        m[0][1], m[1][1], m[2][1], m[3][1], // column 1
        m[0][2], m[1][2], m[2][2], m[3][2], // column 2
        m[0][3], m[1][3], m[2][3], m[3][3], // column 3
    ]
}
