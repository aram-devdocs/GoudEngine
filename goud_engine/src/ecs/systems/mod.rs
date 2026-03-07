//! ECS System implementations.
//!
//! This module provides built-in systems for common game engine tasks:
//! - **Rendering**: 2D sprite rendering with batching (see [`crate::rendering`])
//! - **Physics**: Transform propagation, collision detection
//! - **Audio**: Spatial audio updates

pub mod animation;
pub mod animation_controller;
pub mod skeletal_animation;
pub mod transform;

pub use animation::update_sprite_animations;
pub use animation_controller::update_animation_controllers;
pub use skeletal_animation::{deform_skeletal_meshes, update_skeletal_animations};
pub use transform::TransformPropagationSystem;
