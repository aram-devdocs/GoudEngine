//! Tests for the sprite animation system.

use super::update_sprite_animations;
use crate::assets::{loaders::TextureAsset, AssetServer};
use crate::core::event::Events;
use crate::core::math::Rect;
use crate::ecs::components::sprite::Sprite;
use crate::ecs::components::sprite_animator::events::{AnimationEventFired, EventPayload};
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

#[test]
fn test_animation_system_zero_frame_duration_no_hang() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    // Create an animator with frame_duration = 0.0 (edge case that could cause infinite loop)
    let clip = AnimationClip::new(sample_frames(), 0.0);
    world.insert(entity, SpriteAnimator::new(clip));

    // This should not hang and should skip the animator gracefully
    update_sprite_animations(&mut world, 0.1);

    // Verify the animator state is unchanged (still on frame 0, elapsed should be 0.0)
    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 0);
    assert_eq!(animator.elapsed, 0.0);
    assert!(animator.playing);
}

// =========================================================================
// AnimationLayerStack system tests
// =========================================================================

use crate::ecs::components::animation_layer::AnimationLayerStack;
use crate::ecs::systems::animation::BlendMode;

#[test]
fn test_layer_stack_blends_two_layers() {
    let (mut world, entity) = setup_world_with_sprite();

    let base_frames = vec![Rect::new(0.0, 0.0, 32.0, 32.0)];
    let overlay_frames = vec![Rect::new(64.0, 0.0, 64.0, 64.0)];

    let base_clip = AnimationClip::new(base_frames, 0.1);
    let overlay_clip = AnimationClip::new(overlay_frames, 0.1);

    let stack = AnimationLayerStack::new()
        .with_layer("base", base_clip, 1.0, BlendMode::Override)
        .with_layer("overlay", overlay_clip, 0.5, BlendMode::Override);

    world.insert(entity, stack);

    // Run the system (dt doesn't matter much for single-frame clips in Loop mode)
    update_sprite_animations(&mut world, 0.01);

    let sprite = world.get::<Sprite>(entity).unwrap();
    let rect = sprite
        .source_rect
        .expect("should have a blended source_rect");

    // Base: (0, 0, 32, 32), Overlay: (64, 0, 64, 64) at weight 0.5
    // blend_rects((0,0,32,32), (64,0,64,64), 0.5) = (32, 0, 48, 48)
    assert!((rect.x - 32.0).abs() < f32::EPSILON);
    assert!((rect.y - 0.0).abs() < f32::EPSILON);
    assert!((rect.width - 48.0).abs() < f32::EPSILON);
    assert!((rect.height - 48.0).abs() < f32::EPSILON);
}

#[test]
fn test_layer_weight_zero_ignored() {
    let (mut world, entity) = setup_world_with_sprite();

    let base_frames = vec![Rect::new(10.0, 20.0, 30.0, 40.0)];
    let ignored_frames = vec![Rect::new(999.0, 999.0, 999.0, 999.0)];

    let base_clip = AnimationClip::new(base_frames, 0.1);
    let ignored_clip = AnimationClip::new(ignored_frames, 0.1);

    let stack = AnimationLayerStack::new()
        .with_layer("base", base_clip, 1.0, BlendMode::Override)
        .with_layer("ignored", ignored_clip, 0.0, BlendMode::Override);

    world.insert(entity, stack);
    update_sprite_animations(&mut world, 0.01);

    let sprite = world.get::<Sprite>(entity).unwrap();
    let rect = sprite.source_rect.expect("should have source_rect");

    // Zero-weight layer should be ignored, result == base
    assert!((rect.x - 10.0).abs() < f32::EPSILON);
    assert!((rect.y - 20.0).abs() < f32::EPSILON);
    assert!((rect.width - 30.0).abs() < f32::EPSILON);
    assert!((rect.height - 40.0).abs() < f32::EPSILON);
}

#[test]
fn test_layer_stack_advances_frames_with_dt() {
    let (mut world, entity) = setup_world_with_sprite();

    let frames = vec![
        Rect::new(0.0, 0.0, 32.0, 32.0),
        Rect::new(32.0, 0.0, 32.0, 32.0),
    ];
    let clip = AnimationClip::new(frames, 0.1);

    let stack = AnimationLayerStack::new().with_layer("base", clip, 1.0, BlendMode::Override);

    world.insert(entity, stack);

    // Advance 0.15s: should move from frame 0 to frame 1
    update_sprite_animations(&mut world, 0.15);

    let stack = world.get::<AnimationLayerStack>(entity).unwrap();
    assert_eq!(stack.layers[0].current_frame, 1);

    let sprite = world.get::<Sprite>(entity).unwrap();
    assert_eq!(sprite.source_rect, Some(Rect::new(32.0, 0.0, 32.0, 32.0)));
}

