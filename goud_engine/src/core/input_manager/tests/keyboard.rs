use crate::core::input_manager::{InputBinding, InputManager};
use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode as Key, MouseButton};

#[test]
fn test_new_input_manager() {
    let input = InputManager::new();
    assert!(!input.key_pressed(Key::A));
    assert!(!input.mouse_button_pressed(MouseButton::Button1));
    assert_eq!(input.mouse_position(), Vec2::zero());
    assert_eq!(input.mouse_delta(), Vec2::zero());
}

#[test]
fn test_default() {
    let input = InputManager::default();
    assert!(!input.key_pressed(Key::W));
}

#[test]
fn test_key_pressed() {
    let mut input = InputManager::new();
    assert!(!input.key_pressed(Key::A));

    input.press_key(Key::A);
    assert!(input.key_pressed(Key::A));

    input.release_key(Key::A);
    assert!(!input.key_pressed(Key::A));
}

#[test]
fn test_key_just_pressed() {
    let mut input = InputManager::new();

    // Press key
    input.press_key(Key::Space);
    assert!(input.key_just_pressed(Key::Space)); // First frame

    // Update to next frame
    input.update();
    assert!(!input.key_just_pressed(Key::Space)); // Still held, but not "just" pressed

    // Release and press again
    input.release_key(Key::Space);
    input.update();
    input.press_key(Key::Space);
    assert!(input.key_just_pressed(Key::Space)); // Just pressed again
}

#[test]
fn test_key_just_released() {
    let mut input = InputManager::new();

    // Press key
    input.press_key(Key::W);
    input.update(); // Make it "previous"

    // Release key
    input.release_key(Key::W);
    assert!(input.key_just_released(Key::W));

    // Update to next frame
    input.update();
    assert!(!input.key_just_released(Key::W)); // No longer "just" released
}

#[test]
fn test_keys_pressed_iterator() {
    let mut input = InputManager::new();
    input.press_key(Key::A);
    input.press_key(Key::B);
    input.press_key(Key::C);

    let pressed_keys: Vec<_> = input.keys_pressed().collect();
    assert_eq!(pressed_keys.len(), 3);
    assert!(pressed_keys.contains(&&Key::A));
    assert!(pressed_keys.contains(&&Key::B));
    assert!(pressed_keys.contains(&&Key::C));
}

#[test]
fn test_update_copies_state() {
    let mut input = InputManager::new();

    input.press_key(Key::Space);
    assert!(input.key_just_pressed(Key::Space));

    input.update();
    assert!(!input.key_just_pressed(Key::Space)); // No longer "just" pressed
    assert!(input.key_pressed(Key::Space)); // Still pressed
}

#[test]
fn test_clone() {
    let mut input = InputManager::new();
    input.press_key(Key::A);

    let cloned = input.clone();
    assert!(cloned.key_pressed(Key::A));
}

#[test]
fn test_debug() {
    let input = InputManager::new();
    let debug_str = format!("{:?}", input);
    assert!(debug_str.contains("InputManager"));
}

#[test]
fn test_input_binding_key() {
    let mut input = InputManager::new();
    let binding = InputBinding::Key(Key::A);

    assert!(!binding.is_pressed(&input));
    assert!(!binding.is_just_pressed(&input));

    input.press_key(Key::A);
    assert!(binding.is_pressed(&input));
    assert!(binding.is_just_pressed(&input));

    input.update();
    assert!(binding.is_pressed(&input));
    assert!(!binding.is_just_pressed(&input));

    input.release_key(Key::A);
    assert!(!binding.is_pressed(&input));
    assert!(binding.is_just_released(&input));
}

#[test]
fn test_input_binding_eq() {
    let binding1 = InputBinding::Key(Key::A);
    let binding2 = InputBinding::Key(Key::A);
    let binding3 = InputBinding::Key(Key::B);

    assert_eq!(binding1, binding2);
    assert_ne!(binding1, binding3);
}

#[test]
fn test_input_binding_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(InputBinding::Key(Key::A));
    set.insert(InputBinding::Key(Key::A)); // Duplicate
    set.insert(InputBinding::Key(Key::B));

    assert_eq!(set.len(), 2); // Duplicate not added
}

#[test]
fn test_consumed_key_is_masked_for_queries() {
    let mut input = InputManager::new();
    input.press_key(Key::Tab);
    assert!(input.key_pressed(Key::Tab));
    assert!(input.key_just_pressed(Key::Tab));

    input.consume_key(Key::Tab);
    assert!(!input.key_pressed(Key::Tab));
    assert!(!input.key_just_pressed(Key::Tab));

    input.update();
    assert!(input.key_pressed(Key::Tab));
}
