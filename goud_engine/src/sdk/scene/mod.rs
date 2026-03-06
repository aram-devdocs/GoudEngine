//! Scene management for multiple isolated ECS worlds.
//!
//! The [`SceneManager`] allows creating, destroying, and switching between
//! multiple [`World`](crate::ecs::World) instances. Each scene is fully
//! isolated: entities in one scene are invisible to another.
//!
//! A "default" scene is always created automatically and cannot be destroyed.

mod manager;

pub use manager::{SceneId, SceneManager, DEFAULT_SCENE_NAME};
