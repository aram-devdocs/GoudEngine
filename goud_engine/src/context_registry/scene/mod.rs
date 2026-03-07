//! Scene management for multiple isolated ECS worlds.
//!
//! The [`SceneManager`] allows creating, destroying, and switching between
//! multiple [`World`](crate::ecs::World) instances. Each scene is fully
//! isolated: entities in one scene are invisible to another.
//!
//! A "default" scene is always created automatically and cannot be destroyed.

pub mod data;
pub mod loading;
mod manager;
pub mod prefab;
pub mod prefab_asset;
pub mod serialization;

pub use data::*;
pub use loading::*;
pub use manager::{SceneId, SceneManager, DEFAULT_SCENE_NAME};
pub use prefab::*;
pub use prefab_asset::*;
pub use serialization::*;
