//! Core `InputManager` struct: construction, frame update, keyboard, mouse, and clear.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

use crate::core::debugger::{self, SyntheticInputEventV1};
use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode as Key, MouseButton, TouchPhase};

use super::types::{BufferedInput, GamepadState, InputBinding, TouchState};

/// Input management resource for ECS integration.
///
/// Tracks keyboard, mouse, and gamepad input state across frames,
/// enabling queries for current state, just pressed, and just released.
/// Also supports action mapping for semantic input handling.
#[derive(Debug, Clone)]
pub struct InputManager {
    // Current frame state
    pub(super) keys_current: HashSet<Key>,
    pub(super) mouse_buttons_current: HashSet<MouseButton>,
    pub(super) consumed_keys: HashSet<Key>,
    pub(super) consumed_mouse_buttons: HashSet<MouseButton>,
    /// Deprecated — use `gamepads` for new code.
    pub(super) gamepad_buttons_current: Vec<HashSet<u32>>,
    pub(super) mouse_position: Vec2,
    pub(super) mouse_delta: Vec2,

    // Previous frame state (for just_pressed/just_released detection)
    pub(super) keys_previous: HashSet<Key>,
    pub(super) mouse_buttons_previous: HashSet<MouseButton>,
    /// Deprecated — use `gamepads_previous` for new code.
    pub(super) gamepad_buttons_previous: Vec<HashSet<u32>>,

    // Gamepad state (current and previous)
    pub(super) gamepads: Vec<GamepadState>,
    pub(super) gamepads_previous: Vec<GamepadState>,

    // Mouse scroll
    pub(super) scroll_delta: Vec2,

    // Action mappings (action_name -> list of bindings)
    pub(super) action_mappings: HashMap<String, Vec<InputBinding>>,

    // Input buffering for sequences and combos
    pub(super) input_buffer: VecDeque<BufferedInput>,
    pub(super) buffer_duration: Duration,
    pub(super) last_update: Instant,

    // Analog deadzone threshold (default 0.1)
    pub(super) analog_deadzone: f32,

    // Touch input state
    pub(super) touches_current: HashMap<u64, TouchState>,
    pub(super) touches_previous: HashMap<u64, TouchState>,
    pub(super) touch_pointer_emulation: bool,
}

impl InputManager {
    /// Creates a new InputManager with no inputs pressed.
    ///
    /// Default buffer duration is 200ms, suitable for most combo detection.
    pub fn new() -> Self {
        Self::with_buffer_duration(Duration::from_millis(200))
    }

    /// Creates a new InputManager with custom buffer duration.
    ///
    /// The buffer duration determines how long inputs are remembered for sequence detection.
    /// - Short durations (50-100ms): Strict, requires fast input
    /// - Medium durations (200-300ms): Balanced, works for most games
    /// - Long durations (500ms+): Lenient, easier for casual players
    pub fn with_buffer_duration(buffer_duration: Duration) -> Self {
        Self {
            keys_current: HashSet::new(),
            mouse_buttons_current: HashSet::new(),
            consumed_keys: HashSet::new(),
            consumed_mouse_buttons: HashSet::new(),
            gamepad_buttons_current: vec![HashSet::new(); 4], // Deprecated, for backward compat
            mouse_position: Vec2::zero(),
            mouse_delta: Vec2::zero(),
            keys_previous: HashSet::new(),
            mouse_buttons_previous: HashSet::new(),
            gamepad_buttons_previous: vec![HashSet::new(); 4], // Deprecated
            gamepads: vec![GamepadState::new(); 4], // Support up to 4 gamepads by default
            gamepads_previous: vec![GamepadState::new(); 4],
            scroll_delta: Vec2::zero(),
            action_mappings: HashMap::new(),
            input_buffer: VecDeque::with_capacity(32),
            buffer_duration,
            last_update: Instant::now(),
            analog_deadzone: 0.1, // 10% deadzone by default
            touches_current: HashMap::new(),
            touches_previous: HashMap::new(),
            touch_pointer_emulation: true,
        }
    }

