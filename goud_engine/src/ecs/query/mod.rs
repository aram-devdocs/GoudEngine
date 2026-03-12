//! Query system for the ECS.
//!
//! The query module provides type-safe access to component data stored in the
//! [`World`](crate::ecs::World). Queries are the primary way systems interact
//! with entities and their components.
//!
//! # Architecture
//!
//! The query system is built around several key concepts:
//!
//! - **WorldQuery**: Trait that defines what data a query fetches
//! - **Query**: The main query type used in systems
//! - **QueryState**: Cached state for efficient query execution
//! - **Filters**: Types like `With<T>` and `Without<T>` that filter entities
//!
//! # Basic Usage
//!
//! ```
//! use goud_engine::ecs::{World, Component, Entity};
//! use goud_engine::ecs::query::{Query, WorldQuery, With, Without};
//!
//! // Define components
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Velocity { x: f32, y: f32 }
//! impl Component for Velocity {}
//!
//! // Create and use a query
//! let mut world = World::new();
//! let entity = world.spawn_empty();
//! world.insert(entity, Position { x: 1.0, y: 2.0 });
//!
//! // In a system, queries are obtained via SystemParam
//! // For direct use:
//! let mut query: Query<&Position> = Query::new(&world);
//! for pos in query.iter(&world) {
//!     println!("Position: {:?}", pos);
//! }
//! ```
//!
//! # Query Types
//!
//! ## Data Queries
//!
//! - `Entity` - Returns the entity ID itself
//! - `&T` - Immutable reference to component T
//! - `&mut T` - Mutable reference to component T
//! - `Option<Q>` - Optional query, matches even if inner doesn't (future)
//!
//! ## Filters
//!
//! - `With<T>` - Match entities that have component T
//! - `Without<T>` - Match entities that don't have component T
//!
//! # Access Conflict Detection
//!
//! The query system tracks read and write access to prevent data races:
//!
//! - `&T` marks a read access to component T
//! - `&mut T` marks a write access to component T
//! - Two queries conflict if one writes a component the other reads/writes
//!
//! Use the [`Access`] type to build and check access patterns.
//!
//! # Using Query as a System Parameter
//!
//! `Query<Q, F>` implements `SystemParam`, allowing it to be used as a
//! function system parameter:
//!
//! ```ignore
//! fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
//!     for (mut pos, vel) in query.iter_mut() {
//!         pos.x += vel.x;
//!         pos.y += vel.y;
//!     }
//! }
//! ```
//!
//! # Design Principles
//!
//! 1. **Type Safety**: Query types checked at compile time
//! 2. **Performance**: State caching avoids runtime lookups
//! 3. **Flexibility**: Compose queries with tuples and filters
//! 4. **Parallel Safety**: Read/write access tracked for safe parallelism

pub mod cache;
pub mod fetch;
pub mod iter;
pub mod param;
pub mod query_type;

// Re-export cache type
pub use cache::QueryArchetypeCache;

// Re-export fetch types
pub use fetch::{
    Access, AccessConflict, AccessType, Added, Changed, ConflictInfo, MutState,
    NonSendConflictInfo, QueryState, ReadOnlyWorldQuery, ResourceConflictInfo, With, Without,
    WorldQuery, WriteAccess,
};

// Re-export the main Query type
pub use query_type::Query;

// Re-export iterators
pub use iter::{QueryIter, QueryIterMut};

// Re-export system param state
pub use param::QuerySystemParamState;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_advanced;
#[cfg(test)]
mod tests_cache;
