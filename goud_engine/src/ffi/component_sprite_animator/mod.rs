//! # FFI Functions for SpriteAnimator Component
//!
//! This module provides C-compatible functions for building animation clips
//! and querying sprite animator state. These functions allow language bindings
//! to construct and inspect `SpriteAnimator` components without duplicating
//! logic across SDKs.
//!
//! ## Design Philosophy
//!
//! All animation logic lives in Rust. SDKs (C#, Python, etc.) are thin
//! wrappers that marshal data and call these FFI functions.
//!
//! ## Module Layout
//!
//! - `factory`  -- clip builder new/add_frame/free and animator creation
//! - `playback` -- read-only queries (current_frame, is_playing, is_finished)

pub(crate) mod factory;
pub(crate) mod playback;

#[cfg(test)]
mod tests;

// Re-export types from core for backward compatibility
pub use crate::core::types::{FfiAnimationClipBuilder, FfiPlaybackMode, FfiSpriteAnimator};

// Re-export all public FFI functions so external consumers see a flat namespace.

// Factory / Builder
pub use factory::{
    goud_animation_clip_builder_add_frame, goud_animation_clip_builder_free,
    goud_animation_clip_builder_new, goud_sprite_animator_from_clip,
};

// Playback queries
pub use playback::{
    goud_sprite_animator_get_current_frame, goud_sprite_animator_is_finished,
    goud_sprite_animator_is_playing,
};
