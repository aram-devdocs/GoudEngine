//! FBX geometry and material extraction.

use std::collections::HashMap;

use crate::assets::loaders::mesh::asset::{
    MeshAsset, MeshBounds, MeshMaterial, MeshVertex, SubMesh,
};
use crate::assets::AssetLoadError;

use fbxcel::low::v7400::AttributeValue;
use fbxcel::tree::v7400::NodeHandle;

use super::helpers::{
    extract_vec2, extract_vec3, find_f64_array, find_i32_array, find_layer_element_data,
    strip_fbx_name, FbxConnections,
};
use super::DEFAULT_FBX_ROUGHNESS;

// ===========================================================================
// Material extraction
// ===========================================================================

/// Extracts material definitions from FBX `Objects > Material` nodes.
///
/// Returns a map from FBX object ID to [`MeshMaterial`] for materials that
/// contain at least a diffuse color property.
pub(super) fn extract_materials(root: &NodeHandle) -> HashMap<i64, MeshMaterial> {
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
                    roughness_factor: DEFAULT_FBX_ROUGHNESS,
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
pub(super) fn extract_geometry(
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

            // Read per-polygon material indices from LayerElementMaterial.
            let poly_material_indices: Option<&[i32]> = geom_node
                .first_child_by_name("LayerElementMaterial")
                .and_then(|layer| find_i32_array(&layer, "Materials"));

            // Collect all materials connected to this geometry's parent Model,
            // preserving connection order (index 0, 1, 2...).
            let connected_materials: Vec<Option<MeshMaterial>> = geom_obj_id
                .and_then(|gid| {
                    let model_ids = conns.parents_of.get(&gid)?;
                    for &model_id in model_ids {
                        if let Some(children) = conns.children_of.get(&model_id) {
                            let mats: Vec<Option<MeshMaterial>> = children
                                .iter()
                                .filter(|cid| materials.contains_key(cid))
                                .map(|cid| materials.get(cid).cloned())
                                .collect();
                            if !mats.is_empty() {
                                return Some(mats);
                            }
                        }
                    }
                    None
                })
                .unwrap_or_default();

            // Triangulate polygons and tag each triangle with its material index.
            struct TriData {
                vis: [usize; 3], // polygon-vertex indices for normal/UV lookup
                cps: [usize; 3], // control point indices for position lookup
                mat_idx: i32,
            }
            let mut triangles: Vec<TriData> = Vec::new();
            let mut polygon_vertex_idx: usize = 0;
            let mut polygon_start: usize = 0;
            let mut polygon_count: usize = 0;
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
                    let mat_idx = poly_material_indices
                        .as_ref()
                        .and_then(|arr| arr.get(polygon_count).copied())
                        .unwrap_or(0);

                    if poly_len >= 3 {
                        let first = polygon_start;
                        for tri in 0..(poly_len - 2) {
                            if first + tri + 2 >= polygon_indices.len() {
                                break;
                            }
                            triangles.push(TriData {
                                vis: [first, first + tri + 1, first + tri + 2],
                                cps: [
                                    polygon_indices[first],
                                    polygon_indices[first + tri + 1],
                                    polygon_indices[first + tri + 2],
                                ],
                                mat_idx,
                            });
                        }
                    }
                    polygon_start = polygon_vertex_idx;
                    polygon_count += 1;
                }
            }

            // Sort triangles by material index so we can create one sub-mesh
            // per material group.
            triangles.sort_by_key(|t| t.mat_idx);

            // Determine unique material groups.
            let mat_groups: Vec<i32> = {
                let mut g: Vec<i32> = triangles.iter().map(|t| t.mat_idx).collect();
                g.dedup();
                g
            };

            for &mat_group in &mat_groups {
                let base_vertex = vertices.len() as u32;
                let start_index = indices.len() as u32;

                for tri in triangles.iter().filter(|t| t.mat_idx == mat_group) {
                    for k in 0..3 {
                        let cp = tri.cps[k];
                        let vi = tri.vis[k];
                        let pos = if cp < positions.len() {
                            positions[cp]
                        } else {
                            [0.0, 0.0, 0.0]
                        };
                        let normal = extract_vec3(raw_normals, vi).unwrap_or([0.0, 0.0, 1.0]);
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

                let index_count = indices.len() as u32 - start_index;
                if index_count == 0 {
                    continue;
                }

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

                let matched_material = if mat_group < 0 {
                    None
                } else {
                    connected_materials
                        .get(mat_group as usize)
                        .and_then(|m| m.clone())
                };

                sub_meshes.push(SubMesh {
                    name: format!("{}_mat{}", geom_name, mat_group),
                    start_index,
                    index_count,
                    material_index: None,
                    material: matched_material,
                    bounds,
                });
            }
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
