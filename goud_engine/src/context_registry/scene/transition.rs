//! Scene transition state machine.
//!
//! Tracks progress of a transition between two scenes, supporting
//! instant, fade, and custom transition types.

use super::SceneId;

/// The type of visual transition between scenes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransitionType {
    /// Scene switches immediately with no animation.
    Instant = 0,
    /// Scene fades out then fades in over the given duration.
    Fade = 1,
    /// A user-defined transition effect. The engine tracks progress; SDKs query it to render custom visuals.
    Custom = 2,
}

impl TryFrom<u8> for TransitionType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Instant),
            1 => Ok(Self::Fade),
            2 => Ok(Self::Custom),
            other => Err(other),
        }
    }
}

/// Tracks the in-progress state of a scene transition.
#[derive(Debug, Clone)]
pub struct TransitionState {
    /// Scene being transitioned away from.
    pub from_scene: SceneId,
    /// Scene being transitioned to.
    pub to_scene: SceneId,
    /// The visual transition style.
    pub transition_type: TransitionType,
    /// Total duration of the transition in seconds.
    pub duration: f32,
    /// Time elapsed so far in the transition.
    pub elapsed: f32,
}

impl TransitionState {
    /// Creates a new transition state.
    pub fn new(
        from_scene: SceneId,
        to_scene: SceneId,
        transition_type: TransitionType,
        duration: f32,
    ) -> Self {
        Self {
            from_scene,
            to_scene,
            transition_type,
            duration,
            elapsed: 0.0,
        }
    }

    /// Returns the progress of the transition as a value in `[0.0, 1.0]`.
    ///
    /// A duration of zero (instant transition) always returns `1.0`.
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Returns `true` when the transition has completed.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Advances the transition by the given delta time.
    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
    }
}

/// Emitted when a transition finishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionComplete {
    /// Scene that was left.
    pub from_scene: SceneId,
    /// Scene that is now active.
    pub to_scene: SceneId,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_type_try_from_valid() {
        assert_eq!(TransitionType::try_from(0), Ok(TransitionType::Instant));
        assert_eq!(TransitionType::try_from(1), Ok(TransitionType::Fade));
        assert_eq!(TransitionType::try_from(2), Ok(TransitionType::Custom));
    }

    #[test]
    fn test_transition_type_try_from_invalid() {
        assert_eq!(TransitionType::try_from(3), Err(3));
        assert_eq!(TransitionType::try_from(255), Err(255));
    }

    #[test]
    fn test_progress_zero_duration_returns_one() {
        let state = TransitionState::new(0, 1, TransitionType::Instant, 0.0);
        assert!((state.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_progress_half_way() {
        let mut state = TransitionState::new(0, 1, TransitionType::Fade, 2.0);
        state.tick(1.0);
        assert!((state.progress() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_progress_clamped_at_one() {
        let mut state = TransitionState::new(0, 1, TransitionType::Fade, 1.0);
        state.tick(5.0);
        assert!((state.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_complete_false_initially() {
        let state = TransitionState::new(0, 1, TransitionType::Fade, 1.0);
        assert!(!state.is_complete());
    }

    #[test]
    fn test_is_complete_true_after_duration() {
        let mut state = TransitionState::new(0, 1, TransitionType::Fade, 1.0);
        state.tick(1.0);
        assert!(state.is_complete());
    }

    #[test]
    fn test_instant_is_complete_immediately() {
        let state = TransitionState::new(0, 1, TransitionType::Instant, 0.0);
        assert!(state.is_complete());
    }

    #[test]
    fn test_tick_accumulates_time() {
        let mut state = TransitionState::new(0, 1, TransitionType::Fade, 2.0);
        state.tick(0.5);
        state.tick(0.3);
        assert!((state.elapsed - 0.8).abs() < f32::EPSILON);
    }
}
