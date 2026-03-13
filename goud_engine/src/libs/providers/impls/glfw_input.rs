//! GLFW input provider -- Layer 1 input provider with state synced from
//! the Layer 2 `InputManager`.
//!
//! This provider maintains its own input state (key/mouse/scroll maps)
//! and exposes a `sync_from_input_manager` method that Layer 2 code
//! (e.g., `GoudGame`) calls each frame to copy state from the
//! `InputManager`. This avoids a Layer 1 -> Layer 2 import violation.

use std::collections::HashSet;

use crate::core::providers::diagnostics::InputDiagnosticsV1;
use crate::libs::error::GoudResult;
use crate::libs::providers::input::InputProvider;
use crate::libs::providers::types::{
    GamepadAxis, GamepadButton, GamepadId, InputCapabilities, KeyCode, MouseButton,
};
use crate::libs::providers::Provider;

/// GLFW-based input provider.
///
/// Stores a snapshot of input state that is updated each frame via
/// [`sync`](GlfwInputProvider::sync). The provider translates between
/// the platform-independent [`KeyCode`]/[`MouseButton`] enums and the
/// internal representation.
pub struct GlfwInputProvider {
    capabilities: InputCapabilities,

    // Keyboard state (stored as KeyCode discriminant values)
    keys_current: HashSet<u32>,
    keys_previous: HashSet<u32>,

    // Mouse state
    mouse_buttons_current: HashSet<u32>,
    mouse_buttons_previous: HashSet<u32>,
    mouse_position: [f32; 2],
    mouse_delta: [f32; 2],
    scroll_delta: [f32; 2],
}

impl GlfwInputProvider {
    /// Creates a new GLFW input provider with no inputs pressed.
    pub fn new() -> Self {
        Self {
            capabilities: InputCapabilities {
                supports_gamepad: true,
                supports_touch: false,
                max_gamepads: 4,
            },
            keys_current: HashSet::new(),
            keys_previous: HashSet::new(),
            mouse_buttons_current: HashSet::new(),
            mouse_buttons_previous: HashSet::new(),
            mouse_position: [0.0, 0.0],
            mouse_delta: [0.0, 0.0],
            scroll_delta: [0.0, 0.0],
        }
    }

    /// Synchronizes provider state from raw input data.
    ///
    /// Called by Layer 2 code each frame after `InputManager::update()`.
    /// The caller extracts the relevant data from `InputManager` and
    /// passes it here as plain types, avoiding a direct dependency on
    /// the `InputManager` type from this Layer 1 module.
    pub fn sync(
        &mut self,
        pressed_keys: &[u32],
        pressed_mouse_buttons: &[u32],
        mouse_pos: [f32; 2],
        mouse_delta: [f32; 2],
        scroll_delta: [f32; 2],
    ) {
        // Rotate current -> previous
        std::mem::swap(&mut self.keys_previous, &mut self.keys_current);
        self.keys_current.clear();
        self.keys_current.extend(pressed_keys);

        std::mem::swap(
            &mut self.mouse_buttons_previous,
            &mut self.mouse_buttons_current,
        );
        self.mouse_buttons_current.clear();
        self.mouse_buttons_current.extend(pressed_mouse_buttons);

        self.mouse_position = mouse_pos;
        self.mouse_delta = mouse_delta;
        self.scroll_delta = scroll_delta;
    }
}

impl Default for GlfwInputProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for GlfwInputProvider {
    fn name(&self) -> &str {
        "glfw"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl InputProvider for GlfwInputProvider {
    fn input_capabilities(&self) -> &InputCapabilities {
        &self.capabilities
    }

    fn update_input(&mut self) -> GoudResult<()> {
        // State is updated via sync() called by the game loop.
        Ok(())
    }

    fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_current.contains(&(key as u32))
    }

    fn key_just_pressed(&self, key: KeyCode) -> bool {
        let code = key as u32;
        self.keys_current.contains(&code) && !self.keys_previous.contains(&code)
    }

    fn key_just_released(&self, key: KeyCode) -> bool {
        let code = key as u32;
        !self.keys_current.contains(&code) && self.keys_previous.contains(&code)
    }

    fn mouse_position(&self) -> [f32; 2] {
        self.mouse_position
    }

    fn mouse_delta(&self) -> [f32; 2] {
        self.mouse_delta
    }

    fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_current.contains(&(button as u32))
    }

    fn scroll_delta(&self) -> [f32; 2] {
        self.scroll_delta
    }

    fn gamepad_connected(&self, _id: GamepadId) -> bool {
        // Gamepad infrastructure is not yet wired through the provider
        // layer. Returns false until gamepad support is added.
        false
    }

    fn gamepad_axis(&self, _id: GamepadId, _axis: GamepadAxis) -> f32 {
        0.0
    }

