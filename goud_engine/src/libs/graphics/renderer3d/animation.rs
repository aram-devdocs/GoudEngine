//! Skeletal animation playback, blending, and transitions for the 3D renderer.

use crate::core::types::{KeyframeAnimation, SkeletonData};

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
    /// Pre-allocated scratch buffer for local transforms (avoids per-frame allocation).
    pub scratch_local: Vec<[f32; 16]>,
    /// Pre-allocated scratch buffer for global transforms (avoids per-frame allocation).
    pub scratch_global: Vec<[f32; 16]>,
}

impl AnimationPlayer {
    /// Create a new animation player for a skeleton with `bone_count` bones.
    pub fn new(bone_count: usize) -> Self {
        Self {
            primary: None,
            secondary: None,
            blend_factor: 0.0,
            transition: None,
            bone_matrices: vec![IDENTITY_MAT4; bone_count],
            scratch_local: vec![IDENTITY_MAT4; bone_count],
            scratch_global: vec![IDENTITY_MAT4; bone_count],
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
    ///
    /// Both animations loop by default. This is safe because the caller explicitly controls
    /// the blend factor; when they're done blending, they can stop or transition to a new clip.
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
        // Fallback: build property names per call when no cached names available.
        let fallback: Vec<BonePropertyNames> = (0..skeleton.bones.len())
            .map(BonePropertyNames::new)
            .collect();
        self.update_with_names(dt, skeleton, animations, &fallback);
    }

    /// Advance animation time and compute bone matrices using pre-cached
    /// property names (zero per-frame string allocations).
    pub fn update_with_names(
        &mut self,
        dt: f32,
        skeleton: &SkeletonData,
        animations: &[KeyframeAnimation],
        prop_names: &[BonePropertyNames],
    ) {
        let bone_count = self.advance_and_prepare(dt, skeleton, animations);
        if bone_count == 0 {
            return;
        }

        // Sample primary bone poses into self.bone_matrices using scratch buffers.
        if let Some(ref state) = self.primary {
            if let Some(anim) = animations.get(state.clip_index) {
                compute_bone_matrices_into(
                    skeleton,
                    anim,
                    state.time,
                    prop_names,
                    &mut self.scratch_local,
                    &mut self.scratch_global,
                    &mut self.bone_matrices,
                );
            } else {
                self.reset_bone_matrices_to_identity();
            }
        } else {
            self.reset_bone_matrices_to_identity();
        }

        // If blending, sample secondary and lerp in-place.
        if self.blend_factor > f32::EPSILON {
            if let Some(ref state) = self.secondary {
                if let Some(anim) = animations.get(state.clip_index) {
                    let secondary =
                        compute_bone_matrices_with_names(skeleton, anim, state.time, prop_names);
                    self.blend_secondary(&secondary);
                }
            }
        }
    }

    /// Advance animation time and compute bone matrices using pre-built
    /// [`BoneChannelMap`]s (zero per-frame string lookups or HashMap access).
    ///
    /// This is the fast path used when channel maps have been built at model
    /// load time.
    pub fn update_with_channel_maps(
        &mut self,
        dt: f32,
        skeleton: &SkeletonData,
        animations: &[KeyframeAnimation],
        channel_maps: &[BoneChannelMap],
    ) {
        let bone_count = self.advance_and_prepare(dt, skeleton, animations);
        if bone_count == 0 {
            return;
        }

        // Sample primary bone poses using the fast channel-map path.
        if let Some(ref state) = self.primary {
            if let Some(anim) = animations.get(state.clip_index) {
                if let Some(cm) = channel_maps.get(state.clip_index) {
                    compute_bone_matrices_into_fast(
                        skeleton,
                        anim,
                        state.time,
                        cm,
                        &mut self.scratch_local,
                        &mut self.scratch_global,
                        &mut self.bone_matrices,
                    );
                } else {
                    // Fallback: no channel map for this clip (should not happen).
                    let fallback: Vec<BonePropertyNames> =
                        (0..bone_count).map(BonePropertyNames::new).collect();
                    compute_bone_matrices_into(
                        skeleton,
                        anim,
                        state.time,
                        &fallback,
                        &mut self.scratch_local,
                        &mut self.scratch_global,
                        &mut self.bone_matrices,
                    );
                }
            } else {
                self.reset_bone_matrices_to_identity();
            }
        } else {
            self.reset_bone_matrices_to_identity();
        }

        // If blending, sample secondary and lerp in-place.
        if self.blend_factor > f32::EPSILON {
            if let Some(ref state) = self.secondary {
                if let Some(anim) = animations.get(state.clip_index) {
                    let secondary = if let Some(cm) = channel_maps.get(state.clip_index) {
                        compute_bone_matrices_with_channel_map(skeleton, anim, state.time, cm)
                    } else {
                        let fallback: Vec<BonePropertyNames> =
                            (0..bone_count).map(BonePropertyNames::new).collect();
                        compute_bone_matrices_with_names(skeleton, anim, state.time, &fallback)
                    };
                    self.blend_secondary(&secondary);
                }
            }
        }
    }

    /// Shared setup: advance states, handle transitions, resize buffers.
    /// Returns bone count (0 means caller should return early).
    fn advance_and_prepare(
        &mut self,
        dt: f32,
        skeleton: &SkeletonData,
        animations: &[KeyframeAnimation],
    ) -> usize {
        advance_state(&mut self.primary, dt, animations);
        advance_state(&mut self.secondary, dt, animations);

        if let Some(ref mut tr) = self.transition {
            tr.elapsed += dt;
            if tr.elapsed >= tr.duration {
                self.transition = None;
                self.secondary = None;
                self.blend_factor = 0.0;
            } else {
                self.blend_factor = 1.0 - (tr.elapsed / tr.duration);
            }
        }

        let bone_count = skeleton.bones.len();
        if bone_count == 0 {
            return 0;
        }

        if self.scratch_local.len() != bone_count {
            self.scratch_local.resize(bone_count, IDENTITY_MAT4);
            self.scratch_global.resize(bone_count, IDENTITY_MAT4);
        }
        if self.bone_matrices.len() != bone_count {
            self.bone_matrices.resize(bone_count, IDENTITY_MAT4);
        }

        bone_count
    }

    /// Reset all bone matrices to identity.
    fn reset_bone_matrices_to_identity(&mut self) {
        for m in self.bone_matrices.iter_mut() {
            *m = IDENTITY_MAT4;
        }
    }

    /// Blend secondary bone matrices into primary using `self.blend_factor`.
    fn blend_secondary(&mut self, secondary: &[[f32; 16]]) {
        let t = self.blend_factor;
        let inv_t = 1.0 - t;
        for (primary, sec) in self.bone_matrices.iter_mut().zip(secondary.iter()) {
            for (p, &s) in primary.iter_mut().zip(sec.iter()) {
                *p = *p * inv_t + s * t;
            }
        }
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Public wrapper for `advance_state` used by `core_model_animation` for
/// shared evaluation (G5) — advances the animation clock without recomputing
/// bone matrices.
pub(in crate::libs::graphics::renderer3d) fn advance_state_pub(
    state: &mut Option<AnimationState>,
    dt: f32,
    animations: &[KeyframeAnimation],
) {
    advance_state(state, dt, animations);
}

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
            // Clamp to valid range to prevent floating-point drift from
            // producing an out-of-bounds sample time after wrapping.
            s.time = s.time.clamp(0.0, anim.duration);
        }
    }
}

// Re-export types and functions from animation_sampling so existing callers
// continue to find them under `super::animation::*`.
#[allow(unused_imports)] // Used by animation_tests
pub(crate) use super::animation_sampling::{build_trs_matrix, mat4_mul};
pub(in crate::libs::graphics::renderer3d) use super::animation_sampling::{
    compute_bone_matrices_into, compute_bone_matrices_into_fast,
    compute_bone_matrices_with_channel_map, compute_bone_matrices_with_names,
};
pub use super::animation_sampling::{BoneChannelMap, BonePropertyNames};

pub(crate) const IDENTITY_MAT4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];
