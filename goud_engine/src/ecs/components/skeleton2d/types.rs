//! Core types for 2D skeletal animation.
//!
//! Defines [`BoneTransform`], [`Bone2D`], and [`Skeleton2D`] which together
//! represent a hierarchical bone structure for mesh deformation.

use crate::core::math::Vec2;
use crate::ecs::Component;

/// Affine transform for a single bone: position, rotation (radians), scale.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoneTransform {
    /// Translation offset relative to the parent bone (or world origin for root).
    pub position: Vec2,
    /// Rotation in radians (counter-clockwise positive).
    pub rotation: f32,
    /// Non-uniform scale.
    pub scale: Vec2,
}

impl Default for BoneTransform {
    /// Identity transform: zero position, zero rotation, unit scale.
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::one(),
        }
    }
}

impl BoneTransform {
    /// Linearly interpolates between `self` and `other` by factor `t`.
    ///
    /// Position and scale use [`Vec2::lerp`]; rotation uses scalar lerp.
    /// For small angle differences, linear interpolation of the angle is adequate.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Builds a 3x3 affine matrix (scale * rotate * translate) stored row-major.
    ///
    /// The matrix applies scale, then rotation, then translation:
    /// ```text
    /// | sx*cos  -sy*sin  tx |
    /// | sx*sin   sy*cos  ty |
    /// |   0        0      1 |
    /// ```
    pub fn to_matrix(&self) -> [[f32; 3]; 3] {
        let (sin, cos) = self.rotation.sin_cos();
        let sx = self.scale.x;
        let sy = self.scale.y;
        [
            [sx * cos, -sy * sin, self.position.x],
            [sx * sin, sy * cos, self.position.y],
            [0.0, 0.0, 1.0],
        ]
    }

    /// Composes parent and child transforms: `parent * child`.
    ///
    /// Applies the parent's rotation/scale to the child's position, accumulates
    /// rotations additively, and multiplies scales component-wise.
    pub fn combine(parent: &Self, child: &Self) -> Self {
        let (sin, cos) = parent.rotation.sin_cos();
        let rotated = Vec2::new(
            child.position.x * cos - child.position.y * sin,
            child.position.x * sin + child.position.y * cos,
        );
        Self {
            position: Vec2::new(
                parent.position.x + rotated.x * parent.scale.x,
                parent.position.y + rotated.y * parent.scale.y,
            ),
            rotation: parent.rotation + child.rotation,
            scale: Vec2::new(
                parent.scale.x * child.scale.x,
                parent.scale.y * child.scale.y,
            ),
        }
    }
}

/// A single bone in a [`Skeleton2D`].
#[derive(Debug, Clone)]
pub struct Bone2D {
    /// Index of this bone (matches its position in [`Skeleton2D::bones`]).
    pub id: usize,
    /// Human-readable name for editor integration and lookups.
    pub name: String,
    /// Index of the parent bone, or `None` for root bones.
    pub parent_id: Option<usize>,
    /// Local-space transform relative to the parent bone.
    pub local_transform: BoneTransform,
    /// Inverse of the bind pose used during mesh skinning.
    pub bind_pose_inverse: BoneTransform,
}

/// Hierarchical bone skeleton attached as an ECS component.
///
/// Contains the bone definitions and their computed world-space transforms.
/// The [`update_skeletal_animations`](crate::ecs::systems::skeletal_animation)
/// system keeps `world_transforms` up to date each frame.
#[derive(Debug, Clone)]
pub struct Skeleton2D {
    /// Ordered list of bones (parents appear before children).
    pub bones: Vec<Bone2D>,
    /// World-space transforms computed by hierarchy propagation.
    pub world_transforms: Vec<BoneTransform>,
}

impl Component for Skeleton2D {}

impl Skeleton2D {
    /// Creates a new skeleton and computes initial world transforms.
    ///
    /// Bones must be ordered so that every parent appears before its children.
    pub fn new(bones: Vec<Bone2D>) -> Self {
        let mut world_transforms = vec![BoneTransform::default(); bones.len()];
        for i in 0..bones.len() {
            let local = bones[i].local_transform;
            world_transforms[i] = match bones[i].parent_id {
                Some(pid) => BoneTransform::combine(&world_transforms[pid], &local),
                None => local,
            };
        }
        Self {
            bones,
            world_transforms,
        }
    }

    /// Returns the number of bones.
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Finds a bone index by name, returning `None` if not found.
    pub fn find_bone(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name == name)
    }
}
