//! Keyframe interpolation helpers for skeletal animation.
//!
//! Given a time `t` and a sorted list of [`BoneKeyframe`]s, finds the
//! surrounding pair and returns the interpolated [`BoneTransform`].

use crate::ecs::components::skeleton2d::{BoneKeyframe, BoneTransform};

/// Samples a bone track at time `t`, interpolating between surrounding keyframes.
///
/// - If `keyframes` is empty, returns [`BoneTransform::default()`].
/// - If `t` is before the first keyframe, returns the first keyframe's transform.
/// - If `t` is at or after the last keyframe, returns the last keyframe's transform.
/// - Otherwise, linearly interpolates between the two surrounding keyframes.
pub fn sample_track(keyframes: &[BoneKeyframe], t: f32) -> BoneTransform {
    if keyframes.is_empty() {
        return BoneTransform::default();
    }

    if keyframes.len() == 1 || t <= keyframes[0].time {
        return keyframes[0].transform;
    }

    let last = keyframes.len() - 1;
    if t >= keyframes[last].time {
        return keyframes[last].transform;
    }

    // Find the pair of keyframes surrounding `t`.
    // Binary search for the first keyframe with time > t.
    let right = match keyframes
        .binary_search_by(|kf| kf.time.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Equal))
    {
        Ok(i) => return keyframes[i].transform, // exact match
        Err(i) => i,
    };

    let left = right - 1;
    let kf_a = &keyframes[left];
    let kf_b = &keyframes[right];
    let span = kf_b.time - kf_a.time;

    if span <= 0.0 {
        return kf_a.transform;
    }

    let factor = (t - kf_a.time) / span;
    kf_a.transform.lerp(kf_b.transform, factor)
}
