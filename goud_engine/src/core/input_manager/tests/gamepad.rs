use glfw::GamepadAxis;

use crate::core::input_manager::{InputBinding, InputManager};
use crate::core::math::Vec2;

#[test]
fn test_gamepad_button_pressed() {
    let mut input = InputManager::new();

    input.press_gamepad_button(0, 1);
    assert!(input.gamepad_button_pressed(0, 1));
    assert!(!input.gamepad_button_pressed(0, 2));
    assert!(!input.gamepad_button_pressed(1, 1)); // Different gamepad

    input.release_gamepad_button(0, 1);
    assert!(!input.gamepad_button_pressed(0, 1));
}

#[test]
fn test_gamepad_button_just_pressed() {
    let mut input = InputManager::new();

    input.press_gamepad_button(0, 5);
    assert!(input.gamepad_button_just_pressed(0, 5));

    input.update();
    assert!(!input.gamepad_button_just_pressed(0, 5));
}

#[test]
fn test_gamepad_button_just_released() {
    let mut input = InputManager::new();

    input.press_gamepad_button(1, 3);
    input.update();
    input.release_gamepad_button(1, 3);
    assert!(input.gamepad_button_just_released(1, 3));

    input.update();
    assert!(!input.gamepad_button_just_released(1, 3));
}

#[test]
fn test_gamepad_multiple_gamepads() {
    let mut input = InputManager::new();

    input.press_gamepad_button(0, 1);
    input.press_gamepad_button(1, 1);
    input.press_gamepad_button(2, 2);

    assert!(input.gamepad_button_pressed(0, 1));
    assert!(input.gamepad_button_pressed(1, 1));
    assert!(input.gamepad_button_pressed(2, 2));
    assert!(!input.gamepad_button_pressed(2, 1));
}

#[test]
fn test_gamepad_capacity_expansion() {
    let mut input = InputManager::new();

    // Should expand to support gamepad 5
    input.press_gamepad_button(5, 1);
    assert!(input.gamepad_button_pressed(5, 1));
}

#[test]
fn test_input_binding_gamepad_button() {
    let mut input = InputManager::new();
    let binding = InputBinding::GamepadButton {
        gamepad_id: 0,
        button: 5,
    };

    assert!(!binding.is_pressed(&input));

    input.press_gamepad_button(0, 5);
    assert!(binding.is_pressed(&input));
    assert!(binding.is_just_pressed(&input));
}

#[test]
fn test_gamepad_axis_basic() {
    let mut input = InputManager::new();

    // Initially zero
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0);

    // Set axis value
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.5);
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.5);

    // Negative value
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftY, -0.75);
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftY), -0.75);
}

#[test]
fn test_gamepad_axis_deadzone() {
    let mut input = InputManager::new();

    // Default deadzone is 0.1
    assert_eq!(input.analog_deadzone(), 0.1);

    // Values within deadzone should be zeroed
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.05);
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0);

    // Values outside deadzone should pass through
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.15);
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.15);
}

#[test]
fn test_set_analog_deadzone() {
    let mut input = InputManager::new();

    // Set custom deadzone
    input.set_analog_deadzone(0.2);
    assert_eq!(input.analog_deadzone(), 0.2);

    // Value within new deadzone is zeroed
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.15);
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0);

    // Value outside new deadzone passes through
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.25);
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.25);
}

#[test]
fn test_gamepad_left_stick() {
    let mut input = InputManager::new();

    // Initially zero
    assert_eq!(input.gamepad_left_stick(0), Vec2::zero());

    // Set stick values
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.8);
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftY, -0.6);

    let stick = input.gamepad_left_stick(0);
    assert_eq!(stick.x, 0.8);
    assert_eq!(stick.y, -0.6);
}

#[test]
fn test_gamepad_right_stick() {
    let mut input = InputManager::new();

    // Set right stick values
    input.set_gamepad_axis(0, GamepadAxis::AxisRightX, -0.5);
    input.set_gamepad_axis(0, GamepadAxis::AxisRightY, 0.3);

    let stick = input.gamepad_right_stick(0);
    assert_eq!(stick.x, -0.5);
    assert_eq!(stick.y, 0.3);
}

#[test]
fn test_gamepad_triggers() {
    let mut input = InputManager::new();

    // Triggers are normalized from -1.0..1.0 to 0.0..1.0
    // Set left trigger (axis value -1.0 = 0.0, axis value 1.0 = 1.0)
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftTrigger, -1.0);
    assert_eq!(input.gamepad_left_trigger(0), 0.0);

    input.set_gamepad_axis(0, GamepadAxis::AxisLeftTrigger, 1.0);
    assert_eq!(input.gamepad_left_trigger(0), 1.0);

    // Mid-press (axis 0.0 = trigger 0.5)
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftTrigger, 0.0);
    assert_eq!(input.gamepad_left_trigger(0), 0.5);

    // Right trigger
    input.set_gamepad_axis(0, GamepadAxis::AxisRightTrigger, 0.5);
    assert_eq!(input.gamepad_right_trigger(0), 0.75);
}

#[test]
fn test_gamepad_axis_nonexistent_gamepad() {
    let input = InputManager::new();

    // Querying nonexistent gamepad returns 0.0 for axes
    assert_eq!(input.gamepad_axis(10, GamepadAxis::AxisLeftX), 0.0);
    assert_eq!(input.gamepad_left_stick(10), Vec2::zero());

    // Triggers normalize from -1.0..1.0 to 0.0..1.0
    // So axis value 0.0 (default) becomes trigger value 0.5
    assert_eq!(input.gamepad_left_trigger(10), 0.5);
    assert_eq!(input.gamepad_right_trigger(10), 0.5);
}

