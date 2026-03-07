//! Sprite animation component and supporting types.

use crate::core::math::Rect;
use crate::ecs::Component;

// =============================================================================
// PlaybackMode
// =============================================================================

/// Controls how an animation behaves when it reaches the last frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlaybackMode {
    /// Restart from frame 0 after the last frame.
    Loop,
    /// Stop at the last frame and mark the animation as finished.
    OneShot,
}

// =============================================================================
// AnimationClip
// =============================================================================

/// Defines the frame sequence and timing for a sprite animation.
///
/// `AnimationClip` is a plain data struct, not a component. It is stored
/// inside a [`SpriteAnimator`] component.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::sprite_animator::{AnimationClip, PlaybackMode};
/// use goud_engine::core::math::Rect;
///
/// let frames = vec![
///     Rect::new(0.0, 0.0, 32.0, 32.0),
///     Rect::new(32.0, 0.0, 32.0, 32.0),
///     Rect::new(64.0, 0.0, 32.0, 32.0),
/// ];
///
/// let clip = AnimationClip::new(frames, 0.1);
/// assert_eq!(clip.mode, PlaybackMode::Loop);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AnimationClip {
    /// Source rectangles for each frame (pixel coordinates).
    pub frames: Vec<Rect>,
    /// Seconds per frame.
    pub frame_duration: f32,
    /// Playback mode (Loop or OneShot).
    pub mode: PlaybackMode,
}

impl AnimationClip {
    /// Creates a new animation clip with the given frames and frame duration.
    ///
    /// Defaults to `PlaybackMode::Loop`.
    #[inline]
    pub fn new(frames: Vec<Rect>, frame_duration: f32) -> Self {
        Self {
            frames,
            frame_duration,
            mode: PlaybackMode::Loop,
        }
    }

    /// Sets the playback mode for this clip (builder pattern).
    #[inline]
    pub fn with_mode(mut self, mode: PlaybackMode) -> Self {
        self.mode = mode;
        self
    }

    /// Creates a looping animation clip.
    #[inline]
    pub fn looping(frames: Vec<Rect>, frame_duration: f32) -> Self {
        Self::new(frames, frame_duration).with_mode(PlaybackMode::Loop)
    }

    /// Creates a one-shot animation clip.
    #[inline]
    pub fn one_shot(frames: Vec<Rect>, frame_duration: f32) -> Self {
        Self::new(frames, frame_duration).with_mode(PlaybackMode::OneShot)
    }
}

// =============================================================================
// SpriteAnimator
// =============================================================================

/// ECS component that drives sprite sheet animation.
///
/// Attach this to an entity alongside a [`Sprite`](crate::ecs::components::Sprite)
/// to animate the sprite's `source_rect` through a sequence of frames.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::sprite_animator::{SpriteAnimator, AnimationClip};
/// use goud_engine::core::math::Rect;
///
/// let clip = AnimationClip::new(
///     vec![Rect::new(0.0, 0.0, 32.0, 32.0), Rect::new(32.0, 0.0, 32.0, 32.0)],
///     0.1,
/// );
///
/// let animator = SpriteAnimator::new(clip);
/// assert!(animator.playing);
/// assert!(!animator.finished);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpriteAnimator {
    /// The animation clip driving this animator.
    pub clip: AnimationClip,
    /// Index of the current frame in `clip.frames`.
    pub current_frame: usize,
    /// Accumulated time since the last frame advance.
    pub elapsed: f32,
    /// Whether the animation is currently playing.
    pub playing: bool,
    /// Whether a OneShot animation has completed.
    pub finished: bool,
}

impl SpriteAnimator {
    /// Creates a new animator from the given clip, starting playback immediately.
    #[inline]
    pub fn new(clip: AnimationClip) -> Self {
        Self {
            clip,
            current_frame: 0,
            elapsed: 0.0,
            playing: true,
            finished: false,
        }
    }

    /// Starts (or restarts) playback from frame 0.
    #[inline]
    pub fn play(&mut self) {
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.playing = true;
        self.finished = false;
    }

    /// Pauses playback without resetting frame position.
    #[inline]
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resumes playback from the current frame position.
    #[inline]
    pub fn resume(&mut self) {
        if !self.finished {
            self.playing = true;
        }
    }

    /// Resets the animator to its initial state (frame 0, not playing).
    #[inline]
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.playing = false;
        self.finished = false;
    }

    /// Returns `true` if a OneShot animation has completed.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Returns the source `Rect` for the current frame, or `None` if
    /// the clip has no frames.
    #[inline]
    pub fn current_rect(&self) -> Option<Rect> {
        self.clip.frames.get(self.current_frame).copied()
    }
}

impl Component for SpriteAnimator {}
