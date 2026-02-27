//! Input management system for ECS integration.
//!
//! The `InputManager` resource provides a centralized interface for querying input state
//! within the ECS. It tracks keyboard keys, mouse buttons, mouse position, and gamepad state
//! across frames, enabling queries for:
//! - Current state (is pressed)
//! - Just pressed (pressed this frame, not last frame)
//! - Just released (released this frame, was pressed last frame)
//!
//! # Architecture
//!
//! The InputManager sits between the platform layer (GLFW) and the game systems:
//!
//! ```text
//! GLFW Events → InputHandler → InputManager → Game Systems
//!                (platform)     (ECS resource)   (queries)
//! ```
//!
//! # Usage
//!
//! ## Raw Input Queries
//!
//! ```ignore
//! use goud_engine::ecs::{InputManager, Resource};
//! use glfw::Key;
//!
//! // In your setup system:
//! world.insert_resource(InputManager::new());
//!
//! // In a system:
//! fn player_movement_system(input: Res<InputManager>) {
//!     if input.key_pressed(Key::W) {
//!         // Move forward continuously while held
//!     }
//!     if input.key_just_pressed(Key::Space) {
//!         // Jump only once per press
//!     }
//! }
//! ```
//!
//! ## Action Mapping
//!
//! Action mapping allows semantic names for input, supporting multiple bindings:
//!
//! ```ignore
//! use goud_engine::ecs::{InputManager, InputBinding};
//! use glfw::Key;
//!
//! let mut input = InputManager::new();
//!
//! // Map "Jump" to Space, W key, or gamepad button 0
//! input.map_action("Jump", InputBinding::Key(Key::Space));
//! input.map_action("Jump", InputBinding::Key(Key::W));
//! input.map_action("Jump", InputBinding::GamepadButton { gamepad_id: 0, button: 0 });
//!
//! // Query action state (returns true if ANY binding is pressed)
//! if input.action_pressed("Jump") {
//!     player.jump();
//! }
//!
//! if input.action_just_pressed("Attack") {
//!     player.attack();
//! }
//! ```
//!
//! # Frame Management
//!
//! Call `update()` at the start of each frame to advance the input state:
//!
//! ```ignore
//! fn input_update_system(mut input: ResMut<InputManager>) {
//!     input.update();
//! }
//! ```

use glfw::{GamepadAxis, Key, MouseButton};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

use crate::core::math::Vec2;

/// Gamepad state for a single controller.
///
/// Tracks buttons, axes (analog sticks, triggers), connection status, and vibration.
#[derive(Debug, Clone)]
struct GamepadState {
    /// Currently pressed buttons (using GLFW's button indices)
    buttons: HashSet<u32>,
    /// Analog axis values (-1.0 to 1.0)
    axes: HashMap<GamepadAxis, f32>,
    /// Whether this gamepad is currently connected
    connected: bool,
    /// Rumble/vibration intensity (0.0-1.0)
    vibration: f32,
}

impl GamepadState {
    fn new() -> Self {
        Self {
            buttons: HashSet::new(),
            axes: HashMap::new(),
            connected: false,
            vibration: 0.0,
        }
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.buttons.is_empty() && self.axes.is_empty()
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
struct BufferedInput {
    /// The input binding that was pressed
    binding: InputBinding,
    /// When the input was pressed
    timestamp: Instant,
}

impl BufferedInput {
    /// Creates a new buffered input.
    fn new(binding: InputBinding, timestamp: Instant) -> Self {
        Self { binding, timestamp }
    }

    /// Returns the age of this input in seconds.
    fn age(&self, now: Instant) -> f32 {
        now.duration_since(self.timestamp).as_secs_f32()
    }

    /// Returns true if this input has expired given the buffer duration.
    fn is_expired(&self, now: Instant, buffer_duration: Duration) -> bool {
        now.duration_since(self.timestamp) > buffer_duration
    }
}

/// Input management resource for ECS integration.
///
/// Tracks keyboard, mouse, and gamepad input state across frames,
/// enabling queries for current state, just pressed, and just released.
/// Also supports action mapping for semantic input handling.
#[derive(Debug, Clone)]
pub struct InputManager {
    // Current frame state
    keys_current: HashSet<Key>,
    mouse_buttons_current: HashSet<MouseButton>,
    gamepad_buttons_current: Vec<HashSet<u32>>, // gamepad_id -> buttons (deprecated, use gamepads)
    mouse_position: Vec2,
    mouse_delta: Vec2,

    // Previous frame state (for just_pressed/just_released detection)
    keys_previous: HashSet<Key>,
    mouse_buttons_previous: HashSet<MouseButton>,
    gamepad_buttons_previous: Vec<HashSet<u32>>, // deprecated, use gamepads_previous

    // Gamepad state (current and previous)
    gamepads: Vec<GamepadState>,
    gamepads_previous: Vec<GamepadState>,

    // Mouse scroll
    scroll_delta: Vec2,

    // Action mappings (action_name -> list of bindings)
    action_mappings: HashMap<String, Vec<InputBinding>>,

    // Input buffering for sequences and combos
    input_buffer: VecDeque<BufferedInput>,
    buffer_duration: Duration,
    last_update: Instant,

    // Analog deadzone threshold (default 0.1)
    analog_deadzone: f32,
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

        // Reset deltas
        self.mouse_delta = Vec2::zero();
        self.scroll_delta = Vec2::zero();

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
    }

    /// Sets a key as released.
    pub fn release_key(&mut self, key: Key) {
        self.keys_current.remove(&key);
    }

    /// Returns true if the key is currently pressed.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_current.contains(&key)
    }

    /// Returns true if the key was just pressed this frame.
    ///
    /// True only on the first frame the key is pressed.
    pub fn key_just_pressed(&self, key: Key) -> bool {
        self.keys_current.contains(&key) && !self.keys_previous.contains(&key)
    }

    /// Returns true if the key was just released this frame.
    ///
    /// True only on the first frame the key is released.
    pub fn key_just_released(&self, key: Key) -> bool {
        !self.keys_current.contains(&key) && self.keys_previous.contains(&key)
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
    }

    /// Sets a mouse button as released.
    pub fn release_mouse_button(&mut self, button: MouseButton) {
        self.mouse_buttons_current.remove(&button);
    }

