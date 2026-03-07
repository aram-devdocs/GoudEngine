//! Tests for the animation controller system.

use super::update_animation_controllers;
use crate::core::math::Rect;
use crate::ecs::components::animation_controller::{
    AnimationController, TransitionCondition,
};
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
        ctrl.transition_progress.is_some()
            || ctrl.current_state_name() == "run",
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
    assert!(
        ctrl.transition_progress.is_some()
            || ctrl.current_state_name() == "idle"
    );
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
