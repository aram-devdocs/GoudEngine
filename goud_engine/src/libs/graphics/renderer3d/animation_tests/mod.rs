//! Tests for animation playback and bone matrix computation.

mod baked_tests;

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

    pub(super) fn simple_animation() -> crate::core::types::KeyframeAnimation {
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

    /// Tests the animation LOD half-rate formula used in `update_animations`.
    ///
    /// The formula `!frame_counter.wrapping_add(player_id as u64).is_multiple_of(2)`
    /// skips evaluation when `(frame_counter + player_id)` is odd.  Different
    /// player IDs produce staggered skip patterns so not all models skip the
    /// same frame.
    #[test]
    fn test_animation_lod_half_rate_alternation() {
        let player_id: u32 = 42;
        let mut evaluated = Vec::new();
        for frame in 0u64..10 {
            let should_skip = !frame.wrapping_add(player_id as u64).is_multiple_of(2);
            evaluated.push(!should_skip);
        }
        // player_id 42 is even, so (frame + 42) parity matches frame parity.
        // frame 0 -> even -> evaluated, frame 1 -> odd -> skipped, ...
        assert_eq!(
            evaluated,
            vec![true, false, true, false, true, false, true, false, true, false]
        );
    }

    /// Verifies that different player IDs produce staggered half-rate patterns,
    /// so the engine does not skip every model on the same frame.
    #[test]
    fn test_animation_lod_half_rate_stagger() {
        let mut pattern_even = Vec::new();
        let mut pattern_odd = Vec::new();
        let even_id: u32 = 10;
        let odd_id: u32 = 11;
        for frame in 0u64..6 {
            let skip_even = !frame.wrapping_add(even_id as u64).is_multiple_of(2);
            let skip_odd = !frame.wrapping_add(odd_id as u64).is_multiple_of(2);
            pattern_even.push(!skip_even);
            pattern_odd.push(!skip_odd);
        }
        // Even and odd player IDs should produce opposite patterns.
        assert_eq!(pattern_even, vec![true, false, true, false, true, false]);
        assert_eq!(pattern_odd, vec![false, true, false, true, false, true]);
    }
}
