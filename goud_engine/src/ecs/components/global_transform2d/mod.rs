//! GlobalTransform2D component for 2D world-space transformations.
//!
//! The [`GlobalTransform2D`] component stores the computed world-space transformation
//! for 2D entities in a hierarchy. Unlike
//! [`Transform2D`](crate::ecs::components::Transform2D) which stores local-space data
//! relative to the parent, `GlobalTransform2D` stores the absolute world-space result.
//!
//! # Purpose
//!
//! When entities are arranged in a parent-child hierarchy, each child's
//! [`Transform2D`](crate::ecs::components::Transform2D)
//! is relative to its parent. To render, perform physics, or do other world-space
//! operations, we need the final world-space transformation.
//!
//! For example:
//! - Parent at position (100, 0)
//! - Child at local position (50, 0)
//! - Child's world position is (150, 0)
//!
//! The 2D transform propagation system computes these world-space values and stores
//! them in `GlobalTransform2D`.
//!
//! # Usage
//!
//! `GlobalTransform2D` is typically:
//! 1. Added automatically when spawning entities with `Transform2D`
//! 2. Updated by the 2D transform propagation system each frame
//! 3. Read by rendering systems, physics, etc.
//!
//! **Never modify `GlobalTransform2D` directly.** Always modify `Transform2D` and let
//! the propagation system compute the global value.
//!
//! ```
//! use goud_engine::ecs::components::{Transform2D, GlobalTransform2D};
//! use goud_engine::core::math::Vec2;
//!
//! // Create local transform
//! let local = Transform2D::from_position(Vec2::new(50.0, 0.0));
//!
//! // GlobalTransform2D would be computed by the propagation system
//! // For a root entity, it equals the local transform
//! let global = GlobalTransform2D::from(local);
//!
//! assert!((global.translation() - Vec2::new(50.0, 0.0)).length() < 0.001);
//! ```
//!
//! # Memory Layout
//!
//! GlobalTransform2D stores a pre-computed 3x3 affine transformation matrix (36 bytes).
//! While this uses more memory than Transform2D (20 bytes), it provides:
//!
//! - **Direct use**: Matrix can be sent to GPU without further computation
//! - **Composability**: Easy to combine with parent transforms
//! - **Decomposability**: Can extract position/rotation/scale when needed
//!
//! # FFI Safety
//!
//! GlobalTransform2D is `#[repr(C)]` and can be safely passed across FFI boundaries.

mod impls;
mod operations;
mod types;

#[cfg(test)]
mod tests;

pub use types::GlobalTransform2D;
