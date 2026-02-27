//! ECS System implementations.
//!
//! This module provides built-in systems for common game engine tasks:
//! - **Rendering**: 2D sprite rendering with batching
//! - **Physics**: Transform propagation, collision detection
//! - **Audio**: Spatial audio updates
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::ecs::systems::SpriteRenderSystem;
//! use goud_engine::ecs::Schedule;
//!
//! let mut schedule = Schedule::default();
//! schedule.add_system(SpriteRenderSystem::default());
//! ```

pub mod rendering;
pub mod transform;

pub use rendering::SpriteRenderSystem;
pub use transform::TransformPropagationSystem;