    /// Updates the input state for the next frame.
    ///
    /// This should be called at the start of each frame, before any input queries.
    /// It copies current state to previous state and resets deltas.
    pub fn update(&mut self) {
        let now = Instant::now();

        // Copy current to previous
        self.keys_previous = self.keys_current.clone();
        self.mouse_buttons_previous = self.mouse_buttons_current.clone();
        self.gamepad_buttons_previous = self.gamepad_buttons_current.clone();
        self.gamepads_previous = self.gamepads.clone();
        self.clear_consumed_inputs();

        // Reset deltas
        self.mouse_delta = Vec2::zero();
        self.scroll_delta = Vec2::zero();

        // Cycle touch state: remove ended/cancelled, copy current to previous
        self.touches_previous = self.touches_current.clone();
        self.touches_current.retain(|_, state| {
            state.phase != TouchPhase::Ended && state.phase != TouchPhase::Cancelled
        });
        // Reset moved touches back to started for next frame's delta detection
        for state in self.touches_current.values_mut() {
            state.previous_position = state.position;
        }

        // Clean up expired inputs from buffer
        self.input_buffer
            .retain(|input| !input.is_expired(now, self.buffer_duration));

        self.last_update = now;
    }

    // === Keyboard Input ===

    /// Sets a key as pressed.
    pub fn press_key(&mut self, key: Key) {
        // Only buffer if this is a new press (not already held)
        if !self.keys_current.contains(&key) {
            self.buffer_input(InputBinding::Key(key));
        }
        self.keys_current.insert(key);
        if let Some(key_name) = normalized_key_name(key) {
            record_debugger_input_event(SyntheticInputEventV1 {
                device: "keyboard".to_string(),
                action: "press".to_string(),
                key: Some(key_name.to_string()),
                button: None,
                position: None,
                delta: None,
            });
        }
    }

    /// Sets a key as released.
    pub fn release_key(&mut self, key: Key) {
        self.keys_current.remove(&key);
        if let Some(key_name) = normalized_key_name(key) {
            record_debugger_input_event(SyntheticInputEventV1 {
                device: "keyboard".to_string(),
                action: "release".to_string(),
                key: Some(key_name.to_string()),
                button: None,
                position: None,
                delta: None,
            });
        }
    }

