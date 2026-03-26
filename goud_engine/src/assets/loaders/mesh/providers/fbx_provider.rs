//! FBX [`ModelProvider`] implementation.
//!
//! Uses the `fbxcel` crate to parse binary FBX files.  Extracts mesh
//! geometry, skeleton (bones + skin weights), and keyframe animations.

use std::collections::HashMap;

use crate::assets::loaders::animation::keyframe::{AnimationChannel, EasingFunction, Keyframe};
use crate::assets::loaders::animation::KeyframeAnimation;
use crate::assets::loaders::mesh::asset::{
    MeshAsset, MeshBounds, MeshMaterial, MeshVertex, SubMesh,
};
use crate::assets::{AssetLoadError, LoadContext};

use super::super::provider::{BoneData, ModelData, ModelProvider, SkeletonData};

use fbxcel::low::v7400::AttributeValue;
use fbxcel::tree::v7400::NodeHandle;

/// FBX time ticks per second (standard FBX time unit from FBX SDK).
const FBX_TICKS_PER_SECOND: f64 = 46_186_158_000.0;

/// Maximum bone influences per vertex.
const MAX_INFLUENCES: usize = 4;

/// FBX model provider backed by the `fbxcel` crate.
#[derive(Debug, Clone, Copy, Default)]
pub struct FbxProvider;

impl ModelProvider for FbxProvider {
    fn name(&self) -> &str {
        "FBX"
    }

    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }

    fn load(&self, bytes: &[u8], _context: &mut LoadContext) -> Result<ModelData, AssetLoadError> {
        use fbxcel::pull_parser::any::AnyParser;
        use fbxcel::tree::v7400::Loader as TreeLoader;
        use std::io::Cursor;

        let cursor = Cursor::new(bytes);
        let any_parser = AnyParser::from_seekable_reader(cursor)
            .map_err(|e| AssetLoadError::decode_failed(format!("FBX parse error: {e}")))?;
        let mut parser = match any_parser {
            AnyParser::V7400(p) => p,
            _ => return Err(AssetLoadError::decode_failed("Unsupported FBX version")),
        };
        let (tree, _footer) = TreeLoader::new()
            .load(&mut parser)
            .map_err(|e| AssetLoadError::decode_failed(format!("FBX tree load error: {e}")))?;

        let root = tree.root();
        let conns = parse_connections(&root);
        let materials = extract_materials(&root);
        let (mesh, vertex_to_cp, geom_id) = extract_geometry(&root, &conns, &materials)?;
        let skeleton = extract_skeleton(&root, &conns, geom_id, &vertex_to_cp, mesh.vertices.len());
        let animations = extract_animations(&root, &conns, &skeleton);

        Ok(ModelData {
            mesh,
            skeleton,
            animations,
        })
    }
}

// ===========================================================================
// Connection graph
// ===========================================================================

/// Parsed FBX connection graph mapping source/destination object IDs.
#[derive(Default)]
struct FbxConnections {
    /// destination_id -> list of source_ids (children of this object).
    children_of: HashMap<i64, Vec<i64>>,
    /// source_id -> list of destination_ids (parents of this object).
    parents_of: HashMap<i64, Vec<i64>>,
    /// For OP connections: (src_id, dst_id) -> property name.
    properties: HashMap<(i64, i64), String>,
}

fn parse_connections(root: &NodeHandle) -> FbxConnections {
    let mut conns = FbxConnections::default();
    for section in root.children_by_name("Connections") {
        for c in section.children_by_name("C") {
            let attrs = c.attributes();
            let conn_type = attrs
                .first()
                .and_then(AttributeValue::get_string)
                .unwrap_or("");
            let src = attrs.get(1).and_then(AttributeValue::get_i64).unwrap_or(0);
            let dst = attrs.get(2).and_then(AttributeValue::get_i64).unwrap_or(0);
            if src == 0 && dst == 0 {
                continue;
            }

            conns.children_of.entry(dst).or_default().push(src);
            conns.parents_of.entry(src).or_default().push(dst);

            if conn_type == "OP" {
                if let Some(prop) = attrs.get(3).and_then(AttributeValue::get_string) {
                    conns.properties.insert((src, dst), prop.to_string());
                }
            }
        }
    }
    conns
}

// ===========================================================================
// Material extraction
// ===========================================================================

