//! Gamepad input methods: buttons, analog axes, connection status, and vibration.

#[cfg(feature = "native")]
use glfw::GamepadAxis;

use crate::core::math::Vec2;

use super::manager::InputManager;
use super::types::{GamepadState, InputBinding};

impl InputManager {
    // === Gamepad Input ===

    /// Sets a gamepad button as pressed.
    ///
    /// # Panics
    /// Panics if gamepad_id >= 4.
    pub fn press_gamepad_button(&mut self, gamepad_id: usize, button: u32) {
        self.ensure_gamepad_capacity(gamepad_id);

        // Only buffer if this is a new press
        let is_new = !self.gamepad_buttons_current[gamepad_id].contains(&button);
        if is_new {
            self.buffer_input(InputBinding::GamepadButton { gamepad_id, button });
        }

        self.gamepad_buttons_current[gamepad_id].insert(button);
    }

    /// Sets a gamepad button as released.
    ///
    /// # Panics
    /// Panics if gamepad_id >= 4.
    pub fn release_gamepad_button(&mut self, gamepad_id: usize, button: u32) {
        self.ensure_gamepad_capacity(gamepad_id);
        self.gamepad_buttons_current[gamepad_id].remove(&button);
    }

    /// Returns true if the gamepad button is currently pressed.
    ///
    /// Returns false if gamepad_id is invalid.
    pub fn gamepad_button_pressed(&self, gamepad_id: usize, button: u32) -> bool {
        self.gamepad_buttons_current
            .get(gamepad_id)
            .is_some_and(|buttons| buttons.contains(&button))
    }

    /// Returns true if the gamepad button was just pressed this frame.
    pub fn gamepad_button_just_pressed(&self, gamepad_id: usize, button: u32) -> bool {
        let current = self
            .gamepad_buttons_current
            .get(gamepad_id)
            .is_some_and(|buttons| buttons.contains(&button));
        let previous = self
            .gamepad_buttons_previous
            .get(gamepad_id)
            .is_some_and(|buttons| buttons.contains(&button));
        current && !previous
    }

    /// Returns true if the gamepad button was just released this frame.
    pub fn gamepad_button_just_released(&self, gamepad_id: usize, button: u32) -> bool {
        let current = self
            .gamepad_buttons_current
            .get(gamepad_id)
            .is_some_and(|buttons| buttons.contains(&button));
        let previous = self
            .gamepad_buttons_previous
            .get(gamepad_id)
            .is_some_and(|buttons| buttons.contains(&button));
        !current && previous
    }

    /// Ensures gamepad capacity for the given ID.
    pub(super) fn ensure_gamepad_capacity(&mut self, gamepad_id: usize) {
        while self.gamepad_buttons_current.len() <= gamepad_id {
            self.gamepad_buttons_current
                .push(std::collections::HashSet::new());
        }
        while self.gamepad_buttons_previous.len() <= gamepad_id {
            self.gamepad_buttons_previous
                .push(std::collections::HashSet::new());
        }
        while self.gamepads.len() <= gamepad_id {
            self.gamepads.push(GamepadState::new());
        }
        while self.gamepads_previous.len() <= gamepad_id {
            self.gamepads_previous.push(GamepadState::new());
        }
    }

    // === Gamepad Analog Axes ===

    /// Sets a gamepad analog axis value (-1.0 to 1.0).
    ///
    /// Values within the deadzone threshold are clamped to 0.0.
    pub fn set_gamepad_axis(&mut self, gamepad_id: usize, axis: GamepadAxis, value: f32) {
        self.ensure_gamepad_capacity(gamepad_id);

        // Apply deadzone
        let deadzone_value = if value.abs() < self.analog_deadzone {
            0.0
        } else {
            value
        };

        self.gamepads[gamepad_id].axes.insert(axis, deadzone_value);
    }

    /// Returns the current value of a gamepad analog axis (-1.0 to 1.0).
    ///
    /// Returns 0.0 if the gamepad or axis doesn't exist.
    pub fn gamepad_axis(&self, gamepad_id: usize, axis: GamepadAxis) -> f32 {
        self.gamepads
            .get(gamepad_id)
            .and_then(|gamepad| gamepad.axes.get(&axis).copied())
            .unwrap_or(0.0)
    }

    /// Returns the left stick as a Vec2 (x, y) where -1.0 is left/down and 1.0 is right/up.
    ///
    /// Returns Vec2::zero() if the gamepad doesn't exist.
    pub fn gamepad_left_stick(&self, gamepad_id: usize) -> Vec2 {
        Vec2::new(
            self.gamepad_axis(gamepad_id, GamepadAxis::AxisLeftX),
            self.gamepad_axis(gamepad_id, GamepadAxis::AxisLeftY),
        )
    }

