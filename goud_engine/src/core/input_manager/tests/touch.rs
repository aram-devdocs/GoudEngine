//! Touch input tests for `InputManager`.

use crate::core::input_manager::InputManager;
use crate::core::math::Vec2;
use crate::core::providers::input_types::MouseButton;

#[test]
fn touch_start_registers_active() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));

    assert!(input.touch_active(0));
    assert_eq!(input.touch_position(0), Some(Vec2::new(100.0, 200.0)));
    assert_eq!(input.touch_count(), 1);
}

#[test]
fn touch_end_deactivates() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));
    input.touch_end(0);

    assert!(!input.touch_active(0));
    assert_eq!(input.touch_count(), 0);
}

#[test]
fn touch_move_updates_position_and_delta() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));
    input.touch_move(0, Vec2::new(150.0, 250.0));

    assert_eq!(input.touch_position(0), Some(Vec2::new(150.0, 250.0)));
    let delta = input.touch_delta(0);
    assert!((delta.x - 50.0).abs() < f32::EPSILON);
    assert!((delta.y - 50.0).abs() < f32::EPSILON);
}

#[test]
fn multi_touch_tracks_independently() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 100.0));
    input.touch_start(1, Vec2::new(200.0, 200.0));
    input.touch_start(2, Vec2::new(300.0, 300.0));

    assert_eq!(input.touch_count(), 3);
    assert!(input.touch_active(0));
    assert!(input.touch_active(1));
    assert!(input.touch_active(2));

    input.touch_end(1);
    assert_eq!(input.touch_count(), 2);
    assert!(input.touch_active(0));
    assert!(!input.touch_active(1));
    assert!(input.touch_active(2));
}

#[test]
fn touch_just_pressed_only_on_start_frame() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));

    assert!(input.touch_just_pressed(0));

    // After update cycle, touch is still active but not "just pressed"
    input.update();
    assert!(input.touch_active(0));
    assert!(!input.touch_just_pressed(0));
}

#[test]
fn touch_just_released_only_on_end_frame() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));
    input.update();

    input.touch_end(0);
    assert!(input.touch_just_released(0));

    input.update();
    assert!(!input.touch_just_released(0));
}

#[test]
fn touch_cancel_behaves_like_end() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));
    input.touch_cancel(0);

    assert!(!input.touch_active(0));
    assert_eq!(input.touch_count(), 0);
}

#[test]
fn pointer_emulation_maps_touch_to_mouse() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));

    // Touch 0 should set mouse position and press left button
    assert!(input.mouse_button_pressed(MouseButton::Left));
    let pos = input.mouse_position();
    assert!((pos.x - 100.0).abs() < f32::EPSILON);
    assert!((pos.y - 200.0).abs() < f32::EPSILON);

    input.touch_end(0);
    assert!(!input.mouse_button_pressed(MouseButton::Left));
}

#[test]
fn pointer_emulation_disabled_skips_mouse() {
    let mut input = InputManager::new();
    input.set_touch_pointer_emulation(false);
    input.touch_start(0, Vec2::new(100.0, 200.0));

    assert!(!input.mouse_button_pressed(MouseButton::Left));
}

#[test]
fn pointer_emulation_only_affects_touch_zero() {
    let mut input = InputManager::new();
    input.touch_start(1, Vec2::new(100.0, 200.0));

    // Touch 1 should NOT trigger mouse emulation
    assert!(!input.mouse_button_pressed(MouseButton::Left));
}

#[test]
fn clear_resets_touch_state() {
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));
    input.touch_start(1, Vec2::new(200.0, 300.0));

    input.clear();
    assert_eq!(input.touch_count(), 0);
    assert!(!input.touch_active(0));
    assert!(!input.touch_active(1));
}

#[test]
fn same_frame_tap_not_detected_as_released() {
    // A touch that starts and ends within the same frame (before update())
    // is not detected as "just released" because it was never in touches_previous.
    // This documents the expected behavior for very fast taps.
    let mut input = InputManager::new();
    input.touch_start(0, Vec2::new(100.0, 200.0));
    input.touch_end(0);

    // just_pressed is true because touches_current has the entry and
    // touches_previous does not.
    assert!(input.touch_just_pressed(0));
    // just_released is FALSE because touches_previous does not contain the ID
    // (no update() was called between start and end). This is correct: the
    // touch was never observed as "active in a previous frame".
    assert!(!input.touch_just_released(0));
}

#[test]
fn touch_delta_zero_for_unknown_id() {
    let input = InputManager::new();
    let delta = input.touch_delta(999);
    assert!((delta.x).abs() < f32::EPSILON);
    assert!((delta.y).abs() < f32::EPSILON);
}

#[test]
fn touch_position_none_for_unknown_id() {
    let input = InputManager::new();
    assert_eq!(input.touch_position(999), None);
}
