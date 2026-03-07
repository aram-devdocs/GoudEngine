//! 2D skeletal animation components.
//!
//! This module provides the data types needed for bone-driven mesh deformation:
//!
//! - [`Skeleton2D`] / [`Bone2D`] / [`BoneTransform`]: Bone hierarchy and transforms
//! - [`SkeletalAnimation`] / [`BoneTrack`] / [`BoneKeyframe`]: Keyframe animation data
//! - [`SkeletalMesh2D`] / [`SkeletalVertex`] / [`BoneWeight`]: Weighted mesh vertices
//! - [`SkeletalAnimator`]: Playback controller component

mod animation;
mod mesh;
mod playback;
mod types;

pub use animation::{BoneKeyframe, BoneTrack, SkeletalAnimation};
pub use mesh::{BoneWeight, SkeletalMesh2D, SkeletalVertex};
pub use playback::SkeletalAnimator;
pub use types::{Bone2D, BoneTransform, Skeleton2D};
