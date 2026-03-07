use std::any::Any;

use super::super::entity::Entity;
use super::super::sparse_set::SparseSet;
use super::super::Component;

/// Type-erased function pointer for removing an entity from storage.
///
/// This is used to perform type-erased removal without knowing the concrete
/// component type at runtime.
pub(super) type RemoveEntityFn = fn(storage: &mut dyn Any, entity: Entity) -> bool;

/// Type-erased function pointer for cloning a component from one entity to another.
///
/// Returns `true` if the source entity had the component and it was cloned
/// to the target entity.
pub(super) type CloneToFn = fn(storage: &mut dyn Any, source: Entity, target: Entity) -> bool;

/// Internal wrapper for type-erased component storage.
///
/// This struct allows us to:
/// 1. Perform type-erased operations (remove) via stored function pointers
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

    /// Optional function pointer for cloning a component between entities.
    /// Only set for component types registered as cloneable via
    /// `World::register_cloneable`.
    clone_to_fn: Option<CloneToFn>,
}

impl ComponentStorageEntry {
    /// Creates a new storage entry for a specific component type.
    pub(super) fn new<T: Component>() -> Self {
        Self {
            storage: Box::new(SparseSet::<T>::new()),
            remove_entity_fn: Self::remove_entity_impl::<T>,
            clone_to_fn: None,
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

    /// Registers a clone function for component type `T`.
    ///
    /// After calling this, `clone_to` can copy components of type `T`
    /// between entities without knowing the concrete type at the call site.
    pub(super) fn set_clone_fn<T: Component + Clone>(&mut self) {
        self.clone_to_fn = Some(Self::clone_to_impl::<T>);
    }

    /// Type-erased implementation of component cloning for `SparseSet<T>`.
    fn clone_to_impl<T: Component + Clone>(
        storage: &mut dyn Any,
        source: Entity,
        target: Entity,
    ) -> bool {
        if let Some(sparse_set) = storage.downcast_mut::<SparseSet<T>>() {
            if let Some(component) = sparse_set.get(source).cloned() {
                sparse_set.insert(target, component);
                return true;
            }
        }
        false
    }

    /// Clones a component from `source` to `target` using the registered
    /// clone function.
    ///
    /// Returns `true` if the component was cloned. Returns `false` if:
    /// - No clone function has been registered for this storage
    /// - The source entity does not have this component
    pub(super) fn clone_to(&mut self, source: Entity, target: Entity) -> bool {
        if let Some(clone_fn) = self.clone_to_fn {
            (clone_fn)(self.storage.as_mut(), source, target)
        } else {
            false
        }
    }
}

impl std::fmt::Debug for ComponentStorageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStorageEntry")
            .field("storage", &"<type-erased>")
            .finish()
    }
}
