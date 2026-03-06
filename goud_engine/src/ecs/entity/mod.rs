//! Entity identifiers for the ECS.
//!
//! Entities are lightweight identifiers that serve as keys for component data.
//! Unlike [`Handle<T>`](crate::core::handle::Handle), entities are not generic -
//! they identify an entity across all component storages rather than a specific
//! resource type.
//!
//! # Design Pattern: Generational Indices
//!
//! Entities use the same generational index pattern as handles:
//!
//! 1. Each entity has an index (slot in the entity array) and a generation
//! 2. When an entity is despawned, its generation increments
//! 3. Old entity references become stale (generation mismatch)
//! 4. The slot can be reused for new entities with the new generation
//!
//! This prevents "dangling entity" bugs where code holds a reference to a
//! despawned entity and accidentally accesses data from a new entity that
//! reused the same slot.
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::entity::Entity;
//!
//! // Entities are typically created by EntityAllocator, but can be constructed directly
//! let entity = Entity::new(0, 1);
//!
//! assert_eq!(entity.index(), 0);
//! assert_eq!(entity.generation(), 1);
//!
//! // PLACEHOLDER is a special sentinel value
//! assert!(Entity::PLACEHOLDER.is_placeholder());
//! ```
//!
//! # FFI Safety
//!
//! Entity uses `#[repr(C)]` for predictable memory layout across FFI boundaries.
//! The struct is exactly 8 bytes: 4 bytes for index + 4 bytes for generation.

mod allocator;
mod bulk;
mod types;

pub use allocator::EntityAllocator;
pub use types::Entity;

#[cfg(test)]
mod tests;
