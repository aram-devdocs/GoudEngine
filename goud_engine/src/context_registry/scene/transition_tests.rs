//! Tests for scene transition integration with [`SceneManager`].

use super::*;
use crate::context_registry::scene::transition::TransitionType;

#[test]
fn test_start_transition_activates_both_scenes() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("level_2").unwrap();

    // `to` is not active initially.
    assert!(!mgr.is_active(to));

    mgr.start_transition(from, to, TransitionType::Fade, 1.0)
        .unwrap();

    // Both scenes should now be active during the transition.
    assert!(mgr.is_active(from));
    assert!(mgr.is_active(to));
}

#[test]
fn test_tick_transition_completes_at_correct_time() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("next").unwrap();

    mgr.start_transition(from, to, TransitionType::Fade, 1.0)
        .unwrap();

    // Not complete yet.
    let result = mgr.tick_transition(0.5);
    assert!(result.is_none());
    assert!(mgr.is_transitioning());

    // Complete now.
    let result = mgr.tick_transition(0.5);
    assert!(result.is_some());
    let complete = result.unwrap();
    assert_eq!(complete.from_scene, from);
    assert_eq!(complete.to_scene, to);
    assert!(!mgr.is_transitioning());
}

#[test]
fn test_fade_transition_completes_in_correct_frames_at_60fps() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("next").unwrap();

    // A 0.5 second fade at 60 FPS takes exactly 30 frames because
    // 30 * (1/60) = 0.5 with no floating-point remainder.
    let duration = 0.5_f32;
    let dt = 1.0 / 60.0; // 60 FPS
    let expected_frames: u32 = 30;

    mgr.start_transition(from, to, TransitionType::Fade, duration)
        .unwrap();

    let mut frames = 0u32;
    loop {
        frames += 1;
        if mgr.tick_transition(dt).is_some() {
            break;
        }
    }

    assert_eq!(frames, expected_frames);
    assert!(!mgr.is_transitioning());
    assert!(!mgr.is_active(from));
    assert!(mgr.is_active(to));
}

#[test]
fn test_instant_transition_completes_immediately() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("next").unwrap();

    mgr.start_transition(from, to, TransitionType::Instant, 5.0)
        .unwrap();

    // Even though caller said 5.0, instant forces duration to 0.
    let result = mgr.tick_transition(0.0);
    assert!(result.is_some());
    assert!(!mgr.is_transitioning());
}

#[test]
fn test_start_transition_while_transitioning_fails() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("a").unwrap();
    let to2 = mgr.create_scene("b").unwrap();

    mgr.start_transition(from, to, TransitionType::Fade, 1.0)
        .unwrap();

    let result = mgr.start_transition(from, to2, TransitionType::Fade, 1.0);
    assert!(result.is_err());
}

#[test]
fn test_transition_deactivates_old_scene_on_completion() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("next").unwrap();

    mgr.start_transition(from, to, TransitionType::Fade, 0.5)
        .unwrap();

    assert!(mgr.is_active(from));

    // Complete the transition.
    mgr.tick_transition(0.5);

    assert!(!mgr.is_active(from));
    assert!(mgr.is_active(to));
}

#[test]
fn test_start_transition_invalid_scene_fails() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();

    // Invalid target scene.
    let result = mgr.start_transition(from, 999, TransitionType::Fade, 1.0);
    assert!(result.is_err());

    // Invalid source scene.
    let to = mgr.create_scene("valid").unwrap();
    let result = mgr.start_transition(999, to, TransitionType::Fade, 1.0);
    assert!(result.is_err());
}

#[test]
fn test_transition_progress_reports_correctly() {
    let mut mgr = SceneManager::new();
    let from = mgr.default_scene();
    let to = mgr.create_scene("next").unwrap();

    // No transition -> None.
    assert!(mgr.transition_progress().is_none());

    mgr.start_transition(from, to, TransitionType::Fade, 2.0)
        .unwrap();

    mgr.tick_transition(1.0);
    let progress = mgr.transition_progress().unwrap();
    assert!((progress - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_tick_transition_noop_when_none() {
    let mut mgr = SceneManager::new();
    assert!(mgr.tick_transition(1.0).is_none());
}
