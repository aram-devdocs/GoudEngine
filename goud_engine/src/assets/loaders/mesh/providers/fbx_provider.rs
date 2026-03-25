//! FBX [`ModelProvider`] implementation.
//!
//! Uses Assimp via `russimp` so FBX assets retain skeletons, animations, and
//! material metadata instead of degrading to mesh-only imports.

use crate::assets::loaders::animation::keyframe::{AnimationChannel, EasingFunction, Keyframe};
use crate::assets::loaders::mesh::asset::{
    MeshAsset, MeshBounds, MeshMaterial, MeshVertex, SubMesh,
};
use crate::assets::{AssetLoadError, LoadContext};
use crate::core::types::{BoneData, KeyframeAnimation, ModelData, SkeletonData};

use super::super::provider::ModelProvider;

use russimp::animation::NodeAnim;
use russimp::material::{Material, PropertyTypeInfo};
use russimp::mesh::Mesh;
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// FBX model provider backed by Assimp.
#[derive(Debug, Clone, Copy, Default)]
pub struct FbxProvider;

impl ModelProvider for FbxProvider {
    fn name(&self) -> &str {
        "FBX"
    }

    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }

    fn load(&self, _bytes: &[u8], context: &mut LoadContext) -> Result<ModelData, AssetLoadError> {
        parse_fbx(context.path_str())
    }
}

fn parse_fbx(path: &str) -> Result<ModelData, AssetLoadError> {
    let scene = Scene::from_file(
        path,
        vec![
            PostProcess::Triangulate,
            PostProcess::JoinIdenticalVertices,
            PostProcess::SortByPrimitiveType,
            PostProcess::LimitBoneWeights,
            PostProcess::ValidateDataStructure,
        ],
    )
    .map_err(|e| AssetLoadError::decode_failed(format!("FBX import error: {e}")))?;

    if scene.meshes.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "FBX file contains no mesh data",
        ));
    }

    let root = scene
        .root
        .as_ref()
        .ok_or_else(|| AssetLoadError::decode_failed("FBX scene has no root node"))?;
    let mut nodes_by_name: HashMap<String, Rc<Node>> = HashMap::new();
    collect_nodes(root, &mut nodes_by_name);

    let skeleton = extract_skeleton(&scene, &nodes_by_name);
    let bone_index_by_name = skeleton
        .as_ref()
        .map(build_bone_name_index)
        .unwrap_or_default();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut sub_meshes = Vec::new();
    let mut merged_bone_indices = Vec::new();
    let mut merged_bone_weights = Vec::new();

    for mesh in &scene.meshes {
        let base_vertex = vertices.len() as u32;
        let start_index = indices.len() as u32;
        let material = scene
            .materials
            .get(mesh.material_index as usize)
            .map(material_from_assimp);

        let local_weights = build_mesh_bone_weights(mesh, &bone_index_by_name);
        let mut sub_positions = Vec::with_capacity(mesh.vertices.len());

        for (vertex_index, position) in mesh.vertices.iter().enumerate() {
            let normal = mesh
                .normals
                .get(vertex_index)
                .map(|n| [n.x, n.y, n.z])
                .unwrap_or([0.0, 1.0, 0.0]);
            let uv = mesh
                .texture_coords
                .first()
                .and_then(|coords| coords.as_ref())
                .and_then(|coords| coords.get(vertex_index))
                .map(|coord| [coord.x, coord.y])
                .unwrap_or([0.0, 0.0]);

            let pos = [position.x, position.y, position.z];
            vertices.push(MeshVertex {
                position: pos,
                normal,
                uv,
            });
            sub_positions.push(pos);

            if let Some((bone_ids, bone_weights)) = local_weights.get(vertex_index) {
                merged_bone_indices.push(*bone_ids);
                merged_bone_weights.push(normalize_bone_weights(*bone_weights));
            } else if skeleton.is_some() {
                merged_bone_indices.push([0; 4]);
                merged_bone_weights.push([0.0; 4]);
            }
        }

        for face in &mesh.faces {
            if face.0.len() != 3 {
                continue;
            }
            indices.extend(face.0.iter().map(|index| base_vertex + *index));
        }

        let bounds = MeshBounds::from_positions(&sub_positions);
        sub_meshes.push(SubMesh {
            name: mesh.name.clone(),
            start_index,
            index_count: indices.len() as u32 - start_index,
            material_index: Some(mesh.material_index),
            material,
            bounds,
        });
    }

    if vertices.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "FBX file contains no supported triangle meshes",
        ));
    }

    let mesh_bounds = MeshBounds::from_positions(
        &vertices.iter().map(|vertex| vertex.position).collect::<Vec<_>>(),
    );

    let skeleton = skeleton.map(|mut skeleton| {
        skeleton.bone_indices = merged_bone_indices;
        skeleton.bone_weights = merged_bone_weights;
        skeleton
    });
    let animations = extract_animations(&scene, &bone_index_by_name);

    Ok(ModelData {
        mesh: MeshAsset {
            vertices,
            indices,
            sub_meshes,
            bounds: mesh_bounds,
        },
        skeleton,
        animations,
    })
}

