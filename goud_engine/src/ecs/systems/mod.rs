//! ECS System implementations.
//!
//! This module provides built-in systems for common game engine tasks:
//! - **Rendering**: 2D sprite rendering with batching (see [`crate::rendering`])
//! - **Physics**: Transform propagation, collision detection
//! - **Audio**: Spatial audio updates

pub mod animation;
pub mod animation_controller;
pub mod physics_sync_2d;
pub mod physics_sync_3d;
pub mod skeletal_animation;
pub mod transform;

pub use animation::{blend_rects, compute_blended_rect, update_sprite_animations, BlendMode};
pub use animation_controller::update_animation_controllers;
pub use physics_sync_2d::{PhysicsHandleMap2D, PhysicsStepSystem2D};
pub use physics_sync_3d::{PhysicsHandleMap3D, PhysicsStepSystem3D};
pub use skeletal_animation::{deform_skeletal_meshes, update_skeletal_animations};
pub use transform::TransformPropagationSystem;
