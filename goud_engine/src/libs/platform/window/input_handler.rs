use glfw::{Action, Key, MouseButton, WindowEvent};
use std::collections::{HashMap, HashSet};

use crate::types::MousePosition;

#[repr(C)]
pub struct InputHandler {
    keys_pressed: HashSet<Key>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    gamepad_buttons_pressed: HashMap<u32, HashSet<u32>>, // gamepad_id -> buttons pressed
    mouse_position: MousePosition,                       // Track mouse position
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            gamepad_buttons_pressed: HashMap::new(),
            mouse_position: MousePosition { x: 0.0, y: 0.0 },
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            // Key press and release events
            WindowEvent::Key(key, _, action, _) => match action {
                Action::Press => {
                    self.keys_pressed.insert(*key);
                }
                Action::Release => {
                    self.keys_pressed.remove(key);
                }
                _ => {} // Optionally handle repeat events here
            },

            // Mouse button events
            WindowEvent::MouseButton(button, action, _) => match action {
                Action::Press => {
                    self.mouse_buttons_pressed.insert(*button);
                }
                Action::Release => {
                    self.mouse_buttons_pressed.remove(button);
                }
                _ => {}
            },

            // Mouse position update
            WindowEvent::CursorPos(x, y) => {
                self.mouse_position = MousePosition { x: *x, y: *y };
            }

            // TODO: Handle gamepad events
            // Gamepad button events (assuming an external gamepad event handler updates these)
            // Example: External handler pushes events into InputHandler, e.g., `input_handler.handle_gamepad_event(...)`
            _ => {}
        }
    }

    // Check if a specific key is pressed
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    // Check if a specific mouse button is pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    // Get current mouse position
    pub fn get_mouse_position(&self) -> MousePosition {
        self.mouse_position
    }

    // Handle gamepad button press (example function for external handling)
    pub fn handle_gamepad_button(&mut self, gamepad_id: u32, button: u32, pressed: bool) {
        let buttons = self
            .gamepad_buttons_pressed
            .entry(gamepad_id)
            .or_insert_with(HashSet::new);
        if pressed {
            buttons.insert(button);
        } else {
            buttons.remove(&button);
        }
    }

    // Check if a specific gamepad button is pressed
    pub fn is_gamepad_button_pressed(&self, gamepad_id: u32, button: u32) -> bool {
        self.gamepad_buttons_pressed
            .get(&gamepad_id)
            .map_or(false, |buttons| buttons.contains(&button))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glfw::{Action, Key, MouseButton, WindowEvent};

    #[test]
    fn test_keyboard_input() {
        let mut input_handler = InputHandler::new();

        // Test single key press/release
        let key_event = WindowEvent::Key(Key::A, 0, Action::Press, glfw::Modifiers::empty());
        input_handler.handle_event(&key_event);
        assert!(input_handler.is_key_pressed(Key::A));

        let key_event = WindowEvent::Key(Key::A, 0, Action::Release, glfw::Modifiers::empty());
        input_handler.handle_event(&key_event);
        assert!(!input_handler.is_key_pressed(Key::A));

        // Test multiple simultaneous key presses
        let key_events = [
            WindowEvent::Key(Key::W, 0, Action::Press, glfw::Modifiers::empty()),
            WindowEvent::Key(Key::S, 0, Action::Press, glfw::Modifiers::empty()),
            WindowEvent::Key(Key::Space, 0, Action::Press, glfw::Modifiers::empty()),
        ];

        for event in key_events.iter() {
            input_handler.handle_event(event);
        }

        assert!(input_handler.is_key_pressed(Key::W));
        assert!(input_handler.is_key_pressed(Key::S));
        assert!(input_handler.is_key_pressed(Key::Space));
        assert!(!input_handler.is_key_pressed(Key::A));
    }

    #[test]
    fn test_mouse_input() {
        let mut input_handler = InputHandler::new();

        // Test mouse button press/release
        let press = WindowEvent::MouseButton(
            MouseButton::Button1,
            Action::Press,
            glfw::Modifiers::empty(),
        );
        let release = WindowEvent::MouseButton(
            MouseButton::Button1,
            Action::Release,
            glfw::Modifiers::empty(),
        );

        input_handler.handle_event(&press);
        assert!(input_handler.is_mouse_button_pressed(MouseButton::Button1));

        input_handler.handle_event(&release);
        assert!(!input_handler.is_mouse_button_pressed(MouseButton::Button1));

        // Test mouse movement
        let positions = [
            WindowEvent::CursorPos(0.0, 0.0),
            WindowEvent::CursorPos(100.5, 200.5),
            WindowEvent::CursorPos(-50.0, -25.0),
        ];

        for pos in positions.iter() {
            input_handler.handle_event(pos);
            match pos {
                WindowEvent::CursorPos(x, y) => {
                    let pos = input_handler.get_mouse_position();
                    assert_eq!(pos.x, *x);
                    assert_eq!(pos.y, *y);
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_gamepad_input() {
        let mut input_handler = InputHandler::new();

        // Test multiple gamepads with multiple buttons
        let gamepad_inputs = [
            (1, 0, true),
            (1, 1, true),
            (2, 0, true),
            (2, 5, true),
            (3, 2, true),
        ];

        for (pad, btn, state) in gamepad_inputs.iter() {
            input_handler.handle_gamepad_button(*pad, *btn, *state);
            assert!(input_handler.is_gamepad_button_pressed(*pad, *btn));
        }

        // Test button release
        input_handler.handle_gamepad_button(1, 0, false);
        assert!(!input_handler.is_gamepad_button_pressed(1, 0));
        assert!(input_handler.is_gamepad_button_pressed(1, 1));

        // Test non-existent gamepad/button
        assert!(!input_handler.is_gamepad_button_pressed(99, 0));
        assert!(!input_handler.is_gamepad_button_pressed(1, 99));
    }
}
