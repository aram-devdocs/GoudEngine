//! Component storage trait definitions.
//!
//! This module defines the [`ComponentStorage`] trait, which provides an abstract
//! interface for component storage backends, and [`AnyComponentStorage`], which
//! is the object-safe type-erased variant.

use crate::ecs::{Component, Entity};

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
