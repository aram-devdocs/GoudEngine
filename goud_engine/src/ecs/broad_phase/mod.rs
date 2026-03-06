//! Broad phase collision detection using spatial hashing.
//!
//! The broad phase is the first step of collision detection. It efficiently identifies
//! pairs of objects that *might* be colliding, filtering out objects that are too far
//! apart to possibly collide. The narrow phase then performs precise collision tests
//! on these candidate pairs.
//!
//! # Spatial Hash
//!
//! A spatial hash divides space into a uniform grid of cells. Each object is assigned
//! to one or more cells based on its AABB. Objects in the same cell are potential
//! collision pairs.
//!
//! Benefits:
//! - O(1) insertion and removal
//! - O(n) query for nearby objects (where n = objects per cell)
//! - Simple implementation
//! - Good cache locality
//! - Predictable performance
//!
//! Trade-offs:
//! - Struggles with objects of vastly different sizes
//! - Uniform grid doesn't adapt to object distribution
//! - Memory usage proportional to covered area
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::broad_phase::SpatialHash;
//! use goud_engine::ecs::Entity;
//! use goud_engine::core::math::Rect;
//!
//! // Create spatial hash with 64-pixel cells
//! let mut hash = SpatialHash::new(64.0);
//!
//! // Insert entities with their AABBs
//! let entity = Entity::new(0, 0);
//! let aabb = Rect::new(0.0, 0.0, 32.0, 32.0);
//! hash.insert(entity, aabb);
//!
//! // Query for potential collisions
//! let pairs = hash.query_pairs();
//! for (a, b) in pairs {
//!     // Perform narrow phase collision test on (a, b)
//! }
//! ```

mod grid;
mod queries;
mod spatial_hash;
mod stats;
mod tests;

pub use spatial_hash::SpatialHash;
pub use stats::SpatialHashStats;
