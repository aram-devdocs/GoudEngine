//! Sprite animation system.
//!
//! Provides [`update_sprite_animations`], a system function that advances
//! all [`SpriteAnimator`](crate::ecs::components::SpriteAnimator) components
//! each frame and applies the current frame's source rectangle to the
//! entity's [`Sprite`](crate::ecs::components::Sprite) component (if present).

pub mod blend;
mod system;

#[cfg(test)]
mod tests;

pub use blend::{blend_rects, compute_blended_rect, BlendMode};
pub use system::update_sprite_animations;
