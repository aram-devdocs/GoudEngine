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
            .map_or(false, |buttons| buttons.contains(&button))
    }
}
