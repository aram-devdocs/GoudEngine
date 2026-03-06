//! Input provider trait definition.
//!
//! The `InputProvider` trait abstracts input handling, enabling runtime
//! selection between GLFW-based input, synthetic test input, or null (no-op).
//!
//! Separated from `WindowProvider` because input has a different update
//! cadence and can be mocked without a real window (e.g., test harnesses
//! injecting synthetic events).

use super::types::{
    GamepadAxis, GamepadButton, GamepadId, InputCapabilities, KeyCode, MouseButton,
};
use super::Provider;
use crate::core::error::GoudResult;

/// Trait for input backends.
///
/// Uses platform-independent enum types defined in `types.rs` rather than
/// GLFW-specific types, allowing input providers to work without a windowing
/// dependency.
///
/// The trait is object-safe and stored as `Box<dyn InputProvider>`.
pub trait InputProvider: Provider {
    /// Returns the typed input capabilities for this provider.
    fn input_capabilities(&self) -> &InputCapabilities;

    /// Per-frame input update. Processes queued events and updates state.
    fn update_input(&mut self) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Keyboard
    // -------------------------------------------------------------------------

    /// Returns true if the key is currently held down.
    fn key_pressed(&self, key: KeyCode) -> bool;

    /// Returns true if the key was pressed this frame (not held from previous).
    fn key_just_pressed(&self, key: KeyCode) -> bool;

    /// Returns true if the key was released this frame.
    fn key_just_released(&self, key: KeyCode) -> bool;

    // -------------------------------------------------------------------------
    // Mouse
    // -------------------------------------------------------------------------

    /// Returns the current mouse position as [x, y] in window coordinates.
    fn mouse_position(&self) -> [f32; 2];

    /// Returns the mouse movement delta since the last frame as [dx, dy].
    fn mouse_delta(&self) -> [f32; 2];

    /// Returns true if the mouse button is currently held down.
    fn mouse_button_pressed(&self, button: MouseButton) -> bool;

    /// Returns the scroll wheel delta since the last frame as [dx, dy].
    fn scroll_delta(&self) -> [f32; 2];

    // -------------------------------------------------------------------------
    // Gamepad
    // -------------------------------------------------------------------------

    /// Returns true if a gamepad with the given ID is connected.
    fn gamepad_connected(&self, id: GamepadId) -> bool;

    /// Returns the value of a gamepad axis (-1.0 to 1.0).
    fn gamepad_axis(&self, id: GamepadId, axis: GamepadAxis) -> f32;

    /// Returns true if a gamepad button is currently held down.
    fn gamepad_button_pressed(&self, id: GamepadId, button: GamepadButton) -> bool;
}