fn extract_skeleton(
    scene: &Scene,
    nodes_by_name: &HashMap<String, Rc<Node>>,
) -> Option<SkeletonData> {
    let mut bone_names = HashSet::new();
    let mut inverse_bind_by_name = HashMap::new();

    for mesh in &scene.meshes {
        for bone in &mesh.bones {
            bone_names.insert(bone.name.clone());
            inverse_bind_by_name
                .entry(bone.name.clone())
                .or_insert_with(|| assimp_matrix_to_column_major(&bone.offset_matrix));
        }
    }

    if bone_names.is_empty() {
        return None;
    }

    let mut ordered_names = Vec::new();
    if let Some(root) = scene.root.as_ref() {
        collect_bone_names_in_hierarchy(root, &bone_names, &mut ordered_names);
    }
    let ordered_set: HashSet<&str> = ordered_names.iter().map(String::as_str).collect();
    let mut leftovers: Vec<String> = bone_names
        .iter()
        .filter(|name| !ordered_set.contains(name.as_str()))
        .cloned()
        .collect();
    leftovers.sort();
    ordered_names.extend(leftovers);

    let bone_index_by_name: HashMap<String, usize> = ordered_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect();

    let bones = ordered_names
        .iter()
        .map(|name| {
            let node = nodes_by_name.get(name);
            let parent_index = node
                .and_then(|node| nearest_bone_parent(node, &bone_index_by_name))
                .map(|index| index as i32)
                .unwrap_or(-1);
            let local_bind_transform = node
                .map(|node| assimp_matrix_to_column_major(&node.transformation))
                .unwrap_or(IDENTITY_MAT4);
            let inverse_bind_matrix = inverse_bind_by_name
                .get(name)
                .copied()
                .unwrap_or(IDENTITY_MAT4);

            BoneData {
                name: name.clone(),
                parent_index,
                local_bind_transform,
                inverse_bind_matrix,
            }
        })
        .collect();

    Some(SkeletonData {
        bones,
        bone_indices: Vec::new(),
        bone_weights: Vec::new(),
    })
}

fn build_bone_name_index(skeleton: &SkeletonData) -> HashMap<String, usize> {
    skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(index, bone)| (bone.name.clone(), index))
        .collect()
}

fn build_mesh_bone_weights(
    mesh: &Mesh,
    bone_index_by_name: &HashMap<String, usize>,
) -> Vec<([u32; 4], [f32; 4])> {
    let mut weights = vec![([0; 4], [0.0; 4]); mesh.vertices.len()];

    for bone in &mesh.bones {
        let Some(&bone_index) = bone_index_by_name.get(&bone.name) else {
            continue;
        };

        for weight in &bone.weights {
            let vertex_index = weight.vertex_id as usize;
            if vertex_index >= weights.len() {
                continue;
            }
            let (bone_ids, bone_weights) = &mut weights[vertex_index];
            insert_bone_weight(
                bone_ids,
                bone_weights,
                bone_index as u32,
                weight.weight,
            );
        }
    }

    weights
}

