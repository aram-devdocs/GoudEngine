//! GlobalTransform component for world-space transformations.
//!
//! The [`GlobalTransform`] component stores the computed world-space transformation
//! for entities in a hierarchy. Unlike
//! [`Transform`](crate::ecs::components::Transform) which stores local-space data
//! relative to the parent, `GlobalTransform` stores the absolute world-space result.
//!
//! # Purpose
//!
//! When entities are arranged in a parent-child hierarchy, each child's
//! [`Transform`](crate::ecs::components::Transform)
//! is relative to its parent. To render, perform physics, or do other world-space
//! operations, we need the final world-space transformation.
//!
//! For example:
//! - Parent at position (10, 0, 0)
//! - Child at local position (5, 0, 0)
//! - Child's world position is (15, 0, 0)
//!
//! The transform propagation system computes these world-space values and stores
//! them in `GlobalTransform`.
//!
//! # Usage
//!
//! `GlobalTransform` is typically:
//! 1. Added automatically when spawning entities with `Transform`
//! 2. Updated by the transform propagation system each frame
//! 3. Read by rendering systems, physics, etc.
//!
//! **Never modify `GlobalTransform` directly.** Always modify `Transform` and let
//! the propagation system compute the global value.
//!
//! ```
//! use goud_engine::ecs::components::{Transform, GlobalTransform};
//! use goud_engine::core::math::Vec3;
//!
//! // Create local transform
//! let local = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
//!
//! // GlobalTransform would be computed by the propagation system
//! // For a root entity, it equals the local transform
//! let global = GlobalTransform::from(local);
//!
//! assert_eq!(global.translation(), Vec3::new(5.0, 0.0, 0.0));
//! ```
//!
//! # Memory Layout
//!
//! GlobalTransform stores a pre-computed 4x4 affine transformation matrix (64 bytes).
//! While this uses more memory than Transform (40 bytes), it provides:
//!
//! - **Direct use**: Matrix can be sent to GPU without further computation
//! - **Composability**: Easy to combine with parent transforms
//! - **Decomposability**: Can extract position/rotation/scale when needed
//!
//! # FFI Safety
//!
//! GlobalTransform uses cgmath's `Matrix4<f32>` internally which is column-major.
//! For FFI, use the `to_cols_array` method to get a flat `[f32; 16]` array.

mod core;
mod decomposition;
mod operations;

#[cfg(test)]
mod tests;

pub use core::GlobalTransform;
