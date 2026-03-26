//! Core mesh and model data types shared between the asset pipeline and the renderer.
//!
//! These types live in the foundation layer so that both `libs/` (Layer 2) and
//! `assets/` (Layer 3) can depend on them without creating upward imports.

use serde::{Deserialize, Serialize};

// =============================================================================
// MeshBounds
// =============================================================================

/// Axis-aligned bounds in object space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MeshBounds {
    /// Minimum corner.
    pub min: [f32; 3],
    /// Maximum corner.
    pub max: [f32; 3],
}

impl Default for MeshBounds {
    fn default() -> Self {
        Self {
            min: [0.0, 0.0, 0.0],
            max: [0.0, 0.0, 0.0],
        }
    }
}

impl MeshBounds {
    /// Builds bounds from a non-empty position slice.
    pub fn from_positions(positions: &[[f32; 3]]) -> Self {
        let mut bounds = Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        };
        for position in positions {
            bounds.include(*position);
        }
        if positions.is_empty() {
            Self::default()
        } else {
            bounds
        }
    }

    /// Expands the bounds to include a point.
    pub fn include(&mut self, position: [f32; 3]) {
        for (axis, value) in position.into_iter().enumerate() {
            self.min[axis] = self.min[axis].min(value);
            self.max[axis] = self.max[axis].max(value);
        }
    }

    /// Returns a bounds value that contains both bounds.
    pub fn union(self, other: Self) -> Self {
        Self {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }
}

// =============================================================================
// MeshVertex
// =============================================================================

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

// =============================================================================
// MeshMaterial
// =============================================================================

/// A named sub-range of indices within a mesh.
///
/// Sub-meshes allow a single `MeshAsset` to contain multiple logical
/// parts (e.g., different materials or body parts) without splitting
/// the vertex/index buffers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshMaterial {
    /// Optional source material name.
    pub name: Option<String>,
    /// Base color multiplier.
    pub base_color_factor: [f32; 4],
    /// Base color texture path, if any.
    pub base_color_texture_path: Option<String>,
    /// Normal map texture path, if any.
    pub normal_texture_path: Option<String>,
    /// Metallic-roughness texture path, if any.
    pub metallic_roughness_texture_path: Option<String>,
    /// Emissive texture path, if any.
    pub emissive_texture_path: Option<String>,
    /// Emissive color multiplier.
    pub emissive_factor: [f32; 3],
    /// Metallic factor from the source material.
    pub metallic_factor: f32,
    /// Roughness factor from the source material.
    pub roughness_factor: f32,
    /// Alpha cutoff for masked materials.
    pub alpha_cutoff: Option<f32>,
    /// Whether the source material disables back-face culling.
    pub double_sided: bool,
}

impl Default for MeshMaterial {
    fn default() -> Self {
        Self {
            name: None,
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            base_color_texture_path: None,
            normal_texture_path: None,
            metallic_roughness_texture_path: None,
            emissive_texture_path: None,
            emissive_factor: [0.0, 0.0, 0.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            alpha_cutoff: None,
            double_sided: false,
        }
    }
}

// =============================================================================
// SubMesh
// =============================================================================

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
    /// Imported material metadata for this sub-mesh.
    pub material: Option<MeshMaterial>,
    /// Object-space bounds for this sub-mesh.
    pub bounds: MeshBounds,
}

// =============================================================================
// MeshAsset
// =============================================================================

/// A loaded mesh asset containing CPU-side geometry data.
///
/// `MeshAsset` stores vertices, indices, and optional sub-mesh ranges.
/// It does **not** hold any GPU resources -- those are created by the
/// renderer when the mesh is uploaded.
///
/// # Example
///
/// ```
/// use goud_engine::core::types::mesh_data::{MeshAsset, MeshBounds, MeshVertex, SubMesh};
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
///         material: None,
///         bounds: MeshBounds::default(),
///     }],
///     bounds: MeshBounds::default(),
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
    /// Object-space bounds for the full mesh.
    pub bounds: MeshBounds,
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

    /// Flattens vertices into a packed `[f32]` array matching the skinned
    /// vertex layout (pos3 + norm3 + uv2 + bone_ids4 + bone_weights4 = 16 floats
    /// per vertex).
    ///
    /// `bone_indices` and `bone_weights` must be parallel to `self.vertices`.
    /// If they are shorter than the vertex count, missing entries default to
    /// zero indices and zero weights.
    pub fn to_skinned_interleaved_floats(
        &self,
        bone_indices: &[[u32; 4]],
        bone_weights: &[[f32; 4]],
    ) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.vertices.len() * 16);
        for (i, v) in self.vertices.iter().enumerate() {
            out.extend_from_slice(&v.position);
            out.extend_from_slice(&v.normal);
            out.extend_from_slice(&v.uv);
            // Bone indices stored as floats for the vertex attribute.
            if let Some(bi) = bone_indices.get(i) {
                out.push(bi[0] as f32);
                out.push(bi[1] as f32);
                out.push(bi[2] as f32);
                out.push(bi[3] as f32);
            } else {
                out.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
            }
            if let Some(bw) = bone_weights.get(i) {
                out.extend_from_slice(bw);
            } else {
                out.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
            }
        }
        out
    }

    /// Returns the object-space bounds.
    #[inline]
    pub fn local_bounds(&self) -> MeshBounds {
        self.bounds
    }
}

// =============================================================================
// Skeleton types
// =============================================================================

/// Format-agnostic skeleton data extracted from a model file.
///
/// Bone indices and weights are stored per-vertex, parallel to the mesh
/// vertex array.  Phase A0 providers may return `None` for skeleton data;
/// full extraction lands in Phase B.
#[derive(Debug, Clone)]
pub struct SkeletonData {
    /// Bones in the skeleton hierarchy.
    pub bones: Vec<BoneData>,
    /// Per-vertex bone indices (4 influences per vertex).
    pub bone_indices: Vec<[u32; 4]>,
    /// Per-vertex bone weights (4 influences per vertex).
    pub bone_weights: Vec<[f32; 4]>,
}

/// A single bone in a skeleton hierarchy.
#[derive(Debug, Clone)]
pub struct BoneData {
    /// Human-readable bone name.
    pub name: String,
    /// Index of the parent bone, or -1 for root bones.
    pub parent_index: i32,
    /// Column-major 4x4 inverse bind matrix.
    pub inverse_bind_matrix: [f32; 16],
}

// =============================================================================
// ModelData
// =============================================================================

/// Complete output of a model provider: mesh geometry plus optional
/// skeleton and animation data.
#[derive(Debug, Clone)]
pub struct ModelData {
    /// Parsed mesh geometry (vertices, indices, sub-meshes, bounds).
    pub mesh: MeshAsset,
    /// Optional skeleton for skinned meshes.
    pub skeleton: Option<SkeletonData>,
    /// Animations embedded in the model file.
    pub animations: Vec<super::keyframe_types::KeyframeAnimation>,
}
