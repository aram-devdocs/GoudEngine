//! FFI-safe animation types for sprite animation.

use crate::core::math::Rect;

// =============================================================================
// PlaybackMode
// =============================================================================

/// FFI-safe playback mode enum.
///
/// Maps to the Rust-side `PlaybackMode` enum but uses a fixed integer
/// representation for ABI stability.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiPlaybackMode {
    /// Restart from frame 0 after the last frame.
    Loop = 0,
    /// Stop at the last frame and mark the animation as finished.
    OneShot = 1,
}

// =============================================================================
// FfiSpriteAnimator
// =============================================================================

/// FFI-safe, flat representation of a `SpriteAnimator` snapshot.
///
/// This struct contains only C-compatible primitive types. The `frames`
/// data from the underlying `AnimationClip` is not included here because
/// variable-length arrays cannot cross FFI safely as struct fields. Instead,
/// frames are managed through the `FfiAnimationClipBuilder`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FfiSpriteAnimator {
    /// Index of the current frame in the animation clip.
    pub current_frame: u32,
    /// Accumulated time since the last frame advance (seconds).
    pub elapsed: f32,
    /// Whether the animation is currently playing.
    pub playing: bool,
    /// Whether a OneShot animation has completed.
    pub finished: bool,
    /// Seconds per frame.
    pub frame_duration: f32,
    /// Playback mode.
    pub mode: FfiPlaybackMode,
    /// Total number of frames in the clip.
    pub frame_count: u32,
}

impl FfiSpriteAnimator {
    /// Returns a default (empty, stopped) animator snapshot.
    pub fn default_value() -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
            playing: false,
            finished: false,
            frame_duration: 0.0,
            mode: FfiPlaybackMode::Loop,
            frame_count: 0,
        }
    }
}


// =============================================================================
// FfiAnimationClipBuilder
// =============================================================================

/// Heap-allocated builder for constructing an `AnimationClip` across FFI.
///
/// Accumulates frames (as `Rect` values) and configuration, then produces
/// an `FfiSpriteAnimator` snapshot when built.
///
/// ## Memory Management
///
/// - Allocated by `goud_animation_clip_builder_new()`
/// - Consumed by `goud_sprite_animator_from_clip()` (frees builder)
/// - If not consumed, must be freed with `goud_animation_clip_builder_free()`
// NOT repr(C): this is an opaque heap-allocated handle. Callers only hold
// a raw pointer and never inspect the struct layout across FFI.
pub struct FfiAnimationClipBuilder {
    /// Accumulated frame rectangles.
    pub frames: Vec<Rect>,
    /// Seconds per frame.
    pub frame_duration: f32,
    /// Playback mode.
    pub mode: FfiPlaybackMode,
}
