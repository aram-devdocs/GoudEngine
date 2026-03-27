//! FBX skeleton extraction: bones, inverse bind matrices, and skin weights.

use std::collections::HashMap;

use crate::assets::loaders::mesh::provider::{BoneData, SkeletonData};

use fbxcel::low::v7400::AttributeValue;
use fbxcel::tree::v7400::NodeHandle;

use super::helpers::{
    find_connected_id, find_connected_model_id, find_f64_array, find_i32_array, is_connected,
    strip_fbx_name, FbxConnections,
};

/// Maximum bone influences per vertex.
const MAX_INFLUENCES: usize = 4;

/// Identity 4x4 matrix in column-major order.
const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, // col 0
    0.0, 1.0, 0.0, 0.0, // col 1
    0.0, 0.0, 1.0, 0.0, // col 2
    0.0, 0.0, 0.0, 1.0, // col 3
];

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

/// Extracts the skeleton hierarchy, inverse bind matrices, and per-vertex
/// bone weights from FBX Deformer/Cluster nodes connected to the geometry.
///
/// Returns `None` when the geometry has no skin deformer or no valid clusters.
pub(super) fn extract_skeleton(
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
            // FBX names are often "Model::BoneName\x00\x01" -- strip prefix.
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
        bone_ids_ordered.truncate(128);
        // Rebuild bone_id_to_index to only include the first 128 bones.
        bone_id_to_index.retain(|_, idx| *idx < 128);
    }
    let bone_count = bone_ids_ordered.len(); // re-bind after truncation

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
