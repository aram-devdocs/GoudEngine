//! Point-based spatial grid for non-physics proximity queries.
//!
//! Provides efficient O(1) insertion, removal, and update of entities by position,
//! with fast radius-based neighbor queries. Independent of the physics/collision
//! system.
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::spatial_grid::SpatialGrid;
//! use goud_engine::ecs::Entity;
//! use goud_engine::core::math::Vec2;
//!
//! let mut grid = SpatialGrid::new(32.0);
//!
//! let entity = Entity::new(0, 0);
//! grid.insert(entity, Vec2::new(100.0, 200.0));
//!
//! let nearby = grid.query_radius(Vec2::new(100.0, 200.0), 50.0);
//! assert!(nearby.contains(&entity));
//! ```

mod core;
mod grid;
mod queries;
#[cfg(test)]
mod tests;

pub use self::core::SpatialGrid;
