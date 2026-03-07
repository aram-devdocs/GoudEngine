//! Skeletal mesh types for bone-driven vertex deformation.
//!
//! A [`SkeletalMesh2D`] holds vertices with bone weight assignments.
//! The skeletal animation system uses these weights to deform vertex
//! positions each frame, storing results in `deformed_positions`.

use crate::core::math::Vec2;
use crate::ecs::Component;

/// Influence of a single bone on a vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BoneWeight {
    /// Index into [`Skeleton2D::bones`](super::Skeleton2D).
    pub bone_id: usize,
    /// Blend weight in `[0.0, 1.0]`. Weights for one vertex should sum to 1.
    pub weight: f32,
}

/// A vertex in a skeletal mesh with bone weight assignments.
#[derive(Debug, Clone)]
pub struct SkeletalVertex {
    /// Rest-pose (bind-pose) position.
    pub position: Vec2,
    /// Texture coordinates.
    pub uv: Vec2,
    /// Bone influences (typically up to 4).
    pub weights: Vec<BoneWeight>,
}

/// Deformable 2D mesh driven by a [`Skeleton2D`](super::Skeleton2D).
///
/// The mesh stores rest-pose vertices and a parallel `deformed_positions`
/// buffer that the [`deform_skeletal_meshes`](crate::ecs::systems::skeletal_animation)
/// system fills each frame.
#[derive(Debug, Clone)]
pub struct SkeletalMesh2D {
    /// Rest-pose vertices with bone weights.
    pub vertices: Vec<SkeletalVertex>,
    /// Triangle indices into `vertices`.
    pub indices: Vec<u16>,
    /// Output buffer: deformed vertex positions (same length as `vertices`).
    pub deformed_positions: Vec<Vec2>,
}

impl Component for SkeletalMesh2D {}

impl SkeletalMesh2D {
    /// Creates a new skeletal mesh, initialising deformed positions from rest pose.
    pub fn new(vertices: Vec<SkeletalVertex>, indices: Vec<u16>) -> Self {
        let deformed_positions = vertices.iter().map(|v| v.position).collect();
        Self {
            vertices,
            indices,
            deformed_positions,
        }
    }
}
