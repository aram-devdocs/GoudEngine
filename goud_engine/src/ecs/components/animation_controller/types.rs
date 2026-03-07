//! Animation controller types for state machine-based animation.

use std::collections::HashMap;

use crate::ecs::components::AnimationClip;
use crate::ecs::Component;

// =============================================================================
// AnimationState
// =============================================================================

/// A named animation state containing a clip to play.
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Unique name identifying this state.
    pub name: String,
    /// The animation clip to play while in this state.
    pub clip: AnimationClip,
}

// =============================================================================
// TransitionCondition
// =============================================================================

/// A condition that must be satisfied for a transition to fire.
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    /// Parameter `param` must equal `value`.
    BoolEquals {
        /// Parameter name.
        param: String,
        /// Expected value.
        value: bool,
    },
    /// Parameter `param` must be greater than `threshold`.
    FloatGreaterThan {
        /// Parameter name.
        param: String,
        /// Threshold value.
        threshold: f32,
    },
    /// Parameter `param` must be less than `threshold`.
    FloatLessThan {
        /// Parameter name.
        param: String,
        /// Threshold value.
        threshold: f32,
    },
}

// =============================================================================
// AnimationTransition
// =============================================================================

/// Defines a transition between two animation states.
#[derive(Debug, Clone)]
pub struct AnimationTransition {
    /// Source state name.
    pub from: String,
    /// Target state name.
    pub to: String,
    /// All conditions that must be met for this transition to fire.
    pub conditions: Vec<TransitionCondition>,
    /// Duration in seconds to blend between states.
    pub blend_duration: f32,
}

// =============================================================================
// AnimParam
// =============================================================================

/// A parameter value used to drive animation transitions.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimParam {
    /// Boolean parameter.
    Bool(bool),
    /// Floating-point parameter.
    Float(f32),
}

// =============================================================================
// TransitionProgress
// =============================================================================

/// Tracks an in-progress transition between states.
#[derive(Debug, Clone)]
pub struct TransitionProgress {
    /// State transitioning from.
    pub from_state: String,
    /// State transitioning to.
    pub to_state: String,
    /// Time elapsed in the transition.
    pub elapsed: f32,
    /// Total duration of the transition.
    pub duration: f32,
}

// =============================================================================
// AnimationController
// =============================================================================

/// ECS component that manages animation state machine logic.
///
/// The controller holds a set of named states, transitions between them,
/// and parameters that drive transition conditions. Pair with a
/// [`SpriteAnimator`](crate::ecs::components::SpriteAnimator) component
/// and the [`update_animation_controllers`](crate::ecs::systems::update_animation_controllers)
/// system.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::sprite_animator::AnimationClip;
/// use goud_engine::ecs::components::animation_controller::{
///     AnimationController, TransitionCondition,
/// };
/// use goud_engine::core::math::Rect;
///
/// let idle_clip = AnimationClip::new(
///     vec![Rect::new(0.0, 0.0, 32.0, 32.0)],
///     0.1,
/// );
/// let run_clip = AnimationClip::new(
///     vec![Rect::new(32.0, 0.0, 32.0, 32.0)],
///     0.1,
/// );
///
/// let controller = AnimationController::new("idle")
///     .with_state("idle", idle_clip)
///     .with_state("run", run_clip)
///     .with_transition(
///         "idle", "run", 0.1,
///         vec![TransitionCondition::BoolEquals {
///             param: "running".to_string(),
///             value: true,
///         }],
///     );
///
/// assert_eq!(controller.current_state_name(), "idle");
/// ```
#[derive(Debug, Clone)]
pub struct AnimationController {
    /// Named animation states.
    pub states: HashMap<String, AnimationState>,
    /// Transitions between states.
    pub transitions: Vec<AnimationTransition>,
    /// Parameters driving transition conditions.
    pub parameters: HashMap<String, AnimParam>,
    /// Name of the currently active state.
    pub current_state: String,
    /// Active transition, if any.
    pub transition_progress: Option<TransitionProgress>,
}

impl Component for AnimationController {}