    /// Returns true if the key is currently pressed.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_current.contains(&key) && !self.consumed_keys.contains(&key)
    }

    /// Returns true if the key was just pressed this frame.
    ///
    /// True only on the first frame the key is pressed.
    pub fn key_just_pressed(&self, key: Key) -> bool {
        self.keys_current.contains(&key)
            && !self.keys_previous.contains(&key)
            && !self.consumed_keys.contains(&key)
    }

    /// Returns true if the key was just released this frame.
    ///
    /// True only on the first frame the key is released.
    pub fn key_just_released(&self, key: Key) -> bool {
        !self.keys_current.contains(&key)
            && self.keys_previous.contains(&key)
            && !self.consumed_keys.contains(&key)
    }

    /// Returns an iterator over all currently pressed keys.
    pub fn keys_pressed(&self) -> impl Iterator<Item = &Key> {
        self.keys_current.iter()
    }

    // === Mouse Input ===

    /// Sets a mouse button as pressed.
    pub fn press_mouse_button(&mut self, button: MouseButton) {
        // Only buffer if this is a new press
        if !self.mouse_buttons_current.contains(&button) {
            self.buffer_input(InputBinding::MouseButton(button));
        }
        self.mouse_buttons_current.insert(button);
        if let Some(button_name) = normalized_mouse_button_name(button) {
            record_debugger_input_event(SyntheticInputEventV1 {
                device: "mouse".to_string(),
                action: "press".to_string(),
                key: None,
                button: Some(button_name.to_string()),
                position: None,
                delta: None,
            });
        }
    }

    /// Sets a mouse button as released.
    pub fn release_mouse_button(&mut self, button: MouseButton) {
        self.mouse_buttons_current.remove(&button);
        if let Some(button_name) = normalized_mouse_button_name(button) {
            record_debugger_input_event(SyntheticInputEventV1 {
                device: "mouse".to_string(),
                action: "release".to_string(),
                key: None,
                button: Some(button_name.to_string()),
                position: None,
                delta: None,
            });
        }
    }

    /// Returns true if the mouse button is currently pressed.
    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_current.contains(&button)
            && !self.consumed_mouse_buttons.contains(&button)
    }

    /// Returns true if the mouse button was just pressed this frame.
    pub fn mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_current.contains(&button)
            && !self.mouse_buttons_previous.contains(&button)
            && !self.consumed_mouse_buttons.contains(&button)
    }

    /// Returns true if the mouse button was just released this frame.
    pub fn mouse_button_just_released(&self, button: MouseButton) -> bool {
        !self.mouse_buttons_current.contains(&button)
            && self.mouse_buttons_previous.contains(&button)
            && !self.consumed_mouse_buttons.contains(&button)
    }

    /// Returns an iterator over all currently pressed mouse buttons.
    pub fn mouse_buttons_pressed(&self) -> impl Iterator<Item = &MouseButton> {
        self.mouse_buttons_current.iter()
    }

    // === Mouse Position ===

    /// Updates the mouse position.
    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.mouse_delta = position - self.mouse_position;
        self.mouse_position = position;
        record_debugger_input_event(SyntheticInputEventV1 {
            device: "mouse".to_string(),
            action: "move".to_string(),
            key: None,
            button: None,
            position: Some([self.mouse_position.x, self.mouse_position.y]),
            delta: Some([self.mouse_delta.x, self.mouse_delta.y]),
        });
    }

    /// Returns the current mouse position in screen coordinates.
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Returns the mouse movement delta since last frame.
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    // === Mouse Scroll ===

    /// Updates the mouse scroll delta.
    pub fn add_scroll_delta(&mut self, delta: Vec2) {
        self.scroll_delta = self.scroll_delta + delta;
        record_debugger_input_event(SyntheticInputEventV1 {
            device: "mouse".to_string(),
            action: "scroll".to_string(),
            key: None,
            button: None,
            position: None,
            delta: Some([delta.x, delta.y]),
        });
    }

    /// Returns the mouse scroll delta for this frame.
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    /// Clears all input state (useful for focus loss or pausing).
    pub fn clear(&mut self) {
        self.keys_current.clear();
        self.keys_previous.clear();
        self.consumed_keys.clear();
        self.mouse_buttons_current.clear();
        self.mouse_buttons_previous.clear();
        self.consumed_mouse_buttons.clear();
        for buttons in &mut self.gamepad_buttons_current {
            buttons.clear();
        }
        for buttons in &mut self.gamepad_buttons_previous {
            buttons.clear();
        }
        // Clear new gamepad state (buttons and axes)
        for gamepad in &mut self.gamepads {
            gamepad.buttons.clear();
            gamepad.axes.clear();
            // Don't clear connection status or vibration
        }
        for gamepad in &mut self.gamepads_previous {
            gamepad.buttons.clear();
            gamepad.axes.clear();
        }
        self.mouse_delta = Vec2::zero();
        self.scroll_delta = Vec2::zero();
        self.touches_current.clear();
        self.touches_previous.clear();
    }

    /// Adds an input to the buffer for sequence detection.
    pub(super) fn buffer_input(&mut self, binding: InputBinding) {
        let now = Instant::now();
        self.input_buffer
            .push_back(BufferedInput::new(binding, now));

        // Keep buffer size reasonable (prevent memory growth from rapid inputs)
        while self.input_buffer.len() > 32 {
            self.input_buffer.pop_front();
        }
    }

    /// Consumes a key for the current frame.
    ///
    /// Once consumed, key query methods return `false` for this key until the
    /// next frame (or until consumption is manually cleared).
    pub fn consume_key(&mut self, key: Key) {
        self.consumed_keys.insert(key);
    }

    /// Consumes a mouse button for the current frame.
    ///
    /// Once consumed, mouse button query methods return `false` for this button
    /// until the next frame (or until consumption is manually cleared).
    pub fn consume_mouse_button(&mut self, button: MouseButton) {
        self.consumed_mouse_buttons.insert(button);
    }

    /// Clears all per-frame consumed input masks.
    pub fn clear_consumed_inputs(&mut self) {
        self.consumed_keys.clear();
        self.consumed_mouse_buttons.clear();
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

fn record_debugger_input_event(event: SyntheticInputEventV1) {
    debugger::record_synthetic_input_for_current_route(event);
}

fn normalized_key_name(key: Key) -> Option<&'static str> {
    match key {
        Key::Space => Some("space"),
        Key::Enter => Some("enter"),
        Key::Escape => Some("escape"),
        Key::Tab => Some("tab"),
        Key::Left => Some("left"),
        Key::Right => Some("right"),
        Key::Up => Some("up"),
        Key::Down => Some("down"),
        Key::A => Some("a"),
        Key::D => Some("d"),
        Key::S => Some("s"),
        Key::W => Some("w"),
        _ => None,
    }
}

fn normalized_mouse_button_name(button: MouseButton) -> Option<&'static str> {
    match button {
        MouseButton::Left => Some("left"),
        MouseButton::Right => Some("right"),
        MouseButton::Middle => Some("middle"),
        _ => None,
    }
}
