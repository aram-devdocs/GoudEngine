//! Archetype system for grouping entities with identical component sets.
//!
//! Archetypes are a key optimization in ECS. Entities with the same set of
//! components share an archetype, enabling efficient iteration and storage.
//!
//! # Architecture
//!
//! - **ArchetypeId**: Unique identifier for an archetype
//! - **Archetype**: Stores entities with identical component sets
//! - **ArchetypeGraph**: Manages archetype relationships for component transitions
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::archetype::ArchetypeId;
//!
//! // The EMPTY archetype contains entities with no components
//! let empty = ArchetypeId::EMPTY;
//! assert_eq!(empty.index(), 0);
//!
//! // Custom archetype IDs
//! let arch = ArchetypeId::new(42);
//! assert_eq!(arch.index(), 42);
//! ```

mod archetype_id;
mod graph;
mod storage;

#[cfg(test)]
mod tests_archetype;
#[cfg(test)]
mod tests_archetype_id;
#[cfg(test)]
mod tests_graph;

pub use archetype_id::ArchetypeId;
pub use graph::ArchetypeGraph;
pub use storage::Archetype;
