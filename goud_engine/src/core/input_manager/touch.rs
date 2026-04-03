//! Touch input methods for `InputManager`.

use crate::core::math::Vec2;
use crate::core::providers::input_types::{MouseButton, TouchPhase};

use super::manager::InputManager;
use super::types::TouchState;

impl InputManager {
    /// Records a new touch starting at the given position.
    pub fn touch_start(&mut self, id: u64, position: Vec2) {
        self.touches_current.insert(
            id,
            TouchState {
                position,
                previous_position: position,
                phase: TouchPhase::Started,
            },
        );

        // Pointer emulation: first touch acts as mouse left button
        if self.touch_pointer_emulation && id == 0 {
            self.set_mouse_position(position);
            self.press_mouse_button(MouseButton::Left);
        }
    }

    /// Updates the position of an active touch.
    pub fn touch_move(&mut self, id: u64, position: Vec2) {
        if let Some(state) = self.touches_current.get_mut(&id) {
            state.previous_position = state.position;
            state.position = position;
            state.phase = TouchPhase::Moved;
        }

        if self.touch_pointer_emulation && id == 0 {
            self.set_mouse_position(position);
        }
    }

    /// Records a touch ending (finger lifted).
    pub fn touch_end(&mut self, id: u64) {
        if let Some(state) = self.touches_current.get_mut(&id) {
            state.phase = TouchPhase::Ended;
        }

        if self.touch_pointer_emulation && id == 0 {
            self.release_mouse_button(MouseButton::Left);
        }
    }

    /// Records a touch cancellation (system cancelled, e.g. palm rejection).
    pub fn touch_cancel(&mut self, id: u64) {
        if let Some(state) = self.touches_current.get_mut(&id) {
            state.phase = TouchPhase::Cancelled;
        }

        if self.touch_pointer_emulation && id == 0 {
            self.release_mouse_button(MouseButton::Left);
        }
    }

    /// Returns `true` if the given touch ID is currently active.
    pub fn touch_active(&self, id: u64) -> bool {
        self.touches_current
            .get(&id)
            .map(|s| s.phase != TouchPhase::Ended && s.phase != TouchPhase::Cancelled)
            .unwrap_or(false)
    }

    /// Returns the position of the given touch, or `None` if not active.
    pub fn touch_position(&self, id: u64) -> Option<Vec2> {
        self.touches_current.get(&id).map(|s| s.position)
    }

    /// Returns the movement delta of the given touch since last frame.
    pub fn touch_delta(&self, id: u64) -> Vec2 {
        self.touches_current
            .get(&id)
            .map(|s| s.position - s.previous_position)
            .unwrap_or(Vec2::zero())
    }

    /// Returns `true` if the touch began this frame (not active last frame).
    pub fn touch_just_pressed(&self, id: u64) -> bool {
        self.touches_current.contains_key(&id) && !self.touches_previous.contains_key(&id)
    }

    /// Returns `true` if the touch ended this frame (was active last frame).
    pub fn touch_just_released(&self, id: u64) -> bool {
        !self.touch_active(id)
            && self
                .touches_previous
                .get(&id)
                .map(|s| s.phase != TouchPhase::Ended && s.phase != TouchPhase::Cancelled)
                .unwrap_or(false)
    }

    /// Returns the number of currently active touches.
    pub fn touch_count(&self) -> usize {
        self.touches_current
            .values()
            .filter(|s| s.phase != TouchPhase::Ended && s.phase != TouchPhase::Cancelled)
            .count()
    }

    /// Enables or disables touch-to-pointer emulation.
    ///
    /// When enabled (default), touch ID 0 is mapped to `MouseButton::Left`
    /// and sets the mouse position, allowing existing mouse-based game code
    /// to work on touch devices without modification.
    pub fn set_touch_pointer_emulation(&mut self, enabled: bool) {
        self.touch_pointer_emulation = enabled;
    }
}
