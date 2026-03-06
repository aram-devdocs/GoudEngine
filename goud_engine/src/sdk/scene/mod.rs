//! Scene management re-exports.
//!
//! The scene management implementation lives in [`crate::core::scene`].
//! This module re-exports those types for backward compatibility with
//! existing SDK consumers.

pub use crate::core::scene::{SceneId, SceneManager, DEFAULT_SCENE_NAME};
