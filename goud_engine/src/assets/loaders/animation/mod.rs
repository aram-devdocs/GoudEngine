//! Keyframe-based property animation asset loading.
//!
//! This module provides types for loading and managing keyframe animations
//! from `.anim.json` files. These are distinct from sprite-sheet animations
//! (see [`crate::ecs::components::sprite_animator`]).

pub mod asset;
#[cfg(feature = "native")]
pub mod gltf_parser;
pub mod keyframe;
pub mod loader;

pub use asset::KeyframeAnimation;
pub use loader::AnimationLoader;
