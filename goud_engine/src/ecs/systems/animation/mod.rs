//! Sprite animation system.
//!
//! Provides [`update_sprite_animations`], a system function that advances
//! all [`SpriteAnimator`](crate::ecs::components::SpriteAnimator) components
//! each frame and applies the current frame's source rectangle to the
//! entity's [`Sprite`](crate::ecs::components::Sprite) component (if present).

mod system;

#[cfg(test)]
mod tests;

pub use system::update_sprite_animations;
