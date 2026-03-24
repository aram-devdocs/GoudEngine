//! Skeletal animation playback, blending, and transitions for the 3D renderer.

use crate::assets::loaders::animation::keyframe::interpolate;
use crate::assets::loaders::animation::KeyframeAnimation;
use crate::assets::loaders::mesh::SkeletonData;

// ============================================================================
// Animation state types
// ============================================================================

/// Active animation state for one layer.
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Index into the model's animation list.
    pub clip_index: usize,
    /// Current playback time in seconds.
    pub time: f32,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Whether this animation loops.
    pub looping: bool,
    /// Whether the animation is currently playing.
    pub playing: bool,
}

/// Transition between two animation clips.
#[derive(Debug, Clone)]
pub struct AnimationTransition {
    /// Source clip index.
    pub from_clip: usize,
    /// Target clip index.
    pub to_clip: usize,
    /// Total transition duration in seconds.
    pub duration: f32,
    /// Time elapsed since the transition started.
    pub elapsed: f32,
}

/// Animation playback controller for a model instance.
///
/// Manages animation state, blending, and transitions, and computes the final
/// per-bone matrices each frame.
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    /// Primary animation state.
    pub primary: Option<AnimationState>,
    /// Secondary state for manual blending.
    pub secondary: Option<AnimationState>,
    /// Blend factor (0.0 = primary only, 1.0 = secondary only).
    pub blend_factor: f32,
    /// Active transition (overrides manual blend while active).
    pub transition: Option<AnimationTransition>,
    /// Computed bone matrices for the current frame (column-major 4x4).
    pub bone_matrices: Vec<[f32; 16]>,
}

impl AnimationPlayer {
    /// Create a new animation player for a skeleton with `bone_count` bones.
    pub fn new(bone_count: usize) -> Self {
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        Self {
            primary: None,
            secondary: None,
            blend_factor: 0.0,
            transition: None,
            bone_matrices: vec![identity; bone_count],
        }
    }

    /// Start playing an animation clip.
    pub fn play(&mut self, clip_index: usize, looping: bool) {
        self.primary = Some(AnimationState {
            clip_index,
            time: 0.0,
            speed: 1.0,
            looping,
            playing: true,
        });
        self.transition = None;
    }

    /// Stop all animation playback.
    pub fn stop(&mut self) {
        if let Some(ref mut state) = self.primary {
            state.playing = false;
        }
        if let Some(ref mut state) = self.secondary {
            state.playing = false;
        }
        self.transition = None;
    }

    /// Set the playback speed for the primary animation.
    pub fn set_speed(&mut self, speed: f32) {
        if let Some(ref mut state) = self.primary {
            state.speed = speed;
        }
    }

    /// Set up manual blending between two clips.
    pub fn blend(&mut self, clip_a: usize, clip_b: usize, factor: f32) {
        self.primary = Some(AnimationState {
            clip_index: clip_a,
            time: self.primary.as_ref().map_or(0.0, |s| s.time),
            speed: 1.0,
            looping: true,
            playing: true,
        });
        self.secondary = Some(AnimationState {
            clip_index: clip_b,
            time: self.secondary.as_ref().map_or(0.0, |s| s.time),
            speed: 1.0,
            looping: true,
            playing: true,
        });
        self.blend_factor = factor.clamp(0.0, 1.0);
        self.transition = None;
    }

    /// Start a timed transition from the current animation to a target clip.
    pub fn transition(&mut self, target_clip: usize, duration: f32) {
        let from_clip = self
            .primary
            .as_ref()
            .map(|s| s.clip_index)
            .unwrap_or(target_clip);

        // Copy current primary to secondary so we can blend away from it.
        self.secondary = self.primary.clone();

        self.primary = Some(AnimationState {
            clip_index: target_clip,
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: true,
        });

        self.transition = Some(AnimationTransition {
            from_clip,
            to_clip: target_clip,
            duration: duration.max(f32::EPSILON),
            elapsed: 0.0,
        });
        self.blend_factor = 1.0; // Start fully on the old animation.
    }

    /// Returns `true` if any animation is currently playing.
    pub fn is_playing(&self) -> bool {
        self.primary.as_ref().is_some_and(|s| s.playing)
    }

    /// Returns the playback progress of the primary animation (0.0 to 1.0).
    pub fn progress(&self, animations: &[KeyframeAnimation]) -> f32 {
        if let Some(ref state) = self.primary {
            if let Some(anim) = animations.get(state.clip_index) {
                if anim.duration > 0.0 {
                    return (state.time / anim.duration).clamp(0.0, 1.0);
                }
            }
        }
        0.0
    }