/// Extracts material definitions from FBX `Objects > Material` nodes.
///
/// Returns a map from FBX object ID to [`MeshMaterial`] for materials that
/// contain at least a diffuse color property.
fn extract_materials(root: &NodeHandle) -> HashMap<i64, MeshMaterial> {
    let mut materials: HashMap<i64, MeshMaterial> = HashMap::new();

    for objects_node in root.children_by_name("Objects") {
        for mat_node in objects_node.children_by_name("Material") {
            let attrs = mat_node.attributes();
            let obj_id = match attrs.first().and_then(AttributeValue::get_i64) {
                Some(id) => id,
                None => continue,
            };
            let raw_name = attrs.get(1).and_then(|a| a.get_string()).unwrap_or("");
            let material_name = strip_fbx_name(raw_name);

            // Default color: opaque white.
            let mut r: f64 = 1.0;
            let mut g: f64 = 1.0;
            let mut b: f64 = 1.0;
            let mut alpha: f64 = 1.0;

            // Look for Properties70 child and iterate P entries.
            if let Some(props70) = mat_node.first_child_by_name("Properties70") {
                for p_node in props70.children_by_name("P") {
                    let p_attrs = p_node.attributes();
                    let prop_name = match p_attrs.first().and_then(AttributeValue::get_string) {
                        Some(n) => n,
                        None => continue,
                    };

                    match prop_name {
                        "DiffuseColor" | "Maya|baseColor" => {
                            // P attributes: [name, type1, type2, flags, R, G, B]
                            if let (Some(rv), Some(gv), Some(bv)) = (
                                p_attrs.get(4).and_then(AttributeValue::get_f64),
                                p_attrs.get(5).and_then(AttributeValue::get_f64),
                                p_attrs.get(6).and_then(AttributeValue::get_f64),
                            ) {
                                r = rv;
                                g = gv;
                                b = bv;
                            }
                        }
                        "Opacity" => {
                            if let Some(val) = p_attrs.get(4).and_then(AttributeValue::get_f64) {
                                alpha = val;
                            }
                        }
                        "TransparencyFactor" => {
                            // TransparencyFactor is the inverse: 0 = opaque, 1 = transparent.
                            if let Some(val) = p_attrs.get(4).and_then(AttributeValue::get_f64) {
                                alpha = 1.0 - val;
                            }
                        }
                        _ => {}
                    }
                }
            }

            materials.insert(
                obj_id,
                MeshMaterial {
                    name: Some(material_name),
                    base_color_factor: [r as f32, g as f32, b as f32, alpha as f32],
                    base_color_texture_path: None,
                    normal_texture_path: None,
                    metallic_roughness_texture_path: None,
                    emissive_texture_path: None,
                    emissive_factor: [0.0, 0.0, 0.0],
                    metallic_factor: 0.0,
                    roughness_factor: 0.5,
                    alpha_cutoff: None,
                    double_sided: false,
                },
            );
        }
    }

    materials
}

// ===========================================================================
// Geometry extraction
// ===========================================================================

/// Extracts mesh geometry, a per-vertex control point map, and the FBX
/// object ID of the first Geometry node.
///
/// When material data is available, each sub-mesh is matched to its FBX
/// material via the connection graph (Geometry -> Model -> Material).
fn extract_geometry(
    root: &NodeHandle,
    conns: &FbxConnections,
    materials: &HashMap<i64, MeshMaterial>,
) -> Result<(MeshAsset, Vec<usize>, Option<i64>), AssetLoadError> {
    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sub_meshes: Vec<SubMesh> = Vec::new();
    let mut mesh_bounds = MeshBounds::default();
    let mut has_mesh_bounds = false;
    let mut vertex_to_cp: Vec<usize> = Vec::new();
    let mut first_geom_id: Option<i64> = None;

    for objects_node in root.children_by_name("Objects") {
        for geom_node in objects_node.children_by_name("Geometry") {
            let is_mesh = geom_node
                .attributes()
                .get(2)
                .and_then(|a| a.get_string())
                .map(|s| s == "Mesh")
                .unwrap_or(false);
            if !is_mesh {
                continue;
            }

            let geom_obj_id = geom_node
                .attributes()
                .first()
                .and_then(AttributeValue::get_i64);

            if first_geom_id.is_none() {
                first_geom_id = geom_obj_id;
            }

            let geom_name = geom_node
                .attributes()
                .get(1)
                .and_then(|a| a.get_string())
                .unwrap_or("")
                .to_string();

            let raw_positions = match find_f64_array(&geom_node, "Vertices") {
                Some(v) => v,
                None => continue,
            };
            let raw_indices = match find_i32_array(&geom_node, "PolygonVertexIndex") {
                Some(v) => v,
                None => continue,
            };

            let position_count = raw_positions.len() / 3;
            if position_count == 0 {
                continue;
            }

            let positions: Vec<[f32; 3]> = (0..position_count)
                .map(|i| {
                    [
                        raw_positions[i * 3] as f32,
                        raw_positions[i * 3 + 1] as f32,
                        raw_positions[i * 3 + 2] as f32,
                    ]
                })
                .collect();

            let raw_normals = find_layer_element_data(&geom_node, "LayerElementNormal", "Normals");
            let raw_uvs = find_layer_element_data(&geom_node, "LayerElementUV", "UV");

            let base_vertex = vertices.len() as u32;
            let start_index = indices.len() as u32;
            let mut polygon_vertex_idx: usize = 0;
            let mut polygon_start: usize = 0;
            let mut polygon_indices: Vec<usize> = Vec::new();

            for &raw_idx in raw_indices {
                let control_point = if raw_idx < 0 {
                    (!raw_idx) as usize
                } else {
                    raw_idx as usize
                };
                polygon_indices.push(control_point);
                let is_end = raw_idx < 0;
                polygon_vertex_idx += 1;

                if is_end {
                    let poly_len = polygon_vertex_idx - polygon_start;
                    if poly_len >= 3 {
                        let first = polygon_start;
                        for tri in 0..(poly_len - 2) {
                            let i0 = first;
                            let i1 = first + tri + 1;
                            let i2 = first + tri + 2;

                            for &vi in &[i0, i1, i2] {
                                let cp = polygon_indices[vi];
                                let pos = if cp < positions.len() {
                                    positions[cp]
                                } else {
                                    [0.0, 0.0, 0.0]
                                };
                                let normal =
                                    extract_vec3(raw_normals, vi).unwrap_or([0.0, 0.0, 1.0]);
                                let uv = extract_vec2(raw_uvs, vi).unwrap_or([0.0, 0.0]);

                                let vert_index = vertices.len() as u32;
                                vertices.push(MeshVertex {
                                    position: pos,
                                    normal,
                                    uv,
                                });
                                vertex_to_cp.push(cp);
                                indices.push(vert_index);
                            }
                        }
                    }
                    polygon_start = polygon_vertex_idx;
                }
            }

            let index_count = indices.len() as u32 - start_index;
            let sub_positions: Vec<[f32; 3]> = vertices[base_vertex as usize..]
                .iter()
                .map(|v| v.position)
                .collect();
            let bounds = MeshBounds::from_positions(&sub_positions);

            if has_mesh_bounds {
                mesh_bounds = mesh_bounds.union(bounds);
            } else {
                mesh_bounds = bounds;
                has_mesh_bounds = true;
            }

            // Resolve material via the FBX connection graph:
            // Geometry -> (parent) Model -> (child) Material.
            let matched_material = geom_obj_id.and_then(|gid| {
                let model_ids = conns.parents_of.get(&gid)?;
                for &model_id in model_ids {
                    if let Some(children) = conns.children_of.get(&model_id) {
                        for &child_id in children {
                            if let Some(mat) = materials.get(&child_id) {
                                return Some(mat.clone());
                            }
                        }
                    }
                }
                None
            });

            sub_meshes.push(SubMesh {
                name: geom_name,
                start_index,
                index_count,
                material_index: None,
                material: matched_material,
                bounds,
            });
        }
    }

    if vertices.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "FBX file contains no mesh data",
        ));
    }

    let mesh = MeshAsset {
        vertices,
        indices,
        sub_meshes,
        bounds: mesh_bounds,
    };
    Ok((mesh, vertex_to_cp, first_geom_id))
}

