//! Scene management re-exports.
//!
//! The scene management implementation lives in [`crate::context_registry::scene`].
//! This module re-exports those types for backward compatibility with
//! existing SDK consumers.

pub use crate::context_registry::scene::{SceneId, SceneManager, DEFAULT_SCENE_NAME};
