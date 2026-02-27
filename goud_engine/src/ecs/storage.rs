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

use super::{Component, Entity, SparseSet};

/// Trait for component storage backends.
///
/// `ComponentStorage` defines the interface for storing components indexed by entity.
/// Any type implementing this trait can be used as a storage backend for components
/// in the ECS world.
///
/// # Type Parameters
///
/// The associated type `Item` specifies what component type this storage holds.
/// It must implement [`Component`] to ensure thread safety requirements are met.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow the ECS world to be shared
/// across threads. This is enforced by the trait bounds.
///
/// # Object Safety
///
/// This trait is **not** object-safe due to the associated type. For type-erased
/// storage, use [`AnyComponentStorage`] which provides a runtime interface.
///
/// # Example Implementation
///
/// ```
/// use goud_engine::ecs::{Entity, Component, ComponentStorage, SparseSet};
///
/// // Define a custom storage (wrapping SparseSet for this example)
/// struct MyStorage<T: Component> {
///     inner: SparseSet<T>,
/// }
///
/// impl<T: Component> ComponentStorage for MyStorage<T> {
///     type Item = T;
///
///     fn insert(&mut self, entity: Entity, value: T) -> Option<T> {
///         self.inner.insert(entity, value)
///     }
///
///     fn remove(&mut self, entity: Entity) -> Option<T> {
///         self.inner.remove(entity)
///     }
///
///     fn get(&self, entity: Entity) -> Option<&T> {
///         self.inner.get(entity)
///     }
///
///     fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
///         self.inner.get_mut(entity)
///     }
///
///     fn contains(&self, entity: Entity) -> bool {
///         self.inner.contains(entity)
///     }
///
///     fn len(&self) -> usize {
///         self.inner.len()
///     }
///
///     fn is_empty(&self) -> bool {
///         self.inner.is_empty()
///     }
/// }
/// ```
pub trait ComponentStorage: Send + Sync {
    /// The component type stored in this storage.
    type Item: Component;

    /// Inserts a component for the given entity.
    ///
    /// If the entity already has a component, the old value is returned.
    /// Otherwise, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to associate with the component
    /// * `value` - The component value to store
    ///
    /// # Returns
    ///
    /// The previous component value if one existed, or `None`.
    ///
    /// # Panics
    ///
    /// Implementations may panic if `entity` is invalid (e.g., placeholder).
    fn insert(&mut self, entity: Entity, value: Self::Item) -> Option<Self::Item>;

    /// Removes the component for the given entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity whose component to remove
    ///
    /// # Returns
    ///
    /// The removed component if one existed, or `None`.
    fn remove(&mut self, entity: Entity) -> Option<Self::Item>;

    /// Returns a reference to the component for the given entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Returns
    ///
    /// A reference to the component if the entity has one, or `None`.
    fn get(&self, entity: Entity) -> Option<&Self::Item>;

    /// Returns a mutable reference to the component for the given entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Returns
    ///
    /// A mutable reference to the component if the entity has one, or `None`.
    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Item>;

    /// Returns `true` if the entity has a component in this storage.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    fn contains(&self, entity: Entity) -> bool;

    /// Returns the number of components in this storage.
    fn len(&self) -> usize;

    /// Returns `true` if this storage contains no components.
    fn is_empty(&self) -> bool;
}

// =============================================================================
// SparseSet Implementation
// =============================================================================

impl<T: Component> ComponentStorage for SparseSet<T> {
    type Item = T;

    #[inline]
    fn insert(&mut self, entity: Entity, value: T) -> Option<T> {
        SparseSet::insert(self, entity, value)
    }

    #[inline]
    fn remove(&mut self, entity: Entity) -> Option<T> {
        SparseSet::remove(self, entity)
    }

    #[inline]
    fn get(&self, entity: Entity) -> Option<&T> {
        SparseSet::get(self, entity)
    }

    #[inline]
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        SparseSet::get_mut(self, entity)
    }

    #[inline]
    fn contains(&self, entity: Entity) -> bool {
        SparseSet::contains(self, entity)
    }

    #[inline]
    fn len(&self) -> usize {
        SparseSet::len(self)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        SparseSet::is_empty(self)
    }
}

