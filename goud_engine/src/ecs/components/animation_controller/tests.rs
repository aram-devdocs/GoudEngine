//! Tests for animation controller component types.

use super::*;
use crate::core::math::Rect;
use crate::ecs::components::AnimationClip;

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

#[test]
fn test_create_controller_with_initial_state() {
    let ctrl = AnimationController::new("idle");

    assert_eq!(ctrl.current_state_name(), "idle");
    assert!(ctrl.states.is_empty());
    assert!(ctrl.transitions.is_empty());
    assert!(ctrl.parameters.is_empty());
    assert!(ctrl.transition_progress.is_none());
}

#[test]
fn test_add_states_and_transitions() {
    let ctrl = AnimationController::new("idle")
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
        );

    assert_eq!(ctrl.states.len(), 2);
    assert!(ctrl.states.contains_key("idle"));
    assert!(ctrl.states.contains_key("run"));
    assert_eq!(ctrl.transitions.len(), 1);
    assert_eq!(ctrl.transitions[0].from, "idle");
    assert_eq!(ctrl.transitions[0].to, "run");
    assert_eq!(ctrl.transitions[0].blend_duration, 0.2);
    assert_eq!(ctrl.transitions[0].conditions.len(), 1);
}

#[test]
fn test_set_and_get_params() {
    let mut ctrl = AnimationController::new("idle");

    ctrl.set_bool("running", true);
    ctrl.set_float("speed", 5.0);

    assert_eq!(ctrl.get_param("running"), Some(&AnimParam::Bool(true)));
    assert_eq!(ctrl.get_param("speed"), Some(&AnimParam::Float(5.0)));
    assert_eq!(ctrl.get_param("nonexistent"), None);

    // Overwrite
    ctrl.set_bool("running", false);
    assert_eq!(ctrl.get_param("running"), Some(&AnimParam::Bool(false)));
}

#[test]
fn test_current_clip_returns_correct_clip() {
    let ctrl = AnimationController::new("idle")
        .with_state("idle", idle_clip())
        .with_state("run", run_clip());

    let clip = ctrl.current_clip().unwrap();
    assert_eq!(clip.frames.len(), 1);
    assert_eq!(clip.frames[0], Rect::new(0.0, 0.0, 32.0, 32.0));
}

#[test]
fn test_current_clip_returns_none_for_missing_state() {
    let ctrl = AnimationController::new("nonexistent");
    assert!(ctrl.current_clip().is_none());
}

#[test]
fn test_animation_controller_is_component() {
    use crate::ecs::Component;
    fn assert_component<T: Component>() {}
    assert_component::<AnimationController>();
}
