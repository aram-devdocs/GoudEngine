//! Animation layer stack component for multi-layer blended animation.
//!
//! An [`AnimationLayerStack`] holds multiple [`AnimationLayer`] instances,
//! each with independent playback and a blend weight. The animation system
//! combines the layers each frame to produce a single output rect.

mod ops;
mod types;

pub use types::{AnimationLayer, AnimationLayerStack};