#[test]
fn test_gamepad_connection() {
    let mut input = InputManager::new();

    // Initially not connected
    assert!(!input.is_gamepad_connected(0));
    assert_eq!(input.connected_gamepad_count(), 0);

    // Connect gamepad 0
    input.set_gamepad_connected(0, true);
    assert!(input.is_gamepad_connected(0));
    assert_eq!(input.connected_gamepad_count(), 1);

    // Connect gamepad 1
    input.set_gamepad_connected(1, true);
    assert!(input.is_gamepad_connected(1));
    assert_eq!(input.connected_gamepad_count(), 2);

    // Disconnect gamepad 0
    input.set_gamepad_connected(0, false);
    assert!(!input.is_gamepad_connected(0));
    assert!(input.is_gamepad_connected(1));
    assert_eq!(input.connected_gamepad_count(), 1);
}

#[test]
fn test_connected_gamepads_iterator() {
    let mut input = InputManager::new();

    input.set_gamepad_connected(0, true);
    input.set_gamepad_connected(2, true);
    input.set_gamepad_connected(4, true);

    let connected: Vec<_> = input.connected_gamepads().collect();
    assert_eq!(connected.len(), 3);
    assert!(connected.contains(&0));
    assert!(connected.contains(&2));
    assert!(connected.contains(&4));
}

#[test]
fn test_gamepad_connection_nonexistent() {
    let input = InputManager::new();

    // Querying nonexistent gamepad returns false
    assert!(!input.is_gamepad_connected(10));
}

#[test]
fn test_gamepad_vibration() {
    let mut input = InputManager::new();

    // Initially no vibration
    assert_eq!(input.gamepad_vibration(0), 0.0);

    // Set vibration
    input.set_gamepad_vibration(0, 0.75);
    assert_eq!(input.gamepad_vibration(0), 0.75);

    // Clamping to 0.0-1.0
    input.set_gamepad_vibration(0, 1.5);
    assert_eq!(input.gamepad_vibration(0), 1.0);

    input.set_gamepad_vibration(0, -0.5);
    assert_eq!(input.gamepad_vibration(0), 0.0);
}

#[test]
fn test_stop_gamepad_vibration() {
    let mut input = InputManager::new();

    input.set_gamepad_vibration(0, 0.8);
    assert_eq!(input.gamepad_vibration(0), 0.8);

    input.stop_gamepad_vibration(0);
    assert_eq!(input.gamepad_vibration(0), 0.0);
}

#[test]
fn test_stop_all_vibration() {
    let mut input = InputManager::new();

    input.set_gamepad_vibration(0, 0.5);
    input.set_gamepad_vibration(1, 0.7);
    input.set_gamepad_vibration(2, 0.9);

    input.stop_all_vibration();

    assert_eq!(input.gamepad_vibration(0), 0.0);
    assert_eq!(input.gamepad_vibration(1), 0.0);
    assert_eq!(input.gamepad_vibration(2), 0.0);
}

#[test]
fn test_gamepad_state_clear_preserves_connection() {
    let mut input = InputManager::new();

    // Set up gamepad state
    input.set_gamepad_connected(0, true);
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.5);
    input.press_gamepad_button(0, 1);
    input.set_gamepad_vibration(0, 0.8);

    // Clear should remove input state but preserve connection and vibration
    input.clear();

    assert!(input.is_gamepad_connected(0)); // Connection preserved
    assert_eq!(input.gamepad_vibration(0), 0.8); // Vibration preserved
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0); // Axes cleared
    assert!(!input.gamepad_button_pressed(0, 1)); // Buttons cleared
}

#[test]
fn test_gamepad_multiple_gamepads_axes() {
    let mut input = InputManager::new();

    // Set different values for different gamepads
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.5);
    input.set_gamepad_axis(1, GamepadAxis::AxisLeftX, -0.5);
    input.press_gamepad_button(0, 1);
    input.press_gamepad_button(1, 2);

    // Verify isolation
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.5);
    assert_eq!(input.gamepad_axis(1, GamepadAxis::AxisLeftX), -0.5);
    assert!(input.gamepad_button_pressed(0, 1));
    assert!(!input.gamepad_button_pressed(0, 2));
    assert!(input.gamepad_button_pressed(1, 2));
    assert!(!input.gamepad_button_pressed(1, 1));
}

#[test]
fn test_gamepad_axes_update_persistence() {
    let mut input = InputManager::new();

    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.8);

    // Axes should persist across update (unlike deltas)
    input.update();
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.8);

    input.update();
    assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.8);
}

#[test]
fn test_gamepad_stick_magnitude() {
    let mut input = InputManager::new();

    // Set stick to diagonal (0.6, 0.8) - should have magnitude ~1.0
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.6);
    input.set_gamepad_axis(0, GamepadAxis::AxisLeftY, 0.8);

    let stick = input.gamepad_left_stick(0);
    let magnitude = (stick.x * stick.x + stick.y * stick.y).sqrt();

    // Magnitude should be close to 1.0 (floating point precision)
    assert!((magnitude - 1.0).abs() < 0.01);
}

#[test]
fn test_gamepad_expansion() {
    let mut input = InputManager::new();

    // Should automatically expand to support gamepad 10
    input.set_gamepad_axis(10, GamepadAxis::AxisLeftX, 0.5);
    assert_eq!(input.gamepad_axis(10, GamepadAxis::AxisLeftX), 0.5);

    input.set_gamepad_connected(10, true);
    assert!(input.is_gamepad_connected(10));
}
