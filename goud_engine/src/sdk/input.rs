//! # SDK Input API
//!
//! Provides methods on [`GoudGame`](super::GoudGame) for querying input state:
//! keyboard, mouse, and action mapping.
//!
//! The input system is built on the [`InputManager`] ECS resource. The SDK
//! wraps it so game code can query input without reaching into the ECS layer.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop platform with GLFW).
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::sdk::GoudGame;
//! use glfw::Key;
//!
//! let mut game = GoudGame::new(Default::default()).unwrap();
//!
//! // In your game loop:
//! if game.is_key_pressed(Key::W) {
//!     // Move forward
//! }
//! if game.is_key_just_pressed(Key::Space) {
//!     // Jump (once per press)
//! }
//!
//! let (mx, my) = game.mouse_position();
//! let scroll = game.scroll_delta();
//! ```

use super::GoudGame;
use crate::ecs::InputManager;

// Re-export input types for SDK users so callers do not need to depend
// on `glfw` or reach into `ecs::` directly.
pub use crate::ecs::InputBinding;
pub use glfw::{Key, MouseButton};

// =============================================================================
// Input API (annotated for FFI generation)
// =============================================================================

// NOTE: FFI wrappers are hand-written in ffi/input.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl GoudGame {
    // =========================================================================
    // Keyboard Input
    // =========================================================================

    /// Returns `true` if the given key is currently held down.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if game.is_key_pressed(Key::W) {
    ///     // Move forward continuously while held
    /// }
    /// ```
    #[inline]
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_manager.key_pressed(key)
    }

    /// Returns `true` if the given key was pressed this frame (not held from previous frame).
    ///
    /// Use this for one-shot actions like jumping or menu selection.
    #[inline]
    pub fn is_key_just_pressed(&self, key: Key) -> bool {
        self.input_manager.key_just_pressed(key)
    }

    /// Returns `true` if the given key was released this frame.
    #[inline]
    pub fn is_key_just_released(&self, key: Key) -> bool {
        self.input_manager.key_just_released(key)
    }

    // =========================================================================
    // Mouse Input
    // =========================================================================

    /// Returns `true` if the given mouse button is currently held down.
    #[inline]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.input_manager.mouse_button_pressed(button)
    }

    /// Returns `true` if the given mouse button was pressed this frame.
    #[inline]
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.input_manager.mouse_button_just_pressed(button)
    }

    /// Returns `true` if the given mouse button was released this frame.
    #[inline]
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.input_manager.mouse_button_just_released(button)
    }

    /// Returns the current mouse position in window coordinates.
    ///
    /// Returns `(x, y)` where `(0, 0)` is the top-left corner.
    #[inline]
    pub fn mouse_position(&self) -> (f32, f32) {
        let pos = self.input_manager.mouse_position();
        (pos.x, pos.y)
    }

    /// Returns the mouse movement delta since the last frame.
    ///
    /// Returns `(dx, dy)` where positive X is right and positive Y is down.
    #[inline]
    pub fn mouse_delta(&self) -> (f32, f32) {
        let delta = self.input_manager.mouse_delta();
        (delta.x, delta.y)
    }

    /// Returns the scroll wheel delta since the last frame.
    ///
    /// Returns `(horizontal, vertical)` scroll amounts.
    #[inline]
    pub fn scroll_delta(&self) -> (f32, f32) {
        let delta = self.input_manager.scroll_delta();
        (delta.x, delta.y)
    }

    // =========================================================================
    // Action Mapping
    // =========================================================================

    /// Maps a key to a named action.
    ///
    /// An action can have multiple key bindings. When any bound key is pressed,
    /// the action is considered active.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// game.map_action_key("Jump", Key::Space);
    /// game.map_action_key("Jump", Key::W);
    ///
    /// if game.is_action_pressed("Jump") {
    ///     // Triggered by Space OR W
    /// }
    /// ```
    #[inline]
    pub fn map_action_key(&mut self, action: &str, key: Key) {
        self.input_manager
            .map_action(action, InputBinding::Key(key));
    }

    /// Maps a mouse button to a named action.
    #[inline]
    pub fn map_action_mouse_button(&mut self, action: &str, button: MouseButton) {
        self.input_manager
            .map_action(action, InputBinding::MouseButton(button));
    }

    /// Returns `true` if any binding for the named action is currently held.
    #[inline]
    pub fn is_action_pressed(&self, action: &str) -> bool {
        self.input_manager.action_pressed(action)
    }

    /// Returns `true` if any binding for the named action was pressed this frame.
    #[inline]
    pub fn is_action_just_pressed(&self, action: &str) -> bool {
        self.input_manager.action_just_pressed(action)
    }

    /// Returns `true` if any binding for the named action was released this frame.
    #[inline]
    pub fn is_action_just_released(&self, action: &str) -> bool {
        self.input_manager.action_just_released(action)
    }

    // =========================================================================
    // Direct InputManager Access
    // =========================================================================

    /// Returns a reference to the underlying [`InputManager`].
    ///
    /// Use this for advanced input queries (gamepad, input buffering, etc.)
    /// that are not exposed through convenience methods.
    #[inline]
    pub fn input(&self) -> &InputManager {
        &self.input_manager
    }

    /// Returns a mutable reference to the underlying [`InputManager`].
    ///
    /// Use this for advanced configuration (deadzone, buffer duration, etc.).
    #[inline]
    pub fn input_mut(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::GameConfig;

    #[test]
    fn test_input_manager_accessible() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        // InputManager should be initialized and accessible
        let _input = game.input();
    }

    #[test]
    fn test_input_manager_mutable() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let input = game.input_mut();
        // Should be able to configure the input manager
        input.set_analog_deadzone(0.2);
        assert!((input.analog_deadzone() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_key_not_pressed_by_default() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.is_key_pressed(Key::Space));
        assert!(!game.is_key_just_pressed(Key::Space));
        assert!(!game.is_key_just_released(Key::Space));
    }

    #[test]
    fn test_mouse_button_not_pressed_by_default() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.is_mouse_button_pressed(MouseButton::Button1));
        assert!(!game.is_mouse_button_just_pressed(MouseButton::Button1));
    }

    #[test]
    fn test_mouse_position_default() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        let (x, y) = game.mouse_position();
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_action_mapping() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        game.map_action_key("Jump", Key::Space);
        // Action exists but key is not pressed
        assert!(!game.is_action_pressed("Jump"));
    }

    #[test]
    fn test_key_press_and_query() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        // Simulate a key press through the input manager
        game.input_mut().press_key(Key::W);
        assert!(game.is_key_pressed(Key::W));
    }

    #[test]
    fn test_scroll_delta_default() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        let (sx, sy) = game.scroll_delta();
        assert!((sx - 0.0).abs() < 0.001);
        assert!((sy - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_mouse_delta_default() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        let (dx, dy) = game.mouse_delta();
        assert!((dx - 0.0).abs() < 0.001);
        assert!((dy - 0.0).abs() < 0.001);
    }
}
