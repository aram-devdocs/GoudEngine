//! Query fetch traits for the ECS.
//!
//! This module defines the foundational traits for the query system. Queries
//! allow systems to access component data in a type-safe manner. The fetch
//! traits define what can be queried and how data is retrieved.
//!
//! # Architecture
//!
//! The query system is built around two key traits:
//!
//! - [`WorldQuery`]: Defines what data a query fetches and how to access it
//! - [`ReadOnlyWorldQuery`]: Marker trait for queries that don't mutate data
//!
//! # Query Item Lifetime
//!
//! The [`WorldQuery`] trait uses Generic Associated Types (GATs) for the `Item`
//! type. This allows the fetched item to have a lifetime tied to the world
//! borrow, enabling references to component data.
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::{Component, Entity, World};
//! use goud_engine::ecs::query::{WorldQuery, QueryState};
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // WorldQuery is implemented for &T to fetch component references
//! // (Implementation shown in fetch.rs Step 2.5.2)
//! ```
//!
//! # Design Rationale
//!
//! This design is inspired by Bevy's query system, adapted for GoudEngine's
//! needs:
//!
//! 1. **Type Safety**: The trait bounds ensure only valid queries compile
//! 2. **Performance**: State caching avoids repeated archetype lookups
//! 3. **Flexibility**: The generic design supports arbitrary query types
//! 4. **Parallel Safety**: ReadOnlyWorldQuery enables safe concurrent reads

mod access;
mod component_ref;
mod filters;
mod impls;
mod optional;
mod traits;

#[cfg(test)]
mod tests_mod;

pub use access::{
    Access, AccessConflict, AccessType, ConflictInfo, NonSendConflictInfo, ResourceConflictInfo,
};
pub use component_ref::{MutState, WriteAccess};
pub use filters::{With, Without};
pub use traits::{QueryState, ReadOnlyWorldQuery, WorldQuery};
