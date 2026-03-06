//! Collision detection algorithms for 2D physics.
//!
//! This module provides narrow-phase collision detection between various collider shapes.
//! Each collision function returns contact information when shapes are intersecting.
//!
//! # Supported Shape Pairs
//!
//! - Circle-Circle (fastest)
//! - Circle-AABB
//! - Circle-OBB
//! - AABB-AABB
//! - OBB-OBB (SAT algorithm)
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::collision::{circle_circle_collision, Contact};
//! use goud_engine::core::math::Vec2;
//!
//! let circle_a = Vec2::new(0.0, 0.0);
//! let radius_a = 1.0;
//! let circle_b = Vec2::new(1.5, 0.0);
//! let radius_b = 1.0;
//!
//! if let Some(contact) = circle_circle_collision(circle_a, radius_a, circle_b, radius_b) {
//!     println!("Circles colliding! Penetration: {}", contact.penetration);
//!     println!("Normal: {:?}", contact.normal);
//! }
//! ```
//!
//! # Collision Detection Pipeline
//!
//! 1. **Broad Phase**: Spatial hash identifies potentially colliding pairs (see [`crate::ecs::broad_phase`])
//! 2. **Narrow Phase**: This module's algorithms compute exact contact information
//! 3. **Response**: Physics system resolves collisions based on contact data
//!
//! # Performance Notes
//!
//! - Circle-circle is fastest (single distance check)
//! - AABB-AABB is very fast (no rotation)
//! - OBB-OBB uses SAT (more expensive but accurate)
//! - Early exits when no collision detected

pub mod contact;
pub mod detection_box;
pub mod detection_circle;
pub mod events;
pub mod response;

mod tests;

// Re-export public types at the collision module level to preserve the existing public API.
pub use contact::Contact;
pub use detection_box::{aabb_aabb_collision, box_box_collision};
pub use detection_circle::{circle_aabb_collision, circle_circle_collision, circle_obb_collision};
pub use events::{CollisionEnded, CollisionStarted};
pub use response::{compute_position_correction, resolve_collision, CollisionResponse};