// =========================================================================
// Animation event tests
// =========================================================================

fn setup_world_with_events() -> World {
    let mut world = World::new();
    world.insert_resource(Events::<AnimationEventFired>::new());
    world
}

#[test]
fn test_animation_event_fires_at_configured_frame() {
    let mut world = setup_world_with_events();
    let entity = world.spawn_empty();

    let clip = AnimationClip::new(sample_frames(), 0.1).with_event(1, "hit", EventPayload::None);
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance to frame 1 (0.15s > 0.1s)
    update_sprite_animations(&mut world, 0.15);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let mut reader = events.reader();
    let fired: Vec<_> = reader.read().collect();
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].event_name, "hit");
    assert_eq!(fired[0].frame_index, 1);
    assert!(fired[0].involves(entity));
}

#[test]
fn test_animation_event_fires_once_per_loop_cycle() {
    let mut world = setup_world_with_events();
    let entity = world.spawn_empty();

    let clip =
        AnimationClip::looping(sample_frames(), 0.1).with_event(1, "step", EventPayload::None);
    world.insert(entity, SpriteAnimator::new(clip));

    // First cycle: advance to frame 1
    update_sprite_animations(&mut world, 0.15);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let fired: Vec<_> = events.reader().read().collect();
    assert_eq!(fired.len(), 1, "Event should fire once in the first cycle");

    // Clear events for next cycle
    events.update();

    // Advance through full loop and back to frame 1 again.
    update_sprite_animations(&mut world, 0.35);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let fired: Vec<_> = events.reader().read().collect();
    assert_eq!(fired.len(), 1, "Event should fire once in the second cycle");
    assert_eq!(fired[0].event_name, "step");
}

#[test]
fn test_animation_event_one_shot_fires_once() {
    let mut world = setup_world_with_events();
    let entity = world.spawn_empty();

    let clip = AnimationClip::one_shot(sample_frames(), 0.1).with_event(
        2,
        "finish",
        EventPayload::String("done".to_string()),
    );
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance past all frames
    update_sprite_animations(&mut world, 0.35);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let fired: Vec<_> = events.reader().read().collect();
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].event_name, "finish");
    assert_eq!(fired[0].payload, EventPayload::String("done".to_string()));

    // Clear and advance again -- should not fire (animation finished)
    events.update();
    update_sprite_animations(&mut world, 0.35);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let fired: Vec<_> = events.reader().read().collect();
    assert_eq!(fired.len(), 0, "Finished animation should not fire events");
}

#[test]
fn test_animation_event_not_fired_before_frame() {
    let mut world = setup_world_with_events();
    let entity = world.spawn_empty();

    let clip = AnimationClip::new(sample_frames(), 0.1).with_event(2, "late", EventPayload::None);
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance only to frame 1 (not yet frame 2)
    update_sprite_animations(&mut world, 0.15);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let fired: Vec<_> = events.reader().read().collect();
    assert_eq!(
        fired.len(),
        0,
        "Event at frame 2 should not fire at frame 1"
    );
}

#[test]
fn test_multiple_events_same_frame() {
    let mut world = setup_world_with_events();
    let entity = world.spawn_empty();

    let clip = AnimationClip::new(sample_frames(), 0.1)
        .with_event(1, "sound", EventPayload::Int(42))
        .with_event(1, "particle", EventPayload::Float(1.5));
    world.insert(entity, SpriteAnimator::new(clip));

    // Advance to frame 1
    update_sprite_animations(&mut world, 0.15);

    let events = world
        .get_resource_mut::<Events<AnimationEventFired>>()
        .unwrap();
    events.update();
    let fired: Vec<_> = events.reader().read().collect();
    assert_eq!(fired.len(), 2);

    let names: Vec<&str> = fired.iter().map(|e| e.event_name.as_str()).collect();
    assert!(names.contains(&"sound"));
    assert!(names.contains(&"particle"));
}

#[test]
fn test_animation_events_without_resource_no_panic() {
    // World without Events<AnimationEventFired> resource
    let mut world = World::new();
    let entity = world.spawn_empty();

    let clip = AnimationClip::new(sample_frames(), 0.1).with_event(1, "test", EventPayload::None);
    world.insert(entity, SpriteAnimator::new(clip));

    // Should not panic even without the Events resource
    update_sprite_animations(&mut world, 0.15);

    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.current_frame, 1);
}
