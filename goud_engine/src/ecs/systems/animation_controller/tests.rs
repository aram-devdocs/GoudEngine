//! Tests for the animation controller system.

use super::update_animation_controllers;
use crate::assets::{loaders::TextureAsset, AssetServer};
use crate::core::math::Rect;
use crate::ecs::components::animation_controller::{AnimationController, TransitionCondition};
use crate::ecs::components::sprite::Sprite;
use crate::ecs::components::sprite_animator::{AnimationClip, SpriteAnimator};
use crate::ecs::World;

fn idle_clip() -> AnimationClip {
    AnimationClip::new(vec![Rect::new(0.0, 0.0, 32.0, 32.0)], 0.1)
}

fn run_clip() -> AnimationClip {
    AnimationClip::new(
        vec![
            Rect::new(32.0, 0.0, 32.0, 32.0),
            Rect::new(64.0, 0.0, 32.0, 32.0),
        ],
        0.1,
    )
}

fn setup_controller() -> AnimationController {
    AnimationController::new("idle")
        .with_state("idle", idle_clip())
        .with_state("run", run_clip())
        .with_transition(
            "idle",
            "run",
            0.2,
            vec![TransitionCondition::BoolEquals {
                param: "running".to_string(),
                value: true,
            }],
        )
        .with_transition(
            "run",
            "idle",
            0.1,
            vec![TransitionCondition::BoolEquals {
                param: "running".to_string(),
                value: false,
            }],
        )
}

#[test]
fn test_transition_triggers_on_bool_param() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let mut ctrl = setup_controller();
    ctrl.set_bool("running", true);
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(idle_clip()));

    // First update starts the transition
    update_animation_controllers(&mut world, 0.0);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert!(
        ctrl.transition_progress.is_some(),
        "Transition should have started"
    );
    let progress = ctrl.transition_progress.as_ref().unwrap();
    assert_eq!(progress.from_state, "idle");
    assert_eq!(progress.to_state, "run");

    // Advance past blend duration (0.2s)
    update_animation_controllers(&mut world, 0.25);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(ctrl.current_state_name(), "run");
    assert!(ctrl.transition_progress.is_none());

    // Verify animator clip was updated to run clip
    let animator = world.get::<SpriteAnimator>(entity).unwrap();
    assert_eq!(animator.clip.frames.len(), 2);
    assert_eq!(animator.clip.frames[0], Rect::new(32.0, 0.0, 32.0, 32.0));
    assert_eq!(animator.current_frame, 0);
}

#[test]
fn test_transition_triggers_on_float_param() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let ctrl = AnimationController::new("idle")
        .with_state("idle", idle_clip())
        .with_state("run", run_clip())
        .with_transition(
            "idle",
            "run",
            0.0, // instant transition
            vec![TransitionCondition::FloatGreaterThan {
                param: "speed".to_string(),
                threshold: 1.0,
            }],
        );
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(idle_clip()));

    // Set speed above threshold
    world
        .get_mut::<AnimationController>(entity)
        .unwrap()
        .set_float("speed", 5.0);

    // First update: starts transition with duration 0
    update_animation_controllers(&mut world, 0.01);

    // With 0 duration, the transition starts and will complete on next tick
    // since elapsed (0.0) < duration (0.0) is false, it completes immediately
    // Actually: 0.0 + dt >= 0.0 is true on first check, so let's verify:
    let ctrl = world.get::<AnimationController>(entity).unwrap();
    // Duration is 0.0, so the transition starts on one tick. On the next update
    // elapsed (0.0) + dt >= duration (0.0) triggers completion.
    // But the start action sets elapsed=0 and duration=0. On the NEXT update,
    // 0 + dt >= 0 is true, so it completes.
    // Let's run another update to be sure.
    update_animation_controllers(&mut world, 0.01);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(ctrl.current_state_name(), "run");
}

#[test]
fn test_blend_duration_delays_switch() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let mut ctrl = setup_controller();
    ctrl.set_bool("running", true);
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(idle_clip()));

    // Start transition
    update_animation_controllers(&mut world, 0.0);

    // Advance partially (0.1s of 0.2s blend)
    update_animation_controllers(&mut world, 0.1);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(
        ctrl.current_state_name(),
        "idle",
        "Should still be in idle during blend"
    );
    assert!(ctrl.transition_progress.is_some());
    let progress = ctrl.transition_progress.as_ref().unwrap();
    assert!((progress.elapsed - 0.1).abs() < f32::EPSILON);

    // Advance past blend
    update_animation_controllers(&mut world, 0.15);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(ctrl.current_state_name(), "run");
    assert!(ctrl.transition_progress.is_none());
}

#[test]
fn test_no_transition_when_conditions_not_met() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let ctrl = setup_controller();
    // "running" param not set, so BoolEquals(true) is not met
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(idle_clip()));

    update_animation_controllers(&mut world, 0.1);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(ctrl.current_state_name(), "idle");
    assert!(ctrl.transition_progress.is_none());
}