fn insert_bone_weight(
    bone_indices: &mut [u32; 4],
    bone_weights: &mut [f32; 4],
    bone_index: u32,
    weight: f32,
) {
    if weight <= 0.0 {
        return;
    }

    let mut insert_slot = None;
    for (slot, existing_weight) in bone_weights.iter().enumerate() {
        if *existing_weight < weight {
            insert_slot = Some(slot);
            break;
        }
    }

    let Some(slot) = insert_slot else {
        return;
    };

    for index in (slot + 1..4).rev() {
        bone_indices[index] = bone_indices[index - 1];
        bone_weights[index] = bone_weights[index - 1];
    }

    bone_indices[slot] = bone_index;
    bone_weights[slot] = weight;
}

fn normalize_bone_weights(weights: [f32; 4]) -> [f32; 4] {
    let total = weights.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return weights;
    }

    [
        weights[0] / total,
        weights[1] / total,
        weights[2] / total,
        weights[3] / total,
    ]
}

fn extract_animations(scene: &Scene, bone_index_by_name: &HashMap<String, usize>) -> Vec<KeyframeAnimation> {
    let mut animations = Vec::new();

    for animation in &scene.animations {
        let ticks_per_second = if animation.ticks_per_second > 0.0 {
            animation.ticks_per_second as f32
        } else {
            24.0
        };

        let mut channels = Vec::new();
        let mut max_time = 0.0f32;

        for channel in &animation.channels {
            let Some(&bone_index) = bone_index_by_name.get(&channel.name) else {
                continue;
            };
            channels.extend(build_animation_channels(
                bone_index,
                channel,
                ticks_per_second,
                &mut max_time,
            ));
        }

        if channels.is_empty() {
            continue;
        }

        animations.push(KeyframeAnimation::new(
            clean_animation_name(&animation.name),
            max_time.max((animation.duration as f32) / ticks_per_second),
            channels,
        ));
    }

    animations
}

fn build_animation_channels(
    bone_index: usize,
    node_anim: &NodeAnim,
    ticks_per_second: f32,
    max_time: &mut f32,
) -> Vec<AnimationChannel> {
    let mut channels = Vec::new();

    let translation_components = ["x", "y", "z"];
    for (component, suffix) in translation_components.iter().enumerate() {
        let keyframes = node_anim
            .position_keys
            .iter()
            .map(|key| {
                let time = (key.time as f32) / ticks_per_second;
                *max_time = (*max_time).max(time);
                let value = match component {
                    0 => key.value.x,
                    1 => key.value.y,
                    _ => key.value.z,
                };
                Keyframe {
                    time,
                    value,
                    easing: EasingFunction::Linear,
                }
            })
            .collect::<Vec<_>>();
        if !keyframes.is_empty() {
            channels.push(AnimationChannel {
                target_property: format!("node_{bone_index}.translation.{suffix}"),
                keyframes,
            });
        }
    }

    let rotation_components = ["x", "y", "z", "w"];
    for (component, suffix) in rotation_components.iter().enumerate() {
        let keyframes = node_anim
            .rotation_keys
            .iter()
            .map(|key| {
                let time = (key.time as f32) / ticks_per_second;
                *max_time = (*max_time).max(time);
                let value = match component {
                    0 => key.value.x,
                    1 => key.value.y,
                    2 => key.value.z,
                    _ => key.value.w,
                };
                Keyframe {
                    time,
                    value,
                    easing: EasingFunction::Linear,
                }
            })
            .collect::<Vec<_>>();
        if !keyframes.is_empty() {
            channels.push(AnimationChannel {
                target_property: format!("node_{bone_index}.rotation.{suffix}"),
                keyframes,
            });
        }
    }

    let scale_components = ["x", "y", "z"];
    for (component, suffix) in scale_components.iter().enumerate() {
        let keyframes = node_anim
            .scaling_keys
            .iter()
            .map(|key| {
                let time = (key.time as f32) / ticks_per_second;
                *max_time = (*max_time).max(time);
                let value = match component {
                    0 => key.value.x,
                    1 => key.value.y,
                    _ => key.value.z,
                };
                Keyframe {
                    time,
                    value,
                    easing: EasingFunction::Linear,
                }
            })
            .collect::<Vec<_>>();
        if !keyframes.is_empty() {
            channels.push(AnimationChannel {
                target_property: format!("node_{bone_index}.scale.{suffix}"),
                keyframes,
            });
        }
    }

    channels
}

