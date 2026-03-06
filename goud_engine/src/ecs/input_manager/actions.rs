//! Action mapping methods for semantic input handling.

use super::manager::InputManager;
use super::types::InputBinding;

impl InputManager {
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