// ===========================================================================
// Skeleton extraction
// ===========================================================================

/// Per-cluster data extracted from the FBX Deformer/Cluster nodes.
struct ClusterInfo {
    /// FBX object ID of the bone Model this cluster references.
    bone_id: i64,
    /// Control point indices affected by this bone.
    cp_indices: Vec<i32>,
    /// Weights for each affected control point (parallel to cp_indices).
    weights: Vec<f64>,
    /// Inverse bind matrix as column-major f32x16.
    inverse_bind_matrix: [f32; 16],
}

fn extract_skeleton(
    root: &NodeHandle,
    conns: &FbxConnections,
    geom_id: Option<i64>,
    vertex_to_cp: &[usize],
    vertex_count: usize,
) -> Option<SkeletonData> {
    let geom_id = geom_id?;

    // Collect all Deformer node IDs by subclass.
    let mut skin_ids: Vec<i64> = Vec::new();
    let mut cluster_nodes: HashMap<i64, NodeHandle> = HashMap::new();

    for objects_node in root.children_by_name("Objects") {
        for deformer in objects_node.children_by_name("Deformer") {
            let attrs = deformer.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            let subclass = attrs.get(2).and_then(|a| a.get_string()).unwrap_or("");
            match subclass {
                "Skin" => skin_ids.push(obj_id),
                "Cluster" => {
                    cluster_nodes.insert(obj_id, deformer);
                }
                _ => {}
            }
        }
    }

    // Find a Skin connected to our geometry (check both directions).
    let skin_id = find_connected_id(&skin_ids, geom_id, conns)?;

    // Find all Clusters connected to this Skin.
    let cluster_ids: Vec<i64> = cluster_nodes
        .keys()
        .copied()
        .filter(|&cid| is_connected(cid, skin_id, conns))
        .collect();

    if cluster_ids.is_empty() {
        return None;
    }

    // Collect all Model node IDs that could be bones.
    let mut model_nodes: HashMap<i64, String> = HashMap::new();
    for objects_node in root.children_by_name("Objects") {
        for model in objects_node.children_by_name("Model") {
            let attrs = model.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            let raw_name = attrs.get(1).and_then(|a| a.get_string()).unwrap_or("");
            // FBX names are often "Model::BoneName\x00\x01" — strip prefix.
            let name = strip_fbx_name(raw_name);
            model_nodes.insert(obj_id, name);
        }
    }

    // Extract ClusterInfo for each cluster.
    let mut clusters: Vec<ClusterInfo> = Vec::new();
    for &cid in &cluster_ids {
        let node = match cluster_nodes.get(&cid) {
            Some(n) => n,
            None => continue,
        };

        // Find the bone Model connected to this cluster.
        let bone_id = match find_connected_model_id(cid, conns, &model_nodes) {
            Some(id) => id,
            None => continue,
        };

        let cp_indices: Vec<i32> = find_i32_array(node, "Indexes")
            .map(|s| s.to_vec())
            .unwrap_or_default();
        let weights: Vec<f64> = find_f64_array(node, "Weights")
            .map(|s| s.to_vec())
            .unwrap_or_default();
        let ibm = read_transform_link(node);

        clusters.push(ClusterInfo {
            bone_id,
            cp_indices,
            weights,
            inverse_bind_matrix: ibm,
        });
    }

    if clusters.is_empty() {
        return None;
    }

    // Build ordered bone list: assign each unique bone_id an index.
    let mut bone_id_to_index: HashMap<i64, usize> = HashMap::new();
    let mut bone_ids_ordered: Vec<i64> = Vec::new();
    for cluster in &clusters {
        bone_id_to_index.entry(cluster.bone_id).or_insert_with(|| {
            let idx = bone_ids_ordered.len();
            bone_ids_ordered.push(cluster.bone_id);
            idx
        });
    }

    let bone_count = bone_ids_ordered.len();
    if bone_count > 128 {
        log::warn!("FBX skeleton has {bone_count} bones; clamping to 128");
    }

    // Build parent hierarchy from Model->Model connections.
    let mut bones: Vec<BoneData> = Vec::with_capacity(bone_count);
    for &bid in &bone_ids_ordered {
        let name = model_nodes
            .get(&bid)
            .cloned()
            .unwrap_or_else(|| format!("bone_{}", bones.len()));

        // Find parent: look at this bone's parents in the connection graph
        // and see if any are also bones.
        let parent_index = conns
            .parents_of
            .get(&bid)
            .and_then(|parents| {
                parents
                    .iter()
                    .find_map(|pid| bone_id_to_index.get(pid).map(|&i| i as i32))
            })
            .unwrap_or(-1);

        let ibm = clusters
            .iter()
            .find(|c| c.bone_id == bid)
            .map(|c| c.inverse_bind_matrix)
            .unwrap_or(IDENTITY_MATRIX);

        bones.push(BoneData {
            name,
            parent_index,
            inverse_bind_matrix: ibm,
        });
    }

    // Build per-vertex bone indices and weights from cluster data.
    // Clusters store weights per control point; we map to output vertices
    // via vertex_to_cp.
    let cp_count = vertex_to_cp.iter().copied().max().unwrap_or(0) + 1;
    let mut cp_bones: Vec<Vec<(u32, f32)>> = vec![Vec::new(); cp_count];

    for cluster in &clusters {
        let bone_idx = match bone_id_to_index.get(&cluster.bone_id) {
            Some(&idx) => idx as u32,
            None => continue,
        };
        for (i, &cp_idx) in cluster.cp_indices.iter().enumerate() {
            if cp_idx < 0 || cp_idx as usize >= cp_count {
                continue;
            }
            let weight = cluster.weights.get(i).copied().unwrap_or(0.0) as f32;
            if weight > 1e-6 {
                cp_bones[cp_idx as usize].push((bone_idx, weight));
            }
        }
    }

    // Sort each control point's bone list by weight (descending), keep top 4.
    for influences in &mut cp_bones {
        influences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        influences.truncate(MAX_INFLUENCES);
    }

    // Map control point bone data to output vertices.
    let mut bone_indices: Vec<[u32; 4]> = Vec::with_capacity(vertex_count);
    let mut bone_weights: Vec<[f32; 4]> = Vec::with_capacity(vertex_count);

    for &cp in vertex_to_cp {
        let influences = if cp < cp_bones.len() {
            &cp_bones[cp]
        } else {
            &[][..]
        };

        let mut bi = [0u32; 4];
        let mut bw = [0.0f32; 4];
        let mut total_weight = 0.0f32;

        for (i, &(idx, w)) in influences.iter().enumerate().take(MAX_INFLUENCES) {
            bi[i] = idx.min(127); // Clamp to MAX_BONES-1
            bw[i] = w;
            total_weight += w;
        }

        // Normalize weights to sum to 1.0.
        if total_weight > 1e-6 {
            for w in &mut bw {
                *w /= total_weight;
            }
        }

        bone_indices.push(bi);
        bone_weights.push(bw);
    }

    Some(SkeletonData {
        bones,
        bone_indices,
        bone_weights,
    })
}

