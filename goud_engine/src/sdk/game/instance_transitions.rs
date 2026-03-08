//! Scene transition convenience methods for [`GoudGame`].

use crate::context_registry::scene::transition::{TransitionComplete, TransitionType};
use crate::context_registry::scene::SceneId;
use crate::core::error::GoudError;

use super::GoudGame;

impl GoudGame {
    /// Starts a transition from one scene to another.
    pub fn transition_to(
        &mut self,
        from: SceneId,
        to: SceneId,
        transition_type: TransitionType,
        duration: f32,
    ) -> Result<(), GoudError> {
        self.scene_manager
            .start_transition(from, to, transition_type, duration)
    }

    /// Returns `true` if a scene transition is currently in progress.
    #[inline]
    pub fn is_transitioning(&self) -> bool {
        self.scene_manager.is_transitioning()
    }

    /// Returns the progress of the active transition in `[0.0, 1.0]`.
    #[inline]
    pub fn transition_progress(&self) -> Option<f32> {
        self.scene_manager.transition_progress()
    }

    /// Takes the most recent transition completion result, if any.
    ///
    /// Returns `Some(TransitionComplete)` exactly once after a transition
    /// finishes, then `None` until the next transition completes.
    #[inline]
    pub fn take_transition_complete(&mut self) -> Option<TransitionComplete> {
        self.last_transition_complete.take()
    }
}
