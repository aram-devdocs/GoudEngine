//! Skeletal animation data types.
//!
//! Defines keyframe tracks that drive bone transforms over time.
//! A [`SkeletalAnimation`] contains one [`BoneTrack`] per animated bone,
//! each holding a sorted list of [`BoneKeyframe`]s.

use super::types::BoneTransform;

/// A single keyframe storing a bone transform at a specific time.
#[derive(Debug, Clone)]
pub struct BoneKeyframe {
    /// Time in seconds from the start of the animation.
    pub time: f32,
    /// Bone transform at this keyframe.
    pub transform: BoneTransform,
}

/// Timeline of keyframes for one bone.
#[derive(Debug, Clone)]
pub struct BoneTrack {
    /// Index into [`Skeleton2D::bones`](super::Skeleton2D) identifying
    /// which bone this track animates.
    pub bone_id: usize,
    /// Keyframes sorted by ascending `time`.
    pub keyframes: Vec<BoneKeyframe>,
}

/// A complete skeletal animation clip.
#[derive(Debug, Clone)]
pub struct SkeletalAnimation {
    /// Human-readable name (e.g. "walk", "idle").
    pub name: String,
    /// Total duration in seconds.
    pub duration: f32,
    /// Per-bone keyframe tracks.
    pub tracks: Vec<BoneTrack>,
    /// Whether the animation loops back to the start.
    pub looping: bool,
}
