//! Shared input types: `GamepadState`, `BufferedInput`, `InputBinding`, and `TouchState`.

use crate::core::math::Vec2;
use crate::core::providers::input_types::{GamepadAxis, KeyCode as Key, MouseButton, TouchPhase};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use super::manager::InputManager;

/// Gamepad state for a single controller.
///
/// Tracks buttons, axes (analog sticks, triggers), connection status, and vibration.
#[derive(Debug, Clone)]
pub(super) struct GamepadState {
    /// Currently pressed buttons using the engine's neutral button indices.
    pub(super) buttons: HashSet<u32>,
    /// Analog axis values (-1.0 to 1.0)
    pub(super) axes: HashMap<GamepadAxis, f32>,
    /// Whether this gamepad is currently connected
    pub(super) connected: bool,
    /// Rumble/vibration intensity (0.0-1.0)
    pub(super) vibration: f32,
}

impl GamepadState {
    pub(super) fn new() -> Self {
        Self {
            buttons: HashSet::new(),
            axes: HashMap::new(),
            connected: false,
            vibration: 0.0,
        }
    }
}

/// Represents a single input binding that can be mapped to an action.
///
/// An action can have multiple bindings, allowing for keyboard, mouse, and gamepad
/// inputs to all trigger the same action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    /// A keyboard key.
    Key(Key),
    /// A mouse button.
    MouseButton(MouseButton),
    /// A gamepad button for a specific gamepad.
    GamepadButton {
        /// The ID of the gamepad (0-indexed).
        gamepad_id: usize,
        /// The button index, following platform conventions.
        button: u32,
    },
}

impl InputBinding {
    /// Returns true if this binding is currently pressed.
    pub fn is_pressed(&self, input: &InputManager) -> bool {
        match self {
            InputBinding::Key(key) => input.key_pressed(*key),
            InputBinding::MouseButton(button) => input.mouse_button_pressed(*button),
            InputBinding::GamepadButton { gamepad_id, button } => {
                input.gamepad_button_pressed(*gamepad_id, *button)
            }
        }
    }

    /// Returns true if this binding was just pressed this frame.
    pub fn is_just_pressed(&self, input: &InputManager) -> bool {
        match self {
            InputBinding::Key(key) => input.key_just_pressed(*key),
            InputBinding::MouseButton(button) => input.mouse_button_just_pressed(*button),
            InputBinding::GamepadButton { gamepad_id, button } => {
                input.gamepad_button_just_pressed(*gamepad_id, *button)
            }
        }
    }

    /// Returns true if this binding was just released this frame.
    pub fn is_just_released(&self, input: &InputManager) -> bool {
        match self {
            InputBinding::Key(key) => input.key_just_released(*key),
            InputBinding::MouseButton(button) => input.mouse_button_just_released(*button),
            InputBinding::GamepadButton { gamepad_id, button } => {
                input.gamepad_button_just_released(*gamepad_id, *button)
            }
        }
    }
}

impl std::fmt::Display for InputBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputBinding::Key(key) => write!(f, "Key({:?})", key),
            InputBinding::MouseButton(button) => write!(f, "MouseButton({:?})", button),
            InputBinding::GamepadButton { gamepad_id, button } => {
                write!(
                    f,
                    "GamepadButton(gamepad={}, button={})",
                    gamepad_id, button
                )
            }
        }
    }
}

/// Represents a buffered input event with a timestamp.
///
/// Used for detecting input sequences and combos within a time window.
#[derive(Debug, Clone)]
pub(super) struct BufferedInput {
    /// The input binding that was pressed
    pub(super) binding: InputBinding,
    /// When the input was pressed
    pub(super) timestamp: Instant,
}

impl BufferedInput {
    /// Creates a new buffered input.
    pub(super) fn new(binding: InputBinding, timestamp: Instant) -> Self {
        Self { binding, timestamp }
    }

    /// Returns the age of this input in seconds.
    pub(super) fn age(&self, now: Instant) -> f32 {
        now.duration_since(self.timestamp).as_secs_f32()
    }

    /// Returns true if this input has expired given the buffer duration.
    pub(super) fn is_expired(&self, now: Instant, buffer_duration: Duration) -> bool {
        now.duration_since(self.timestamp) > buffer_duration
    }
}

/// Internal state for a single active touch point.
#[derive(Debug, Clone)]
pub(super) struct TouchState {
    pub(super) position: Vec2,
    pub(super) previous_position: Vec2,
    pub(super) phase: TouchPhase,
}
