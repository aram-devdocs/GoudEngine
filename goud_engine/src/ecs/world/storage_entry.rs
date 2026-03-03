use std::any::Any;

use super::super::entity::Entity;
use super::super::sparse_set::SparseSet;
use super::super::Component;

/// Type-erased function pointer for removing an entity from storage.
///
/// This is used to perform type-erased removal without knowing the concrete
/// component type at runtime.
pub(super) type RemoveEntityFn = fn(storage: &mut dyn Any, entity: Entity) -> bool;

/// Type-erased function pointer for clearing all entities from storage.
pub(super) type ClearStorageFn = fn(storage: &mut dyn Any);

/// Internal wrapper for type-erased component storage.
///
/// This struct allows us to both:
/// 1. Perform type-erased operations (remove, clear) via stored function pointers
/// 2. Downcast to the concrete `SparseSet<T>` for typed operations
///
/// The function pointers are captured at creation time when we know the concrete
/// type, allowing us to call them later without knowing T.
pub(super) struct ComponentStorageEntry {
    /// The actual storage, stored as type-erased Any.
    /// This is always a `SparseSet<T>` for some T: Component.
    storage: Box<dyn Any + Send + Sync>,

    /// Function pointer to remove an entity from this storage.
    /// Returns true if a component was removed.
    remove_entity_fn: RemoveEntityFn,

    /// Function pointer to clear all entities from this storage.
    #[allow(dead_code)] // Used in World::clear and future batch operations
    clear_fn: ClearStorageFn,
}

impl ComponentStorageEntry {
    /// Creates a new storage entry for a specific component type.
    pub(super) fn new<T: Component>() -> Self {
        Self {
            storage: Box::new(SparseSet::<T>::new()),
            remove_entity_fn: Self::remove_entity_impl::<T>,
            clear_fn: Self::clear_impl::<T>,
        }
    }

    /// Type-erased implementation of entity removal for `SparseSet<T>`.
    fn remove_entity_impl<T: Component>(storage: &mut dyn Any, entity: Entity) -> bool {
        if let Some(sparse_set) = storage.downcast_mut::<SparseSet<T>>() {
            sparse_set.remove(entity).is_some()
        } else {
            false
        }
    }

    /// Type-erased implementation of storage clearing for `SparseSet<T>`.
    fn clear_impl<T: Component>(storage: &mut dyn Any) {
        if let Some(sparse_set) = storage.downcast_mut::<SparseSet<T>>() {
            sparse_set.clear();
        }
    }

    /// Returns a reference to the underlying storage as `dyn Any`.
    #[allow(dead_code)]
    pub(super) fn as_any(&self) -> &dyn Any {
        self.storage.as_ref()
    }

    /// Returns a mutable reference to the underlying storage as `dyn Any`.
    #[allow(dead_code)]
    pub(super) fn as_any_mut(&mut self) -> &mut dyn Any {
        self.storage.as_mut()
    }

    /// Attempts to downcast to a specific `SparseSet<T>`.
    pub(super) fn downcast_ref<T: Component>(&self) -> Option<&SparseSet<T>> {
        self.storage.downcast_ref::<SparseSet<T>>()
    }

    /// Attempts to downcast to a mutable `SparseSet<T>`.
    pub(super) fn downcast_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>> {
        self.storage.downcast_mut::<SparseSet<T>>()
    }

    /// Removes an entity from this storage using type-erased removal.
    ///
    /// Returns `true` if the entity had a component that was removed.
    pub(super) fn remove_entity(&mut self, entity: Entity) -> bool {
        (self.remove_entity_fn)(self.storage.as_mut(), entity)
    }

    /// Clears all entities from this storage.
    #[allow(dead_code)] // Used in World::clear and future batch operations
    pub(super) fn clear(&mut self) {
        (self.clear_fn)(self.storage.as_mut())
    }
}

impl std::fmt::Debug for ComponentStorageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStorageEntry")
            .field("storage", &"<type-erased>")
            .finish()
    }
}