fn material_from_assimp(material: &Material) -> MeshMaterial {
    let diffuse = material_color(material, "$clr.diffuse")
        .filter(|color| color_luminance(*color) > 0.06)
        .unwrap_or([0.78, 0.74, 0.68]);
    let emissive = material_color(material, "$clr.emissive").unwrap_or([0.0, 0.0, 0.0]);
    let opacity = material_scalar(material, "$mat.opacity").unwrap_or(1.0);
    let metallic = material_scalar(material, "$mat.reflectivity").unwrap_or(0.0);
    let roughness = material_scalar(material, "$mat.roughnessFactor").unwrap_or(1.0);

    MeshMaterial {
        name: material_name(material),
        base_color_factor: [diffuse[0], diffuse[1], diffuse[2], opacity],
        emissive_factor: emissive,
        metallic_factor: metallic,
        roughness_factor: roughness,
        ..MeshMaterial::default()
    }
}

fn material_name(material: &Material) -> Option<String> {
    material
        .properties
        .iter()
        .find(|property| property.key == "?mat.name")
        .and_then(|property| match &property.data {
            PropertyTypeInfo::String(name) => Some(name.clone()),
            _ => None,
        })
}

fn material_color(material: &Material, key: &str) -> Option<[f32; 3]> {
    material
        .properties
        .iter()
        .find(|property| property.key == key)
        .and_then(|property| match &property.data {
            PropertyTypeInfo::FloatArray(values) if values.len() >= 3 => {
                Some([values[0], values[1], values[2]])
            }
            _ => None,
        })
}

fn material_scalar(material: &Material, key: &str) -> Option<f32> {
    material
        .properties
        .iter()
        .find(|property| property.key == key)
        .and_then(|property| match &property.data {
            PropertyTypeInfo::FloatArray(values) => values.first().copied(),
            _ => None,
        })
}

fn color_luminance(color: [f32; 3]) -> f32 {
    0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]
}

fn collect_nodes(node: &Rc<Node>, nodes_by_name: &mut HashMap<String, Rc<Node>>) {
    nodes_by_name.insert(node.name.clone(), Rc::clone(node));
    for child in node.children.borrow().iter() {
        collect_nodes(child, nodes_by_name);
    }
}

fn collect_bone_names_in_hierarchy(
    node: &Rc<Node>,
    bone_names: &HashSet<String>,
    ordered_names: &mut Vec<String>,
) {
    if bone_names.contains(&node.name) {
        ordered_names.push(node.name.clone());
    }
    for child in node.children.borrow().iter() {
        collect_bone_names_in_hierarchy(child, bone_names, ordered_names);
    }
}

fn nearest_bone_parent(
    node: &Rc<Node>,
    bone_index_by_name: &HashMap<String, usize>,
) -> Option<usize> {
    let mut current = node.parent.upgrade();
    while let Some(parent) = current {
        if let Some(&bone_index) = bone_index_by_name.get(&parent.name) {
            return Some(bone_index);
        }
        current = parent.parent.upgrade();
    }
    None
}

fn assimp_matrix_to_column_major(matrix: &russimp::Matrix4x4) -> [f32; 16] {
    [
        matrix.a1, matrix.b1, matrix.c1, matrix.d1, matrix.a2, matrix.b2, matrix.c2, matrix.d2,
        matrix.a3, matrix.b3, matrix.c3, matrix.d3, matrix.a4, matrix.b4, matrix.c4, matrix.d4,
    ]
}

fn clean_animation_name(name: &str) -> String {
    name.split('|').next_back().unwrap_or(name).to_string()
}

const IDENTITY_MAT4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, 1.0, 0.0, //
    0.0, 0.0, 0.0, 1.0,
];
