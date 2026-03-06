//! Collider component for physics collision detection.
//!
//! The [`Collider`] component defines the collision shape for an entity in the physics
//! simulation. It works in conjunction with [`RigidBody`](crate::ecs::components::RigidBody) to enable collision detection
//! and response.
//!
//! # Collision Shapes
//!
//! GoudEngine supports the following 2D collision shapes:
//!
//! - **Circle**: Defined by a radius, fastest collision detection
//! - **Box**: Axis-aligned or rotated rectangle (AABB or OBB)
//! - **Capsule**: Rounded rectangle, good for characters
//! - **Polygon**: Convex polygons for complex shapes
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::{Collider, ColliderShape};
//! use goud_engine::core::math::Vec2;
//!
//! // Create a circle collider (player, ball)
//! let ball = Collider::circle(0.5)
//!     .with_restitution(0.8) // Bouncy
//!     .with_friction(0.2);
//!
//! // Create a box collider (walls, platforms)
//! let wall = Collider::aabb(Vec2::new(10.0, 1.0))
//!     .with_friction(0.5);
//!
//! // Create a capsule collider (character controller)
//! let player = Collider::capsule(0.3, 1.0)
//!     .with_friction(0.0) // Frictionless movement
//!     .with_is_sensor(true); // Trigger only
//! ```
//!
//! # Collision Layers
//!
//! Colliders support layer-based filtering to control which objects can collide:
//!
//! ```
//! use goud_engine::ecs::components::Collider;
//!
//! // Player collides with enemies and walls
//! let player = Collider::circle(0.5)
//!     .with_layer(0b0001)
//!     .with_mask(0b0110);
//!
//! // Enemy collides with player and walls
//! let enemy = Collider::circle(0.4)
//!     .with_layer(0b0010)
//!     .with_mask(0b0101);
//! ```
//!
//! # Sensors (Triggers)
//!
//! Set `is_sensor` to true to create a trigger volume that detects collisions but
//! doesn't produce collision response:
//!
//! ```
//! use goud_engine::ecs::components::Collider;
//! use goud_engine::core::math::Vec2;
//!
//! // Pickup item trigger
//! let pickup = Collider::aabb(Vec2::new(1.0, 1.0))
//!     .with_is_sensor(true);
//! ```
//!
//! # Integration with RigidBody
//!
//! Colliders work with RigidBody components:
//!
//! - **Dynamic bodies**: Full collision detection and response
//! - **Kinematic bodies**: Collision detection only (no response)
//! - **Static bodies**: Acts as immovable obstacles
//!
//! # Thread Safety
//!
//! Collider is `Send + Sync` and can be safely used in parallel systems.

mod component;
mod shape;

/// Utilities for Axis-Aligned Bounding Box (AABB) calculations and operations.
///
/// These functions are used for broad-phase collision detection, spatial queries,
/// and efficient geometric tests.
pub mod aabb {
    pub use super::aabb_utils::*;
}

mod aabb_utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod aabb_tests;

pub use component::Collider;
pub use shape::ColliderShape;
