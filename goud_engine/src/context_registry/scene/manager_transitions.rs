//! Scene transition methods on [`SceneManager`].

use super::manager::{SceneId, SceneManager};
use crate::context_registry::scene::transition::{
    TransitionComplete, TransitionState, TransitionType,
};
use crate::core::error::GoudError;

impl SceneManager {
    // =========================================================================
    // Scene Transitions
    // =========================================================================

    /// Starts a transition from one scene to another.
    ///
    /// Both scenes must exist and no transition may already be in progress.
    /// For [`TransitionType::Instant`], the duration is forced to zero.
    /// The target scene is activated if it is not already active.
    pub fn start_transition(
        &mut self,
        from: SceneId,
        to: SceneId,
        transition_type: TransitionType,
        duration: f32,
    ) -> Result<(), GoudError> {
        if !self.scene_exists(from) {
            return Err(GoudError::ResourceNotFound(format!(
                "Source scene id {} not found",
                from
            )));
        }
        if !self.scene_exists(to) {
            return Err(GoudError::ResourceNotFound(format!(
                "Target scene id {} not found",
                to
            )));
        }
        if self.active_transition.is_some() {
            return Err(GoudError::InvalidState(
                "A transition is already in progress".to_string(),
            ));
        }
        if duration < 0.0 {
            return Err(GoudError::InvalidState(
                "Transition duration must be >= 0".to_string(),
            ));
        }

        let actual_duration = if transition_type == TransitionType::Instant {
            0.0
        } else {
            duration
        };

        // Ensure the target scene is active so both scenes participate.
        if !self.active_scenes.contains(&to) {
            self.active_scenes.push(to);
        }

        self.active_transition = Some(TransitionState::new(
            from,
            to,
            transition_type,
            actual_duration,
        ));

        Ok(())
    }

    /// Advances the active transition by `delta_time` seconds.
    ///
    /// Returns [`Some(TransitionComplete)`] when the transition finishes.
    /// On completion the source scene is deactivated and the transition
    /// state is cleared.
    pub fn tick_transition(&mut self, delta_time: f32) -> Option<TransitionComplete> {
        let transition = self.active_transition.as_mut()?;
        transition.tick(delta_time);

        if transition.is_complete() {
            let from = transition.from_scene;
            let to = transition.to_scene;
            self.active_scenes.retain(|&s| s != from);
            self.active_transition = None;
            Some(TransitionComplete {
                from_scene: from,
                to_scene: to,
            })
        } else {
            None
        }
    }

    /// Returns the progress of the active transition in `[0.0, 1.0]`,
    /// or `None` if no transition is in progress.
    pub fn transition_progress(&self) -> Option<f32> {
        self.active_transition.as_ref().map(|t| t.progress())
    }

    /// Returns `true` if a transition is currently in progress.
    pub fn is_transitioning(&self) -> bool {
        self.active_transition.is_some()
    }

    /// Returns a reference to the active transition state, if any.
    pub fn active_transition(&self) -> Option<&TransitionState> {
        self.active_transition.as_ref()
    }
}
