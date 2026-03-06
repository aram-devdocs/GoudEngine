//! Tests for the sprite animation system.

use super::update_sprite_animations;
use crate::assets::{loaders::TextureAsset, AssetServer};
use crate::core::math::Rect;
use crate::ecs::components::sprite::Sprite;
use crate::ecs::components::sprite_animator::{AnimationClip, SpriteAnimator};
use crate::ecs::World;

fn sample_frames() -> Vec<Rect> {
    vec![
        Rect::new(0.0, 0.0, 32.0, 32.0),
        Rect::new(32.0, 0.0, 32.0, 32.0),
        Rect::new(64.0, 0.0, 32.0, 32.0),
    ]
}

fn setup_world_with_sprite() -> (World, crate::ecs::entity::Entity) {
    let mut world = World::new();
    let mut asset_server = AssetServer::new();
    let texture = asset_server.load::<TextureAsset>("test.png");

    let entity = world.spawn_empty();
    world.insert(entity, Sprite::new(texture));
    (world, entity)
}

// =========================================================================
// System-level SpriteAnimator tests (advance logic)
// =========================================================================

#[test]
fn test_sprite_animator_loop_restart() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let clip = AnimationClip::new(sample_frames(), 0.1);
    world.insert(entity, SpriteAnimator::new(clip));

    // 3 frames * 0.1s = 0.3s to complete one cycle
    // Advance 0.35s: wraps to frame 0 with 0.05s elapsed
    update_sprite_animations(&mut world, 0.35);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 0);
    assert!(!animator.finished);
    assert!(animator.playing);
}

#[test]
fn test_sprite_animator_one_shot_completion() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let clip = AnimationClip::one_shot(sample_frames(), 0.1);
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance past all frames: 3 frames * 0.1s = 0.3s
    update_sprite_animations(&mut world, 0.35);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert!(animator.finished);
    assert!(!animator.playing);
    assert_eq!(animator.current_frame, 2); // last frame
}

// =========================================================================
// Full system tests
// =========================================================================

#[test]
fn test_animation_system_updates_sprite_source_rect() {
    let (mut world, entity) = setup_world_with_sprite();

    let clip = AnimationClip::new(sample_frames(), 0.1);
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance enough to move to frame 1 (0.15s > 0.1s)
    update_sprite_animations(&mut world, 0.15);

    let sprite = world.get::<Sprite>(entity).unwrap();
    assert_eq!(
        sprite.source_rect,
        Some(Rect::new(32.0, 0.0, 32.0, 32.0)),
        "Sprite source_rect should be updated to frame 1"
    );

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 1);
}

#[test]
fn test_animation_system_loop_wraps() {
    let (mut world, entity) = setup_world_with_sprite();

    let clip = AnimationClip::looping(sample_frames(), 0.1);
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance past all 3 frames: 0.35s wraps to frame 0
    update_sprite_animations(&mut world, 0.35);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 0);
    assert!(!animator.finished);
    assert!(animator.playing);

    let sprite = world.get::<Sprite>(entity).unwrap();
    assert_eq!(sprite.source_rect, Some(Rect::new(0.0, 0.0, 32.0, 32.0)));
}

#[test]
fn test_animation_system_one_shot_stops() {
    let (mut world, entity) = setup_world_with_sprite();

    let clip = AnimationClip::one_shot(sample_frames(), 0.1);
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance well past the end
    update_sprite_animations(&mut world, 0.5);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert!(animator.finished);
    assert!(!animator.playing);
    assert_eq!(animator.current_frame, 2);

    let sprite = world.get::<Sprite>(entity).unwrap();
    assert_eq!(sprite.source_rect, Some(Rect::new(64.0, 0.0, 32.0, 32.0)));
}

#[test]
fn test_animation_system_skips_paused() {
    let (mut world, entity) = setup_world_with_sprite();

    let clip = AnimationClip::new(sample_frames(), 0.1);
    let mut animator = SpriteAnimator::new(clip);
    animator.pause();
    world.insert(entity, animator);

    update_sprite_animations(&mut world, 0.5);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 0);
    assert_eq!(animator.elapsed, 0.0);
}

#[test]
fn test_animation_system_no_sprite_no_crash() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let clip = AnimationClip::new(sample_frames(), 0.1);
    world.insert(entity, SpriteAnimator::new(clip));

    // Should not panic even without a Sprite component
    update_sprite_animations(&mut world, 0.15);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 1);
}