#[test]
fn test_multiple_conditions_all_must_match() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let ctrl = AnimationController::new("idle")
        .with_state("idle", idle_clip())
        .with_state("run", run_clip())
        .with_transition(
            "idle",
            "run",
            0.0,
            vec![
                TransitionCondition::BoolEquals {
                    param: "grounded".to_string(),
                    value: true,
                },
                TransitionCondition::FloatGreaterThan {
                    param: "speed".to_string(),
                    threshold: 0.5,
                },
            ],
        );
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(idle_clip()));

    // Only one condition met
    world
        .get_mut::<AnimationController>(entity)
        .unwrap()
        .set_bool("grounded", true);

    update_animation_controllers(&mut world, 0.01);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(
        ctrl.current_state_name(),
        "idle",
        "Should not transition with only one condition met"
    );

    // Now meet both conditions
    world
        .get_mut::<AnimationController>(entity)
        .unwrap()
        .set_float("speed", 1.0);

    update_animation_controllers(&mut world, 0.01);

    // Transition started
    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert!(
        ctrl.transition_progress.is_some() || ctrl.current_state_name() == "run",
        "Transition should have started or completed (duration=0)"
    );
}

#[test]
fn test_float_less_than_condition() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let ctrl = AnimationController::new("run")
        .with_state("run", run_clip())
        .with_state("idle", idle_clip())
        .with_transition(
            "run",
            "idle",
            0.0,
            vec![TransitionCondition::FloatLessThan {
                param: "speed".to_string(),
                threshold: 0.5,
            }],
        );
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(run_clip()));

    // Set speed below threshold
    world
        .get_mut::<AnimationController>(entity)
        .unwrap()
        .set_float("speed", 0.1);

    update_animation_controllers(&mut world, 0.01);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert!(ctrl.transition_progress.is_some() || ctrl.current_state_name() == "idle");
}

#[test]
fn test_entity_without_sprite_animator_is_skipped() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let mut ctrl = setup_controller();
    ctrl.set_bool("running", true);
    world.insert(entity, ctrl);
    // No SpriteAnimator inserted

    // Should not panic
    update_animation_controllers(&mut world, 0.1);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert_eq!(ctrl.current_state_name(), "idle");
}

// =========================================================================
// Crossfade blending tests
// =========================================================================

fn setup_world_with_sprite_and_controller() -> (World, crate::ecs::entity::Entity) {
    let mut world = World::new();
    let mut asset_server = AssetServer::new();
    let texture = asset_server.load::<TextureAsset>("test.png");

    let entity = world.spawn_empty();
    world.insert(entity, Sprite::new(texture));
    (world, entity)
}

#[test]
fn test_crossfade_transition_blends_rect() {
    let (mut world, entity) = setup_world_with_sprite_and_controller();

    // idle: single frame at (0,0,32,32)
    // run:  first frame at (32,0,32,32)
    let mut ctrl = setup_controller();
    ctrl.set_bool("running", true);
    world.insert(entity, ctrl);
    world.insert(entity, SpriteAnimator::new(idle_clip()));

    // Tick 1: start transition (dt=0, no advance yet)
    update_animation_controllers(&mut world, 0.0);

    let ctrl = world.get::<AnimationController>(entity).unwrap();
    assert!(ctrl.transition_progress.is_some());

    // Tick 2: advance to 50% of 0.2s blend = 0.1s elapsed
    update_animation_controllers(&mut world, 0.1);

    // At 50% blend: lerp((0,0,32,32), (32,0,32,32), 0.5) = (16,0,32,32)
    let sprite = world.get::<Sprite>(entity).unwrap();
    let rect = sprite
        .source_rect
        .expect("should have blended rect during crossfade");
    assert!(
        (rect.x - 16.0).abs() < f32::EPSILON,
        "x should be 16.0 at 50% blend, got {}",
        rect.x
    );
    assert!(
        (rect.y - 0.0).abs() < f32::EPSILON,
        "y should be 0.0, got {}",
        rect.y
    );
    assert!(
        (rect.width - 32.0).abs() < f32::EPSILON,
        "width should be 32.0, got {}",
        rect.width
    );
}

#[test]
fn test_crossfade_frame_rate_independent() {
    // Two different frame rates should produce the same blend at the same elapsed time.
    let make_world = || -> (World, crate::ecs::entity::Entity) {
        let (mut world, entity) = setup_world_with_sprite_and_controller();
        let mut ctrl = setup_controller();
        ctrl.set_bool("running", true);
        world.insert(entity, ctrl);
        world.insert(entity, SpriteAnimator::new(idle_clip()));
        // Start transition
        update_animation_controllers(&mut world, 0.0);
        (world, entity)
    };

    // Scenario A: one big step of 0.1s
    let (mut world_a, entity_a) = make_world();
    update_animation_controllers(&mut world_a, 0.1);

    // Scenario B: two small steps of 0.05s each
    let (mut world_b, entity_b) = make_world();
    update_animation_controllers(&mut world_b, 0.05);
    update_animation_controllers(&mut world_b, 0.05);

    let sprite_a = world_a.get::<Sprite>(entity_a).unwrap();
    let sprite_b = world_b.get::<Sprite>(entity_b).unwrap();

    let rect_a = sprite_a.source_rect.expect("A should have rect");
    let rect_b = sprite_b.source_rect.expect("B should have rect");

    // Both should be at 50% blend (0.1s out of 0.2s duration)
    assert!(
        (rect_a.x - rect_b.x).abs() < 0.01,
        "x values should match: {} vs {}",
        rect_a.x,
        rect_b.x
    );
    assert!(
        (rect_a.width - rect_b.width).abs() < 0.01,
        "width values should match: {} vs {}",
        rect_a.width,
        rect_b.width
    );
}
