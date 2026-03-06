//! ECS System implementations.
//!
//! This module provides built-in systems for common game engine tasks:
//! - **Rendering**: 2D sprite rendering with batching (see [`crate::rendering`])
//! - **Physics**: Transform propagation, collision detection
//! - **Audio**: Spatial audio updates

pub mod transform;

pub use transform::TransformPropagationSystem;