// ===========================================================================
// Animation extraction
// ===========================================================================

fn extract_animations(
    root: &NodeHandle,
    conns: &FbxConnections,
    skeleton: &Option<SkeletonData>,
) -> Vec<KeyframeAnimation> {
    let skeleton = match skeleton {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Build bone name -> bone index map for looking up bones by FBX name.
    let bone_name_to_idx: HashMap<&str, usize> = skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(i, b)| (b.name.as_str(), i))
        .collect();

    // Collect all Model node IDs with their names (needed to resolve bone connections).
    let mut model_id_to_name: HashMap<i64, String> = HashMap::new();
    let mut model_name_to_id: HashMap<String, i64> = HashMap::new();
    for objects_node in root.children_by_name("Objects") {
        for model in objects_node.children_by_name("Model") {
            let attrs = model.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            let raw_name = attrs.get(1).and_then(|a| a.get_string()).unwrap_or("");
            let name = strip_fbx_name(raw_name);
            model_id_to_name.insert(obj_id, name.clone());
            model_name_to_id.insert(name, obj_id);
        }
    }

    // Build a map from Model ID -> bone index.
    let mut model_id_to_bone_idx: HashMap<i64, usize> = HashMap::new();
    for (name, &model_id) in &model_name_to_id {
        if let Some(&idx) = bone_name_to_idx.get(name.as_str()) {
            model_id_to_bone_idx.insert(model_id, idx);
        }
    }

    // Collect AnimationStack, AnimationLayer, AnimationCurveNode, AnimationCurve nodes.
    let mut anim_stacks: Vec<(i64, String)> = Vec::new();
    let mut anim_curve_nodes_map: HashMap<i64, NodeHandle> = HashMap::new();

    for objects_node in root.children_by_name("Objects") {
        for node in objects_node.children_by_name("AnimationStack") {
            let attrs = node.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            let raw_name = attrs.get(1).and_then(|a| a.get_string()).unwrap_or("");
            anim_stacks.push((obj_id, strip_fbx_name(raw_name)));
        }
        for node in objects_node.children_by_name("AnimationCurveNode") {
            let attrs = node.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            anim_curve_nodes_map.insert(obj_id, node);
        }
    }

    // Collect AnimationCurve nodes (KeyTime + KeyValueFloat).
    let mut anim_curves: HashMap<i64, CurveData> = HashMap::new();
    for objects_node in root.children_by_name("Objects") {
        for node in objects_node.children_by_name("AnimationCurve") {
            let attrs = node.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);

            let key_times: Vec<i64> = find_i64_array(&node, "KeyTime")
                .map(|s| s.to_vec())
                .unwrap_or_default();
            let key_values: Vec<f32> = find_f32_array(&node, "KeyValueFloat")
                .map(|s| s.to_vec())
                .or_else(|| {
                    find_f64_array(&node, "KeyValueFloat")
                        .map(|s| s.iter().map(|&v| v as f32).collect())
                })
                .unwrap_or_default();

            if !key_times.is_empty() && key_times.len() == key_values.len() {
                anim_curves.insert(
                    obj_id,
                    CurveData {
                        key_times,
                        key_values,
                    },
                );
            }
        }
    }

    // For each AnimationStack, build a KeyframeAnimation.
    let mut animations: Vec<KeyframeAnimation> = Vec::new();

    for (stack_id, stack_name) in &anim_stacks {
        // Find layers connected to this stack.
        let layer_ids: Vec<i64> = conns.children_of.get(stack_id).cloned().unwrap_or_default();

        // Collect per-bone Euler rotation curves for later quaternion conversion.
        // bone_index -> (component_index, Vec<(time_sec, value_deg)>)
        let mut bone_euler_curves: HashMap<usize, [Vec<(f32, f32)>; 3]> = HashMap::new();
        let mut channels: Vec<AnimationChannel> = Vec::new();
        let mut max_time: f32 = 0.0;

        for &layer_id in &layer_ids {
            // Find CurveNodes connected to this layer.
            let curve_node_ids_for_layer: Vec<i64> = conns
                .children_of
                .get(&layer_id)
                .cloned()
                .unwrap_or_default();

            for &cn_id in &curve_node_ids_for_layer {
                if !anim_curve_nodes_map.contains_key(&cn_id) {
                    continue;
                }

                // Find which bone this CurveNode targets (via OP connection).
                let (bone_idx, property) =
                    match find_curve_node_target(cn_id, conns, &model_id_to_bone_idx) {
                        Some(v) => v,
                        None => continue,
                    };

                // Find the individual AnimationCurve children (X, Y, Z components).
                let child_curve_ids = conns.children_of.get(&cn_id).cloned().unwrap_or_default();

                for &curve_id in &child_curve_ids {
                    let curve = match anim_curves.get(&curve_id) {
                        Some(c) => c,
                        None => continue,
                    };

                    // Determine component from OP property: "d|X", "d|Y", "d|Z".
                    let component = conns
                        .properties
                        .get(&(curve_id, cn_id))
                        .map(|p| p.as_str())
                        .unwrap_or("");
                    let suffix = match component {
                        "d|X" => "x",
                        "d|Y" => "y",
                        "d|Z" => "z",
                        _ => continue,
                    };
                    let comp_idx = match suffix {
                        "x" => 0usize,
                        "y" => 1,
                        "z" => 2,
                        _ => continue,
                    };

                    // Convert FBX ticks to seconds and build keyframes.
                    let keyframes: Vec<(f32, f32)> = curve
                        .key_times
                        .iter()
                        .zip(curve.key_values.iter())
                        .map(|(&t, &v)| {
                            let time_sec = (t as f64 / FBX_TICKS_PER_SECOND) as f32;
                            if time_sec > max_time {
                                max_time = time_sec;
                            }
                            (time_sec, v)
                        })
                        .collect();

                    match property.as_str() {
                        "translation" => {
                            let target = format!("node_{bone_idx}.translation.{suffix}");
                            channels.push(AnimationChannel {
                                target_property: target,
                                keyframes: keyframes
                                    .iter()
                                    .map(|&(time, value)| Keyframe {
                                        time,
                                        value,
                                        easing: EasingFunction::Linear,
                                    })
                                    .collect(),
                            });
                        }
                        "rotation" => {
                            // Accumulate Euler curves; we'll convert to
                            // quaternion after collecting all 3 axes.
                            let entry = bone_euler_curves
                                .entry(bone_idx)
                                .or_insert_with(|| [Vec::new(), Vec::new(), Vec::new()]);
                            entry[comp_idx] = keyframes;
                        }
                        "scale" => {
                            let target = format!("node_{bone_idx}.scale.{suffix}");
                            channels.push(AnimationChannel {
                                target_property: target,
                                keyframes: keyframes
                                    .iter()
                                    .map(|&(time, value)| Keyframe {
                                        time,
                                        value,
                                        easing: EasingFunction::Linear,
                                    })
                                    .collect(),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // Convert accumulated Euler rotation curves to quaternion channels.
        for (&bone_idx, euler_curves) in &bone_euler_curves {
            let quat_channels =
                euler_curves_to_quat_channels(bone_idx, euler_curves, &mut max_time);
            channels.extend(quat_channels);
        }

        if !channels.is_empty() {
            animations.push(KeyframeAnimation::new(
                stack_name.clone(),
                max_time,
                channels,
            ));
        }
    }

    animations
}

// ===========================================================================
// Euler → Quaternion conversion
// ===========================================================================

/// Converts three Euler rotation curves (X, Y, Z in degrees) into four
/// quaternion channels (x, y, z, w) for a single bone.
fn euler_curves_to_quat_channels(
    bone_idx: usize,
    euler: &[Vec<(f32, f32)>; 3],
    max_time: &mut f32,
) -> Vec<AnimationChannel> {
    // Merge all unique timestamps from the three Euler curves.
    let mut times: Vec<f32> = Vec::new();
    for curve in euler {
        for &(t, _) in curve {
            if !times.iter().any(|&existing| (existing - t).abs() < 1e-6) {
                times.push(t);
            }
        }
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if times.is_empty() {
        return Vec::new();
    }

    // At each timestamp, interpolate all 3 Euler curves and convert to quaternion.
    let mut kf_x = Vec::with_capacity(times.len());
    let mut kf_y = Vec::with_capacity(times.len());
    let mut kf_z = Vec::with_capacity(times.len());
    let mut kf_w = Vec::with_capacity(times.len());

    for &t in &times {
        let rx = lerp_curve_at(t, &euler[0]);
        let ry = lerp_curve_at(t, &euler[1]);
        let rz = lerp_curve_at(t, &euler[2]);

        let [qx, qy, qz, qw] = euler_xyz_to_quat(rx, ry, rz);

        if t > *max_time {
            *max_time = t;
        }

        let easing = EasingFunction::Linear;
        kf_x.push(Keyframe {
            time: t,
            value: qx,
            easing: easing.clone(),
        });
        kf_y.push(Keyframe {
            time: t,
            value: qy,
            easing: easing.clone(),
        });
        kf_z.push(Keyframe {
            time: t,
            value: qz,
            easing: easing.clone(),
        });
        kf_w.push(Keyframe {
            time: t,
            value: qw,
            easing,
        });
    }

    vec![
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.x"),
            keyframes: kf_x,
        },
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.y"),
            keyframes: kf_y,
        },
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.z"),
            keyframes: kf_z,
        },
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.w"),
            keyframes: kf_w,
        },
    ]
}

/// Linearly interpolates a curve's value at a given time.
fn lerp_curve_at(t: f32, curve: &[(f32, f32)]) -> f32 {
    if curve.is_empty() {
        return 0.0;
    }
    if curve.len() == 1 || t <= curve[0].0 {
        return curve[0].1;
    }
    if t >= curve.last().unwrap().0 {
        return curve.last().unwrap().1;
    }
    // Binary search for the bracketing keyframes.
    let idx = curve.partition_point(|&(ct, _)| ct < t);
    if idx == 0 {
        return curve[0].1;
    }
    let (t0, v0) = curve[idx - 1];
    let (t1, v1) = curve[idx];
    let dt = t1 - t0;
    if dt.abs() < 1e-9 {
        return v1;
    }
    let frac = (t - t0) / dt;
    v0 + frac * (v1 - v0)
}

/// Converts Euler XYZ rotation (degrees) to quaternion (x, y, z, w).
///
/// Uses FBX's default rotation order: intrinsic X-Y-Z, which corresponds
/// to the matrix multiplication R = Rx * Ry * Rz.
fn euler_xyz_to_quat(rx_deg: f32, ry_deg: f32, rz_deg: f32) -> [f32; 4] {
    let rx = rx_deg.to_radians() * 0.5;
    let ry = ry_deg.to_radians() * 0.5;
    let rz = rz_deg.to_radians() * 0.5;

    let (sx, cx) = rx.sin_cos();
    let (sy, cy) = ry.sin_cos();
    let (sz, cz) = rz.sin_cos();

    // Q = Qx * Qy * Qz (FBX eEulerXYZ)
    [
        sx * cy * cz + cx * sy * sz, // x
        cx * sy * cz - sx * cy * sz, // y
        cx * cy * sz + sx * sy * cz, // z
        cx * cy * cz - sx * sy * sz, // w
    ]
}

// ===========================================================================
// Animation helpers
// ===========================================================================

struct CurveData {
    key_times: Vec<i64>,
    key_values: Vec<f32>,
}

/// Finds which bone a CurveNode targets via OP connections.
///
/// Returns (bone_index, property_type) where property_type is one of
/// "translation", "rotation", "scale".
fn find_curve_node_target(
    cn_id: i64,
    conns: &FbxConnections,
    model_id_to_bone_idx: &HashMap<i64, usize>,
) -> Option<(usize, String)> {
    // CurveNode connects to a bone Model via OP connection with property
    // "Lcl Translation", "Lcl Rotation", or "Lcl Scaling".
    let parents = conns.parents_of.get(&cn_id)?;
    for &dst in parents {
        if let Some(&bone_idx) = model_id_to_bone_idx.get(&dst) {
            if let Some(prop) = conns.properties.get(&(cn_id, dst)) {
                let property_type = match prop.as_str() {
                    "Lcl Translation" => "translation",
                    "Lcl Rotation" => "rotation",
                    "Lcl Scaling" => "scale",
                    _ => continue,
                };
                return Some((bone_idx, property_type.to_string()));
            }
        }
    }
    None
}

// ===========================================================================
// Skeleton helpers
// ===========================================================================

/// Identity 4x4 matrix in column-major order.
const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, // col 0
    0.0, 1.0, 0.0, 0.0, // col 1
    0.0, 0.0, 1.0, 0.0, // col 2
    0.0, 0.0, 0.0, 1.0, // col 3
];

/// Reads the TransformLink from a Cluster node and returns its **inverse**
/// as the inverse bind matrix in column-major f32[16].
///
/// FBX `TransformLink` is the bone's global transform at bind time.
/// The engine needs the *inverse* of this for skinning.
fn read_transform_link(cluster: &NodeHandle) -> [f32; 16] {
    let raw = match find_f64_array(cluster, "TransformLink") {
        Some(v) if v.len() >= 16 => v,
        _ => return IDENTITY_MATRIX,
    };

    // FBX stores matrices row-major: [row0_col0, row0_col1, ...].
    // Convert to column-major first, then invert.
    let col_major = [
        raw[0] as f32,
        raw[4] as f32,
        raw[8] as f32,
        raw[12] as f32,
        raw[1] as f32,
        raw[5] as f32,
        raw[9] as f32,
        raw[13] as f32,
        raw[2] as f32,
        raw[6] as f32,
        raw[10] as f32,
        raw[14] as f32,
        raw[3] as f32,
        raw[7] as f32,
        raw[11] as f32,
        raw[15] as f32,
    ];

    invert_4x4(&col_major).unwrap_or(IDENTITY_MATRIX)
}

/// Inverts a column-major 4x4 matrix using cofactor expansion.
fn invert_4x4(m: &[f32; 16]) -> Option<[f32; 16]> {
    // Column-major: m[col*4 + row]
    let m = |r: usize, c: usize| m[c * 4 + r];

    let s0 = m(0, 0) * m(1, 1) - m(1, 0) * m(0, 1);
    let s1 = m(0, 0) * m(1, 2) - m(1, 0) * m(0, 2);
    let s2 = m(0, 0) * m(1, 3) - m(1, 0) * m(0, 3);
    let s3 = m(0, 1) * m(1, 2) - m(1, 1) * m(0, 2);
    let s4 = m(0, 1) * m(1, 3) - m(1, 1) * m(0, 3);
    let s5 = m(0, 2) * m(1, 3) - m(1, 2) * m(0, 3);

    let c5 = m(2, 2) * m(3, 3) - m(3, 2) * m(2, 3);
    let c4 = m(2, 1) * m(3, 3) - m(3, 1) * m(2, 3);
    let c3 = m(2, 1) * m(3, 2) - m(3, 1) * m(2, 2);
    let c2 = m(2, 0) * m(3, 3) - m(3, 0) * m(2, 3);
    let c1 = m(2, 0) * m(3, 2) - m(3, 0) * m(2, 2);
    let c0 = m(2, 0) * m(3, 1) - m(3, 0) * m(2, 1);

    let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;

    // Result in column-major order: r[col * 4 + row].
    #[allow(clippy::identity_op, clippy::erasing_op)]
    let r = [
        // Column 0
        (m(1, 1) * c5 - m(1, 2) * c4 + m(1, 3) * c3) * inv_det,
        (-m(0, 1) * c5 + m(0, 2) * c4 - m(0, 3) * c3) * inv_det,
        (m(3, 1) * s5 - m(3, 2) * s4 + m(3, 3) * s3) * inv_det,
        (-m(2, 1) * s5 + m(2, 2) * s4 - m(2, 3) * s3) * inv_det,
        // Column 1
        (-m(1, 0) * c5 + m(1, 2) * c2 - m(1, 3) * c1) * inv_det,
        (m(0, 0) * c5 - m(0, 2) * c2 + m(0, 3) * c1) * inv_det,
        (-m(3, 0) * s5 + m(3, 2) * s2 - m(3, 3) * s1) * inv_det,
        (m(2, 0) * s5 - m(2, 2) * s2 + m(2, 3) * s1) * inv_det,
        // Column 2
        (m(1, 0) * c4 - m(1, 1) * c2 + m(1, 3) * c0) * inv_det,
        (-m(0, 0) * c4 + m(0, 1) * c2 - m(0, 3) * c0) * inv_det,
        (m(3, 0) * s4 - m(3, 1) * s2 + m(3, 3) * s0) * inv_det,
        (-m(2, 0) * s4 + m(2, 1) * s2 - m(2, 3) * s0) * inv_det,
        // Column 3
        (-m(1, 0) * c3 + m(1, 1) * c1 - m(1, 2) * c0) * inv_det,
        (m(0, 0) * c3 - m(0, 1) * c1 + m(0, 2) * c0) * inv_det,
        (-m(3, 0) * s3 + m(3, 1) * s1 - m(3, 2) * s0) * inv_det,
        (m(2, 0) * s3 - m(2, 1) * s1 + m(2, 2) * s0) * inv_det,
    ];

    Some(r)
}

/// Finds an object from `candidates` that is connected to `target_id`
/// in either direction.
fn find_connected_id(candidates: &[i64], target_id: i64, conns: &FbxConnections) -> Option<i64> {
    candidates
        .iter()
        .copied()
        .find(|&cid| is_connected(cid, target_id, conns))
}

/// Returns true if `a` and `b` are connected in either direction.
fn is_connected(a: i64, b: i64, conns: &FbxConnections) -> bool {
    conns.children_of.get(&b).is_some_and(|v| v.contains(&a))
        || conns.children_of.get(&a).is_some_and(|v| v.contains(&b))
}

/// Finds the Model (bone) ID connected to a Cluster, checking both
/// connection directions.
fn find_connected_model_id(
    cluster_id: i64,
    conns: &FbxConnections,
    model_nodes: &HashMap<i64, String>,
) -> Option<i64> {
    // Check objects connected to this cluster in both directions.
    if let Some(parents) = conns.parents_of.get(&cluster_id) {
        for &pid in parents {
            if model_nodes.contains_key(&pid) {
                return Some(pid);
            }
        }
    }
    if let Some(children) = conns.children_of.get(&cluster_id) {
        for &cid in children {
            if model_nodes.contains_key(&cid) {
                return Some(cid);
            }
        }
    }
    None
}

/// Strips FBX name prefixes like "Model::" and null terminators.
fn strip_fbx_name(raw: &str) -> String {
    let s = raw.split('\0').next().unwrap_or(raw);
    // Strip "Type::Name" prefix if present.
    s.rsplit("::").next().unwrap_or(s).to_string()
}

// ===========================================================================
// FBX tree helpers
// ===========================================================================

/// Finds a direct child node by name and extracts its first attribute as `&[f64]`.
fn find_f64_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [f64]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_f64)
}

