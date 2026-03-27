//! Tests for BakedAnimationData and bake_animations.

#[cfg(test)]
mod tests {
    use super::super::super::animation::*;
    use super::super::tests::simple_animation;
    use crate::core::types::{BoneData, SkeletonData};

    fn simple_skeleton() -> SkeletonData {
        SkeletonData {
            bones: vec![
                BoneData {
                    name: "root".to_string(),
                    parent_index: -1,
                    inverse_bind_matrix: IDENTITY_MAT4,
                },
                BoneData {
                    name: "child".to_string(),
                    parent_index: 0,
                    inverse_bind_matrix: IDENTITY_MAT4,
                },
            ],
            bone_indices: vec![],
            bone_weights: vec![],
        }
    }

    /// Helper: build a BakedAnimationData with one clip containing known matrices.
    fn make_baked_data(
        bone_count: usize,
        frame_count: usize,
        sample_rate: f32,
        duration: f32,
        fill: impl Fn(usize, usize) -> [f32; 16],
    ) -> BakedAnimationData {
        let mut matrices = Vec::with_capacity(frame_count * bone_count);
        for frame in 0..frame_count {
            for bone in 0..bone_count {
                matrices.push(fill(frame, bone));
            }
        }
        BakedAnimationData {
            clips: vec![BakedClip {
                sample_rate,
                frame_count,
                duration,
                matrices,
            }],
            bone_count,
        }
    }

    #[test]
    fn test_baked_sample_empty_clips() {
        let baked = BakedAnimationData {
            clips: vec![],
            bone_count: 2,
        };
        let mut output = [IDENTITY_MAT4; 2];
        let ok = baked.sample(0, 0.0, &mut output);
        assert!(!ok, "sample() should return false when clips vec is empty");
    }

    #[test]
    fn test_baked_sample_zero_bone_count() {
        let baked = BakedAnimationData {
            clips: vec![BakedClip {
                sample_rate: 30.0,
                frame_count: 2,
                duration: 1.0,
                matrices: vec![],
            }],
            bone_count: 0,
        };
        let mut output: [[f32; 16]; 0] = [];
        let ok = baked.sample(0, 0.0, &mut output);
        assert!(!ok, "sample() should return false when bone_count is zero");
    }

    #[test]
    fn test_baked_sample_short_output_returns_false() {
        let baked = make_baked_data(2, 2, 30.0, 1.0, |_frame, _bone| IDENTITY_MAT4);
        let mut output = [IDENTITY_MAT4; 1];
        let ok = baked.sample(0, 0.0, &mut output);
        assert!(
            !ok,
            "sample() should return false when output is shorter than bone_count"
        );
    }

    #[test]
    fn test_baked_sample_exact_frame() {
        let mut frame0_mat = IDENTITY_MAT4;
        frame0_mat[12] = 5.0;
        frame0_mat[13] = 10.0;

        let mut frame1_mat = IDENTITY_MAT4;
        frame1_mat[12] = 15.0;
        frame1_mat[13] = 20.0;

        let baked = BakedAnimationData {
            clips: vec![BakedClip {
                sample_rate: 1.0,
                frame_count: 2,
                duration: 1.0,
                matrices: vec![frame0_mat, frame1_mat],
            }],
            bone_count: 1,
        };
        let mut output = [IDENTITY_MAT4; 1];

        let ok = baked.sample(0, 0.0, &mut output);
        assert!(ok, "sample() should succeed");
        assert!(
            (output[0][12] - 5.0).abs() < f32::EPSILON,
            "Expected x=5.0 at frame 0, got {}",
            output[0][12]
        );
        assert!(
            (output[0][13] - 10.0).abs() < f32::EPSILON,
            "Expected y=10.0 at frame 0, got {}",
            output[0][13]
        );
    }

    #[test]
    fn test_baked_sample_interpolation() {
        let mut frame0_mat = IDENTITY_MAT4;
        frame0_mat[12] = 0.0;

        let mut frame1_mat = IDENTITY_MAT4;
        frame1_mat[12] = 10.0;

        let baked = BakedAnimationData {
            clips: vec![BakedClip {
                sample_rate: 1.0,
                frame_count: 2,
                duration: 1.0,
                matrices: vec![frame0_mat, frame1_mat],
            }],
            bone_count: 1,
        };
        let mut output = [IDENTITY_MAT4; 1];

        let ok = baked.sample(0, 0.5, &mut output);
        assert!(ok, "sample() should succeed");
        assert!(
            (output[0][12] - 5.0).abs() < 0.01,
            "Expected interpolated x=5.0 at t=0.5, got {}",
            output[0][12]
        );
    }

    #[test]
    fn test_baked_sample_invalid_clip_index() {
        let baked = make_baked_data(1, 1, 30.0, 0.0, |_f, _b| IDENTITY_MAT4);
        let mut output = [IDENTITY_MAT4; 1];
        let ok = baked.sample(5, 0.0, &mut output);
        assert!(
            !ok,
            "sample() should return false for out-of-range clip index"
        );
    }

    #[test]
    fn test_baked_sample_single_frame_clip() {
        let mut mat = IDENTITY_MAT4;
        mat[12] = 42.0;

        let baked = BakedAnimationData {
            clips: vec![BakedClip {
                sample_rate: 30.0,
                frame_count: 1,
                duration: 0.0,
                matrices: vec![mat],
            }],
            bone_count: 1,
        };
        let mut output = [IDENTITY_MAT4; 1];

        for t in [0.0, 0.5, 1.0, 100.0] {
            let ok = baked.sample(0, t, &mut output);
            assert!(ok, "sample() should succeed for single-frame clip at t={t}");
            assert!(
                (output[0][12] - 42.0).abs() < f32::EPSILON,
                "Single-frame clip should always return frame 0 data, got x={} at t={t}",
                output[0][12]
            );
        }
    }

    // =========================================================================
    // bake_animations tests
    // =========================================================================

    #[test]
    fn test_bake_animations_empty_input() {
        let skeleton = simple_skeleton();
        let animations: Vec<crate::core::types::KeyframeAnimation> = vec![];
        let channel_maps: Vec<BoneChannelMap> = vec![];

        let baked = bake_animations(&skeleton, &animations, &channel_maps, 30.0);

        assert!(
            baked.clips.is_empty(),
            "bake_animations with empty input should produce empty clips"
        );
        assert_eq!(baked.bone_count, 2);
    }

    #[test]
    fn test_bake_animations_frame_count() {
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];
        let channel_maps: Vec<BoneChannelMap> = vec![];

        let baked = bake_animations(&skeleton, &animations, &channel_maps, 30.0);

        assert_eq!(baked.clips.len(), 1);
        assert_eq!(baked.clips[0].frame_count, 30);
        assert_eq!(baked.clips[0].matrices.len(), 30 * 2);
        assert_eq!(baked.bone_count, 2);
    }
}
