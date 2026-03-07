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

/// Type-erased function pointer for serializing a component from storage.
///
/// Given a storage and an entity, returns the serialized JSON value if the
/// entity has this component.
pub(super) type SerializeFn = fn(storage: &dyn Any, entity: Entity) -> Option<serde_json::Value>;

/// Type-erased function pointer for deserializing a component from JSON.
///
/// Returns a boxed Any containing the deserialized component, or None on
/// failure.
pub(super) type DeserializeFn = fn(value: &serde_json::Value) -> Option<Box<dyn Any + Send + Sync>>;

/// Type-erased function pointer for inserting a boxed component into storage.
///
/// The component is passed as `Box<dyn Any + Send + Sync>` and downcast to
/// the concrete type internally. Returns true on success.
pub(super) type InsertAnyFn =
    fn(storage: &mut dyn Any, entity: Entity, component: Box<dyn Any + Send + Sync>) -> bool;

/// Type-erased function pointer that returns the type name of a component.
pub(super) type TypeNameFn = fn() -> &'static str;

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

    /// Optional function pointer for serializing a component to JSON.
    /// Only set for component types registered as serializable.
    serialize_fn: Option<SerializeFn>,

    /// Optional function pointer for deserializing a component from JSON.
    /// Only set for component types registered as serializable.
    deserialize_fn: Option<DeserializeFn>,

    /// Optional function pointer for inserting a type-erased component.
    /// Only set for component types registered as serializable.
    insert_any_fn: Option<InsertAnyFn>,

    /// Optional function pointer that returns the component type name.
    /// Only set for component types registered as serializable.
    type_name_fn: Option<TypeNameFn>,
}

impl ComponentStorageEntry {
    /// Creates a new storage entry for a specific component type.
    pub(super) fn new<T: Component>() -> Self {
        Self {
            storage: Box::new(SparseSet::<T>::new()),
            remove_entity_fn: Self::remove_entity_impl::<T>,
            clone_to_fn: None,
            serialize_fn: None,
            deserialize_fn: None,
            insert_any_fn: None,
            type_name_fn: None,
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

    // =========================================================================
    // Serialization
    // =========================================================================

    /// Registers serialize/deserialize/insert functions for component type `T`.
    pub(super) fn set_serialize_fns<
        T: Component + serde::Serialize + for<'de> serde::Deserialize<'de>,
    >(
        &mut self,
    ) {
        self.serialize_fn = Some(Self::serialize_impl::<T>);
        self.deserialize_fn = Some(Self::deserialize_impl::<T>);
        self.insert_any_fn = Some(Self::insert_any_impl::<T>);
        self.type_name_fn = Some(Self::type_name_impl::<T>);
    }

    /// Type-erased serialize implementation.
    fn serialize_impl<T: Component + serde::Serialize>(
        storage: &dyn Any,
        entity: Entity,
    ) -> Option<serde_json::Value> {
        let sparse_set = storage.downcast_ref::<SparseSet<T>>()?;
        let component = sparse_set.get(entity)?;
        serde_json::to_value(component).ok()
    }

    /// Type-erased deserialize implementation.
    fn deserialize_impl<T: Component + for<'de> serde::Deserialize<'de>>(
        value: &serde_json::Value,
    ) -> Option<Box<dyn Any + Send + Sync>> {
        let component: T = serde_json::from_value(value.clone()).ok()?;
        Some(Box::new(component))
    }

    /// Type-erased insert implementation for boxed components.
    fn insert_any_impl<T: Component>(
        storage: &mut dyn Any,
        entity: Entity,
        component: Box<dyn Any + Send + Sync>,
    ) -> bool {
        if let Some(sparse_set) = storage.downcast_mut::<SparseSet<T>>() {
            if let Ok(typed) = component.downcast::<T>() {
                sparse_set.insert(entity, *typed);
                return true;
            }
        }
        false
    }

    /// Returns the type name for the component.
    fn type_name_impl<T: Component>() -> &'static str {
        std::any::type_name::<T>()
    }

    /// Serializes a component for the given entity, if serialization is
    /// registered.
    pub(super) fn serialize_component(&self, entity: Entity) -> Option<serde_json::Value> {
        let serialize_fn = self.serialize_fn?;
        (serialize_fn)(self.storage.as_ref(), entity)
    }

    /// Deserializes a component from JSON, if deserialization is registered.
    pub(super) fn deserialize_component(
        &self,
        value: &serde_json::Value,
    ) -> Option<Box<dyn Any + Send + Sync>> {
        let deserialize_fn = self.deserialize_fn?;
        (deserialize_fn)(value)
    }

    /// Inserts a type-erased component for the given entity.
    pub(super) fn insert_any(
        &mut self,
        entity: Entity,
        component: Box<dyn Any + Send + Sync>,
    ) -> bool {
        if let Some(insert_fn) = self.insert_any_fn {
            (insert_fn)(self.storage.as_mut(), entity, component)
        } else {
            false
        }
    }

    /// Returns the component type name, if registered.
    pub(super) fn type_name(&self) -> Option<&'static str> {
        self.type_name_fn.map(|f| (f)())
    }
}

impl std::fmt::Debug for ComponentStorageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStorageEntry")
            .field("storage", &"<type-erased>")
            .finish()
    }
}
