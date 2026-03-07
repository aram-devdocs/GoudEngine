//! Skeletal animation playback controller.
//!
//! [`SkeletalAnimator`] drives a single [`SkeletalAnimation`] at runtime,
//! tracking elapsed time, play/pause state, and playback speed.

use super::animation::SkeletalAnimation;
use crate::ecs::Component;

/// Controls playback of a [`SkeletalAnimation`] on an entity.
///
/// Attach alongside a [`Skeleton2D`](super::Skeleton2D) component.
/// The [`update_skeletal_animations`](crate::ecs::systems::skeletal_animation)
/// system advances the timer and samples keyframes each frame.
#[derive(Debug, Clone)]
pub struct SkeletalAnimator {
    /// The animation clip being played.
    pub animation: SkeletalAnimation,
    /// Current playback position in seconds.
    pub current_time: f32,
    /// Whether the animator is actively advancing time.
    pub playing: bool,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
}

impl Component for SkeletalAnimator {}

impl SkeletalAnimator {
    /// Creates a new animator for the given animation, starting paused at t=0.
    pub fn new(animation: SkeletalAnimation) -> Self {
        Self {
            animation,
            current_time: 0.0,
            playing: false,
            speed: 1.0,
        }
    }

    /// Starts or resumes playback.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pauses playback without resetting the time.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resets playback to t=0 without changing play/pause state.
    pub fn reset(&mut self) {
        self.current_time = 0.0;
    }

    /// Returns `true` if a non-looping animation has reached its end.
    pub fn is_finished(&self) -> bool {
        !self.animation.looping && self.current_time >= self.animation.duration
    }
}
