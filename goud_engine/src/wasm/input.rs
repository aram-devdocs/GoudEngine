//! Input handling methods for WasmGame.
//!
//! Provides key/mouse state queries and action mapping.

use wasm_bindgen::prelude::*;

use super::WasmGame;

// ---------------------------------------------------------------------------
// Input — setters (called from JS event handlers)
// ---------------------------------------------------------------------------

#[wasm_bindgen]
impl WasmGame {
    pub fn press_key(&mut self, key_code: u32) {
        self.keys_current.insert(key_code);
        self.keys_pressed_buffer.insert(key_code);
    }

    pub fn release_key(&mut self, key_code: u32) {
        self.keys_current.remove(&key_code);
        self.keys_released_buffer.insert(key_code);
    }

    pub fn press_mouse_button(&mut self, button: u32) {
        self.mouse_buttons_current.insert(button);
        self.mouse_pressed_buffer.insert(button);
    }

    pub fn release_mouse_button(&mut self, button: u32) {
        self.mouse_buttons_current.remove(&button);
        self.mouse_released_buffer.insert(button);
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn add_scroll_delta(&mut self, dx: f32, dy: f32) {
        self.scroll_dx += dx;
        self.scroll_dy += dy;
    }

    // ======================================================================
    // Input — queries (called from game logic)
    // ======================================================================

    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        self.keys_current.contains(&key_code)
    }

    pub fn is_key_just_pressed(&self, key_code: u32) -> bool {
        self.frame_keys_just_pressed.contains(&key_code)
    }

    pub fn is_key_just_released(&self, key_code: u32) -> bool {
        self.frame_keys_just_released.contains(&key_code)
    }

    pub fn is_mouse_button_pressed(&self, button: u32) -> bool {
        self.mouse_buttons_current.contains(&button)
    }

    pub fn is_mouse_button_just_pressed(&self, button: u32) -> bool {
        self.frame_mouse_just_pressed.contains(&button)
    }

    pub fn is_mouse_button_just_released(&self, button: u32) -> bool {
        self.frame_mouse_just_released.contains(&button)
    }

    pub fn mouse_x(&self) -> f32 {
        self.mouse_x
    }

    pub fn mouse_y(&self) -> f32 {
        self.mouse_y
    }

    pub fn scroll_dx(&self) -> f32 {
        self.scroll_dx
    }

    pub fn scroll_dy(&self) -> f32 {
        self.scroll_dy
    }

    // ======================================================================
    // Action mapping
    // ======================================================================

    /// Maps an action name to a key code. Multiple keys can be mapped
    /// to the same action. Returns `true` on success.
    pub fn map_action_key(&mut self, action: String, key: u32) -> bool {
        self.action_map.entry(action).or_default().push(key);
        true
    }

    /// Returns `true` if any key mapped to the given action is currently held.
    pub fn is_action_pressed(&self, action: String) -> bool {
        self.action_map
            .get(&action)
            .map(|keys| keys.iter().any(|k| self.keys_current.contains(k)))
            .unwrap_or(false)
    }

    /// Returns `true` if any key mapped to the given action was just pressed
    /// this frame (not held from previous frame).
    pub fn is_action_just_pressed(&self, action: String) -> bool {
        self.action_map
            .get(&action)
            .map(|keys| {
                keys.iter()
                    .any(|k| self.frame_keys_just_pressed.contains(k))
            })
            .unwrap_or(false)
    }

    /// Returns `true` if any key mapped to the given action was just released
    /// this frame (held previous frame but not current).
    pub fn is_action_just_released(&self, action: String) -> bool {
        self.action_map
            .get(&action)
            .map(|keys| {
                keys.iter()
                    .any(|k| self.frame_keys_just_released.contains(k))
            })
            .unwrap_or(false)
    }
}