    /// Advance animation time and compute bone matrices.
    pub fn update(&mut self, dt: f32, skeleton: &SkeletonData, animations: &[KeyframeAnimation]) {
        // 1. Advance time on primary and secondary states.
        advance_state(&mut self.primary, dt, animations);
        advance_state(&mut self.secondary, dt, animations);

        // 2. Handle transitions.
        if let Some(ref mut tr) = self.transition {
            tr.elapsed += dt;
            if tr.elapsed >= tr.duration {
                // Transition complete: primary is now the sole animation.
                self.transition = None;
                self.secondary = None;
                self.blend_factor = 0.0;
            } else {
                // Interpolate blend factor from 1.0 (old) to 0.0 (new primary).
                self.blend_factor = 1.0 - (tr.elapsed / tr.duration);
            }
        }

        let bone_count = skeleton.bones.len();
        if bone_count == 0 {
            return;
        }

        // 3. Sample primary bone poses.
        let primary_matrices = if let Some(ref state) = self.primary {
            if let Some(anim) = animations.get(state.clip_index) {
                compute_bone_matrices(skeleton, anim, state.time)
            } else {
                identity_matrices(bone_count)
            }
        } else {
            identity_matrices(bone_count)
        };

        // 4. If blending, sample secondary and lerp.
        let final_matrices = if self.blend_factor > f32::EPSILON {
            if let Some(ref state) = self.secondary {
                if let Some(anim) = animations.get(state.clip_index) {
                    let secondary_matrices = compute_bone_matrices(skeleton, anim, state.time);
                    lerp_matrices(&primary_matrices, &secondary_matrices, self.blend_factor)
                } else {
                    primary_matrices
                }
            } else {
                primary_matrices
            }
        } else {
            primary_matrices
        };

        // 5. Store results.
        self.bone_matrices = final_matrices;
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

fn advance_state(state: &mut Option<AnimationState>, dt: f32, animations: &[KeyframeAnimation]) {
    if let Some(ref mut s) = state {
        if !s.playing {
            return;
        }
        if let Some(anim) = animations.get(s.clip_index) {
            s.time += dt * s.speed;
            if s.time >= anim.duration {
                if s.looping {
                    if anim.duration > 0.0 {
                        s.time %= anim.duration;
                    } else {
                        s.time = 0.0;
                    }
                } else {
                    s.time = anim.duration;
                    s.playing = false;
                }
            }
        }
    }
}

/// Compute bone matrices for a skeleton at a given time in an animation clip.
///
/// Steps:
/// 1. For each bone, sample the animation channels for translation, rotation, scale.
/// 2. Build a local transform matrix from the sampled T/R/S.
/// 3. Walk the hierarchy to compute global transforms.
/// 4. Multiply global transform by inverse bind matrix.
fn compute_bone_matrices(
    skeleton: &SkeletonData,
    anim: &KeyframeAnimation,
    time: f32,
) -> Vec<[f32; 16]> {
    let bone_count = skeleton.bones.len();
    let mut local_transforms = Vec::with_capacity(bone_count);
    let mut global_transforms = vec![IDENTITY_MAT4; bone_count];
    let mut result = vec![IDENTITY_MAT4; bone_count];

    for (bone_idx, _bone) in skeleton.bones.iter().enumerate() {
        let (tx, ty, tz) = sample_translation(anim, bone_idx, time);
        let (rx, ry, rz, rw) = sample_rotation(anim, bone_idx, time);
        let (sx, sy, sz) = sample_scale(anim, bone_idx, time);

        let local = build_trs_matrix(tx, ty, tz, rx, ry, rz, rw, sx, sy, sz);
        local_transforms.push(local);
    }

    // Walk hierarchy: compute global transforms.
    for i in 0..bone_count {
        let parent = skeleton.bones[i].parent_index;
        if parent >= 0 && (parent as usize) < bone_count {
            global_transforms[i] =
                mat4_mul(&global_transforms[parent as usize], &local_transforms[i]);
        } else {
            global_transforms[i] = local_transforms[i];
        }
    }

    // Final: global * inverse_bind.
    for i in 0..bone_count {
        result[i] = mat4_mul(
            &global_transforms[i],
            &skeleton.bones[i].inverse_bind_matrix,
        );
    }

    result
}

fn sample_translation(anim: &KeyframeAnimation, bone_idx: usize, time: f32) -> (f32, f32, f32) {
    let prefix = format!("node_{bone_idx}.translation");
    let x = sample_channel(anim, &format!("{prefix}.x"), time).unwrap_or(0.0);
    let y = sample_channel(anim, &format!("{prefix}.y"), time).unwrap_or(0.0);
    let z = sample_channel(anim, &format!("{prefix}.z"), time).unwrap_or(0.0);
    (x, y, z)
}

fn sample_rotation(anim: &KeyframeAnimation, bone_idx: usize, time: f32) -> (f32, f32, f32, f32) {
    let prefix = format!("node_{bone_idx}.rotation");
    let x = sample_channel(anim, &format!("{prefix}.x"), time).unwrap_or(0.0);
    let y = sample_channel(anim, &format!("{prefix}.y"), time).unwrap_or(0.0);
    let z = sample_channel(anim, &format!("{prefix}.z"), time).unwrap_or(0.0);
    let w = sample_channel(anim, &format!("{prefix}.w"), time).unwrap_or(1.0);
    // Normalize quaternion.
    let len = (x * x + y * y + z * z + w * w).sqrt();
    if len > f32::EPSILON {
        (x / len, y / len, z / len, w / len)
    } else {
        (0.0, 0.0, 0.0, 1.0)
    }
}

fn sample_scale(anim: &KeyframeAnimation, bone_idx: usize, time: f32) -> (f32, f32, f32) {
    let prefix = format!("node_{bone_idx}.scale");
    let x = sample_channel(anim, &format!("{prefix}.x"), time).unwrap_or(1.0);
    let y = sample_channel(anim, &format!("{prefix}.y"), time).unwrap_or(1.0);
    let z = sample_channel(anim, &format!("{prefix}.z"), time).unwrap_or(1.0);
    (x, y, z)
}

fn sample_channel(anim: &KeyframeAnimation, property: &str, time: f32) -> Option<f32> {
    anim.channel_by_property(property)
        .map(|ch| interpolate(&ch.keyframes, time))
}

/// Build a column-major 4x4 TRS matrix from translation, quaternion rotation, and scale.
fn build_trs_matrix(
    tx: f32,
    ty: f32,
    tz: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
    sx: f32,
    sy: f32,
    sz: f32,
) -> [f32; 16] {
    // Rotation matrix from quaternion (column-major).
    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    // Column-major: [col0, col1, col2, col3]
    [
        // Column 0
        sx * (1.0 - 2.0 * (yy + zz)),
        sx * (2.0 * (xy + wz)),
        sx * (2.0 * (xz - wy)),
        0.0,
        // Column 1
        sy * (2.0 * (xy - wz)),
        sy * (1.0 - 2.0 * (xx + zz)),
        sy * (2.0 * (yz + wx)),
        0.0,
        // Column 2
        sz * (2.0 * (xz + wy)),
        sz * (2.0 * (yz - wx)),
        sz * (1.0 - 2.0 * (xx + yy)),
        0.0,
        // Column 3
        tx,
        ty,
        tz,
        1.0,
    ]
}

const IDENTITY_MAT4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// Multiply two column-major 4x4 matrices.
fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0f32;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = sum;
        }
    }
    out
}

