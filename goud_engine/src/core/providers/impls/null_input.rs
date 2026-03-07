//! Null input provider -- silent no-op for headless testing.

use crate::core::error::GoudResult;
use crate::core::providers::input::InputProvider;
use crate::core::providers::types::{
    GamepadAxis, GamepadButton, GamepadId, InputCapabilities, KeyCode, MouseButton,
};
use crate::core::providers::Provider;

/// An input provider that does nothing. Used for headless testing and as
/// a default when no input system is available.
pub struct NullInputProvider {
    capabilities: InputCapabilities,
}

impl NullInputProvider {
    /// Create a new null input provider.
    pub fn new() -> Self {
        Self {
            capabilities: InputCapabilities {
                supports_gamepad: false,
                supports_touch: false,
                max_gamepads: 0,
            },
        }
    }
}

impl Default for NullInputProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NullInputProvider {
    fn name(&self) -> &str {
        "null"
    }

    fn version(&self) -> &str {
        "0.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl InputProvider for NullInputProvider {
    fn input_capabilities(&self) -> &InputCapabilities {
        &self.capabilities
    }

    fn update_input(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn key_pressed(&self, _key: KeyCode) -> bool {
        false
    }

    fn key_just_pressed(&self, _key: KeyCode) -> bool {
        false
    }

    fn key_just_released(&self, _key: KeyCode) -> bool {
        false
    }

    fn mouse_position(&self) -> [f32; 2] {
        [0.0, 0.0]
    }

    fn mouse_delta(&self) -> [f32; 2] {
        [0.0, 0.0]
    }

    fn mouse_button_pressed(&self, _button: MouseButton) -> bool {
        false
    }

    fn scroll_delta(&self) -> [f32; 2] {
        [0.0, 0.0]
    }

    fn gamepad_connected(&self, _id: GamepadId) -> bool {
        false
    }

    fn gamepad_axis(&self, _id: GamepadId, _axis: GamepadAxis) -> f32 {
        0.0
    }

    fn gamepad_button_pressed(&self, _id: GamepadId, _button: GamepadButton) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_input_construction() {
        let provider = NullInputProvider::new();
        assert_eq!(provider.name(), "null");
        assert_eq!(provider.version(), "0.0.0");
    }

    #[test]
    fn test_null_input_default() {
        let provider = NullInputProvider::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_input_capabilities() {
        let provider = NullInputProvider::new();
        let caps = provider.input_capabilities();
        assert!(!caps.supports_gamepad);
        assert!(!caps.supports_touch);
        assert_eq!(caps.max_gamepads, 0);
    }

    #[test]
    fn test_null_input_update() {
        let mut provider = NullInputProvider::new();
        assert!(provider.update_input().is_ok());
    }

    #[test]
    fn test_null_input_keyboard() {
        let provider = NullInputProvider::new();
        assert!(!provider.key_pressed(KeyCode::Space));
        assert!(!provider.key_just_pressed(KeyCode::Enter));
        assert!(!provider.key_just_released(KeyCode::Escape));
    }

    #[test]
    fn test_null_input_mouse() {
        let provider = NullInputProvider::new();
        assert_eq!(provider.mouse_position(), [0.0, 0.0]);
        assert_eq!(provider.mouse_delta(), [0.0, 0.0]);
        assert!(!provider.mouse_button_pressed(MouseButton::Left));
        assert_eq!(provider.scroll_delta(), [0.0, 0.0]);
    }

    #[test]
    fn test_null_input_gamepad() {
        let provider = NullInputProvider::new();
        let id = GamepadId(0);
        assert!(!provider.gamepad_connected(id));
        assert_eq!(provider.gamepad_axis(id, GamepadAxis::LeftStickX), 0.0);
        assert!(!provider.gamepad_button_pressed(id, GamepadButton::South));
    }
}
