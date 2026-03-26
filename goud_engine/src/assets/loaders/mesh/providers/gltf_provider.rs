//! glTF/GLB [`ModelProvider`] implementation.
//!
//! Wraps the existing [`parse_gltf`](super::super::gltf_parser::parse_gltf)
//! parser behind the provider trait.  Extracts skeleton (skin) data and all
//! animations from the glTF document.

use crate::assets::loaders::animation::keyframe::{AnimationChannel, EasingFunction, Keyframe};
use crate::assets::loaders::animation::KeyframeAnimation;
use crate::assets::{AssetLoadError, LoadContext};

use super::super::provider::{BoneData, ModelData, ModelProvider, SkeletonData};
use super::gltf_helpers::{process_embedded_images, process_node};

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
        use crate::core::types::{MeshAsset, MeshBounds, MeshVertex, SubMesh};

        let gltf = gltf::Gltf::from_slice(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("GLTF parse error: {e}")))?;

        let buffers = load_gltf_buffers(&gltf, context)?;

        // Process embedded images first to map image indices to asset paths.
        let image_paths = process_embedded_images(&gltf, &buffers, context)?;

        // Iterate all meshes from scene nodes to flatten multiple nodes into a single mesh.
        // This ensures multi-node scenes produce a merged vertex buffer.
        let scenes: Vec<_> = gltf.scenes().collect();
        if scenes.is_empty() {
            return Err(AssetLoadError::decode_failed("GLTF contains no scenes"));
        }

        let mut vertices: Vec<MeshVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut sub_meshes: Vec<SubMesh> = Vec::new();
        let mut mesh_bounds = MeshBounds::default();
        let mut has_bounds = false;
        // Accumulate per-vertex bone data inline with vertex extraction to
        // guarantee alignment between the two arrays.
        let mut all_bone_indices: Vec<[u32; 4]> = Vec::new();
        let mut all_bone_weights: Vec<[f32; 4]> = Vec::new();

        for scene in scenes {
            for root_node in scene.nodes() {
                process_node(
                    &root_node,
                    &buffers,
                    &image_paths,
                    &mut vertices,
                    &mut indices,
                    &mut sub_meshes,
                    &mut mesh_bounds,
                    &mut has_bounds,
                    &mut all_bone_indices,
                    &mut all_bone_weights,
                );
            }
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

        let skeleton = extract_skeleton(&gltf, &buffers, all_bone_indices, all_bone_weights);
        let animations = extract_all_animations(&gltf, &buffers);

        Ok(ModelData {
            mesh,
            skeleton,
            animations,
        })
    }
}
/// Load buffer data from glTF (embedded GLB, data URIs, and external file URIs).
fn load_gltf_buffers(
    gltf: &gltf::Gltf,
    context: &mut LoadContext,
) -> Result<Vec<Vec<u8>>, AssetLoadError> {
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
                    // External file URI — resolve via LoadContext reader and track as dependency.
                    let base = context.path().directory().unwrap_or("");
                    let resolved = if base.is_empty() {
                        uri.to_string()
                    } else {
                        format!("{base}/{uri}")
                    };
                    context.add_dependency(&resolved);
                    context.read_asset_bytes(&resolved)?
                }
            }
        };
        buffers.push(data);
    }
    Ok(buffers)
}

/// Process embedded images in the glTF document and return a mapping of image index to asset path.
fn extract_skeleton(
    gltf: &gltf::Gltf,
    buffers: &[Vec<u8>],
    mut bone_indices: Vec<[u32; 4]>,
    bone_weights: Vec<[f32; 4]>,
) -> Option<SkeletonData> {
    let skin = gltf.skins().next()?;
    let joints: Vec<gltf::Node> = skin.joints().collect();
    let joint_count = joints.len();

    // Build a map from node index to joint index.
    let joint_index_map: std::collections::HashMap<usize, usize> = joints
        .iter()
        .enumerate()
        .map(|(i, node)| (node.index(), i))
        .collect();

    // Build a parent map: node_index -> parent_node_index.
    let node_parent_map = build_node_parent_map(gltf);

    // Read inverse bind matrices.
    let reader = skin.reader(|buf| buffers.get(buf.index()).map(|b| b.as_slice()));
    let inverse_bind_matrices: Vec<[[f32; 4]; 4]> = reader
        .read_inverse_bind_matrices()
        .map(|ibm| ibm.collect())
        .unwrap_or_default();

    // Phase 4: validate inverse bind matrices are present for skinned meshes.
    if inverse_bind_matrices.is_empty() {
        log::warn!(
            "glTF skin has {} joints but no inverse bind matrices; using identity",
            joint_count
        );
    }
    let identity_ibm: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    // Phase 4: detect parent hierarchy cycles.
    let mut visited = vec![false; joint_count];
    let mut in_stack = vec![false; joint_count];

    let mut bones = Vec::with_capacity(joint_count);
    for (i, joint_node) in joints.iter().enumerate() {
        let mut parent_index = node_parent_map
            .get(&joint_node.index())
            .and_then(|&parent_node_idx| joint_index_map.get(&parent_node_idx).copied())
            .map(|pi| pi as i32)
            .unwrap_or(-1);

        // Ensure parent index is in range.
        if parent_index >= joint_count as i32 {
            log::warn!(
                "Bone {} parent {} exceeds joint count; resetting to -1",
                i,
                parent_index
            );
            parent_index = -1;
        }
        let ibm = flatten_mat4(inverse_bind_matrices.get(i).unwrap_or(&identity_ibm));

        bones.push(BoneData {
            name: joint_node.name().unwrap_or("unnamed_bone").to_string(),
            parent_index,
            inverse_bind_matrix: ibm,
        });
    }

    // Phase 4: detect cycles by DFS on the parent graph.
    for start in 0..joint_count {
        if visited[start] {
            continue;
        }
        let mut path = vec![start];
        in_stack[start] = true;
        loop {
            let current = *path.last().unwrap();
            let pi = bones[current].parent_index;
            if pi < 0 || (pi as usize) >= joint_count {
                break;
            }
            let pu = pi as usize;
            if in_stack[pu] {
                log::error!(
                    "Bone hierarchy cycle detected at bone {}; breaking cycle",
                    pu
                );
                bones[current].parent_index = -1;
                break;
            }
            if visited[pu] {
                break;
            }
            in_stack[pu] = true;
            path.push(pu);
        }
        for &node in &path {
            visited[node] = true;
            in_stack[node] = false;
        }
    }

    // Phase 4: validate bone indices don't exceed joint count.
    for bi in &mut bone_indices {
        for idx in bi.iter_mut() {
            if (*idx as usize) >= joint_count {
                log::warn!(
                    "Bone index {} exceeds joint count {}; clamping to 0",
                    *idx,
                    joint_count
                );
                *idx = 0;
            }
        }
    }

    Some(SkeletonData {
        bones,
        bone_indices,
        bone_weights,
    })
}

/// Extract all animations, mapping glTF node indices to joint indices so `compute_bone_matrices` can
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

/// Flatten `mat[col][row]` to column-major `[f32; 16]` without transposing.
fn flatten_mat4(m: &[[f32; 4]; 4]) -> [f32; 16] {
    [
        m[0][0], m[0][1], m[0][2], m[0][3], // column 0
        m[1][0], m[1][1], m[1][2], m[1][3], // column 1
        m[2][0], m[2][1], m[2][2], m[2][3], // column 2
        m[3][0], m[3][1], m[3][2], m[3][3], // column 3
    ]
}
