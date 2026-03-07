//! 2D skeletal animation system.
//!
//! Provides two system functions that run each frame:
//!
//! - [`update_skeletal_animations`]: Advances animation time, samples keyframes,
//!   and propagates bone transforms through the hierarchy.
//! - [`deform_skeletal_meshes`]: Applies bone world transforms to mesh vertices
//!   via linear blend skinning.

pub mod interpolation;
mod system;

#[cfg(test)]
mod tests;

pub use system::{deform_skeletal_meshes, update_skeletal_animations};