    /// Returns true if the mouse button is currently pressed.
    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_current.contains(&button)
    }

    /// Returns true if the mouse button was just pressed this frame.
    pub fn mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_current.contains(&button)
            && !self.mouse_buttons_previous.contains(&button)
    }

    /// Returns true if the mouse button was just released this frame.
    pub fn mouse_button_just_released(&self, button: MouseButton) -> bool {
        !self.mouse_buttons_current.contains(&button)
            && self.mouse_buttons_previous.contains(&button)
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
    }

    /// Returns the mouse scroll delta for this frame.
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

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
    fn ensure_gamepad_capacity(&mut self, gamepad_id: usize) {
        while self.gamepad_buttons_current.len() <= gamepad_id {
            self.gamepad_buttons_current.push(HashSet::new());
        }
        while self.gamepad_buttons_previous.len() <= gamepad_id {
            self.gamepad_buttons_previous.push(HashSet::new());
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

    // === Input Buffering ===

    /// Adds an input to the buffer for sequence detection.
    fn buffer_input(&mut self, binding: InputBinding) {
        let now = Instant::now();
        self.input_buffer
            .push_back(BufferedInput::new(binding, now));

        // Keep buffer size reasonable (prevent memory growth from rapid inputs)
        while self.input_buffer.len() > 32 {
            self.input_buffer.pop_front();
        }
    }

    /// Returns the current buffer duration.
    pub fn buffer_duration(&self) -> Duration {
        self.buffer_duration
    }

    /// Sets the buffer duration for input sequences.
    ///
    /// This determines how long inputs are remembered for combo detection.
    pub fn set_buffer_duration(&mut self, duration: Duration) {
        self.buffer_duration = duration;
    }

    /// Returns the number of inputs currently in the buffer.
    pub fn buffer_size(&self) -> usize {
        self.input_buffer.len()
    }

    /// Clears the input buffer.
    ///
    /// Useful when resetting combos or canceling sequences.
    pub fn clear_buffer(&mut self) {
        self.input_buffer.clear();
    }

    /// Checks if a sequence of inputs was pressed within the buffer duration.
    ///
    /// Returns true if all bindings in the sequence were pressed in order,
    /// with each subsequent input occurring within the buffer window.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::{InputManager, InputBinding};
    /// use glfw::Key;
    ///
    /// let mut input = InputManager::new();
    ///
    /// // Detect "Down, Down, Forward, Punch" combo (fighting game)
    /// let combo = vec![
    ///     InputBinding::Key(Key::Down),
    ///     InputBinding::Key(Key::Down),
    ///     InputBinding::Key(Key::Right),
    ///     InputBinding::Key(Key::Space),
    /// ];
    ///
    /// if input.sequence_detected(&combo) {
    ///     player.perform_special_move();
    /// }
    /// ```
    pub fn sequence_detected(&self, sequence: &[InputBinding]) -> bool {
        if sequence.is_empty() || self.input_buffer.is_empty() {
            return false;
        }

        let now = Instant::now();
        let mut seq_index = 0;

        // Scan buffer from oldest to newest
        for buffered in &self.input_buffer {
            // Skip expired inputs
            if buffered.is_expired(now, self.buffer_duration) {
                continue;
            }

            // Check if this matches the next input in sequence
            if buffered.binding == sequence[seq_index] {
                seq_index += 1;

                // Entire sequence matched
                if seq_index == sequence.len() {
                    return true;
                }
            }
        }

        false
    }

    /// Checks if a sequence was pressed and clears the buffer if detected.
    ///
    /// This is useful for consuming combos so they don't trigger multiple times.
    ///
    /// Returns true if the sequence was detected and consumed.
    pub fn consume_sequence(&mut self, sequence: &[InputBinding]) -> bool {
        if self.sequence_detected(sequence) {
            self.clear_buffer();
            true
        } else {
            false
        }
    }

    /// Returns the time since the last buffered input in seconds.
    ///
    /// Returns None if the buffer is empty.
    pub fn time_since_last_input(&self) -> Option<f32> {
        self.input_buffer
            .back()
            .map(|input| input.age(Instant::now()))
    }

    /// Returns all inputs in the buffer (oldest to newest).
    ///
    /// Useful for debugging or visualizing input history.
    pub fn buffered_inputs(&self) -> impl Iterator<Item = (InputBinding, f32)> + '_ {
        let now = Instant::now();
        self.input_buffer
            .iter()
            .map(move |input| (input.binding, input.age(now)))
    }

    /// Clears all input state (useful for focus loss or pausing).
    pub fn clear(&mut self) {
        self.keys_current.clear();
        self.keys_previous.clear();
        self.mouse_buttons_current.clear();
        self.mouse_buttons_previous.clear();
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
    }

    // === Action Mapping ===

    /// Maps an input binding to an action.
    ///
    /// An action can have multiple bindings. If the action already exists,
    /// the binding is added to its list. Duplicate bindings are allowed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::{InputManager, InputBinding};
    /// use glfw::Key;
    ///
    /// let mut input = InputManager::new();
    /// input.map_action("Jump", InputBinding::Key(Key::Space));
    /// input.map_action("Jump", InputBinding::Key(Key::W)); // Alternative binding
    /// ```
    pub fn map_action(&mut self, action: impl Into<String>, binding: InputBinding) {
        self.action_mappings
            .entry(action.into())
            .or_default()
            .push(binding);
    }

    /// Unmaps a specific input binding from an action.
    ///
    /// Returns true if the binding was removed, false if it wasn't found.
    pub fn unmap_action(&mut self, action: &str, binding: InputBinding) -> bool {
        if let Some(bindings) = self.action_mappings.get_mut(action) {
            if let Some(pos) = bindings.iter().position(|b| *b == binding) {
                bindings.remove(pos);
                return true;
            }
        }
        false
    }

    /// Removes all bindings for an action.
    ///
    /// Returns true if the action existed and was removed.
    pub fn clear_action(&mut self, action: &str) -> bool {
        self.action_mappings.remove(action).is_some()
    }

    /// Removes all action mappings.
    pub fn clear_all_actions(&mut self) {
        self.action_mappings.clear();
    }

    /// Returns all bindings for an action.
    ///
    /// Returns an empty slice if the action doesn't exist.
    pub fn get_action_bindings(&self, action: &str) -> &[InputBinding] {
        self.action_mappings
            .get(action)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns true if the action has any bindings.
    pub fn has_action(&self, action: &str) -> bool {
        self.action_mappings.contains_key(action)
    }

    /// Returns an iterator over all action names.
    pub fn action_names(&self) -> impl Iterator<Item = &str> {
        self.action_mappings.keys().map(|s| s.as_str())
    }

    /// Returns the number of registered actions.
    pub fn action_count(&self) -> usize {
        self.action_mappings.len()
    }

    /// Returns true if ANY binding for the action is currently pressed.
    ///
    /// Returns false if the action doesn't exist or has no bindings.
    pub fn action_pressed(&self, action: &str) -> bool {
        self.action_mappings
            .get(action)
            .is_some_and(|bindings| bindings.iter().any(|b| b.is_pressed(self)))
    }

    /// Returns true if ANY binding for the action was just pressed this frame.
    ///
    /// Returns false if the action doesn't exist or has no bindings.
    pub fn action_just_pressed(&self, action: &str) -> bool {
        self.action_mappings
            .get(action)
            .is_some_and(|bindings| bindings.iter().any(|b| b.is_just_pressed(self)))
    }

    /// Returns true if ANY binding for the action was just released this frame.
    ///
    /// Returns false if the action doesn't exist or has no bindings.
    pub fn action_just_released(&self, action: &str) -> bool {
        self.action_mappings
            .get(action)
            .is_some_and(|bindings| bindings.iter().any(|b| b.is_just_released(self)))
    }

    /// Returns the strength of the action (0.0-1.0).
    ///
    /// For digital inputs (keys, buttons), this returns 1.0 if pressed, 0.0 otherwise.
    /// This method exists for future analog input support (triggers, analog sticks).
    pub fn action_strength(&self, action: &str) -> f32 {
        if self.action_pressed(action) {
            1.0
        } else {
            0.0
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input_manager() {
        let input = InputManager::new();
        assert!(!input.key_pressed(Key::A));
        assert!(!input.mouse_button_pressed(MouseButton::Button1));
        assert_eq!(input.mouse_position(), Vec2::zero());
        assert_eq!(input.mouse_delta(), Vec2::zero());
    }

    #[test]
    fn test_default() {
        let input = InputManager::default();
        assert!(!input.key_pressed(Key::W));
    }

    // === Keyboard Tests ===

    #[test]
    fn test_key_pressed() {
        let mut input = InputManager::new();
        assert!(!input.key_pressed(Key::A));

        input.press_key(Key::A);
        assert!(input.key_pressed(Key::A));

        input.release_key(Key::A);
        assert!(!input.key_pressed(Key::A));
    }

    #[test]
    fn test_key_just_pressed() {
        let mut input = InputManager::new();

        // Press key
        input.press_key(Key::Space);
        assert!(input.key_just_pressed(Key::Space)); // First frame

        // Update to next frame
        input.update();
        assert!(!input.key_just_pressed(Key::Space)); // Still held, but not "just" pressed

        // Release and press again
        input.release_key(Key::Space);
        input.update();
        input.press_key(Key::Space);
        assert!(input.key_just_pressed(Key::Space)); // Just pressed again
    }

    #[test]
    fn test_key_just_released() {
        let mut input = InputManager::new();

        // Press key
        input.press_key(Key::W);
        input.update(); // Make it "previous"

        // Release key
        input.release_key(Key::W);
        assert!(input.key_just_released(Key::W));

        // Update to next frame
        input.update();
        assert!(!input.key_just_released(Key::W)); // No longer "just" released
    }

    #[test]
    fn test_keys_pressed_iterator() {
        let mut input = InputManager::new();
        input.press_key(Key::A);
        input.press_key(Key::B);
        input.press_key(Key::C);

        let pressed_keys: Vec<_> = input.keys_pressed().collect();
        assert_eq!(pressed_keys.len(), 3);
        assert!(pressed_keys.contains(&&Key::A));
        assert!(pressed_keys.contains(&&Key::B));
        assert!(pressed_keys.contains(&&Key::C));
    }

    // === Mouse Button Tests ===

    #[test]
    fn test_mouse_button_pressed() {
        let mut input = InputManager::new();
        assert!(!input.mouse_button_pressed(MouseButton::Button1));

        input.press_mouse_button(MouseButton::Button1);
        assert!(input.mouse_button_pressed(MouseButton::Button1));

        input.release_mouse_button(MouseButton::Button1);
        assert!(!input.mouse_button_pressed(MouseButton::Button1));
    }

    #[test]
    fn test_mouse_button_just_pressed() {
        let mut input = InputManager::new();

        input.press_mouse_button(MouseButton::Button2);
        assert!(input.mouse_button_just_pressed(MouseButton::Button2));

        input.update();
        assert!(!input.mouse_button_just_pressed(MouseButton::Button2));
    }

    #[test]
    fn test_mouse_button_just_released() {
        let mut input = InputManager::new();

        input.press_mouse_button(MouseButton::Button1);
        input.update();
        input.release_mouse_button(MouseButton::Button1);
        assert!(input.mouse_button_just_released(MouseButton::Button1));

        input.update();
        assert!(!input.mouse_button_just_released(MouseButton::Button1));
    }

    #[test]
    fn test_mouse_buttons_pressed_iterator() {
        let mut input = InputManager::new();
        input.press_mouse_button(MouseButton::Button1);
        input.press_mouse_button(MouseButton::Button2);

        let pressed_buttons: Vec<_> = input.mouse_buttons_pressed().collect();
        assert_eq!(pressed_buttons.len(), 2);
    }

    // === Mouse Position Tests ===

    #[test]
    fn test_mouse_position() {
        let mut input = InputManager::new();
        assert_eq!(input.mouse_position(), Vec2::zero());

        let pos = Vec2::new(100.0, 200.0);
        input.set_mouse_position(pos);
        assert_eq!(input.mouse_position(), pos);
    }

    #[test]
    fn test_mouse_delta() {
        let mut input = InputManager::new();

        // First position
        input.set_mouse_position(Vec2::new(100.0, 100.0));
        assert_eq!(input.mouse_delta(), Vec2::new(100.0, 100.0)); // Delta from (0,0)

        // Second position
        input.set_mouse_position(Vec2::new(150.0, 120.0));
        assert_eq!(input.mouse_delta(), Vec2::new(50.0, 20.0)); // Delta from previous
    }

    #[test]
    fn test_mouse_delta_reset_on_update() {
        let mut input = InputManager::new();

        input.set_mouse_position(Vec2::new(100.0, 100.0));
        assert_ne!(input.mouse_delta(), Vec2::zero());

        input.update(); // Reset delta
        assert_eq!(input.mouse_delta(), Vec2::zero());
    }

    // === Scroll Tests ===

    #[test]
    fn test_scroll_delta() {
        let mut input = InputManager::new();
        assert_eq!(input.scroll_delta(), Vec2::zero());

        input.add_scroll_delta(Vec2::new(0.0, 1.0));
        assert_eq!(input.scroll_delta(), Vec2::new(0.0, 1.0));

        input.add_scroll_delta(Vec2::new(0.0, 2.0));
        assert_eq!(input.scroll_delta(), Vec2::new(0.0, 3.0)); // Accumulates
    }

    #[test]
    fn test_scroll_delta_reset_on_update() {
        let mut input = InputManager::new();

        input.add_scroll_delta(Vec2::new(0.0, 5.0));
        input.update();
        assert_eq!(input.scroll_delta(), Vec2::zero());
    }

    // === Gamepad Tests ===

    #[test]
    fn test_gamepad_button_pressed() {
        let mut input = InputManager::new();

        input.press_gamepad_button(0, 1);
        assert!(input.gamepad_button_pressed(0, 1));
        assert!(!input.gamepad_button_pressed(0, 2));
        assert!(!input.gamepad_button_pressed(1, 1)); // Different gamepad

        input.release_gamepad_button(0, 1);
        assert!(!input.gamepad_button_pressed(0, 1));
    }

    #[test]
    fn test_gamepad_button_just_pressed() {
        let mut input = InputManager::new();

        input.press_gamepad_button(0, 5);
        assert!(input.gamepad_button_just_pressed(0, 5));

        input.update();
        assert!(!input.gamepad_button_just_pressed(0, 5));
    }

    #[test]
    fn test_gamepad_button_just_released() {
        let mut input = InputManager::new();

        input.press_gamepad_button(1, 3);
        input.update();
        input.release_gamepad_button(1, 3);
        assert!(input.gamepad_button_just_released(1, 3));

        input.update();
        assert!(!input.gamepad_button_just_released(1, 3));
    }

    #[test]
    fn test_gamepad_multiple_gamepads() {
        let mut input = InputManager::new();

        input.press_gamepad_button(0, 1);
        input.press_gamepad_button(1, 1);
        input.press_gamepad_button(2, 2);

        assert!(input.gamepad_button_pressed(0, 1));
        assert!(input.gamepad_button_pressed(1, 1));
        assert!(input.gamepad_button_pressed(2, 2));
        assert!(!input.gamepad_button_pressed(2, 1));
    }

    #[test]
    fn test_gamepad_capacity_expansion() {
        let mut input = InputManager::new();

        // Should expand to support gamepad 5
        input.press_gamepad_button(5, 1);
        assert!(input.gamepad_button_pressed(5, 1));
    }

    // === Clear Tests ===

    #[test]
    fn test_clear() {
        let mut input = InputManager::new();

        // Set various inputs
        input.press_key(Key::A);
        input.press_mouse_button(MouseButton::Button1);
        input.set_mouse_position(Vec2::new(100.0, 100.0));
        input.add_scroll_delta(Vec2::new(0.0, 1.0));
        input.press_gamepad_button(0, 5);

        // Clear all
        input.clear();

        // Verify all cleared
        assert!(!input.key_pressed(Key::A));
        assert!(!input.mouse_button_pressed(MouseButton::Button1));
        assert_eq!(input.mouse_delta(), Vec2::zero());
        assert_eq!(input.scroll_delta(), Vec2::zero());
        assert!(!input.gamepad_button_pressed(0, 5));
    }

    // === Update Tests ===

    #[test]
    fn test_update_copies_state() {
        let mut input = InputManager::new();

        input.press_key(Key::Space);
        assert!(input.key_just_pressed(Key::Space));

        input.update();
        assert!(!input.key_just_pressed(Key::Space)); // No longer "just" pressed
        assert!(input.key_pressed(Key::Space)); // Still pressed
    }

    #[test]
    fn test_clone() {
        let mut input = InputManager::new();
        input.press_key(Key::A);

        let cloned = input.clone();
        assert!(cloned.key_pressed(Key::A));
    }

    #[test]
    fn test_debug() {
        let input = InputManager::new();
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("InputManager"));
    }

    // === InputBinding Tests ===

    #[test]
    fn test_input_binding_key() {
        let mut input = InputManager::new();
        let binding = InputBinding::Key(Key::A);

        assert!(!binding.is_pressed(&input));
        assert!(!binding.is_just_pressed(&input));

        input.press_key(Key::A);
        assert!(binding.is_pressed(&input));
        assert!(binding.is_just_pressed(&input));

        input.update();
        assert!(binding.is_pressed(&input));
        assert!(!binding.is_just_pressed(&input));

        input.release_key(Key::A);
        assert!(!binding.is_pressed(&input));
        assert!(binding.is_just_released(&input));
    }

    #[test]
    fn test_input_binding_mouse_button() {
        let mut input = InputManager::new();
        let binding = InputBinding::MouseButton(MouseButton::Button1);

        assert!(!binding.is_pressed(&input));

        input.press_mouse_button(MouseButton::Button1);
        assert!(binding.is_pressed(&input));
        assert!(binding.is_just_pressed(&input));
    }

    #[test]
    fn test_input_binding_gamepad_button() {
        let mut input = InputManager::new();
        let binding = InputBinding::GamepadButton {
            gamepad_id: 0,
            button: 5,
        };

        assert!(!binding.is_pressed(&input));

        input.press_gamepad_button(0, 5);
        assert!(binding.is_pressed(&input));
        assert!(binding.is_just_pressed(&input));
    }

    #[test]
    fn test_input_binding_display() {
        let key_binding = InputBinding::Key(Key::Space);
        let mouse_binding = InputBinding::MouseButton(MouseButton::Button1);
        let gamepad_binding = InputBinding::GamepadButton {
            gamepad_id: 2,
            button: 10,
        };

        let key_str = format!("{}", key_binding);
        let mouse_str = format!("{}", mouse_binding);
        let gamepad_str = format!("{}", gamepad_binding);

        assert!(key_str.contains("Key"));
        assert!(mouse_str.contains("MouseButton"));
        assert!(gamepad_str.contains("GamepadButton"));
        assert!(gamepad_str.contains("gamepad=2"));
        assert!(gamepad_str.contains("button=10"));
    }

    #[test]
    fn test_input_binding_eq() {
        let binding1 = InputBinding::Key(Key::A);
        let binding2 = InputBinding::Key(Key::A);
        let binding3 = InputBinding::Key(Key::B);

        assert_eq!(binding1, binding2);
        assert_ne!(binding1, binding3);
    }

    #[test]
    fn test_input_binding_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(InputBinding::Key(Key::A));
        set.insert(InputBinding::Key(Key::A)); // Duplicate
        set.insert(InputBinding::Key(Key::B));

        assert_eq!(set.len(), 2); // Duplicate not added
    }

    // === Action Mapping Tests ===

    #[test]
    fn test_map_action_basic() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));
        assert!(input.has_action("Jump"));
        assert_eq!(input.get_action_bindings("Jump").len(), 1);
    }

    #[test]
    fn test_map_action_multiple_bindings() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));
        input.map_action("Jump", InputBinding::Key(Key::W));
        input.map_action(
            "Jump",
            InputBinding::GamepadButton {
                gamepad_id: 0,
                button: 0,
            },
        );

        let bindings = input.get_action_bindings("Jump");
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_unmap_action() {
        let mut input = InputManager::new();
        let binding = InputBinding::Key(Key::Space);

        input.map_action("Jump", binding);
        assert_eq!(input.get_action_bindings("Jump").len(), 1);

        assert!(input.unmap_action("Jump", binding));
        assert_eq!(input.get_action_bindings("Jump").len(), 0);

        // Unmapping non-existent binding returns false
        assert!(!input.unmap_action("Jump", binding));
    }

    #[test]
    fn test_clear_action() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));
        input.map_action("Jump", InputBinding::Key(Key::W));

        assert!(input.clear_action("Jump"));
        assert!(!input.has_action("Jump"));

        // Clearing non-existent action returns false
        assert!(!input.clear_action("Jump"));
    }

    #[test]
    fn test_clear_all_actions() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));
        input.map_action("Attack", InputBinding::Key(Key::E));
        input.map_action("Defend", InputBinding::Key(Key::Q));

        assert_eq!(input.action_count(), 3);

        input.clear_all_actions();
        assert_eq!(input.action_count(), 0);
        assert!(!input.has_action("Jump"));
        assert!(!input.has_action("Attack"));
        assert!(!input.has_action("Defend"));
    }

    #[test]
    fn test_get_action_bindings_nonexistent() {
        let input = InputManager::new();
        let bindings = input.get_action_bindings("NonExistent");
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_has_action() {
        let mut input = InputManager::new();

        assert!(!input.has_action("Jump"));

        input.map_action("Jump", InputBinding::Key(Key::Space));
        assert!(input.has_action("Jump"));
    }

    #[test]
    fn test_action_names() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));
        input.map_action("Attack", InputBinding::Key(Key::E));

        let names: Vec<_> = input.action_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Jump"));
        assert!(names.contains(&"Attack"));
    }

    #[test]
    fn test_action_count() {
        let mut input = InputManager::new();

        assert_eq!(input.action_count(), 0);

        input.map_action("Jump", InputBinding::Key(Key::Space));
        assert_eq!(input.action_count(), 1);

        input.map_action("Attack", InputBinding::Key(Key::E));
        assert_eq!(input.action_count(), 2);
    }

    #[test]
    fn test_action_pressed() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));
        input.map_action("Jump", InputBinding::Key(Key::W));

        // No keys pressed
        assert!(!input.action_pressed("Jump"));

        // One key pressed
        input.press_key(Key::Space);
        assert!(input.action_pressed("Jump"));

        // Other key pressed
        input.release_key(Key::Space);
        input.press_key(Key::W);
        assert!(input.action_pressed("Jump"));

        // Both keys pressed
        input.press_key(Key::Space);
        assert!(input.action_pressed("Jump"));

        // No keys pressed again
        input.release_key(Key::Space);
        input.release_key(Key::W);
        assert!(!input.action_pressed("Jump"));
    }

    #[test]
    fn test_action_pressed_nonexistent() {
        let input = InputManager::new();
        assert!(!input.action_pressed("NonExistent"));
    }

    #[test]
    fn test_action_just_pressed() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));

        // Press key
        input.press_key(Key::Space);
        assert!(input.action_just_pressed("Jump"));

        // Update to next frame
        input.update();
        assert!(!input.action_just_pressed("Jump")); // Still held, not "just" pressed
    }

    #[test]
    fn test_action_just_released() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));

        // Press key
        input.press_key(Key::Space);
        input.update();

        // Release key
        input.release_key(Key::Space);
        assert!(input.action_just_released("Jump"));

        // Update to next frame
        input.update();
        assert!(!input.action_just_released("Jump"));
    }

    #[test]
    fn test_action_strength() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));

        // Not pressed
        assert_eq!(input.action_strength("Jump"), 0.0);

        // Pressed
        input.press_key(Key::Space);
        assert_eq!(input.action_strength("Jump"), 1.0);
    }

    #[test]
    fn test_action_strength_nonexistent() {
        let input = InputManager::new();
        assert_eq!(input.action_strength("NonExistent"), 0.0);
    }

    #[test]
    fn test_action_multiple_input_types() {
        let mut input = InputManager::new();

        // Map action to key, mouse button, and gamepad button
        input.map_action("Fire", InputBinding::Key(Key::Space));
        input.map_action("Fire", InputBinding::MouseButton(MouseButton::Button1));
        input.map_action(
            "Fire",
            InputBinding::GamepadButton {
                gamepad_id: 0,
                button: 0,
            },
        );

        // Test keyboard
        input.press_key(Key::Space);
        assert!(input.action_pressed("Fire"));
        input.release_key(Key::Space);
        assert!(!input.action_pressed("Fire"));

        // Test mouse
        input.press_mouse_button(MouseButton::Button1);
        assert!(input.action_pressed("Fire"));
        input.release_mouse_button(MouseButton::Button1);
        assert!(!input.action_pressed("Fire"));

        // Test gamepad
        input.press_gamepad_button(0, 0);
        assert!(input.action_pressed("Fire"));
    }

    #[test]
    fn test_action_mapping_string_ownership() {
        let mut input = InputManager::new();

        // Test with &str
        input.map_action("Jump", InputBinding::Key(Key::Space));
        assert!(input.has_action("Jump"));

        // Test with String
        let action_name = String::from("Attack");
        input.map_action(action_name.clone(), InputBinding::Key(Key::E));
        assert!(input.has_action(&action_name));
    }

    #[test]
    fn test_action_mapping_persistence_across_update() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));

        // Action mappings should persist across frame updates
        input.update();
        assert!(input.has_action("Jump"));

        input.update();
        assert!(input.has_action("Jump"));
    }

    #[test]
    fn test_action_mapping_persistence_across_clear() {
        let mut input = InputManager::new();

        input.map_action("Jump", InputBinding::Key(Key::Space));

        // Action mappings should persist across input state clear
        input.clear();
        assert!(input.has_action("Jump"));
    }

    // === Input Buffering Tests ===

    #[test]
    fn test_with_buffer_duration() {
        let duration = Duration::from_millis(500);
        let input = InputManager::with_buffer_duration(duration);
        assert_eq!(input.buffer_duration(), duration);
    }

    #[test]
    fn test_set_buffer_duration() {
        let mut input = InputManager::new();
        let new_duration = Duration::from_millis(300);

        input.set_buffer_duration(new_duration);
        assert_eq!(input.buffer_duration(), new_duration);
    }

    #[test]
    fn test_buffer_size() {
        let mut input = InputManager::new();
        assert_eq!(input.buffer_size(), 0);

        input.press_key(Key::A);
        assert_eq!(input.buffer_size(), 1);

        input.press_key(Key::B);
        assert_eq!(input.buffer_size(), 2);
    }

    #[test]
    fn test_clear_buffer() {
        let mut input = InputManager::new();

        input.press_key(Key::A);
        input.press_key(Key::B);
        assert_eq!(input.buffer_size(), 2);

        input.clear_buffer();
        assert_eq!(input.buffer_size(), 0);
    }

    #[test]
    fn test_buffer_only_new_presses() {
        let mut input = InputManager::new();

        // First press should buffer
        input.press_key(Key::A);
        assert_eq!(input.buffer_size(), 1);

        // Pressing again while held should not buffer
        input.press_key(Key::A);
        assert_eq!(input.buffer_size(), 1);

        // Release and press again should buffer
        input.release_key(Key::A);
        input.press_key(Key::A);
        assert_eq!(input.buffer_size(), 2);
    }

    #[test]
    fn test_sequence_detected_basic() {
        let mut input = InputManager::new();

        // Input sequence: A -> B -> C
        input.press_key(Key::A);
        input.press_key(Key::B);
        input.press_key(Key::C);

        let sequence = vec![
            InputBinding::Key(Key::A),
            InputBinding::Key(Key::B),
            InputBinding::Key(Key::C),
        ];

        assert!(input.sequence_detected(&sequence));
    }

    #[test]
    fn test_sequence_detected_wrong_order() {
        let mut input = InputManager::new();

        // Input sequence: A -> C -> B (wrong order)
        input.press_key(Key::A);
        input.press_key(Key::C);
        input.press_key(Key::B);

        let sequence = vec![
            InputBinding::Key(Key::A),
            InputBinding::Key(Key::B),
            InputBinding::Key(Key::C),
        ];

        assert!(!input.sequence_detected(&sequence));
    }

    #[test]
    fn test_sequence_detected_partial() {
        let mut input = InputManager::new();

        // Input only part of sequence
        input.press_key(Key::A);
        input.press_key(Key::B);

        let sequence = vec![
            InputBinding::Key(Key::A),
            InputBinding::Key(Key::B),
            InputBinding::Key(Key::C),
        ];

        assert!(!input.sequence_detected(&sequence));
    }

    #[test]
    fn test_sequence_detected_empty() {
        let input = InputManager::new();

        // Empty sequence should return false
        assert!(!input.sequence_detected(&[]));
    }

    #[test]
    fn test_sequence_detected_with_extra_inputs() {
        let mut input = InputManager::new();

        // Input sequence with extra inputs in between
        input.press_key(Key::A);
        input.press_key(Key::X); // Extra input
        input.press_key(Key::B);
        input.press_key(Key::Y); // Extra input
        input.press_key(Key::C);

        let sequence = vec![
            InputBinding::Key(Key::A),
            InputBinding::Key(Key::B),
            InputBinding::Key(Key::C),
        ];

        // Should still detect sequence (allows for extra inputs)
        assert!(input.sequence_detected(&sequence));
    }

    #[test]
    fn test_consume_sequence() {
        let mut input = InputManager::new();

        input.press_key(Key::A);
        input.press_key(Key::B);

        let sequence = vec![InputBinding::Key(Key::A), InputBinding::Key(Key::B)];

        // First consume should succeed
        assert!(input.consume_sequence(&sequence));
        assert_eq!(input.buffer_size(), 0); // Buffer cleared

        // Second consume should fail (buffer cleared)
        assert!(!input.consume_sequence(&sequence));
    }

    #[test]
    fn test_consume_sequence_not_detected() {
        let mut input = InputManager::new();

        input.press_key(Key::A);

        let sequence = vec![InputBinding::Key(Key::A), InputBinding::Key(Key::B)];

        // Sequence not complete, should not consume
        assert!(!input.consume_sequence(&sequence));
        assert_eq!(input.buffer_size(), 1); // Buffer not cleared
    }

    #[test]
    fn test_time_since_last_input() {
        let mut input = InputManager::new();

        // No inputs, should return None
        assert!(input.time_since_last_input().is_none());

        input.press_key(Key::A);

        // Should return Some(small value)
        let time = input.time_since_last_input();
        assert!(time.is_some());
        assert!(time.unwrap() < 0.1); // Should be very recent
    }

    #[test]
    fn test_buffered_inputs_iterator() {
        let mut input = InputManager::new();

        input.press_key(Key::A);
        input.press_key(Key::B);
        input.press_key(Key::C);

        let buffered: Vec<_> = input.buffered_inputs().collect();
        assert_eq!(buffered.len(), 3);

        // Check bindings (ages will be very small)
        assert_eq!(buffered[0].0, InputBinding::Key(Key::A));
        assert_eq!(buffered[1].0, InputBinding::Key(Key::B));
        assert_eq!(buffered[2].0, InputBinding::Key(Key::C));

        // Ages should be very small (recent)
        assert!(buffered[0].1 < 0.1);
        assert!(buffered[1].1 < 0.1);
        assert!(buffered[2].1 < 0.1);
    }

    #[test]
    fn test_buffer_expiration() {
        use std::thread;

        let mut input = InputManager::with_buffer_duration(Duration::from_millis(50));

        input.press_key(Key::A);
        assert_eq!(input.buffer_size(), 1);

        // Wait for buffer to expire
        thread::sleep(Duration::from_millis(60));

        // Update should clean expired inputs
        input.update();
        assert_eq!(input.buffer_size(), 0);
    }

    #[test]
    fn test_buffer_max_size() {
        let mut input = InputManager::new();

        // Fill buffer beyond max size (32)
        for i in 0..40 {
            input.press_key(Key::A);
            input.release_key(Key::A);
        }

        // Should cap at 32
        assert!(input.buffer_size() <= 32);
    }

    #[test]
    fn test_sequence_mixed_input_types() {
        let mut input = InputManager::new();

        // Sequence with keyboard, mouse, and gamepad
        input.press_key(Key::A);
        input.press_mouse_button(MouseButton::Button1);
        input.press_gamepad_button(0, 5);

        let sequence = vec![
            InputBinding::Key(Key::A),
            InputBinding::MouseButton(MouseButton::Button1),
            InputBinding::GamepadButton {
                gamepad_id: 0,
                button: 5,
            },
        ];

        assert!(input.sequence_detected(&sequence));
    }

    #[test]
    fn test_fighting_game_combo() {
        let mut input = InputManager::new();

        // Classic "hadouken" combo: Down -> Down (double tap) -> Forward -> Punch
        input.press_key(Key::Down);
        input.release_key(Key::Down); // Release for double tap
        input.press_key(Key::Down); // Second Down press
        input.press_key(Key::Right);
        input.press_key(Key::Space);

        let hadouken = vec![
            InputBinding::Key(Key::Down),
            InputBinding::Key(Key::Down),
            InputBinding::Key(Key::Right),
            InputBinding::Key(Key::Space),
        ];

        assert!(input.sequence_detected(&hadouken));
    }

    #[test]
    fn test_sequence_persistence_across_update() {
        let mut input = InputManager::new();

        input.press_key(Key::A);
        input.press_key(Key::B);

        // Update shouldn't clear recent buffer
        input.update();

        let sequence = vec![InputBinding::Key(Key::A), InputBinding::Key(Key::B)];
        assert!(input.sequence_detected(&sequence));
    }

    #[test]
    fn test_buffer_not_cleared_by_state_clear() {
        let mut input = InputManager::new();

        input.press_key(Key::A);
        assert_eq!(input.buffer_size(), 1);

        // clear() only clears input state, not buffer
        input.clear();
        assert_eq!(input.buffer_size(), 1);
    }

    #[test]
    fn test_double_tap_detection() {
        let mut input = InputManager::new();

        // Double tap: A -> A
        input.press_key(Key::W);
        input.release_key(Key::W);
        input.press_key(Key::W);

        let double_tap = vec![InputBinding::Key(Key::W), InputBinding::Key(Key::W)];

        assert!(input.sequence_detected(&double_tap));
    }

    // === Gamepad Analog Axes Tests ===

    #[test]
    fn test_gamepad_axis_basic() {
        let mut input = InputManager::new();

        // Initially zero
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0);

        // Set axis value
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.5);
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.5);

        // Negative value
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftY, -0.75);
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftY), -0.75);
    }

    #[test]
    fn test_gamepad_axis_deadzone() {
        let mut input = InputManager::new();

        // Default deadzone is 0.1
        assert_eq!(input.analog_deadzone(), 0.1);

        // Values within deadzone should be zeroed
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.05);
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0);

        // Values outside deadzone should pass through
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.15);
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.15);
    }

    #[test]
    fn test_set_analog_deadzone() {
        let mut input = InputManager::new();

        // Set custom deadzone
        input.set_analog_deadzone(0.2);
        assert_eq!(input.analog_deadzone(), 0.2);

        // Value within new deadzone is zeroed
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.15);
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0);

        // Value outside new deadzone passes through
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.25);
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.25);
    }

    #[test]
    fn test_gamepad_left_stick() {
        let mut input = InputManager::new();

        // Initially zero
        assert_eq!(input.gamepad_left_stick(0), Vec2::zero());

        // Set stick values
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.8);
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftY, -0.6);

        let stick = input.gamepad_left_stick(0);
        assert_eq!(stick.x, 0.8);
        assert_eq!(stick.y, -0.6);
    }

    #[test]
    fn test_gamepad_right_stick() {
        let mut input = InputManager::new();

        // Set right stick values
        input.set_gamepad_axis(0, GamepadAxis::AxisRightX, -0.5);
        input.set_gamepad_axis(0, GamepadAxis::AxisRightY, 0.3);

        let stick = input.gamepad_right_stick(0);
        assert_eq!(stick.x, -0.5);
        assert_eq!(stick.y, 0.3);
    }

    #[test]
    fn test_gamepad_triggers() {
        let mut input = InputManager::new();

        // Triggers are normalized from -1.0..1.0 to 0.0..1.0
        // Set left trigger (axis value -1.0 = 0.0, axis value 1.0 = 1.0)
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftTrigger, -1.0);
        assert_eq!(input.gamepad_left_trigger(0), 0.0);

        input.set_gamepad_axis(0, GamepadAxis::AxisLeftTrigger, 1.0);
        assert_eq!(input.gamepad_left_trigger(0), 1.0);

        // Mid-press (axis 0.0 = trigger 0.5)
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftTrigger, 0.0);
        assert_eq!(input.gamepad_left_trigger(0), 0.5);

        // Right trigger
        input.set_gamepad_axis(0, GamepadAxis::AxisRightTrigger, 0.5);
        assert_eq!(input.gamepad_right_trigger(0), 0.75);
    }

    #[test]
    fn test_gamepad_axis_nonexistent_gamepad() {
        let input = InputManager::new();

        // Querying nonexistent gamepad returns 0.0 for axes
        assert_eq!(input.gamepad_axis(10, GamepadAxis::AxisLeftX), 0.0);
        assert_eq!(input.gamepad_left_stick(10), Vec2::zero());

        // Triggers normalize from -1.0..1.0 to 0.0..1.0
        // So axis value 0.0 (default) becomes trigger value 0.5
        assert_eq!(input.gamepad_left_trigger(10), 0.5);
        assert_eq!(input.gamepad_right_trigger(10), 0.5);
    }

    // === Gamepad Connection Tests ===

    #[test]
    fn test_gamepad_connection() {
        let mut input = InputManager::new();

        // Initially not connected
        assert!(!input.is_gamepad_connected(0));
        assert_eq!(input.connected_gamepad_count(), 0);

        // Connect gamepad 0
        input.set_gamepad_connected(0, true);
        assert!(input.is_gamepad_connected(0));
        assert_eq!(input.connected_gamepad_count(), 1);

        // Connect gamepad 1
        input.set_gamepad_connected(1, true);
        assert!(input.is_gamepad_connected(1));
        assert_eq!(input.connected_gamepad_count(), 2);

        // Disconnect gamepad 0
        input.set_gamepad_connected(0, false);
        assert!(!input.is_gamepad_connected(0));
        assert!(input.is_gamepad_connected(1));
        assert_eq!(input.connected_gamepad_count(), 1);
    }

    #[test]
    fn test_connected_gamepads_iterator() {
        let mut input = InputManager::new();

        input.set_gamepad_connected(0, true);
        input.set_gamepad_connected(2, true);
        input.set_gamepad_connected(4, true);

        let connected: Vec<_> = input.connected_gamepads().collect();
        assert_eq!(connected.len(), 3);
        assert!(connected.contains(&0));
        assert!(connected.contains(&2));
        assert!(connected.contains(&4));
    }

    #[test]
    fn test_gamepad_connection_nonexistent() {
        let input = InputManager::new();

        // Querying nonexistent gamepad returns false
        assert!(!input.is_gamepad_connected(10));
    }

    // === Gamepad Vibration Tests ===

    #[test]
    fn test_gamepad_vibration() {
        let mut input = InputManager::new();

        // Initially no vibration
        assert_eq!(input.gamepad_vibration(0), 0.0);

        // Set vibration
        input.set_gamepad_vibration(0, 0.75);
        assert_eq!(input.gamepad_vibration(0), 0.75);

        // Clamping to 0.0-1.0
        input.set_gamepad_vibration(0, 1.5);
        assert_eq!(input.gamepad_vibration(0), 1.0);

        input.set_gamepad_vibration(0, -0.5);
        assert_eq!(input.gamepad_vibration(0), 0.0);
    }

    #[test]
    fn test_stop_gamepad_vibration() {
        let mut input = InputManager::new();

        input.set_gamepad_vibration(0, 0.8);
        assert_eq!(input.gamepad_vibration(0), 0.8);

        input.stop_gamepad_vibration(0);
        assert_eq!(input.gamepad_vibration(0), 0.0);
    }

    #[test]
    fn test_stop_all_vibration() {
        let mut input = InputManager::new();

        input.set_gamepad_vibration(0, 0.5);
        input.set_gamepad_vibration(1, 0.7);
        input.set_gamepad_vibration(2, 0.9);

        input.stop_all_vibration();

        assert_eq!(input.gamepad_vibration(0), 0.0);
        assert_eq!(input.gamepad_vibration(1), 0.0);
        assert_eq!(input.gamepad_vibration(2), 0.0);
    }

    // === Gamepad Integration Tests ===

    #[test]
    fn test_gamepad_state_clear_preserves_connection() {
        let mut input = InputManager::new();

        // Set up gamepad state
        input.set_gamepad_connected(0, true);
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.5);
        input.press_gamepad_button(0, 1);
        input.set_gamepad_vibration(0, 0.8);

        // Clear should remove input state but preserve connection and vibration
        input.clear();

        assert!(input.is_gamepad_connected(0)); // Connection preserved
        assert_eq!(input.gamepad_vibration(0), 0.8); // Vibration preserved
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.0); // Axes cleared
        assert!(!input.gamepad_button_pressed(0, 1)); // Buttons cleared
    }

    #[test]
    fn test_gamepad_multiple_gamepads_axes() {
        let mut input = InputManager::new();

        // Set different values for different gamepads
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.5);
        input.set_gamepad_axis(1, GamepadAxis::AxisLeftX, -0.5);
        input.press_gamepad_button(0, 1);
        input.press_gamepad_button(1, 2);

        // Verify isolation
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.5);
        assert_eq!(input.gamepad_axis(1, GamepadAxis::AxisLeftX), -0.5);
        assert!(input.gamepad_button_pressed(0, 1));
        assert!(!input.gamepad_button_pressed(0, 2));
        assert!(input.gamepad_button_pressed(1, 2));
        assert!(!input.gamepad_button_pressed(1, 1));
    }

    #[test]
    fn test_gamepad_axes_update_persistence() {
        let mut input = InputManager::new();

        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.8);

        // Axes should persist across update (unlike deltas)
        input.update();
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.8);

        input.update();
        assert_eq!(input.gamepad_axis(0, GamepadAxis::AxisLeftX), 0.8);
    }

    #[test]
    fn test_gamepad_stick_magnitude() {
        let mut input = InputManager::new();

        // Set stick to diagonal (0.6, 0.8) - should have magnitude ~1.0
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftX, 0.6);
        input.set_gamepad_axis(0, GamepadAxis::AxisLeftY, 0.8);

        let stick = input.gamepad_left_stick(0);
        let magnitude = (stick.x * stick.x + stick.y * stick.y).sqrt();

        // Magnitude should be close to 1.0 (floating point precision)
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gamepad_expansion() {
        let mut input = InputManager::new();

        // Should automatically expand to support gamepad 10
        input.set_gamepad_axis(10, GamepadAxis::AxisLeftX, 0.5);
        assert_eq!(input.gamepad_axis(10, GamepadAxis::AxisLeftX), 0.5);

        input.set_gamepad_connected(10, true);
        assert!(input.is_gamepad_connected(10));
    }
}