    fn gamepad_button_pressed(&self, _id: GamepadId, _button: GamepadButton) -> bool {
        false
    }

    fn input_diagnostics(&self) -> InputDiagnosticsV1 {
        InputDiagnosticsV1 {
            pressed_keys: self.keys_current.iter().map(|k| format!("{}", k)).collect(),
            mouse_position: self.mouse_position,
            mouse_buttons_pressed: self
                .mouse_buttons_current
                .iter()
                .map(|b| format!("{}", b))
                .collect(),
            connected_gamepads: 0,
            scroll_delta: self.scroll_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glfw_input_construction() {
        let provider = GlfwInputProvider::new();
        assert_eq!(provider.name(), "glfw");
        assert_eq!(provider.version(), "1.0.0");
    }

    #[test]
    fn test_glfw_input_default() {
        let provider = GlfwInputProvider::default();
        assert_eq!(provider.name(), "glfw");
    }

    #[test]
    fn test_glfw_input_capabilities() {
        let provider = GlfwInputProvider::new();
        let caps = provider.input_capabilities();
        assert!(caps.supports_gamepad);
        assert!(!caps.supports_touch);
        assert_eq!(caps.max_gamepads, 4);
    }

    #[test]
    fn test_glfw_input_default_state() {
        let provider = GlfwInputProvider::new();
        assert!(!provider.key_pressed(KeyCode::Space));
        assert!(!provider.key_just_pressed(KeyCode::A));
        assert!(!provider.key_just_released(KeyCode::Escape));
        assert_eq!(provider.mouse_position(), [0.0, 0.0]);
        assert_eq!(provider.mouse_delta(), [0.0, 0.0]);
        assert!(!provider.mouse_button_pressed(MouseButton::Left));
        assert_eq!(provider.scroll_delta(), [0.0, 0.0]);
    }

    #[test]
    fn test_glfw_input_sync_key_pressed() {
        let mut provider = GlfwInputProvider::new();

        // Sync with Space pressed
        provider.sync(
            &[KeyCode::Space as u32],
            &[],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        );

        assert!(provider.key_pressed(KeyCode::Space));
        assert!(provider.key_just_pressed(KeyCode::Space));
        assert!(!provider.key_just_released(KeyCode::Space));
    }

    #[test]
    fn test_glfw_input_sync_key_held() {
        let mut provider = GlfwInputProvider::new();

        // Frame 1: press Space
        provider.sync(
            &[KeyCode::Space as u32],
            &[],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        );

        // Frame 2: Space still held
        provider.sync(
            &[KeyCode::Space as u32],
            &[],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        );

        assert!(provider.key_pressed(KeyCode::Space));
        assert!(!provider.key_just_pressed(KeyCode::Space));
    }

    #[test]
    fn test_glfw_input_sync_key_released() {
        let mut provider = GlfwInputProvider::new();

        // Frame 1: press Space
        provider.sync(
            &[KeyCode::Space as u32],
            &[],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        );

        // Frame 2: release Space
        provider.sync(&[], &[], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]);

        assert!(!provider.key_pressed(KeyCode::Space));
        assert!(provider.key_just_released(KeyCode::Space));
    }

    #[test]
    fn test_glfw_input_sync_mouse() {
        let mut provider = GlfwInputProvider::new();

        provider.sync(
            &[],
            &[MouseButton::Left as u32],
            [100.0, 200.0],
            [5.0, -3.0],
            [0.0, 1.0],
        );

        assert!(provider.mouse_button_pressed(MouseButton::Left));
        assert!(!provider.mouse_button_pressed(MouseButton::Right));
        assert_eq!(provider.mouse_position(), [100.0, 200.0]);
        assert_eq!(provider.mouse_delta(), [5.0, -3.0]);
        assert_eq!(provider.scroll_delta(), [0.0, 1.0]);
    }

    #[test]
    fn test_glfw_input_update_noop() {
        let mut provider = GlfwInputProvider::new();
        assert!(provider.update_input().is_ok());
    }

    #[test]
    fn test_glfw_input_gamepad_stubs() {
        let provider = GlfwInputProvider::new();
        let id = GamepadId(0);
        assert!(!provider.gamepad_connected(id));
        assert_eq!(provider.gamepad_axis(id, GamepadAxis::LeftStickX), 0.0);
        assert!(!provider.gamepad_button_pressed(id, GamepadButton::South));
    }

    #[test]
    fn test_glfw_input_generic_capabilities() {
        let provider = GlfwInputProvider::new();
        let caps = provider.capabilities();
        let input_caps = caps.downcast_ref::<InputCapabilities>().unwrap();
        assert!(input_caps.supports_gamepad);
    }
}
