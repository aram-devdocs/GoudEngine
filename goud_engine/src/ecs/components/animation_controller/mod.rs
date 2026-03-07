//! Animation controller component for state machine-driven animation.
//!
//! The [`AnimationController`] manages transitions between named animation
//! states based on parameter conditions. Pair it with a
//! [`SpriteAnimator`](crate::ecs::components::SpriteAnimator) and the
//! [`update_animation_controllers`](crate::ecs::systems::update_animation_controllers)
//! system to drive sprite animations via a state machine.

mod ops;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    AnimParam, AnimationController, AnimationState, AnimationTransition, TransitionCondition,
    TransitionProgress,
};
