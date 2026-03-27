//! Skeletal mesh types for the 3D renderer.

use cgmath::Vector3;

// ============================================================================
// Skeletal Mesh
// ============================================================================

/// Maximum bones per skeleton for GPU skinning.
///
/// This constant must match the shader's `u_bones[128]` uniform array size.
/// The runtime config field [`SkinningConfig::max_bones_per_mesh`] defaults to
/// this value and is used for validation; changing it does NOT resize the
/// shader array.
pub const MAX_BONES: usize = 128;

/// Maximum bone influences per vertex.
pub const MAX_BONE_INFLUENCES: usize = 4;

/// A single bone in a skeleton hierarchy.
#[derive(Debug, Clone)]
pub struct Bone3D {
    /// Bone name for lookup.
    pub name: String,
    /// Index of the parent bone (-1 for root).
    pub parent_index: i32,
    /// Inverse bind-pose matrix (column-major 4x4).
    pub inverse_bind_matrix: [f32; 16],
}

/// A complete skeleton definition.
#[derive(Debug, Clone)]
pub struct Skeleton3D {
    /// Ordered list of bones.
    pub bones: Vec<Bone3D>,
}

impl Skeleton3D {
    /// Create a new empty skeleton.
    pub fn new() -> Self {
        Self { bones: Vec::new() }
    }

    /// Return the number of bones in this skeleton.
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }
}

impl Default for Skeleton3D {
    fn default() -> Self {
        Self::new()
    }
}

/// A mesh with per-vertex bone weights for GPU skinning.
#[derive(Debug)]
pub struct SkinnedMesh3D {
    /// Vertex data (position + normal + texcoord + bone indices + bone weights).
    pub vertices: Vec<f32>,
    /// The skeleton driving this mesh.
    pub skeleton: Skeleton3D,
    /// Current bone matrices (column-major, one per bone).
    pub bone_matrices: Vec<[f32; 16]>,
    /// Associated GPU buffer handle.
    pub(in crate::libs::graphics::renderer3d) buffer: crate::libs::graphics::backend::BufferHandle,
    /// Number of vertices.
    pub(in crate::libs::graphics::renderer3d) vertex_count: i32,
    /// Position in world space.
    pub(in crate::libs::graphics::renderer3d) position: Vector3<f32>,
    /// Rotation (pitch, yaw, roll) in degrees.
    pub(in crate::libs::graphics::renderer3d) rotation: Vector3<f32>,
    /// Scale.
    pub(in crate::libs::graphics::renderer3d) scale: Vector3<f32>,
    /// Base color for the skinned mesh (RGBA).
    pub(in crate::libs::graphics::renderer3d) color: [f32; 4],
}
