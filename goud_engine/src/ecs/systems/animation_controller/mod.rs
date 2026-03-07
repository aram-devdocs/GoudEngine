//! Animation controller system.
//!
//! Provides [`update_animation_controllers`], a system function that evaluates
//! animation state machine transitions and updates
//! [`SpriteAnimator`](crate::ecs::components::SpriteAnimator) clips accordingly.

mod system;

#[cfg(test)]
mod tests;

pub use system::update_animation_controllers;
