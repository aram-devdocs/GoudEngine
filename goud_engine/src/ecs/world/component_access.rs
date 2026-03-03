use super::super::component::ComponentId;
use super::super::entity::Entity;
use super::super::sparse_set::SparseSet;
use super::super::Component;
use super::storage_entry::ComponentStorageEntry;
use super::World;

impl World {
    // =========================================================================
    // Component Type Statistics
    // =========================================================================

    /// Returns the number of registered component types.
    ///
    /// A component type is registered when the first entity receives a
    /// component of that type. This count represents the number of unique
    /// component types that have been used in this world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.component_type_count(), 0);
    /// ```
    #[inline]
    pub fn component_type_count(&self) -> usize {
        self.storages.len()
    }

    /// Checks if a component type has been registered in this world.
    ///
    /// A component type is registered when the first entity receives a
    /// component of that type, creating storage for it.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component, ComponentId};
    ///
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// let world = World::new();
    ///
    /// // No components registered yet
    /// assert!(!world.has_component_type::<Health>());
    /// ```
    #[inline]
    pub fn has_component_type<T: Component>(&self) -> bool {
        self.storages.contains_key(&ComponentId::of::<T>())
    }

    // =========================================================================
    // Component Access
    // =========================================================================

    /// Gets an immutable reference to a component on an entity.
    ///
    /// Returns `None` if the entity doesn't exist or doesn't have the component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to retrieve
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to get the component from
    ///
    /// # Returns
    ///
    /// `Some(&T)` if the entity exists and has the component, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    ///
    /// // Entity without Position component
    /// let entity = world.spawn_empty();
    ///
    /// // Returns None because entity doesn't have Position
    /// assert!(world.get::<Position>(entity).is_none());
    /// ```
    ///
    /// # Performance
    ///
    /// This is an O(1) operation using sparse set lookup.
    #[inline]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        // Check if entity is alive first (prevents accessing components of dead entities)
        if !self.is_alive(entity) {
            return None;
        }

        // Get storage and lookup component
        self.get_storage::<T>()?.get(entity)
    }

    /// Gets a mutable reference to a component on an entity.
    ///
    /// Returns `None` if the entity doesn't exist or doesn't have the component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to retrieve
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to get the component from
    ///
    /// # Returns
    ///
    /// `Some(&mut T)` if the entity exists and has the component, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Health(i32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_empty();
    ///
    /// // Entity doesn't have Health yet
    /// assert!(world.get_mut::<Health>(entity).is_none());
    /// ```
    ///
    /// # Performance
    ///
    /// This is an O(1) operation using sparse set lookup.
    #[inline]
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        // Check if entity is alive first (prevents accessing components of dead entities)
        if !self.is_alive(entity) {
            return None;
        }

        // Get storage and lookup component
        self.get_storage_option_mut::<T>()?.get_mut(entity)
    }

    /// Checks if an entity has a specific component type.
    ///
    /// Returns `false` if the entity doesn't exist or doesn't have the component.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to check for
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// `true` if the entity exists and has the component, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// struct Marker;
    /// impl Component for Marker {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_empty();
    ///
    /// // Entity doesn't have Marker component
    /// assert!(!world.has::<Marker>(entity));
    /// ```
    ///
    /// # Performance
    ///
    /// This is an O(1) operation using sparse set lookup.
    #[inline]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        // Check if entity is alive first
        if !self.is_alive(entity) {
            return false;
        }

        // Check storage for component
        self.get_storage::<T>()
            .map(|storage| storage.contains(entity))
            .unwrap_or(false)
    }

    // =========================================================================
    // Storage Access (Internal Helpers)
    // =========================================================================

    /// Gets a reference to the storage for a component type.
    ///
    /// Returns `None` if no entities have ever had this component type.
    ///
    /// # Type Safety
    ///
    /// The returned storage is guaranteed to contain `SparseSet<T>` because:
    /// 1. Storage is only created in `get_or_create_storage_mut` which uses `TypeId`
    /// 2. We downcast using the same `TypeId`
    pub(crate) fn get_storage<T: Component>(&self) -> Option<&SparseSet<T>> {
        let id = ComponentId::of::<T>();
        self.storages
            .get(&id)
            .and_then(|entry| entry.downcast_ref::<T>())
    }

    /// Gets a mutable reference to the storage for a component type.
    ///
    /// Returns `None` if no entities have ever had this component type.
    /// Use [`get_or_create_storage_mut`](Self::get_or_create_storage_mut) if you
    /// need to ensure storage exists.
    ///
    /// # Type Safety
    ///
    /// The returned storage is guaranteed to contain `SparseSet<T>` because:
    /// 1. Storage is only created in `get_or_create_storage_mut` which uses `TypeId`
    /// 2. We downcast using the same `TypeId`
    pub(crate) fn get_storage_option_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>> {
        let id = ComponentId::of::<T>();
        self.storages
            .get_mut(&id)
            .and_then(|entry| entry.downcast_mut::<T>())
    }

    /// Gets a mutable reference to the storage for a component type,
    /// creating it if it doesn't exist.
    ///
    /// This is the primary way to access component storage for modifications
    /// when you need to ensure storage exists (e.g., for insertion).
    ///
    /// # Type Safety
    ///
    /// Storage creation and access are type-safe because:
    /// 1. `ComponentId::of::<T>()` uniquely identifies type T
    /// 2. We only insert `SparseSet<T>` for that `ComponentId`
    /// 3. Downcast uses the same type T
    #[allow(dead_code)] // Used in Step 2.4.5 - Component Insertion
    pub(crate) fn get_or_create_storage_mut<T: Component>(&mut self) -> &mut SparseSet<T> {
        let id = ComponentId::of::<T>();

        // Get or create storage entry
        self.storages
            .entry(id)
            .or_insert_with(ComponentStorageEntry::new::<T>)
            .downcast_mut::<T>()
            .expect("Storage type mismatch - this is a bug")
    }

    // Keep old name as alias for backward compatibility with existing tests
    #[allow(dead_code)]
    pub(crate) fn get_storage_mut<T: Component>(&mut self) -> &mut SparseSet<T> {
        self.get_or_create_storage_mut::<T>()
    }
}
