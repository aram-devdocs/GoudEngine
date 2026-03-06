//! Tests for sprite animation component types.

use super::*;
use crate::core::math::Rect;

fn sample_frames() -> Vec<Rect> {
    vec![
        Rect::new(0.0, 0.0, 32.0, 32.0),
        Rect::new(32.0, 0.0, 32.0, 32.0),
        Rect::new(64.0, 0.0, 32.0, 32.0),
    ]
}

// =========================================================================
// AnimationClip tests
// =========================================================================

#[test]
fn test_animation_clip_new() {
    let clip = AnimationClip::new(sample_frames(), 0.1);

    assert_eq!(clip.frames.len(), 3);
    assert_eq!(clip.frame_duration, 0.1);
    assert_eq!(clip.mode, PlaybackMode::Loop);
}

#[test]
fn test_animation_clip_one_shot() {
    let clip = AnimationClip::one_shot(sample_frames(), 0.2);

    assert_eq!(clip.mode, PlaybackMode::OneShot);
    assert_eq!(clip.frame_duration, 0.2);
    assert_eq!(clip.frames.len(), 3);
}

#[test]
fn test_animation_clip_looping() {
    let clip = AnimationClip::looping(sample_frames(), 0.15);

    assert_eq!(clip.mode, PlaybackMode::Loop);
    assert_eq!(clip.frame_duration, 0.15);
}

#[test]
fn test_animation_clip_with_mode() {
    let clip = AnimationClip::new(sample_frames(), 0.1).with_mode(PlaybackMode::OneShot);

    assert_eq!(clip.mode, PlaybackMode::OneShot);
}

// =========================================================================
// SpriteAnimator tests
// =========================================================================

#[test]
fn test_sprite_animator_new() {
    let clip = AnimationClip::new(sample_frames(), 0.1);
    let animator = SpriteAnimator::new(clip);

    assert_eq!(animator.current_frame, 0);
    assert_eq!(animator.elapsed, 0.0);
    assert!(animator.playing);
    assert!(!animator.finished);
}

#[test]
fn test_sprite_animator_current_rect() {
    let clip = AnimationClip::new(sample_frames(), 0.1);
    let animator = SpriteAnimator::new(clip);

    let rect = animator.current_rect().unwrap();
    assert_eq!(rect, Rect::new(0.0, 0.0, 32.0, 32.0));
}

#[test]
fn test_sprite_animator_current_rect_empty_clip() {
    let clip = AnimationClip::new(vec![], 0.1);
    let animator = SpriteAnimator::new(clip);

    assert!(animator.current_rect().is_none());
}

#[test]
fn test_sprite_animator_pause_resume() {
    let clip = AnimationClip::new(sample_frames(), 0.1);
    let mut animator = SpriteAnimator::new(clip);

    assert!(animator.playing);

    animator.pause();
    assert!(!animator.playing);

    animator.resume();
    assert!(animator.playing);
}

#[test]
fn test_sprite_animator_resume_does_not_resume_finished() {
    let clip = AnimationClip::one_shot(sample_frames(), 0.1);
    let mut animator = SpriteAnimator::new(clip);
    animator.finished = true;
    animator.playing = false;

    animator.resume();
    assert!(!animator.playing);
}

#[test]
fn test_sprite_animator_reset() {
    let clip = AnimationClip::new(sample_frames(), 0.1);
    let mut animator = SpriteAnimator::new(clip);

    animator.current_frame = 2;
    animator.elapsed = 0.5;
    animator.finished = true;

    animator.reset();

    assert_eq!(animator.current_frame, 0);
    assert_eq!(animator.elapsed, 0.0);
    assert!(!animator.playing);
    assert!(!animator.finished);
}

#[test]
fn test_sprite_animator_play_restarts() {
    let clip = AnimationClip::one_shot(sample_frames(), 0.1);
    let mut animator = SpriteAnimator::new(clip);

    animator.current_frame = 2;
    animator.elapsed = 0.5;
    animator.finished = true;
    animator.playing = false;

    animator.play();

    assert_eq!(animator.current_frame, 0);
    assert_eq!(animator.elapsed, 0.0);
    assert!(animator.playing);
    assert!(!animator.finished);
}

#[test]
fn test_sprite_animator_is_component() {
    use crate::ecs::Component;
    fn assert_component<T: Component>() {}
    assert_component::<SpriteAnimator>();
}
