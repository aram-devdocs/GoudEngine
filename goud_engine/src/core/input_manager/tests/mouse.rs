use glfw::MouseButton;

use crate::core::input_manager::{InputBinding, InputManager};
use crate::core::math::Vec2;

#[test]
fn test_mouse_button_pressed() {
    let mut input = InputManager::new();
    assert!(!input.mouse_button_pressed(MouseButton::Button1));

    input.press_mouse_button(MouseButton::Button1);
    assert!(input.mouse_button_pressed(MouseButton::Button1));

    input.release_mouse_button(MouseButton::Button1);
    assert!(!input.mouse_button_pressed(MouseButton::Button1));
}

#[test]
fn test_mouse_button_just_pressed() {
    let mut input = InputManager::new();

    input.press_mouse_button(MouseButton::Button2);
    assert!(input.mouse_button_just_pressed(MouseButton::Button2));

    input.update();
    assert!(!input.mouse_button_just_pressed(MouseButton::Button2));
}

#[test]
fn test_mouse_button_just_released() {
    let mut input = InputManager::new();

    input.press_mouse_button(MouseButton::Button1);
    input.update();
    input.release_mouse_button(MouseButton::Button1);
    assert!(input.mouse_button_just_released(MouseButton::Button1));

    input.update();
    assert!(!input.mouse_button_just_released(MouseButton::Button1));
}

#[test]
fn test_mouse_buttons_pressed_iterator() {
    let mut input = InputManager::new();
    input.press_mouse_button(MouseButton::Button1);
    input.press_mouse_button(MouseButton::Button2);

    let pressed_buttons: Vec<_> = input.mouse_buttons_pressed().collect();
    assert_eq!(pressed_buttons.len(), 2);
}

#[test]
fn test_mouse_position() {
    let mut input = InputManager::new();
    assert_eq!(input.mouse_position(), Vec2::zero());

    let pos = Vec2::new(100.0, 200.0);
    input.set_mouse_position(pos);
    assert_eq!(input.mouse_position(), pos);
}

#[test]
fn test_mouse_delta() {
    let mut input = InputManager::new();

    // First position
    input.set_mouse_position(Vec2::new(100.0, 100.0));
    assert_eq!(input.mouse_delta(), Vec2::new(100.0, 100.0)); // Delta from (0,0)

    // Second position
    input.set_mouse_position(Vec2::new(150.0, 120.0));
    assert_eq!(input.mouse_delta(), Vec2::new(50.0, 20.0)); // Delta from previous
}

#[test]
fn test_mouse_delta_reset_on_update() {
    let mut input = InputManager::new();

    input.set_mouse_position(Vec2::new(100.0, 100.0));
    assert_ne!(input.mouse_delta(), Vec2::zero());

    input.update(); // Reset delta
    assert_eq!(input.mouse_delta(), Vec2::zero());
}

#[test]
fn test_scroll_delta() {
    let mut input = InputManager::new();
    assert_eq!(input.scroll_delta(), Vec2::zero());

    input.add_scroll_delta(Vec2::new(0.0, 1.0));
    assert_eq!(input.scroll_delta(), Vec2::new(0.0, 1.0));

    input.add_scroll_delta(Vec2::new(0.0, 2.0));
    assert_eq!(input.scroll_delta(), Vec2::new(0.0, 3.0)); // Accumulates
}

#[test]
fn test_scroll_delta_reset_on_update() {
    let mut input = InputManager::new();

    input.add_scroll_delta(Vec2::new(0.0, 5.0));
    input.update();
    assert_eq!(input.scroll_delta(), Vec2::zero());
}

#[test]
fn test_input_binding_mouse_button() {
    let mut input = InputManager::new();
    let binding = InputBinding::MouseButton(MouseButton::Button1);

    assert!(!binding.is_pressed(&input));

    input.press_mouse_button(MouseButton::Button1);
    assert!(binding.is_pressed(&input));
    assert!(binding.is_just_pressed(&input));
}

#[test]
fn test_input_binding_display() {
    use glfw::Key;

    let key_binding = InputBinding::Key(Key::Space);
    let mouse_binding = InputBinding::MouseButton(MouseButton::Button1);
    let gamepad_binding = InputBinding::GamepadButton {
        gamepad_id: 2,
        button: 10,
    };

    let key_str = format!("{}", key_binding);
    let mouse_str = format!("{}", mouse_binding);
    let gamepad_str = format!("{}", gamepad_binding);

    assert!(key_str.contains("Key"));
    assert!(mouse_str.contains("MouseButton"));
    assert!(gamepad_str.contains("GamepadButton"));
    assert!(gamepad_str.contains("gamepad=2"));
    assert!(gamepad_str.contains("button=10"));
}

#[test]
fn test_clear() {
    use glfw::Key;

    let mut input = InputManager::new();

    // Set various inputs
    input.press_key(Key::A);
    input.press_mouse_button(MouseButton::Button1);
    input.set_mouse_position(Vec2::new(100.0, 100.0));
    input.add_scroll_delta(Vec2::new(0.0, 1.0));
    input.press_gamepad_button(0, 5);

    // Clear all
    input.clear();

    // Verify all cleared
    assert!(!input.key_pressed(Key::A));
    assert!(!input.mouse_button_pressed(MouseButton::Button1));
    assert_eq!(input.mouse_delta(), Vec2::zero());
    assert_eq!(input.scroll_delta(), Vec2::zero());
    assert!(!input.gamepad_button_pressed(0, 5));
}
