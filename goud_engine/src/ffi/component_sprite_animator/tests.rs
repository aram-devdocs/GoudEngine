//! # Tests for SpriteAnimator FFI
//!
//! Unit tests covering clip builder lifecycle, animator creation,
//! playback queries, and null-safety guarantees.

use crate::core::types::{FfiAnimationClipBuilder, FfiPlaybackMode, FfiSpriteAnimator};
use crate::ffi::component_sprite_animator::factory::{
    goud_animation_clip_builder_add_frame, goud_animation_clip_builder_free,
    goud_animation_clip_builder_new, goud_sprite_animator_from_clip,
};
use crate::ffi::component_sprite_animator::playback::{
    goud_sprite_animator_get_current_frame, goud_sprite_animator_is_finished,
    goud_sprite_animator_is_playing,
};

// =========================================================================
// Builder Lifecycle
// =========================================================================

#[test]
fn test_clip_builder_new_returns_non_null() {
    let builder = goud_animation_clip_builder_new(0.1, FfiPlaybackMode::Loop);
    assert!(!builder.is_null());
    // SAFETY: builder is non-null and was just created.
    unsafe { goud_animation_clip_builder_free(builder) };
}

#[test]
fn test_clip_builder_add_frame_returns_same_pointer() {
    let builder = goud_animation_clip_builder_new(0.1, FfiPlaybackMode::Loop);
    // SAFETY: builder is non-null (asserted by new returning non-null).
    let builder2 = unsafe {
        goud_animation_clip_builder_add_frame(builder, 0.0, 0.0, 32.0, 32.0)
    };
    assert_eq!(builder, builder2);
    // SAFETY: builder is still the valid sole owner.
    unsafe { goud_animation_clip_builder_free(builder) };
}

#[test]
fn test_clip_builder_free_does_not_crash() {
    let builder = goud_animation_clip_builder_new(0.1, FfiPlaybackMode::OneShot);
    assert!(!builder.is_null(), "builder should be non-null before free");
    // SAFETY: builder is non-null and is the sole owner.
    unsafe { goud_animation_clip_builder_free(builder) };
}

// =========================================================================
// Animator Creation
// =========================================================================

#[test]
fn test_animator_from_clip_loop() {
    let builder = goud_animation_clip_builder_new(0.1, FfiPlaybackMode::Loop);
    // SAFETY: builder is non-null; add_frame and from_clip follow the builder contract.
    let animator = unsafe {
        let b = goud_animation_clip_builder_add_frame(builder, 0.0, 0.0, 32.0, 32.0);
        let b = goud_animation_clip_builder_add_frame(b, 32.0, 0.0, 32.0, 32.0);
        let b = goud_animation_clip_builder_add_frame(b, 64.0, 0.0, 32.0, 32.0);
        goud_sprite_animator_from_clip(b)
    };

    assert_eq!(animator.current_frame, 0);
    assert_eq!(animator.elapsed, 0.0);
    assert!(animator.playing);
    assert!(!animator.finished);
    assert_eq!(animator.frame_duration, 0.1);
    assert_eq!(animator.mode, FfiPlaybackMode::Loop);
    assert_eq!(animator.frame_count, 3);
}

#[test]
fn test_animator_from_clip_oneshot() {
    let builder = goud_animation_clip_builder_new(0.2, FfiPlaybackMode::OneShot);
    // SAFETY: builder is non-null; add_frame and from_clip follow the builder contract.
    let animator = unsafe {
        let b = goud_animation_clip_builder_add_frame(builder, 0.0, 0.0, 16.0, 16.0);
        goud_sprite_animator_from_clip(b)
    };

    assert_eq!(animator.mode, FfiPlaybackMode::OneShot);
    assert_eq!(animator.frame_duration, 0.2);
    assert_eq!(animator.frame_count, 1);
    assert!(animator.playing);
}

#[test]
fn test_animator_from_clip_empty() {
    let builder = goud_animation_clip_builder_new(0.1, FfiPlaybackMode::Loop);
    // SAFETY: builder is non-null; from_clip consumes it.
    let animator = unsafe { goud_sprite_animator_from_clip(builder) };

    assert_eq!(animator.frame_count, 0);
    assert!(animator.playing);
}

// =========================================================================
// Playback Queries
// =========================================================================

#[test]
fn test_playback_queries_on_new_animator() {
    let builder = goud_animation_clip_builder_new(0.1, FfiPlaybackMode::Loop);
    // SAFETY: builder is non-null.
    let animator = unsafe {
        let b = goud_animation_clip_builder_add_frame(builder, 0.0, 0.0, 32.0, 32.0);
        goud_sprite_animator_from_clip(b)
    };

    // SAFETY: animator is a valid stack-allocated FfiSpriteAnimator.
    unsafe {
        assert_eq!(goud_sprite_animator_get_current_frame(&animator), 0);
        assert!(goud_sprite_animator_is_playing(&animator));
        assert!(!goud_sprite_animator_is_finished(&animator));
    }
}

#[test]
fn test_playback_queries_on_finished_animator() {
    let mut animator = FfiSpriteAnimator {
        current_frame: 2,
        elapsed: 0.0,
        playing: false,
        finished: true,
        frame_duration: 0.1,
        mode: FfiPlaybackMode::OneShot,
        frame_count: 3,
    };

    // SAFETY: animator is a valid stack-allocated FfiSpriteAnimator.
    unsafe {
        assert_eq!(goud_sprite_animator_get_current_frame(&animator), 2);
        assert!(!goud_sprite_animator_is_playing(&animator));
        assert!(goud_sprite_animator_is_finished(&animator));
    }
}

// =========================================================================
// Null Safety
// =========================================================================

#[test]
fn test_null_safety() {
    let null_builder: *mut FfiAnimationClipBuilder = std::ptr::null_mut();
    let null_animator: *const FfiSpriteAnimator = std::ptr::null();

    // SAFETY: All FFI functions are documented to handle null pointers gracefully.
    unsafe {
        // Builder null safety
        let returned = goud_animation_clip_builder_add_frame(
            null_builder, 0.0, 0.0, 32.0, 32.0,
        );
        assert!(returned.is_null());

        // Free null should not crash
        goud_animation_clip_builder_free(null_builder);

        // from_clip with null returns default
        let animator = goud_sprite_animator_from_clip(null_builder);
        assert_eq!(animator.current_frame, 0);
        assert!(!animator.playing);
        assert!(!animator.finished);
        assert_eq!(animator.frame_count, 0);

        // Playback queries with null
        assert_eq!(goud_sprite_animator_get_current_frame(null_animator), 0);
        assert!(!goud_sprite_animator_is_playing(null_animator));
        assert!(!goud_sprite_animator_is_finished(null_animator));
    }
}
