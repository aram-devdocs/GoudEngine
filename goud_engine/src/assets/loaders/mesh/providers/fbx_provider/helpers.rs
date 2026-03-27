//! FBX tree helpers: connection graph, array extraction, and name utilities.

use std::collections::HashMap;

use fbxcel::low::v7400::AttributeValue;
use fbxcel::tree::v7400::NodeHandle;

// ===========================================================================
// Connection graph
// ===========================================================================

/// Parsed FBX connection graph mapping source/destination object IDs.
#[derive(Default)]
pub(super) struct FbxConnections {
    /// destination_id -> list of source_ids (children of this object).
    pub children_of: HashMap<i64, Vec<i64>>,
    /// source_id -> list of destination_ids (parents of this object).
    pub parents_of: HashMap<i64, Vec<i64>>,
    /// For OP connections: (src_id, dst_id) -> property name.
    pub properties: HashMap<(i64, i64), String>,
}

/// Parses the `Connections` section of the FBX tree into a bidirectional
/// graph of source/destination object IDs, including OP property names.
pub(super) fn parse_connections(root: &NodeHandle) -> FbxConnections {
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
// FBX tree data extraction helpers
// ===========================================================================

/// Finds a direct child node by name and extracts its first attribute as `&[f64]`.
pub(super) fn find_f64_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [f64]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_f64)
}

/// Finds a direct child node by name and extracts its first attribute as `&[i32]`.
pub(super) fn find_i32_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [i32]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_i32)
}

/// Finds a direct child node by name and extracts its first attribute as `&[i64]`.
pub(super) fn find_i64_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [i64]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_i64)
}

/// Finds a direct child node by name and extracts its first attribute as `&[f32]`.
pub(super) fn find_f32_array<'a>(parent: &NodeHandle<'a>, child_name: &str) -> Option<&'a [f32]> {
    let child = parent.first_child_by_name(child_name)?;
    child
        .attributes()
        .first()
        .and_then(AttributeValue::get_arr_f32)
}

/// Finds data inside a layer element (e.g. `LayerElementNormal/Normals`).
pub(super) fn find_layer_element_data<'a>(
    geom: &NodeHandle<'a>,
    layer_name: &str,
    data_child: &str,
) -> Option<&'a [f64]> {
    geom.first_child_by_name(layer_name)
        .and_then(|layer| find_f64_array(&layer, data_child))
}

/// Extracts a 3-component vector from a flat f64 slice at a given vertex index.
pub(super) fn extract_vec3(data: Option<&[f64]>, vertex_index: usize) -> Option<[f32; 3]> {
    let d = data?;
    let base = vertex_index * 3;
    if base + 2 < d.len() {
        Some([d[base] as f32, d[base + 1] as f32, d[base + 2] as f32])
    } else {
        None
    }
}

/// Extracts a 2-component vector from a flat f64 slice at a given vertex index.
pub(super) fn extract_vec2(data: Option<&[f64]>, vertex_index: usize) -> Option<[f32; 2]> {
    let d = data?;
    let base = vertex_index * 2;
    if base + 1 < d.len() {
        Some([d[base] as f32, d[base + 1] as f32])
    } else {
        None
    }
}

// ===========================================================================
// Connection helpers
// ===========================================================================

/// Finds an object from `candidates` that is connected to `target_id`
/// in either direction.
pub(super) fn find_connected_id(
    candidates: &[i64],
    target_id: i64,
    conns: &FbxConnections,
) -> Option<i64> {
    candidates
        .iter()
        .copied()
        .find(|&cid| is_connected(cid, target_id, conns))
}

/// Returns true if `a` and `b` are connected in either direction.
pub(super) fn is_connected(a: i64, b: i64, conns: &FbxConnections) -> bool {
    conns.children_of.get(&b).is_some_and(|v| v.contains(&a))
        || conns.children_of.get(&a).is_some_and(|v| v.contains(&b))
}

/// Finds the Model (bone) ID connected to a Cluster, checking both
/// connection directions.
pub(super) fn find_connected_model_id(
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
pub(super) fn strip_fbx_name(raw: &str) -> String {
    let s = raw.split('\0').next().unwrap_or(raw);
    // Strip "Type::Name" prefix if present.
    s.rsplit("::").next().unwrap_or(s).to_string()
}
