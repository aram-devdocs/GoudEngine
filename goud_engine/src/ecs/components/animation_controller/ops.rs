//! Builder and helper methods for [`AnimationController`].

use std::collections::HashMap;

use crate::ecs::components::AnimationClip;

use super::types::{
    AnimParam, AnimationController, AnimationState, AnimationTransition, TransitionCondition,
};

impl AnimationController {
    /// Creates a new controller with the given initial state name.
    ///
    /// The controller starts with no states, transitions, or parameters.
    /// Use the builder methods to add them.
    ///
    /// **Precondition:** `initial_state` should name a state that will be
    /// registered via [`with_state`](Self::with_state) before the controller
    /// is used. If the state is never added, lookups such as
    /// [`current_clip`](Self::current_clip) will return `None`.
    #[inline]
    pub fn new(initial_state: &str) -> Self {
        Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            parameters: HashMap::new(),
            current_state: initial_state.to_string(),
            transition_progress: None,
        }
    }

    /// Adds an animation state with the given name and clip (builder pattern).
    #[inline]
    pub fn with_state(mut self, name: &str, clip: AnimationClip) -> Self {
        self.states
            .insert(name.to_string(), AnimationState { clip });
        self
    }

    /// Adds a transition between two states (builder pattern).
    #[inline]
    pub fn with_transition(
        mut self,
        from: &str,
        to: &str,
        blend_duration: f32,
        conditions: Vec<TransitionCondition>,
    ) -> Self {
        self.transitions.push(AnimationTransition {
            from: from.to_string(),
            to: to.to_string(),
            conditions,
            blend_duration,
        });
        self
    }

    /// Sets a boolean parameter value.
    #[inline]
    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.parameters
            .insert(name.to_string(), AnimParam::Bool(value));
    }

    /// Sets a float parameter value.
    #[inline]
    pub fn set_float(&mut self, name: &str, value: f32) {
        self.parameters
            .insert(name.to_string(), AnimParam::Float(value));
    }

    /// Returns a reference to the parameter with the given name, if it exists.
    #[inline]
    pub fn get_param(&self, name: &str) -> Option<&AnimParam> {
        self.parameters.get(name)
    }

    /// Returns the name of the current state.
    #[inline]
    pub fn current_state_name(&self) -> &str {
        &self.current_state
    }

    /// Returns a reference to the current state's animation clip, if the
    /// current state exists in the states map.
    #[inline]
    pub fn current_clip(&self) -> Option<&AnimationClip> {
        self.states.get(&self.current_state).map(|s| &s.clip)
    }
}
