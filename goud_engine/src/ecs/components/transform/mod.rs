//! Transform component for 3D spatial transformations.
//!
//! The [`Transform`] component represents an entity's position, rotation, and scale
//! in 3D space. It is one of the most commonly used components in game development,
//! essential for positioning and orienting objects in the world.
//!
//! # Design Philosophy
//!
//! Transform stores position, rotation (as quaternion), and scale separately rather
//! than as a combined matrix. This provides:
//!
//! - **Intuitive manipulation**: Modify position/rotation/scale independently
//! - **Numerical stability**: Quaternions avoid gimbal lock and numerical drift
//! - **Memory efficiency**: 10 floats vs 16 for a full matrix
//! - **Interpolation support**: Easy lerp/slerp for animations
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::Transform;
//! use goud_engine::core::math::Vec3;
//!
//! // Create a transform at position (10, 5, 0)
//! let mut transform = Transform::from_position(Vec3::new(10.0, 5.0, 0.0));
//!
//! // Modify the transform
//! transform.translate(Vec3::new(1.0, 0.0, 0.0));
//! transform.rotate_y(std::f32::consts::PI / 4.0);
//! transform.set_scale(Vec3::new(2.0, 2.0, 2.0));
//!
//! // Get the transformation matrix for rendering
//! let matrix = transform.matrix();
//! ```
//!
//! # Coordinate System
//!
//! GoudEngine uses a right-handed coordinate system:
//! - X axis: Right
//! - Y axis: Up
//! - Z axis: Out of the screen (towards viewer)
//!
//! Rotations follow the right-hand rule: positive rotation around an axis
//! goes counter-clockwise when looking down that axis.
//!
//! # FFI Safety
//!
//! Transform is `#[repr(C)]` and can be safely passed across FFI boundaries.
//! The quaternion is stored as (x, y, z, w) to match common conventions.

pub mod core;
pub mod ops;
pub mod quat;

mod tests;
mod tests_spatial;

pub use core::Transform;
pub use quat::Quat;
