use std::time::Duration;

use glfw::{Key, MouseButton};

use crate::core::input_manager::{InputBinding, InputManager};

#[test]
fn test_with_buffer_duration() {
    let duration = Duration::from_millis(500);
    let input = InputManager::with_buffer_duration(duration);
    assert_eq!(input.buffer_duration(), duration);
}

#[test]
fn test_set_buffer_duration() {
    let mut input = InputManager::new();
    let new_duration = Duration::from_millis(300);

    input.set_buffer_duration(new_duration);
    assert_eq!(input.buffer_duration(), new_duration);
}

#[test]
fn test_buffer_size() {
    let mut input = InputManager::new();
    assert_eq!(input.buffer_size(), 0);

    input.press_key(Key::A);
    assert_eq!(input.buffer_size(), 1);

    input.press_key(Key::B);
    assert_eq!(input.buffer_size(), 2);
}

#[test]
fn test_clear_buffer() {
    let mut input = InputManager::new();

    input.press_key(Key::A);
    input.press_key(Key::B);
    assert_eq!(input.buffer_size(), 2);

    input.clear_buffer();
    assert_eq!(input.buffer_size(), 0);
}

#[test]
fn test_buffer_only_new_presses() {
    let mut input = InputManager::new();

    // First press should buffer
    input.press_key(Key::A);
    assert_eq!(input.buffer_size(), 1);

    // Pressing again while held should not buffer
    input.press_key(Key::A);
    assert_eq!(input.buffer_size(), 1);

    // Release and press again should buffer
    input.release_key(Key::A);
    input.press_key(Key::A);
    assert_eq!(input.buffer_size(), 2);
}

#[test]
fn test_sequence_detected_basic() {
    let mut input = InputManager::new();

    // Input sequence: A -> B -> C
    input.press_key(Key::A);
    input.press_key(Key::B);
    input.press_key(Key::C);

    let sequence = vec![
        InputBinding::Key(Key::A),
        InputBinding::Key(Key::B),
        InputBinding::Key(Key::C),
    ];

    assert!(input.sequence_detected(&sequence));
}

#[test]
fn test_sequence_detected_wrong_order() {
    let mut input = InputManager::new();

    // Input sequence: A -> C -> B (wrong order)
    input.press_key(Key::A);
    input.press_key(Key::C);
    input.press_key(Key::B);

    let sequence = vec![
        InputBinding::Key(Key::A),
        InputBinding::Key(Key::B),
        InputBinding::Key(Key::C),
    ];

    assert!(!input.sequence_detected(&sequence));
}

#[test]
fn test_sequence_detected_partial() {
    let mut input = InputManager::new();

    // Input only part of sequence
    input.press_key(Key::A);
    input.press_key(Key::B);

    let sequence = vec![
        InputBinding::Key(Key::A),
        InputBinding::Key(Key::B),
        InputBinding::Key(Key::C),
    ];

    assert!(!input.sequence_detected(&sequence));
}

#[test]
fn test_sequence_detected_empty() {
    let input = InputManager::new();

    // Empty sequence should return false
    assert!(!input.sequence_detected(&[]));
}

#[test]
fn test_sequence_detected_with_extra_inputs() {
    let mut input = InputManager::new();

    // Input sequence with extra inputs in between
    input.press_key(Key::A);
    input.press_key(Key::X); // Extra input
    input.press_key(Key::B);
    input.press_key(Key::Y); // Extra input
    input.press_key(Key::C);

    let sequence = vec![
        InputBinding::Key(Key::A),
        InputBinding::Key(Key::B),
        InputBinding::Key(Key::C),
    ];

    // Should still detect sequence (allows for extra inputs)
    assert!(input.sequence_detected(&sequence));
}

#[test]
fn test_consume_sequence() {
    let mut input = InputManager::new();

    input.press_key(Key::A);
    input.press_key(Key::B);

    let sequence = vec![InputBinding::Key(Key::A), InputBinding::Key(Key::B)];

    // First consume should succeed
    assert!(input.consume_sequence(&sequence));
    assert_eq!(input.buffer_size(), 0); // Buffer cleared

    // Second consume should fail (buffer cleared)
    assert!(!input.consume_sequence(&sequence));
}