// =============================================================================
// Type-Erased Storage
// =============================================================================

/// Type-erased component storage for runtime operations.
///
/// Unlike [`ComponentStorage`], this trait is object-safe and can be used for
/// type-erased storage in the ECS world. It provides basic operations that
/// don't require knowing the concrete component type.
///
/// # Usage
///
/// This trait is primarily used internally by the ECS world to manage storage
/// for different component types. Most users should interact with components
/// through the typed [`ComponentStorage`] trait.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Entity, Component, SparseSet, AnyComponentStorage};
///
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// let mut storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());
///
/// let entity = Entity::new(0, 1);
///
/// // Type-erased operations
/// assert!(!storage.contains_entity(entity));
/// assert_eq!(storage.storage_len(), 0);
/// assert!(storage.storage_is_empty());
/// ```
pub trait AnyComponentStorage: Send + Sync {
    /// Returns `true` if the entity has a component in this storage.
    fn contains_entity(&self, entity: Entity) -> bool;

    /// Removes the component for the given entity (dropping the value).
    ///
    /// Returns `true` if a component was removed, `false` otherwise.
    fn remove_entity(&mut self, entity: Entity) -> bool;

    /// Returns the number of components in this storage.
    fn storage_len(&self) -> usize;

    /// Returns `true` if this storage contains no components.
    fn storage_is_empty(&self) -> bool;

    /// Clears all components from this storage.
    fn clear(&mut self);

    /// Returns the type name of the stored component (for debugging).
    fn component_type_name(&self) -> &'static str;
}

impl<T: Component> AnyComponentStorage for SparseSet<T> {
    #[inline]
    fn contains_entity(&self, entity: Entity) -> bool {
        self.contains(entity)
    }

    #[inline]
    fn remove_entity(&mut self, entity: Entity) -> bool {
        self.remove(entity).is_some()
    }

