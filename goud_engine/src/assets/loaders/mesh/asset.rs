//! [`MeshAsset`] -- CPU-side mesh data (vertices, indices, sub-meshes).

use crate::assets::{Asset, AssetType};
use serde::{Deserialize, Serialize};

/// A single vertex in a mesh.
///
/// Matches the 3D renderer vertex layout: position (3f) + normal (3f) + uv (2f).
/// This is CPU-only data; GPU buffer creation is handled separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshVertex {
    /// Vertex position in object space.
    pub position: [f32; 3],
    /// Vertex normal vector (should be unit length).
    pub normal: [f32; 3],
    /// Texture coordinates.
    pub uv: [f32; 2],
}

/// A named sub-range of indices within a mesh.
///
/// Sub-meshes allow a single `MeshAsset` to contain multiple logical
/// parts (e.g., different materials or body parts) without splitting
/// the vertex/index buffers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubMesh {
    /// Human-readable name (e.g., from the source file).
    pub name: String,
    /// First index in the parent `MeshAsset::indices` array.
    pub start_index: u32,
    /// Number of indices belonging to this sub-mesh.
    pub index_count: u32,
    /// Optional index into an external material list.
    pub material_index: Option<u32>,
}

/// A loaded mesh asset containing CPU-side geometry data.
///
/// `MeshAsset` stores vertices, indices, and optional sub-mesh ranges.
/// It does **not** hold any GPU resources -- those are created by the
/// renderer when the mesh is uploaded.
///
/// # Example
///
/// ```
/// use goud_engine::assets::loaders::mesh::{MeshAsset, MeshVertex, SubMesh};
///
/// let asset = MeshAsset {
///     vertices: vec![
///         MeshVertex { position: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
///     ],
///     indices: vec![0],
///     sub_meshes: vec![SubMesh {
///         name: "default".into(),
///         start_index: 0,
///         index_count: 1,
///         material_index: None,
///     }],
/// };
///
/// assert_eq!(asset.vertex_count(), 1);
/// assert_eq!(asset.index_count(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshAsset {
    /// All vertices in the mesh.
    pub vertices: Vec<MeshVertex>,
    /// Index buffer (triangles: every 3 indices form a face).
    pub indices: Vec<u32>,
    /// Sub-mesh ranges within the index buffer.
    pub sub_meshes: Vec<SubMesh>,
}

impl MeshAsset {
    /// Returns the number of vertices.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of indices.
    #[inline]
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Returns the number of triangles (index_count / 3).
    #[inline]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Returns true if the mesh has no vertices.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Flattens vertices into a packed `[f32]` array matching the 3D
    /// renderer vertex layout (position + normal + uv = 8 floats per vertex).
    pub fn to_interleaved_floats(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.vertices.len() * 8);
        for v in &self.vertices {
            out.extend_from_slice(&v.position);
            out.extend_from_slice(&v.normal);
            out.extend_from_slice(&v.uv);
        }
        out
    }
}

impl Asset for MeshAsset {
    fn asset_type_name() -> &'static str {
        "Mesh"
    }

    fn asset_type() -> AssetType {
        AssetType::Mesh
    }

    fn extensions() -> &'static [&'static str] {
        &["gltf", "glb", "obj"]
    }
}
