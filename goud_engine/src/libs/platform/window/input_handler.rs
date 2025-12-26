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
        let buttons = self.gamepad_buttons_pressed.entry(gamepad_id).or_default();
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
            .is_some_and(|buttons| buttons.contains(&button))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};

    #[test]
    fn test_new_input_handler() {
        let input_handler = InputHandler::new();
        assert!(input_handler.keys_pressed.is_empty());
        assert!(input_handler.mouse_buttons_pressed.is_empty());
        assert!(input_handler.gamepad_buttons_pressed.is_empty());
        assert_eq!(input_handler.mouse_position.x, 0.0);
        assert_eq!(input_handler.mouse_position.y, 0.0);
    }

    #[test]
    fn test_key_press_and_release() {
        let mut input_handler = InputHandler::new();

        // Test key press
        let key_press_event = WindowEvent::Key(Key::A, 0, Action::Press, Modifiers::empty());
        input_handler.handle_event(&key_press_event);
        assert!(input_handler.is_key_pressed(Key::A));

        // Test key release
        let key_release_event = WindowEvent::Key(Key::A, 0, Action::Release, Modifiers::empty());
        input_handler.handle_event(&key_release_event);
        assert!(!input_handler.is_key_pressed(Key::A));
    }

    #[test]
    fn test_mouse_button_press_and_release() {
        let mut input_handler = InputHandler::new();

        // Test mouse button press
        let mouse_press_event =
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, Modifiers::empty());
        input_handler.handle_event(&mouse_press_event);
        assert!(input_handler.is_mouse_button_pressed(MouseButton::Button1));

        // Test mouse button release
        let mouse_release_event =
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, Modifiers::empty());
        input_handler.handle_event(&mouse_release_event);
        assert!(!input_handler.is_mouse_button_pressed(MouseButton::Button1));
    }

    #[test]
    fn test_mouse_position_update() {
        let mut input_handler = InputHandler::new();

        // Test mouse position update
        let mouse_pos_event = WindowEvent::CursorPos(15.5, 20.0);
        input_handler.handle_event(&mouse_pos_event);

        let position = input_handler.get_mouse_position();
        assert_eq!(position.x, 15.5);
        assert_eq!(position.y, 20.0);
    }

    #[test]
    fn test_gamepad_button_handling() {
        let mut input_handler = InputHandler::new();

        // Test gamepad button press
        input_handler.handle_gamepad_button(0, 1, true);
        assert!(input_handler.is_gamepad_button_pressed(0, 1));

        // Test another gamepad button press
        input_handler.handle_gamepad_button(0, 2, true);
        assert!(input_handler.is_gamepad_button_pressed(0, 2));

        // Test gamepad button release
        input_handler.handle_gamepad_button(0, 1, false);
        assert!(!input_handler.is_gamepad_button_pressed(0, 1));
        assert!(input_handler.is_gamepad_button_pressed(0, 2));

        // Test different gamepad
        input_handler.handle_gamepad_button(1, 1, true);
        assert!(input_handler.is_gamepad_button_pressed(1, 1));
        assert!(!input_handler.is_gamepad_button_pressed(1, 2));
    }
}
