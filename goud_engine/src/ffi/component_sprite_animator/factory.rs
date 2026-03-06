//! # Builder Pattern for SpriteAnimator Component
//!
//! Provides a heap-allocated `FfiAnimationClipBuilder` that accumulates
//! animation frames and configuration, then produces an `FfiSpriteAnimator`.
//!
//! ## Usage (C#)
//!
//! ```csharp
//! var builder = goud_animation_clip_builder_new(0.1f, FfiPlaybackMode.Loop);
//! builder = goud_animation_clip_builder_add_frame(builder, 0, 0, 32, 32);
//! builder = goud_animation_clip_builder_add_frame(builder, 32, 0, 32, 32);
//! var animator = goud_sprite_animator_from_clip(builder); // Consumes builder
//! ```
//!
//! ## Memory Management
//!
//! - `goud_animation_clip_builder_new()` allocates on the heap
//! - `goud_sprite_animator_from_clip()` consumes the builder and frees memory
//! - If you don't call from_clip(), call `goud_animation_clip_builder_free()`

use crate::core::math::Rect;
use crate::core::types::{FfiAnimationClipBuilder, FfiPlaybackMode, FfiSpriteAnimator};
use crate::ecs::components::sprite_animator::{AnimationClip, PlaybackMode, SpriteAnimator};

// =============================================================================
// Conversion impls (live in FFI layer to avoid core -> ecs dependency)
// =============================================================================

impl From<PlaybackMode> for FfiPlaybackMode {
    fn from(mode: PlaybackMode) -> Self {
        match mode {
            PlaybackMode::Loop => FfiPlaybackMode::Loop,
            PlaybackMode::OneShot => FfiPlaybackMode::OneShot,
        }
    }
}

impl From<FfiPlaybackMode> for PlaybackMode {
    fn from(mode: FfiPlaybackMode) -> Self {
        match mode {
            FfiPlaybackMode::Loop => PlaybackMode::Loop,
            FfiPlaybackMode::OneShot => PlaybackMode::OneShot,
        }
    }
}

impl From<&SpriteAnimator> for FfiSpriteAnimator {
    fn from(animator: &SpriteAnimator) -> Self {
        Self {
            current_frame: animator.current_frame as u32,
            elapsed: animator.elapsed,
            playing: animator.playing,
            finished: animator.finished,
            frame_duration: animator.clip.frame_duration,
            mode: animator.clip.mode.into(),
            frame_count: animator.clip.frames.len() as u32,
        }
    }
}

/// Creates a new animation clip builder.
///
/// The builder is heap-allocated and must be either:
/// - Consumed with `goud_sprite_animator_from_clip()`, or
/// - Freed with `goud_animation_clip_builder_free()`
///
/// # Parameters
///
/// - `frame_duration`: Seconds per frame
/// - `mode`: Playback mode (Loop=0, OneShot=1)
///
/// # Returns
///
/// A pointer to a new `FfiAnimationClipBuilder`, or null if allocation fails.
///
/// # Safety
///
/// The returned pointer is owned by the caller.
#[no_mangle]
pub extern "C" fn goud_animation_clip_builder_new(
    frame_duration: f32,
    mode: FfiPlaybackMode,
) -> *mut FfiAnimationClipBuilder {
    let builder = FfiAnimationClipBuilder {
        frames: Vec::new(),
        frame_duration,
        mode,
    };
    Box::into_raw(Box::new(builder))
}

/// Adds a frame rectangle to the animation clip builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder (caller-owned)
/// - `x`: X position of the source rectangle
/// - `y`: Y position of the source rectangle
/// - `w`: Width of the source rectangle
/// - `h`: Height of the source rectangle
///
/// # Returns
///
/// The same builder pointer for chaining. Returns the input pointer
/// unchanged (including null) if builder is null.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_animation_clip_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_animation_clip_builder_add_frame(
    builder: *mut FfiAnimationClipBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> *mut FfiAnimationClipBuilder {
    if builder.is_null() {
        return builder;
    }
    // SAFETY: builder is non-null and was allocated via Box::into_raw
    (*builder).frames.push(Rect::new(x, y, w, h));
    builder
}

/// Frees an animation clip builder without building.
///
/// Use this if you need to abort animation construction and clean up memory.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder to free (caller-owned)
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_animation_clip_builder_new()` or null
/// - After this call, the builder pointer must not be used
#[no_mangle]
pub unsafe extern "C" fn goud_animation_clip_builder_free(builder: *mut FfiAnimationClipBuilder) {
    if !builder.is_null() {
        // SAFETY: builder is non-null and was allocated via Box::into_raw
        drop(Box::from_raw(builder));
    }
}

/// Consumes an animation clip builder and returns an `FfiSpriteAnimator`.
///
/// This function:
/// 1. Builds an `AnimationClip` from the accumulated frames
/// 2. Creates a `SpriteAnimator` from the clip
/// 3. Converts it to an `FfiSpriteAnimator` snapshot
/// 4. Frees the builder memory
///
/// After calling this, the builder pointer is invalid.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder to consume (ownership transfers to this function)
///
/// # Returns
///
/// An `FfiSpriteAnimator` with playback started. If builder is null,
/// returns a default (empty, stopped) animator.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_animation_clip_builder_new()` or null
/// - After this call, the builder pointer must not be used
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_animator_from_clip(
    builder: *mut FfiAnimationClipBuilder,
) -> FfiSpriteAnimator {
    if builder.is_null() {
        return FfiSpriteAnimator::default();
    }
    // SAFETY: builder is non-null and was allocated via Box::into_raw
    let boxed = Box::from_raw(builder);
    let clip = AnimationClip::new(boxed.frames, boxed.frame_duration)
        .with_mode(boxed.mode.into());
    let animator = SpriteAnimator::new(clip);
    FfiSpriteAnimator::from(&animator)
}
