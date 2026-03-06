use glfw::{Key, MouseButton};

use crate::core::input_manager::{InputBinding, InputManager};

#[test]
fn test_map_action_basic() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));
    assert!(input.has_action("Jump"));
    assert_eq!(input.get_action_bindings("Jump").len(), 1);
}

#[test]
fn test_map_action_multiple_bindings() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));
    input.map_action("Jump", InputBinding::Key(Key::W));
    input.map_action(
        "Jump",
        InputBinding::GamepadButton {
            gamepad_id: 0,
            button: 0,
        },
    );

    let bindings = input.get_action_bindings("Jump");
    assert_eq!(bindings.len(), 3);
}

#[test]
fn test_unmap_action() {
    let mut input = InputManager::new();
    let binding = InputBinding::Key(Key::Space);

    input.map_action("Jump", binding);
    assert_eq!(input.get_action_bindings("Jump").len(), 1);

    assert!(input.unmap_action("Jump", binding));
    assert_eq!(input.get_action_bindings("Jump").len(), 0);

    // Unmapping non-existent binding returns false
    assert!(!input.unmap_action("Jump", binding));
}

#[test]
fn test_clear_action() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));
    input.map_action("Jump", InputBinding::Key(Key::W));

    assert!(input.clear_action("Jump"));
    assert!(!input.has_action("Jump"));

    // Clearing non-existent action returns false
    assert!(!input.clear_action("Jump"));
}

#[test]
fn test_clear_all_actions() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));
    input.map_action("Attack", InputBinding::Key(Key::E));
    input.map_action("Defend", InputBinding::Key(Key::Q));

    assert_eq!(input.action_count(), 3);

    input.clear_all_actions();
    assert_eq!(input.action_count(), 0);
    assert!(!input.has_action("Jump"));
    assert!(!input.has_action("Attack"));
    assert!(!input.has_action("Defend"));
}

#[test]
fn test_get_action_bindings_nonexistent() {
    let input = InputManager::new();
    let bindings = input.get_action_bindings("NonExistent");
    assert_eq!(bindings.len(), 0);
}

#[test]
fn test_has_action() {
    let mut input = InputManager::new();

    assert!(!input.has_action("Jump"));

    input.map_action("Jump", InputBinding::Key(Key::Space));
    assert!(input.has_action("Jump"));
}

#[test]
fn test_action_names() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));
    input.map_action("Attack", InputBinding::Key(Key::E));

    let names: Vec<_> = input.action_names().collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Jump"));
    assert!(names.contains(&"Attack"));
}

#[test]
fn test_action_count() {
    let mut input = InputManager::new();

    assert_eq!(input.action_count(), 0);

    input.map_action("Jump", InputBinding::Key(Key::Space));
    assert_eq!(input.action_count(), 1);

    input.map_action("Attack", InputBinding::Key(Key::E));
    assert_eq!(input.action_count(), 2);
}

#[test]
fn test_action_pressed() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));
    input.map_action("Jump", InputBinding::Key(Key::W));

    // No keys pressed
    assert!(!input.action_pressed("Jump"));

    // One key pressed
    input.press_key(Key::Space);
    assert!(input.action_pressed("Jump"));

    // Other key pressed
    input.release_key(Key::Space);
    input.press_key(Key::W);
    assert!(input.action_pressed("Jump"));

    // Both keys pressed
    input.press_key(Key::Space);
    assert!(input.action_pressed("Jump"));

    // No keys pressed again
    input.release_key(Key::Space);
    input.release_key(Key::W);
    assert!(!input.action_pressed("Jump"));
}

#[test]
fn test_action_pressed_nonexistent() {
    let input = InputManager::new();
    assert!(!input.action_pressed("NonExistent"));
}

#[test]
fn test_action_just_pressed() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));

    // Press key
    input.press_key(Key::Space);
    assert!(input.action_just_pressed("Jump"));

    // Update to next frame
    input.update();
    assert!(!input.action_just_pressed("Jump")); // Still held, not "just" pressed
}

#[test]
fn test_action_just_released() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));

    // Press key
    input.press_key(Key::Space);
    input.update();

    // Release key
    input.release_key(Key::Space);
    assert!(input.action_just_released("Jump"));

    // Update to next frame
    input.update();
    assert!(!input.action_just_released("Jump"));
}

#[test]
fn test_action_strength() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));

    // Not pressed
    assert_eq!(input.action_strength("Jump"), 0.0);

    // Pressed
    input.press_key(Key::Space);
    assert_eq!(input.action_strength("Jump"), 1.0);
}

#[test]
fn test_action_strength_nonexistent() {
    let input = InputManager::new();
    assert_eq!(input.action_strength("NonExistent"), 0.0);
}

#[test]
fn test_action_multiple_input_types() {
    let mut input = InputManager::new();

    // Map action to key, mouse button, and gamepad button
    input.map_action("Fire", InputBinding::Key(Key::Space));
    input.map_action("Fire", InputBinding::MouseButton(MouseButton::Button1));
    input.map_action(
        "Fire",
        InputBinding::GamepadButton {
            gamepad_id: 0,
            button: 0,
        },
    );

    // Test keyboard
    input.press_key(Key::Space);
    assert!(input.action_pressed("Fire"));
    input.release_key(Key::Space);
    assert!(!input.action_pressed("Fire"));

    // Test mouse
    input.press_mouse_button(MouseButton::Button1);
    assert!(input.action_pressed("Fire"));
    input.release_mouse_button(MouseButton::Button1);
    assert!(!input.action_pressed("Fire"));

    // Test gamepad
    input.press_gamepad_button(0, 0);
    assert!(input.action_pressed("Fire"));
}

#[test]
fn test_action_mapping_string_ownership() {
    let mut input = InputManager::new();

    // Test with &str
    input.map_action("Jump", InputBinding::Key(Key::Space));
    assert!(input.has_action("Jump"));

    // Test with String
    let action_name = String::from("Attack");
    input.map_action(action_name.clone(), InputBinding::Key(Key::E));
    assert!(input.has_action(&action_name));
}

#[test]
fn test_action_mapping_persistence_across_update() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));

    // Action mappings should persist across frame updates
    input.update();
    assert!(input.has_action("Jump"));

    input.update();
    assert!(input.has_action("Jump"));
}

#[test]
fn test_action_mapping_persistence_across_clear() {
    let mut input = InputManager::new();

    input.map_action("Jump", InputBinding::Key(Key::Space));

    // Action mappings should persist across input state clear
    input.clear();
    assert!(input.has_action("Jump"));
}
