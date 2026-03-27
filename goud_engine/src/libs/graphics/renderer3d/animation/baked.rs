//! Pre-baked animation cache for O(bones) runtime lookups.

use super::{
    compute_bone_matrices_into, compute_bone_matrices_into_fast, BoneChannelMap, BonePropertyNames,
    IDENTITY_MAT4,
};
use crate::core::types::{KeyframeAnimation, SkeletonData};

// ============================================================================
// Pre-baked animation cache
// ============================================================================

/// Pre-baked bone matrices for all frames of all clips.
///
/// Eliminates CPU animation evaluation at runtime by pre-computing bone
/// matrices at a fixed sample rate during model load. At runtime, the
/// update loop performs a simple lookup + lerp instead of full keyframe
/// sampling, hierarchy walk, and inverse-bind multiplication.
#[derive(Debug, Clone)]
pub struct BakedAnimationData {
    /// Per-clip baked data.
    pub clips: Vec<BakedClip>,
    /// Number of bones in the skeleton.
    pub bone_count: usize,
}

/// Pre-baked bone matrices for a single animation clip.
#[derive(Debug, Clone)]
pub struct BakedClip {
    /// Frame rate used for baking (typically 30.0).
    pub sample_rate: f32,
    /// Total number of baked frames.
    pub frame_count: usize,
    /// Duration of the clip in seconds.
    pub duration: f32,
    /// Flat array of bone matrices: `frame_count * bone_count` entries.
    /// Index as: `matrices[frame * bone_count + bone_index]`.
    pub matrices: Vec<[f32; 16]>,
}

impl BakedAnimationData {
    /// Look up the bone matrices for a clip at a given time.
    ///
    /// Performs linear interpolation between adjacent baked frames and
    /// writes the result into `output`. Returns `true` on success, or
    /// `false` if `clip_index` is out of range, there are no frames, or
    /// `output` is too short to hold all bone matrices.
    pub fn sample(&self, clip_index: usize, time: f32, output: &mut [[f32; 16]]) -> bool {
        let clip = match self.clips.get(clip_index) {
            Some(c) => c,
            None => return false,
        };
        if clip.frame_count == 0 || self.bone_count == 0 {
            return false;
        }

        let bc = self.bone_count;
        if output.len() < bc {
            return false;
        }

        // Compute frame position.
        let frame_f = (time * clip.sample_rate).max(0.0);
        let frame0 = (frame_f as usize).min(clip.frame_count - 1);
        let frame1 = (frame0 + 1).min(clip.frame_count - 1);
        let frac = frame_f - frame0 as f32;

        let base0 = frame0 * bc;
        let base1 = frame1 * bc;

        for (i, out_mat) in output.iter_mut().enumerate().take(bc) {
            if frac < 0.001 || frame0 == frame1 {
                // No interpolation needed.
                *out_mat = clip.matrices[base0 + i];
            } else {
                // Linear interpolation between frames.
                let m0 = &clip.matrices[base0 + i];
                let m1 = &clip.matrices[base1 + i];
                for j in 0..16 {
                    out_mat[j] = m0[j] + frac * (m1[j] - m0[j]);
                }
            }
        }
        true
    }
}

impl std::fmt::Display for BakedAnimationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BakedAnimationData({} clips, {} bones",
            self.clips.len(),
            self.bone_count
        )?;
        for (i, clip) in self.clips.iter().enumerate() {
            write!(
                f,
                ", clip[{i}]: {:.2}s @ {:.0}fps ({} frames)",
                clip.duration, clip.sample_rate, clip.frame_count
            )?;
        }
        write!(f, ")")
    }
}

/// Pre-bake bone matrices for all animation clips at a fixed sample rate.
///
/// This function evaluates every animation clip at `sample_rate` fps
/// using the fast channel-map path (falling back to property-name path
/// when no channel map is available) and stores the resulting bone
/// matrices. The baked data is then used at runtime for O(bones) lookups
/// instead of O(bones x channels x keyframes) evaluation.
pub fn bake_animations(
    skeleton: &SkeletonData,
    animations: &[KeyframeAnimation],
    channel_maps: &[BoneChannelMap],
    sample_rate: f32,
) -> BakedAnimationData {
    let bone_count = skeleton.bones.len();
    let mut clips = Vec::with_capacity(animations.len());

    let mut scratch_local = vec![IDENTITY_MAT4; bone_count];
    let mut scratch_global = vec![IDENTITY_MAT4; bone_count];
    let mut result = vec![IDENTITY_MAT4; bone_count];

    for (clip_idx, anim) in animations.iter().enumerate() {
        let duration = anim.duration;
        let frame_count = ((duration * sample_rate).ceil() as usize).max(1);
        let mut matrices = Vec::with_capacity(frame_count * bone_count);

        for frame in 0..frame_count {
            let time = ((frame as f32) / sample_rate).min(duration);

            // Use the fast channel-map path if available, otherwise fall
            // back to the property-name path.
            if let Some(map) = channel_maps.get(clip_idx) {
                compute_bone_matrices_into_fast(
                    skeleton,
                    anim,
                    time,
                    map,
                    &mut scratch_local,
                    &mut scratch_global,
                    &mut result,
                );
            } else {
                let prop_names: Vec<BonePropertyNames> =
                    (0..bone_count).map(BonePropertyNames::new).collect();
                compute_bone_matrices_into(
                    skeleton,
                    anim,
                    time,
                    &prop_names,
                    &mut scratch_local,
                    &mut scratch_global,
                    &mut result,
                );
            }

            matrices.extend_from_slice(&result);
        }

        clips.push(BakedClip {
            sample_rate,
            frame_count,
            duration,
            matrices,
        });
    }

    BakedAnimationData { clips, bone_count }
}
