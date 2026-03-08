//! Transform2D component for 2D spatial transformations.
//!
//! The [`Transform2D`] component represents an entity's position, rotation, and scale
//! in 2D space. It is optimized for 2D games where entities exist on a flat plane.
//!
//! # Design Philosophy
//!
//! Transform2D uses a simpler representation than the 3D [`Transform`](crate::ecs::components::Transform):
//!
//! - **Position**: 2D vector (x, y)
//! - **Rotation**: Single angle in radians (counter-clockwise)
//! - **Scale**: 2D vector for non-uniform scaling
//!
//! This provides:
//! - **Simplicity**: No quaternions, just a rotation angle
//! - **Memory efficiency**: 20 bytes vs 40 bytes for Transform
//! - **Intuitive**: Rotation is a single value in radians
//! - **Performance**: Simpler matrix calculations for 2D
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::Transform2D;
//! use goud_engine::core::math::Vec2;
//! use std::f32::consts::PI;
//!
//! // Create a transform at position (100, 50)
//! let mut transform = Transform2D::from_position(Vec2::new(100.0, 50.0));
//!
//! // Modify the transform
//! transform.translate(Vec2::new(10.0, 0.0));
//! transform.rotate(PI / 4.0); // 45 degrees
//! transform.set_scale(Vec2::new(2.0, 2.0));
//!
//! // Get the transformation matrix for rendering
//! let matrix = transform.matrix();
//! ```
//!
//! # Coordinate System
//!
//! GoudEngine 2D uses a standard screen-space coordinate system:
//! - X axis: Right (positive)
//! - Y axis: Down (positive) or Up (positive) depending on camera
//!
//! Rotation is counter-clockwise when viewed from above (standard mathematical convention).
//!
//! # FFI Safety
//!
//! Transform2D is `#[repr(C)]` and can be safely passed across FFI boundaries.

mod delta;
pub(crate) mod mat3x3;
pub(crate) mod ops;
pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use mat3x3::Mat3x3;
pub use types::Transform2D;