    #[inline]
    fn storage_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn storage_is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        SparseSet::clear(self)
    }

    #[inline]
    fn component_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test components
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, PartialEq)]
    struct Name(String);
    impl Component for Name {}

    // Marker component (zero-sized)
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Player;
    impl Component for Player {}

    // =========================================================================
    // ComponentStorage Trait Tests
    // =========================================================================

    mod component_storage_trait {
        use super::*;

        #[test]
        fn test_sparse_set_implements_component_storage() {
            // Verify SparseSet<T: Component> implements ComponentStorage
            fn assert_component_storage<S: ComponentStorage>() {}
            assert_component_storage::<SparseSet<Position>>();
            assert_component_storage::<SparseSet<Velocity>>();
            assert_component_storage::<SparseSet<Name>>();
            assert_component_storage::<SparseSet<Player>>();
        }

        #[test]
        fn test_component_storage_is_send_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<SparseSet<Position>>();
            assert_send_sync::<SparseSet<Name>>();
        }

        #[test]
        fn test_insert_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            let old = ComponentStorage::insert(&mut storage, entity, Position { x: 1.0, y: 2.0 });

            assert_eq!(old, None);
            assert!(storage.contains(entity));
        }

        #[test]
        fn test_insert_replace_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            ComponentStorage::insert(&mut storage, entity, Position { x: 1.0, y: 2.0 });
            let old = ComponentStorage::insert(&mut storage, entity, Position { x: 3.0, y: 4.0 });

            assert_eq!(old, Some(Position { x: 1.0, y: 2.0 }));
            assert_eq!(
                ComponentStorage::get(&storage, entity),
                Some(&Position { x: 3.0, y: 4.0 })
            );
        }

        #[test]
        fn test_remove_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            storage.insert(entity, Position { x: 1.0, y: 2.0 });
            let removed = ComponentStorage::remove(&mut storage, entity);

            assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
            assert!(!storage.contains(entity));
        }

        #[test]
        fn test_remove_nonexistent_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            let removed = ComponentStorage::remove(&mut storage, entity);
            assert_eq!(removed, None);
        }

        #[test]
        fn test_get_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            storage.insert(entity, Position { x: 5.0, y: 10.0 });
            let pos = ComponentStorage::get(&storage, entity);

            assert_eq!(pos, Some(&Position { x: 5.0, y: 10.0 }));
        }

        #[test]
        fn test_get_nonexistent_via_trait() {
            let storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            assert_eq!(ComponentStorage::get(&storage, entity), None);
        }

        #[test]
        fn test_get_mut_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let entity = Entity::new(0, 1);

            storage.insert(entity, Position { x: 1.0, y: 2.0 });

            if let Some(pos) = ComponentStorage::get_mut(&mut storage, entity) {
                pos.x = 100.0;
                pos.y = 200.0;
            }

            assert_eq!(storage.get(entity), Some(&Position { x: 100.0, y: 200.0 }));
        }

        #[test]
        fn test_contains_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();
            let e1 = Entity::new(0, 1);
            let e2 = Entity::new(1, 1);

            storage.insert(e1, Position { x: 0.0, y: 0.0 });

            assert!(ComponentStorage::contains(&storage, e1));
            assert!(!ComponentStorage::contains(&storage, e2));
        }

        #[test]
        fn test_len_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();

            assert_eq!(ComponentStorage::len(&storage), 0);

            storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
            assert_eq!(ComponentStorage::len(&storage), 1);

            storage.insert(Entity::new(1, 1), Position { x: 1.0, y: 1.0 });
            assert_eq!(ComponentStorage::len(&storage), 2);
        }

        #[test]
        fn test_is_empty_via_trait() {
            let mut storage: SparseSet<Position> = SparseSet::new();

            assert!(ComponentStorage::is_empty(&storage));

            storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
            assert!(!ComponentStorage::is_empty(&storage));
        }

        #[test]
        fn test_generic_function_with_component_storage() {
            // Test that we can write generic code over ComponentStorage
            #[allow(dead_code)]
            fn count_matching<S>(storage: &S, predicate: impl Fn(&Position) -> bool) -> usize
            where
                S: ComponentStorage<Item = Position> + AsRef<SparseSet<Position>>,
            {
                storage
                    .as_ref()
                    .iter()
                    .filter(|(_, pos)| predicate(pos))
                    .count()
            }

            // Note: This requires a custom wrapper or direct iteration
            // For now, just verify the trait works with the methods
            let mut storage: SparseSet<Position> = SparseSet::new();
            storage.insert(Entity::new(0, 1), Position { x: 1.0, y: 2.0 });
            storage.insert(Entity::new(1, 1), Position { x: 3.0, y: 4.0 });

            assert_eq!(ComponentStorage::len(&storage), 2);
        }
    }

    // =========================================================================
    // AnyComponentStorage Tests
    // =========================================================================

    mod any_component_storage {
        use super::*;

        #[test]
        fn test_sparse_set_implements_any_component_storage() {
            fn assert_any_storage<S: AnyComponentStorage>() {}
            assert_any_storage::<SparseSet<Position>>();
            assert_any_storage::<SparseSet<Name>>();
        }

        #[test]
        fn test_any_storage_is_object_safe() {
            // Verify we can create trait objects
            let _storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());
        }

        #[test]
        fn test_contains_entity_via_any() {
            let storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());

            let entity = Entity::new(0, 1);

            assert!(!storage.contains_entity(entity));

            // We need to downcast to insert - this is expected
            // Type-erased storage is for querying, not typed operations
        }

        #[test]
        fn test_storage_len_via_any() {
            let mut storage = SparseSet::<Position>::new();
            storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
            storage.insert(Entity::new(1, 1), Position { x: 1.0, y: 1.0 });

            let any_storage: &dyn AnyComponentStorage = &storage;

            assert_eq!(any_storage.storage_len(), 2);
            assert!(!any_storage.storage_is_empty());
        }

        #[test]
        fn test_remove_entity_via_any() {
            let mut storage = SparseSet::<Position>::new();
            let entity = Entity::new(0, 1);

            storage.insert(entity, Position { x: 1.0, y: 2.0 });

            let any_storage: &mut dyn AnyComponentStorage = &mut storage;

            assert!(any_storage.contains_entity(entity));
            let removed = any_storage.remove_entity(entity);

            assert!(removed);
            assert!(!any_storage.contains_entity(entity));
        }

        #[test]
        fn test_remove_nonexistent_via_any() {
            let mut storage = SparseSet::<Position>::new();
            let entity = Entity::new(0, 1);

            let any_storage: &mut dyn AnyComponentStorage = &mut storage;
            let removed = any_storage.remove_entity(entity);

            assert!(!removed);
        }

        #[test]
        fn test_clear_via_any() {
            let mut storage = SparseSet::<Position>::new();
            storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
            storage.insert(Entity::new(1, 1), Position { x: 1.0, y: 1.0 });

            let any_storage: &mut dyn AnyComponentStorage = &mut storage;
            any_storage.clear();

            assert!(any_storage.storage_is_empty());
            assert_eq!(any_storage.storage_len(), 0);
        }

        #[test]
        fn test_component_type_name_via_any() {
            let storage = SparseSet::<Position>::new();
            let any_storage: &dyn AnyComponentStorage = &storage;

            let name = any_storage.component_type_name();
            assert!(name.contains("Position"));
        }

        #[test]
        fn test_multiple_storages_as_any() {
            let mut positions = SparseSet::<Position>::new();
            let mut velocities = SparseSet::<Velocity>::new();

            positions.insert(Entity::new(0, 1), Position { x: 1.0, y: 2.0 });
            velocities.insert(Entity::new(0, 1), Velocity { x: 3.0, y: 4.0 });

            let storages: Vec<&dyn AnyComponentStorage> = vec![&positions, &velocities];

            assert_eq!(storages[0].storage_len(), 1);
            assert_eq!(storages[1].storage_len(), 1);

            assert!(storages[0].component_type_name().contains("Position"));
            assert!(storages[1].component_type_name().contains("Velocity"));
        }

        #[test]
        fn test_any_storage_is_empty() {
            let storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());
            assert!(storage.storage_is_empty());
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_component_storage_workflow() {
            let mut storage: SparseSet<Position> = SparseSet::new();

            // Create entities
            let e1 = Entity::new(0, 1);
            let e2 = Entity::new(1, 1);
            let e3 = Entity::new(2, 1);

            // Insert via trait
            ComponentStorage::insert(&mut storage, e1, Position { x: 0.0, y: 0.0 });
            ComponentStorage::insert(&mut storage, e2, Position { x: 10.0, y: 20.0 });
            ComponentStorage::insert(&mut storage, e3, Position { x: 100.0, y: 200.0 });

            assert_eq!(ComponentStorage::len(&storage), 3);

            // Modify via trait
            if let Some(pos) = ComponentStorage::get_mut(&mut storage, e2) {
                pos.x += 5.0;
                pos.y += 5.0;
            }

            // Verify
            assert_eq!(
                ComponentStorage::get(&storage, e2),
                Some(&Position { x: 15.0, y: 25.0 })
            );

            // Remove via trait
            let removed = ComponentStorage::remove(&mut storage, e1);
            assert_eq!(removed, Some(Position { x: 0.0, y: 0.0 }));
            assert_eq!(ComponentStorage::len(&storage), 2);

            // Remaining entities still accessible
            assert!(ComponentStorage::contains(&storage, e2));
            assert!(ComponentStorage::contains(&storage, e3));
        }

        #[test]
        fn test_zero_sized_component_storage() {
            let mut storage: SparseSet<Player> = SparseSet::new();

            let e1 = Entity::new(0, 1);
            let e2 = Entity::new(1, 1);

            ComponentStorage::insert(&mut storage, e1, Player);
            ComponentStorage::insert(&mut storage, e2, Player);

            assert_eq!(ComponentStorage::len(&storage), 2);
            assert!(ComponentStorage::contains(&storage, e1));
            assert!(ComponentStorage::contains(&storage, e2));

            // ZST storage works correctly
            assert_eq!(ComponentStorage::get(&storage, e1), Some(&Player));
        }

        #[test]
        fn test_string_component_storage() {
            let mut storage: SparseSet<Name> = SparseSet::new();

            let entity = Entity::new(0, 1);
            ComponentStorage::insert(&mut storage, entity, Name("Player".to_string()));

            assert_eq!(
                ComponentStorage::get(&storage, entity),
                Some(&Name("Player".to_string()))
            );

            // Replace
            let old = ComponentStorage::insert(&mut storage, entity, Name("Enemy".to_string()));
            assert_eq!(old, Some(Name("Player".to_string())));
        }
    }
}
