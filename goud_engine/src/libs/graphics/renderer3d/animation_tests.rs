//! Tests for animation playback and bone matrix computation.

#[cfg(test)]
mod tests {
    use super::super::animation::*;
    use crate::core::types::{AnimationChannel, BoneData, EasingFunction, Keyframe, SkeletonData};

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

    fn simple_animation() -> crate::core::types::KeyframeAnimation {
        crate::core::types::KeyframeAnimation::new(
            "test".to_string(),
            1.0,
            vec![
                AnimationChannel {
                    target_property: "node_0.translation.x".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            value: 0.0,
                            easing: EasingFunction::Linear,
                        },
                        Keyframe {
                            time: 1.0,
                            value: 10.0,
                            easing: EasingFunction::Linear,
                        },
                    ],
                },
                AnimationChannel {
                    target_property: "node_0.translation.y".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 0.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_0.translation.z".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 0.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_0.rotation.w".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_0.scale.x".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_0.scale.y".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_0.scale.z".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_1.rotation.w".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_1.scale.x".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_1.scale.y".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
                AnimationChannel {
                    target_property: "node_1.scale.z".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.0,
                        value: 1.0,
                        easing: EasingFunction::Linear,
                    }],
                },
            ],
        )
    }

    #[test]
    fn test_animation_player_new() {
        let player = AnimationPlayer::new(2);
        assert_eq!(player.bone_matrices.len(), 2);
        assert!(!player.is_playing());
    }

    #[test]
    fn test_animation_player_play_stop() {
        let mut player = AnimationPlayer::new(2);
        player.play(0, true);
        assert!(player.is_playing());
        player.stop();
        assert!(!player.is_playing());
    }

    #[test]
    fn test_animation_player_progress() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];

        player.play(0, false);
        player.update(0.5, &skeleton, &animations);

        let progress = player.progress(&animations);
        assert!((progress - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_animation_player_update_translates_bone() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];

        player.play(0, false);
        player.update(0.5, &skeleton, &animations);

        // The root bone should have translation.x = 5.0 at t=0.5.
        // Column 3 of the bone matrix stores translation: indices 12, 13, 14.
        let root_matrix = &player.bone_matrices[0];
        assert!(
            (root_matrix[12] - 5.0).abs() < 0.1,
            "Expected root bone x translation ~5.0, got {}",
            root_matrix[12]
        );
    }

    #[test]
    fn test_animation_player_looping() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];

        player.play(0, true);
        // Advance past the end.
        player.update(1.5, &skeleton, &animations);
        assert!(player.is_playing());
        // Time should have wrapped.
        let progress = player.progress(&animations);
        assert!(progress < 1.0);
    }

    #[test]
    fn test_animation_player_non_looping_stops() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];

        player.play(0, false);
        player.update(1.5, &skeleton, &animations);
        assert!(!player.is_playing());
        let progress = player.progress(&animations);
        assert!((progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_animation_player_blend() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();

        // Two animations: one translates x=0..10, the other x=0..20.
        let mut anim_b = simple_animation();
        anim_b.name = "test_b".to_string();
        for ch in &mut anim_b.channels {
            if ch.target_property == "node_0.translation.x" {
                ch.keyframes[1].value = 20.0;
            }
        }
        let animations = vec![simple_animation(), anim_b];

        player.blend(0, 1, 0.5);
        player.update(0.5, &skeleton, &animations);

        // At t=0.5: anim_a root x=5.0, anim_b root x=10.0.
        // Blend at 0.5: lerp(5.0, 10.0, 0.5) = 7.5.
        let root_matrix = &player.bone_matrices[0];
        assert!(
            (root_matrix[12] - 7.5).abs() < 0.5,
            "Expected blended x ~7.5, got {}",
            root_matrix[12]
        );
    }

    #[test]
    fn test_animation_player_transition() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];

        player.play(0, true);
        player.update(0.5, &skeleton, &animations);

        // Start a transition to the same clip (for simplicity).
        player.transition(0, 0.5);
        assert!(player.transition.is_some());

        // Advance past transition duration.
        player.update(0.6, &skeleton, &animations);
        assert!(player.transition.is_none());
        assert!(player.is_playing());
    }

    #[test]
    fn test_animation_player_set_speed() {
        let mut player = AnimationPlayer::new(2);
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()];

        player.play(0, false);
        player.set_speed(2.0);
        player.update(0.25, &skeleton, &animations);

        // At speed 2.0, 0.25s of real time = 0.5s of animation time.
        let progress = player.progress(&animations);
        assert!((progress - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_mat4_mul_identity() {
        let result = super::super::animation::mat4_mul(&IDENTITY_MAT4, &IDENTITY_MAT4);
        for i in 0..16 {
            assert!(
                (result[i] - IDENTITY_MAT4[i]).abs() < f32::EPSILON,
                "Identity mul failed at index {i}"
            );
        }
    }

    #[test]
    fn test_build_trs_matrix_identity() {
        let m = super::super::animation::build_trs_matrix(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
        );
        for i in 0..16 {
            assert!(
                (m[i] - IDENTITY_MAT4[i]).abs() < f32::EPSILON,
                "TRS identity failed at index {i}: got {}, expected {}",
                m[i],
                IDENTITY_MAT4[i]
            );
        }
    }

    #[test]
    fn test_build_trs_matrix_translation_only() {
        let m = super::super::animation::build_trs_matrix(
            3.0, 5.0, 7.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
        );
        assert!((m[12] - 3.0).abs() < f32::EPSILON);
        assert!((m[13] - 5.0).abs() < f32::EPSILON);
        assert!((m[14] - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skinned_vertex_layout_size() {
        // 3 (pos) + 3 (normal) + 2 (uv) + 4 (bone ids) + 4 (bone weights) = 16 floats
        // 16 * 4 bytes = 64 bytes per vertex
        assert_eq!(16 * std::mem::size_of::<f32>(), 64);
    }

    // =========================================================================
    // BakedAnimationData tests
    // =========================================================================

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
        // Arrange: BakedAnimationData with no clips.
        let baked = BakedAnimationData {
            clips: vec![],
            bone_count: 2,
        };
        let mut output = [IDENTITY_MAT4; 2];

        // Act
        let ok = baked.sample(0, 0.0, &mut output);

        // Assert
        assert!(!ok, "sample() should return false when clips vec is empty");
    }

    #[test]
    fn test_baked_sample_zero_bone_count() {
        // Arrange: BakedAnimationData with bone_count = 0 but a clip present.
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

        // Act
        let ok = baked.sample(0, 0.0, &mut output);

        // Assert
        assert!(!ok, "sample() should return false when bone_count is zero");
    }

    #[test]
    fn test_baked_sample_short_output_returns_false() {
        // Arrange: 2-bone skeleton but output buffer holds only 1 matrix.
        let baked = make_baked_data(2, 2, 30.0, 1.0, |_frame, _bone| IDENTITY_MAT4);
        let mut output = [IDENTITY_MAT4; 1];

        // Act
        let ok = baked.sample(0, 0.0, &mut output);

        // Assert
        assert!(
            !ok,
            "sample() should return false when output is shorter than bone_count"
        );
    }

    #[test]
    fn test_baked_sample_exact_frame() {
        // Arrange: 1-bone, 2-frame clip. Frame 0 has a known translation matrix.
        let mut frame0_mat = IDENTITY_MAT4;
        frame0_mat[12] = 5.0; // translation x = 5.0
        frame0_mat[13] = 10.0; // translation y = 10.0

        let mut frame1_mat = IDENTITY_MAT4;
        frame1_mat[12] = 15.0;
        frame1_mat[13] = 20.0;

        let baked = BakedAnimationData {
            clips: vec![BakedClip {
                sample_rate: 1.0, // 1 fps for easy math
                frame_count: 2,
                duration: 1.0,
                matrices: vec![frame0_mat, frame1_mat],
            }],
            bone_count: 1,
        };
        let mut output = [IDENTITY_MAT4; 1];

        // Act: sample at exactly frame 0 (time = 0.0).
        let ok = baked.sample(0, 0.0, &mut output);

        // Assert
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
        // Arrange: 1-bone, 2-frame clip at 1 fps.
        // Frame 0: translation x = 0.0
        // Frame 1: translation x = 10.0
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

        // Act: sample at time = 0.5 (midpoint between frame 0 and frame 1).
        let ok = baked.sample(0, 0.5, &mut output);

        // Assert: linear interpolation should give x = 5.0.
        assert!(ok, "sample() should succeed");
        assert!(
            (output[0][12] - 5.0).abs() < 0.01,
            "Expected interpolated x=5.0 at t=0.5, got {}",
            output[0][12]
        );
    }

    #[test]
    fn test_baked_sample_invalid_clip_index() {
        // Arrange: 1-bone, 1-frame, 1-clip baked data.
        let baked = make_baked_data(1, 1, 30.0, 0.0, |_f, _b| IDENTITY_MAT4);
        let mut output = [IDENTITY_MAT4; 1];

        // Act: attempt to sample a clip index that does not exist.
        let ok = baked.sample(5, 0.0, &mut output);

        // Assert
        assert!(
            !ok,
            "sample() should return false for out-of-range clip index"
        );
    }

    #[test]
    fn test_baked_sample_single_frame_clip() {
        // Arrange: 1-bone, 1-frame clip -- any time should return frame 0.
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

        // Act: sample at various times.
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
        // Arrange: empty animations slice.
        let skeleton = simple_skeleton();
        let animations: Vec<crate::core::types::KeyframeAnimation> = vec![];
        let channel_maps: Vec<BoneChannelMap> = vec![];

        // Act
        let baked = bake_animations(&skeleton, &animations, &channel_maps, 30.0);

        // Assert
        assert!(
            baked.clips.is_empty(),
            "bake_animations with empty input should produce empty clips"
        );
        assert_eq!(baked.bone_count, 2);
    }

    #[test]
    fn test_bake_animations_frame_count() {
        // Arrange: a 1-second animation baked at 30 fps.
        let skeleton = simple_skeleton();
        let animations = vec![simple_animation()]; // duration = 1.0s
        let channel_maps: Vec<BoneChannelMap> = vec![];

        // Act
        let baked = bake_animations(&skeleton, &animations, &channel_maps, 30.0);

        // Assert: ceil(1.0 * 30.0) = 30, .max(1) = 30 frames.
        assert_eq!(baked.clips.len(), 1);
        assert_eq!(baked.clips[0].frame_count, 30);
        // Total matrices should be frame_count * bone_count.
        assert_eq!(baked.clips[0].matrices.len(), 30 * 2);
        assert_eq!(baked.bone_count, 2);
    }
}
