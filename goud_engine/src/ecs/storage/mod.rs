//! Component storage traits and implementations.
//!
//! This module defines the [`ComponentStorage`] trait, which provides an abstract
//! interface for component storage backends. The primary implementation is
//! [`SparseSet`], but the trait allows for alternative storage strategies in the future.
//!
//! # Design Philosophy
//!
//! Component storage is separated from the component type itself to allow:
//!
//! - **Flexibility**: Different components can use different storage strategies
//! - **Optimization**: Future storage options (table-based, chunk-based, etc.)
//! - **Abstraction**: The ECS world can work with type-erased storage
//!
//! # Thread Safety
//!
//! All storage implementations must be `Send + Sync` to enable parallel system
//! execution. The storage itself is not internally synchronized - concurrent
//! access must be managed at a higher level (e.g., via `RwLock` or scheduler).
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::{Entity, Component, SparseSet, ComponentStorage};
//!
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // SparseSet implements ComponentStorage
//! let mut storage: SparseSet<Position> = SparseSet::new();
//!
//! let entity = Entity::new(0, 1);
//! storage.insert(entity, Position { x: 10.0, y: 20.0 });
//!
//! assert!(storage.contains(entity));
//! assert_eq!(storage.get(entity), Some(&Position { x: 10.0, y: 20.0 }));
//! ```

mod impls;
mod traits;

#[cfg(test)]
mod tests;

pub use traits::{AnyComponentStorage, ComponentStorage};