    /// Returns the right stick as a Vec2 (x, y) where -1.0 is left/down and 1.0 is right/up.
    ///
    /// Returns Vec2::zero() if the gamepad doesn't exist.
    pub fn gamepad_right_stick(&self, gamepad_id: usize) -> Vec2 {
        Vec2::new(
            self.gamepad_axis(gamepad_id, GamepadAxis::AxisRightX),
            self.gamepad_axis(gamepad_id, GamepadAxis::AxisRightY),
        )
    }

    /// Returns the left trigger value (0.0 to 1.0).
    ///
    /// Returns 0.0 if the gamepad doesn't exist.
    pub fn gamepad_left_trigger(&self, gamepad_id: usize) -> f32 {
        // Left trigger is mapped to axis 2, normalized from -1.0..1.0 to 0.0..1.0
        (self.gamepad_axis(gamepad_id, GamepadAxis::AxisLeftTrigger) + 1.0) * 0.5
    }

    /// Returns the right trigger value (0.0 to 1.0).
    ///
    /// Returns 0.0 if the gamepad doesn't exist.
    pub fn gamepad_right_trigger(&self, gamepad_id: usize) -> f32 {
        // Right trigger is mapped to axis 3, normalized from -1.0..1.0 to 0.0..1.0
        (self.gamepad_axis(gamepad_id, GamepadAxis::AxisRightTrigger) + 1.0) * 0.5
    }

    // === Gamepad Connection ===

    /// Sets the connection status of a gamepad.
    pub fn set_gamepad_connected(&mut self, gamepad_id: usize, connected: bool) {
        self.ensure_gamepad_capacity(gamepad_id);
        self.gamepads[gamepad_id].connected = connected;
    }

    /// Returns true if the gamepad is currently connected.
    pub fn is_gamepad_connected(&self, gamepad_id: usize) -> bool {
        self.gamepads
            .get(gamepad_id)
            .map(|gamepad| gamepad.connected)
            .unwrap_or(false)
    }

    /// Returns the number of connected gamepads.
    pub fn connected_gamepad_count(&self) -> usize {
        self.gamepads
            .iter()
            .filter(|gamepad| gamepad.connected)
            .count()
    }

    /// Returns an iterator over all connected gamepad IDs.
    pub fn connected_gamepads(&self) -> impl Iterator<Item = usize> + '_ {
        self.gamepads
            .iter()
            .enumerate()
            .filter_map(|(id, gamepad)| if gamepad.connected { Some(id) } else { None })
    }

    // === Gamepad Vibration ===

    /// Sets the vibration intensity for a gamepad (0.0-1.0).
    ///
    /// Note: Actual vibration must be implemented by the platform layer.
    /// This only tracks the requested vibration intensity.
    pub fn set_gamepad_vibration(&mut self, gamepad_id: usize, intensity: f32) {
        self.ensure_gamepad_capacity(gamepad_id);
        self.gamepads[gamepad_id].vibration = intensity.clamp(0.0, 1.0);
    }

    /// Returns the current vibration intensity for a gamepad (0.0-1.0).
    pub fn gamepad_vibration(&self, gamepad_id: usize) -> f32 {
        self.gamepads
            .get(gamepad_id)
            .map(|gamepad| gamepad.vibration)
            .unwrap_or(0.0)
    }

    /// Stops vibration for a gamepad (sets intensity to 0.0).
    pub fn stop_gamepad_vibration(&mut self, gamepad_id: usize) {
        self.set_gamepad_vibration(gamepad_id, 0.0);
    }

    /// Stops vibration for all gamepads.
    pub fn stop_all_vibration(&mut self) {
        for gamepad in &mut self.gamepads {
            gamepad.vibration = 0.0;
        }
    }

    // === Analog Deadzone ===

    /// Returns the current analog deadzone threshold (0.0-1.0).
    ///
    /// Default is 0.1 (10%).
    pub fn analog_deadzone(&self) -> f32 {
        self.analog_deadzone
    }

    /// Sets the analog deadzone threshold (0.0-1.0).
    ///
    /// Analog axis values within this threshold are clamped to 0.0.
    /// This prevents stick drift and accidental input.
    pub fn set_analog_deadzone(&mut self, deadzone: f32) {
        self.analog_deadzone = deadzone.clamp(0.0, 1.0);
    }
}