/// Finds a direct child node by name and extracts its first attribute as `&[i32]`.
fn find_i32_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [i32]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_i32)
}

/// Finds a direct child node by name and extracts its first attribute as `&[i64]`.
fn find_i64_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [i64]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_i64)
}

/// Finds a direct child node by name and extracts its first attribute as `&[f32]`.
fn find_f32_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [f32]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_f32)
}

/// Finds data inside a layer element (e.g. `LayerElementNormal/Normals`).
fn find_layer_element_data<'a>(
    geom: &NodeHandle<'a>,
    layer_name: &str,
    data_child: &str,
) -> Option<&'a [f64]> {
    geom.first_child_by_name(layer_name)
        .and_then(|layer| find_f64_array(&layer, data_child))
}

/// Extracts a 3-component vector from a flat f64 slice at a given vertex index.
fn extract_vec3(data: Option<&[f64]>, vertex_index: usize) -> Option<[f32; 3]> {
    let d = data?;
    let base = vertex_index * 3;
    if base + 2 < d.len() {
        Some([d[base] as f32, d[base + 1] as f32, d[base + 2] as f32])
    } else {
        None
    }
}

/// Extracts a 2-component vector from a flat f64 slice at a given vertex index.
fn extract_vec2(data: Option<&[f64]>, vertex_index: usize) -> Option<[f32; 2]> {
    let d = data?;
    let base = vertex_index * 2;
    if base + 1 < d.len() {
        Some([d[base] as f32, d[base + 1] as f32])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euler_xyz_to_quat_identity() {
        let [x, y, z, w] = euler_xyz_to_quat(0.0, 0.0, 0.0);
        assert!(
            (w - 1.0).abs() < 1e-6,
            "w should be 1 for identity rotation"
        );
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
        assert!(z.abs() < 1e-6);
    }

    #[test]
    fn test_euler_xyz_to_quat_90_x() {
        let [x, y, z, w] = euler_xyz_to_quat(90.0, 0.0, 0.0);
        let expected_sin = (45.0f32.to_radians()).sin();
        let expected_cos = (45.0f32.to_radians()).cos();
        assert!((x - expected_sin).abs() < 1e-5);
        assert!(y.abs() < 1e-5);
        assert!(z.abs() < 1e-5);
        assert!((w - expected_cos).abs() < 1e-5);
    }

    #[test]
    fn test_euler_xyz_to_quat_unit_length() {
        let [x, y, z, w] = euler_xyz_to_quat(30.0, 45.0, 60.0);
        let len = (x * x + y * y + z * z + w * w).sqrt();
        assert!((len - 1.0).abs() < 1e-5, "quaternion should be unit length");
    }

    #[test]
    fn test_strip_fbx_name_simple() {
        assert_eq!(strip_fbx_name("BoneName"), "BoneName");
    }

    #[test]
    fn test_strip_fbx_name_with_prefix() {
        assert_eq!(strip_fbx_name("Model::Armature"), "Armature");
    }

    #[test]
    fn test_strip_fbx_name_with_null() {
        assert_eq!(strip_fbx_name("Bone\x00\x01Blender"), "Bone");
    }

    #[test]
    fn test_strip_fbx_name_double_prefix() {
        assert_eq!(strip_fbx_name("Model::Armature::Bone1"), "Bone1");
    }

    #[test]
    fn test_lerp_curve_at_empty() {
        assert!((lerp_curve_at(1.0, &[]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_curve_at_single() {
        let curve = [(0.5, 3.0)];
        assert!((lerp_curve_at(0.0, &curve) - 3.0).abs() < 1e-6);
        assert!((lerp_curve_at(1.0, &curve) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_curve_at_interpolation() {
        let curve = [(0.0, 0.0), (1.0, 10.0)];
        assert!((lerp_curve_at(0.5, &curve) - 5.0).abs() < 1e-5);
        assert!((lerp_curve_at(0.25, &curve) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_curve_at_clamp() {
        let curve = [(1.0, 5.0), (2.0, 15.0)];
        assert!((lerp_curve_at(0.0, &curve) - 5.0).abs() < 1e-6);
        assert!((lerp_curve_at(3.0, &curve) - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_vec3_valid() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(extract_vec3(Some(&data), 0), Some([1.0, 2.0, 3.0]));
        assert_eq!(extract_vec3(Some(&data), 1), Some([4.0, 5.0, 6.0]));
    }

    #[test]
    fn test_extract_vec3_out_of_bounds() {
        let data = [1.0, 2.0];
        assert_eq!(extract_vec3(Some(&data), 0), None);
    }

    #[test]
    fn test_extract_vec3_none() {
        assert_eq!(extract_vec3(None, 0), None);
    }

    #[test]
    fn test_extract_vec2_valid() {
        let data = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(extract_vec2(Some(&data), 0), Some([1.0, 2.0]));
        assert_eq!(extract_vec2(Some(&data), 1), Some([3.0, 4.0]));
    }

    #[test]
    fn test_is_connected_bidirectional() {
        let mut conns = FbxConnections::default();
        conns.children_of.entry(100).or_default().push(200);
        conns.parents_of.entry(200).or_default().push(100);

        assert!(is_connected(200, 100, &conns));
        assert!(is_connected(100, 200, &conns));
        assert!(!is_connected(300, 100, &conns));
    }
}