/// Component-wise linear interpolation between two sets of matrices.
fn lerp_matrices(a: &[[f32; 16]], b: &[[f32; 16]], t: f32) -> Vec<[f32; 16]> {
    a.iter()
        .zip(b.iter())
        .map(|(ma, mb)| {
            let mut result = [0.0f32; 16];
            for i in 0..16 {
                result[i] = ma[i] * (1.0 - t) + mb[i] * t;
            }
            result
        })
        .collect()
}

fn identity_matrices(count: usize) -> Vec<[f32; 16]> {
    vec![IDENTITY_MAT4; count]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::animation::keyframe::{AnimationChannel, EasingFunction, Keyframe};
    use crate::assets::loaders::mesh::BoneData;

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

    fn simple_animation() -> KeyframeAnimation {
        KeyframeAnimation::new(
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
        let result = mat4_mul(&IDENTITY_MAT4, &IDENTITY_MAT4);
        for i in 0..16 {
            assert!(
                (result[i] - IDENTITY_MAT4[i]).abs() < f32::EPSILON,
                "Identity mul failed at index {i}"
            );
        }
    }

    #[test]
    fn test_build_trs_matrix_identity() {
        let m = build_trs_matrix(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
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
        let m = build_trs_matrix(3.0, 5.0, 7.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
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
}