#[test]
fn test_consume_sequence_not_detected() {
    let mut input = InputManager::new();

    input.press_key(Key::A);

    let sequence = vec![InputBinding::Key(Key::A), InputBinding::Key(Key::B)];

    // Sequence not complete, should not consume
    assert!(!input.consume_sequence(&sequence));
    assert_eq!(input.buffer_size(), 1); // Buffer not cleared
}

#[test]
fn test_time_since_last_input() {
    let mut input = InputManager::new();

    // No inputs, should return None
    assert!(input.time_since_last_input().is_none());

    input.press_key(Key::A);

    // Should return Some(small value)
    let time = input.time_since_last_input();
    assert!(time.is_some());
    assert!(time.unwrap() < 0.1); // Should be very recent
}

#[test]
fn test_buffered_inputs_iterator() {
    let mut input = InputManager::new();

    input.press_key(Key::A);
    input.press_key(Key::B);
    input.press_key(Key::C);

    let buffered: Vec<_> = input.buffered_inputs().collect();
    assert_eq!(buffered.len(), 3);

    // Check bindings (ages will be very small)
    assert_eq!(buffered[0].0, InputBinding::Key(Key::A));
    assert_eq!(buffered[1].0, InputBinding::Key(Key::B));
    assert_eq!(buffered[2].0, InputBinding::Key(Key::C));

    // Ages should be very small (recent)
    assert!(buffered[0].1 < 0.1);
    assert!(buffered[1].1 < 0.1);
    assert!(buffered[2].1 < 0.1);
}

#[test]
fn test_buffer_expiration() {
    use std::thread;

    let mut input = InputManager::with_buffer_duration(Duration::from_millis(50));

    input.press_key(Key::A);
    assert_eq!(input.buffer_size(), 1);

    // Wait for buffer to expire
    thread::sleep(Duration::from_millis(60));

    // Update should clean expired inputs
    input.update();
    assert_eq!(input.buffer_size(), 0);
}

#[test]
fn test_buffer_max_size() {
    let mut input = InputManager::new();

    // Fill buffer beyond max size (32)
    for _ in 0..40 {
        input.press_key(Key::A);
        input.release_key(Key::A);
    }

    // Should cap at 32
    assert!(input.buffer_size() <= 32);
}

#[test]
fn test_sequence_mixed_input_types() {
    let mut input = InputManager::new();

    // Sequence with keyboard, mouse, and gamepad
    input.press_key(Key::A);
    input.press_mouse_button(MouseButton::Button1);
    input.press_gamepad_button(0, 5);

    let sequence = vec![
        InputBinding::Key(Key::A),
        InputBinding::MouseButton(MouseButton::Button1),
        InputBinding::GamepadButton {
            gamepad_id: 0,
            button: 5,
        },
    ];

    assert!(input.sequence_detected(&sequence));
}

#[test]
fn test_fighting_game_combo() {
    let mut input = InputManager::new();

    // Classic "hadouken" combo: Down -> Down (double tap) -> Forward -> Punch
    input.press_key(Key::Down);
    input.release_key(Key::Down); // Release for double tap
    input.press_key(Key::Down); // Second Down press
    input.press_key(Key::Right);
    input.press_key(Key::Space);

    let hadouken = vec![
        InputBinding::Key(Key::Down),
        InputBinding::Key(Key::Down),
        InputBinding::Key(Key::Right),
        InputBinding::Key(Key::Space),
    ];

    assert!(input.sequence_detected(&hadouken));
}

#[test]
fn test_sequence_persistence_across_update() {
    let mut input = InputManager::new();

    input.press_key(Key::A);
    input.press_key(Key::B);

    // Update shouldn't clear recent buffer
    input.update();

    let sequence = vec![InputBinding::Key(Key::A), InputBinding::Key(Key::B)];
    assert!(input.sequence_detected(&sequence));
}

#[test]
fn test_buffer_not_cleared_by_state_clear() {
    let mut input = InputManager::new();

    input.press_key(Key::A);
    assert_eq!(input.buffer_size(), 1);

    // clear() only clears input state, not buffer
    input.clear();
    assert_eq!(input.buffer_size(), 1);
}

#[test]
fn test_double_tap_detection() {
    let mut input = InputManager::new();

    // Double tap: W -> W
    input.press_key(Key::W);
    input.release_key(Key::W);
    input.press_key(Key::W);

    let double_tap = vec![InputBinding::Key(Key::W), InputBinding::Key(Key::W)];

    assert!(input.sequence_detected(&double_tap));
}
