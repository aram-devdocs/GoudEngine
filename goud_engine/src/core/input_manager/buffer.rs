//! Input buffering methods for sequence and combo detection.

use std::time::Instant;

use super::manager::InputManager;
use super::types::InputBinding;

impl InputManager {
    // === Input Buffering ===

    /// Returns the current buffer duration.
    pub fn buffer_duration(&self) -> std::time::Duration {
        self.buffer_duration
    }

    /// Sets the buffer duration for input sequences.
    ///
    /// This determines how long inputs are remembered for combo detection.
    pub fn set_buffer_duration(&mut self, duration: std::time::Duration) {
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
}
