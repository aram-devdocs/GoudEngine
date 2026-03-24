//! FBX [`ModelProvider`] implementation.
//!
//! Uses the `fbxcel` crate to parse binary FBX files.  Phase A0 extracts
//! mesh geometry (vertices, normals, UVs, indices).  Skeleton and
//! animation extraction are deferred to Phase B.

use crate::assets::loaders::mesh::asset::{MeshAsset, MeshBounds, MeshVertex, SubMesh};
use crate::assets::{AssetLoadError, LoadContext};

use super::super::provider::{ModelData, ModelProvider};

use fbxcel::low::v7400::AttributeValue;
use fbxcel::tree::v7400::NodeHandle;

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
        let mesh = parse_fbx(bytes)?;
        Ok(ModelData {
            mesh,
            skeleton: None,
            animations: vec![],
        })
    }
}

/// Parses a binary FBX file into a [`MeshAsset`].
///
/// Walks the FBX node tree looking for `Geometry` nodes that contain
/// `Vertices`, `PolygonVertexIndex`, `LayerElementNormal`, and
/// `LayerElementUV` children.
fn parse_fbx(bytes: &[u8]) -> Result<MeshAsset, AssetLoadError> {
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

    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sub_meshes: Vec<SubMesh> = Vec::new();
    let mut mesh_bounds = MeshBounds::default();
    let mut has_mesh_bounds = false;

    // Find all Objects -> Geometry nodes.
    for objects_node in root.children_by_name("Objects") {
        for geom_node in objects_node.children_by_name("Geometry") {
            // Only process mesh geometry (class attribute is typically "Mesh").
            let is_mesh = geom_node
                .attributes()
                .get(2)
                .and_then(|a| a.get_string())
                .map(|s| s == "Mesh")
                .unwrap_or(false);
            if !is_mesh {
                continue;
            }

            let geom_name = geom_node
                .attributes()
                .get(1)
                .and_then(|a| a.get_string())
                .unwrap_or("")
                .to_string();

            let raw_positions = find_f64_array(&geom_node, "Vertices");
            let raw_indices = find_i32_array(&geom_node, "PolygonVertexIndex");

            let raw_positions = match raw_positions {
                Some(v) => v,
                None => continue,
            };
            let raw_indices = match raw_indices {
                Some(v) => v,
                None => continue,
            };

            let position_count = raw_positions.len() / 3;
            if position_count == 0 {
                continue;
            }

            // Build per-control-point positions.
            let positions: Vec<[f32; 3]> = (0..position_count)
                .map(|i| {
                    [
                        raw_positions[i * 3] as f32,
                        raw_positions[i * 3 + 1] as f32,
                        raw_positions[i * 3 + 2] as f32,
                    ]
                })
                .collect();

            // Extract normals (by-polygon-vertex mapping).
            let raw_normals = find_layer_element_data(&geom_node, "LayerElementNormal", "Normals");

            // Extract UVs.
            let raw_uvs = find_layer_element_data(&geom_node, "LayerElementUV", "UV");

            // Triangulate polygons and build vertex/index data.
            let base_vertex = vertices.len() as u32;
            let start_index = indices.len() as u32;
            let mut polygon_vertex_idx: usize = 0;
            let mut polygon_start: usize = 0;

            // Each polygon is terminated by a negative index (bitwise NOT of the actual index).
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
                    // Fan-triangulate the polygon.
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

            sub_meshes.push(SubMesh {
                name: geom_name,
                start_index,
                index_count,
                material_index: None,
                material: None,
                bounds,
            });
        }
    }

    if vertices.is_empty() {
        return Err(AssetLoadError::decode_failed(
            "FBX file contains no mesh data",
        ));
    }

    Ok(MeshAsset {
        vertices,
        indices,
        sub_meshes,
        bounds: mesh_bounds,
    })
}

// ---------------------------------------------------------------------------
// FBX tree helpers
// ---------------------------------------------------------------------------

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
