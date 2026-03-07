//! Sprite animation component for frame-by-frame sprite sheet animation.
//!
//! This module provides [`SpriteAnimator`], an ECS component that advances
//! through a sequence of source rectangles ([`AnimationClip`]) to animate
//! a sprite sheet. Pair it with a [`Sprite`](crate::ecs::components::Sprite)
//! component and the [`update_sprite_animations`](crate::ecs::systems::update_sprite_animations)
//! system.
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::components::sprite_animator::{
//!     SpriteAnimator, AnimationClip, PlaybackMode,
//! };
//! use goud_engine::core::math::Rect;
//!
//! let clip = AnimationClip::new(
//!     vec![
//!         Rect::new(0.0, 0.0, 32.0, 32.0),
//!         Rect::new(32.0, 0.0, 32.0, 32.0),
//!     ],
//!     0.1,
//! );
//!
//! let animator = SpriteAnimator::new(clip);
//! assert!(animator.playing);
//! ```

mod component;

#[cfg(test)]
mod tests;

pub use component::{AnimationClip, PlaybackMode, SpriteAnimator};
