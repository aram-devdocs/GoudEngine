//! World - the central container for all ECS data.
//!
//! The `World` is the heart of the ECS. It manages:
//!
//! - **Entities**: Lightweight identifiers created and destroyed via the world
//! - **Components**: Data attached to entities, stored in type-erased sparse sets
//! - **Archetypes**: Groups of entities with identical component sets
//! - **Resources**: Global data shared across systems (future)
//!
//! # Architecture
//!
//! The World coordinates several subsystems:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                          World                              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  EntityAllocator     - Manages entity ID allocation         │
//! │  ArchetypeGraph      - Tracks entity->component groupings   │
//! │  entity_archetypes   - Maps Entity -> ArchetypeId           │
//! │  storages            - Type-erased component storage        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Thread Safety
//!
//! The World itself is not thread-safe. For concurrent access:
//! - Wrap in `Arc<RwLock<World>>` for shared ownership
//! - Or use the scheduler (future) which manages safe parallel access
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::World;
//!
//! let world = World::new();
//!
//! assert_eq!(world.entity_count(), 0);
//! assert_eq!(world.archetype_count(), 1); // Empty archetype always exists
//! ```

use std::any::Any;
use std::collections::HashMap;

use super::archetype::{ArchetypeGraph, ArchetypeId};
use super::component::ComponentId;
use super::entity::{Entity, EntityAllocator};
use super::resource::{
    NonSend, NonSendMut, NonSendResource, NonSendResources, Res, ResMut, Resource, Resources,
};
use super::sparse_set::SparseSet;
use super::Component;

// =============================================================================
// Component Storage Entry
// =============================================================================

/// Type-erased function pointer for removing an entity from storage.
///
/// This is used to perform type-erased removal without knowing the concrete
/// component type at runtime.
type RemoveEntityFn = fn(storage: &mut dyn Any, entity: Entity) -> bool;

/// Type-erased function pointer for clearing all entities from storage.
type ClearStorageFn = fn(storage: &mut dyn Any);

/// Internal wrapper for type-erased component storage.
///
/// This struct allows us to both:
/// 1. Perform type-erased operations (remove, clear) via stored function pointers
/// 2. Downcast to the concrete `SparseSet<T>` for typed operations
///
/// The function pointers are captured at creation time when we know the concrete
/// type, allowing us to call them later without knowing T.
struct ComponentStorageEntry {
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
    fn new<T: Component>() -> Self {
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
    fn as_any(&self) -> &dyn Any {
        self.storage.as_ref()
    }

    /// Returns a mutable reference to the underlying storage as `dyn Any`.
    #[allow(dead_code)]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.storage.as_mut()
    }

    /// Attempts to downcast to a specific `SparseSet<T>`.
    fn downcast_ref<T: Component>(&self) -> Option<&SparseSet<T>> {
        self.storage.downcast_ref::<SparseSet<T>>()
    }

    /// Attempts to downcast to a mutable `SparseSet<T>`.
    fn downcast_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>> {
        self.storage.downcast_mut::<SparseSet<T>>()
    }

    /// Removes an entity from this storage using type-erased removal.
    ///
    /// Returns `true` if the entity had a component that was removed.
    fn remove_entity(&mut self, entity: Entity) -> bool {
        (self.remove_entity_fn)(self.storage.as_mut(), entity)
    }

    /// Clears all entities from this storage.
    #[allow(dead_code)] // Used in World::clear and future batch operations
    fn clear(&mut self) {
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

// =============================================================================
// World
// =============================================================================

/// The central container for all ECS data.
///
/// `World` is the top-level struct that owns all ECS state. It provides
/// methods to spawn/despawn entities, add/remove/query components, and
/// manage archetypes.
///
/// # Design Philosophy
///
/// The World follows Bevy's architecture pattern:
///
/// 1. **Entity allocation** is handled by `EntityAllocator` using generational indices
/// 2. **Component storage** uses type-erased `SparseSet<T>` instances
/// 3. **Archetype tracking** groups entities by their component signature
/// 4. **Entity-archetype mapping** enables O(1) component lookups
///
/// # Type Erasure
///
/// Component storage is type-erased using `Box<dyn Any + Send + Sync>`.
/// This allows the World to store components of any type without being
/// generic. Type safety is maintained through `ComponentId` keys and
/// careful downcasting.
///
/// # Memory Layout
///
/// Components are stored in `SparseSet<T>` instances, one per component type.
/// Entities are tracked in archetypes for efficient querying. The
/// `entity_archetypes` map provides O(1) lookup of an entity's archetype.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World, Component};
///
/// // Define components
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// // Create world
/// let mut world = World::new();
///
/// // World starts with one archetype (empty)
/// assert_eq!(world.archetype_count(), 1);
/// assert_eq!(world.entity_count(), 0);
/// ```
#[derive(Debug)]
pub struct World {
    /// Entity ID allocator.
    ///
    /// Manages allocation and deallocation of entity IDs with generation
    /// counting to prevent stale entity access.
    entities: EntityAllocator,

    /// Archetype graph.
    ///
    /// Tracks all archetypes and their relationships (add/remove edges).
    /// Always contains at least the EMPTY archetype at index 0.
    archetypes: ArchetypeGraph,

    /// Maps entities to their current archetype.
    ///
    /// Every alive entity has an entry here. When an entity's components
    /// change, it moves to a new archetype and this mapping is updated.
    entity_archetypes: HashMap<Entity, ArchetypeId>,

    /// Type-erased component storage.
    ///
    /// Maps `ComponentId` to a type-erased storage wrapper. The wrapper contains
    /// both a `Box<dyn AnyComponentStorage>` for type-erased operations (like remove)
    /// and a `Box<dyn Any + Send + Sync>` for typed downcasting.
    ///
    /// Type safety is ensured by:
    /// 1. Only inserting `SparseSet<T>` for component type T
    /// 2. Downcasting with the correct type when accessing
    storages: HashMap<ComponentId, ComponentStorageEntry>,

    /// Resource storage.
    ///
    /// Resources are singleton data that exists outside the entity-component model.
    /// Examples include Time, Input state, Asset manager, etc.
    resources: Resources,

    /// Non-send resource storage.
    ///
    /// Resources that cannot be safely sent between threads. These include
    /// window handles, OpenGL contexts, and other thread-local data.
    /// Non-send resources must only be accessed from the main thread.
    non_send_resources: NonSendResources,
}

impl World {
    /// Creates a new, empty world.
    ///
    /// The world starts with:
    /// - No entities
    /// - No component storage
    /// - One archetype (the EMPTY archetype)
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.entity_count(), 0);
    /// assert_eq!(world.archetype_count(), 1);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            archetypes: ArchetypeGraph::new(),
            entity_archetypes: HashMap::new(),
            storages: HashMap::new(),
            resources: Resources::new(),
            non_send_resources: NonSendResources::new(),
        }
    }

    /// Creates a new world with pre-allocated capacity.
    ///
    /// Use this when you know approximately how many entities and component
    /// types you'll have, to avoid reallocations during gameplay.
    ///
    /// # Arguments
    ///
    /// * `entity_capacity` - Expected number of entities
    /// * `component_type_capacity` - Expected number of unique component types
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// // Pre-allocate for a game with ~10,000 entities and ~50 component types
    /// let world = World::with_capacity(10_000, 50);
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    #[inline]
    pub fn with_capacity(entity_capacity: usize, component_type_capacity: usize) -> Self {
        Self {
            entities: EntityAllocator::with_capacity(entity_capacity),
            archetypes: ArchetypeGraph::new(),
            entity_archetypes: HashMap::with_capacity(entity_capacity),
            storages: HashMap::with_capacity(component_type_capacity),
            resources: Resources::new(),
            non_send_resources: NonSendResources::new(),
        }
    }

    // =========================================================================
    // Entity Statistics
    // =========================================================================

    /// Returns the number of currently alive entities.
    ///
    /// This is the total number of entities that have been spawned and not
    /// yet despawned.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if there are no entities in the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert!(world.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    // =========================================================================
    // Archetype Statistics
    // =========================================================================

    /// Returns the number of archetypes in the world.
    ///
    /// This includes the EMPTY archetype which always exists. As entities
    /// gain different combinations of components, new archetypes are created.
    ///
    /// # Performance Note
    ///
    /// A high archetype count can indicate fragmentation. If you have many
    /// archetypes with few entities each, consider consolidating component
    /// combinations.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.archetype_count(), 1); // Empty archetype
    /// ```
    #[inline]
    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    // =========================================================================
    // Entity Spawning
    // =========================================================================

    /// Spawns a new entity with no components.
    ///
    /// The entity is allocated, added to the empty archetype, and returned.
    /// Use this when you want to add components later or need a simple entity.
    ///
    /// # Returns
    ///
    /// The newly created [`Entity`].
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entity = world.spawn_empty();
    ///
    /// assert!(world.is_alive(entity));
    /// assert_eq!(world.entity_count(), 1);
    /// ```
    ///
    /// # Performance
    ///
    /// This is an O(1) operation. The entity is immediately available for
    /// component additions.
    #[inline]
    pub fn spawn_empty(&mut self) -> Entity {
        // Allocate a new entity ID
        let entity = self.entities.allocate();

        // Add to the empty archetype
        let empty_archetype = self
            .archetypes
            .get_mut(ArchetypeId::EMPTY)
            .expect("Empty archetype should always exist");
        empty_archetype.add_entity(entity);

        // Track entity -> archetype mapping
        self.entity_archetypes.insert(entity, ArchetypeId::EMPTY);

        entity
    }

    /// Spawns a new entity and returns a builder for adding components.
    ///
    /// This is the primary way to create entities with components using a
    /// fluent builder pattern. The entity is immediately alive and starts
    /// in the empty archetype.
    ///
    /// # Returns
    ///
    /// An [`EntityWorldMut`] builder for adding components to the entity.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::{World, Component};
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    ///
    /// // Spawn with components (insert will be available in step 2.4.5)
    /// let entity = world.spawn()
    ///     .insert(Position { x: 0.0, y: 0.0 })
    ///     .id();
    ///
    /// assert!(world.is_alive(entity));
    /// ```
    ///
    /// # Builder Pattern
    ///
    /// The returned `EntityWorldMut` holds a mutable borrow of the world.
    /// Call [`.id()`](EntityWorldMut::id) at the end of your chain to get
    /// the entity ID and release the borrow.
    ///
    /// # Performance
    ///
    /// Entity spawning is O(1). Adding components via the builder triggers
    /// archetype transitions as needed.
    #[inline]
    pub fn spawn(&mut self) -> EntityWorldMut<'_> {
        let entity = self.spawn_empty();
        EntityWorldMut::new(self, entity)
    }

    /// Spawns multiple entities at once and returns their IDs.
    ///
    /// This is more efficient than calling [`spawn_empty()`](Self::spawn_empty)
    /// in a loop because it can batch allocations.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of entities to spawn
    ///
    /// # Returns
    ///
    /// A vector of the newly created [`Entity`] IDs.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entities = world.spawn_batch(100);
    ///
    /// assert_eq!(entities.len(), 100);
    /// assert_eq!(world.entity_count(), 100);
    ///
    /// for entity in &entities {
    ///     assert!(world.is_alive(*entity));
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// Batch spawning uses the entity allocator's batch allocation, which
    /// is more efficient than individual allocations. The empty archetype
    /// is also updated in bulk.
    pub fn spawn_batch(&mut self, count: usize) -> Vec<Entity> {
        if count == 0 {
            return Vec::new();
        }

        // Batch allocate entity IDs
        let entities = self.entities.allocate_batch(count);

        // Get the empty archetype
        let empty_archetype = self
            .archetypes
            .get_mut(ArchetypeId::EMPTY)
            .expect("Empty archetype should always exist");

        // Add all entities to empty archetype and track mappings
        for &entity in &entities {
            empty_archetype.add_entity(entity);
            self.entity_archetypes.insert(entity, ArchetypeId::EMPTY);
        }

        entities
    }

    // =========================================================================
    // Entity Despawning
    // =========================================================================

    /// Despawns an entity, removing it and all its components from the world.
    ///
    /// This completely removes the entity from the ECS:
    /// - The entity is removed from its archetype
    /// - All components attached to the entity are dropped
    /// - The entity ID is deallocated and may be recycled with a new generation
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to despawn
    ///
    /// # Returns
    ///
    /// `true` if the entity was successfully despawned, `false` if it was
    /// already dead or invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entity = world.spawn_empty();
    /// assert!(world.is_alive(entity));
    ///
    /// let despawned = world.despawn(entity);
    /// assert!(despawned);
    /// assert!(!world.is_alive(entity));
    ///
    /// // Despawning again returns false
    /// let despawned_again = world.despawn(entity);
    /// assert!(!despawned_again);
    /// ```
    ///
    /// # Component Cleanup
    ///
    /// All components attached to the entity are dropped when the entity is
    /// despawned. This ensures resources are properly released.
    ///
    /// # Performance
    ///
    /// Despawning is O(n) where n is the number of component types registered
    /// in the world, as we must check each storage for the entity's components.
    /// For better performance with many entities, use [`despawn_batch`](Self::despawn_batch).
    pub fn despawn(&mut self, entity: Entity) -> bool {
        // Check if entity is alive
        if !self.entities.is_alive(entity) {
            return false;
        }

        // Get and remove the archetype mapping
        let archetype_id = match self.entity_archetypes.remove(&entity) {
            Some(id) => id,
            None => return false,
        };

        // Get the components this entity has from its archetype
        // We need to clone the component set to avoid borrowing issues
        let component_ids: Vec<ComponentId> = self
            .archetypes
            .get(archetype_id)
            .map(|arch| arch.components().iter().copied().collect())
            .unwrap_or_default();

        // Remove from archetype
        if let Some(archetype) = self.archetypes.get_mut(archetype_id) {
            archetype.remove_entity(entity);
        }

        // Remove components using type-erased removal
        // We only check storages for components the entity actually has (from its archetype)
        for component_id in component_ids {
            if let Some(storage_entry) = self.storages.get_mut(&component_id) {
                storage_entry.remove_entity(entity);
            }
        }

        // Deallocate the entity ID
        self.entities.deallocate(entity);

        true
    }

    /// Despawns multiple entities at once.
    ///
    /// This is more efficient than calling [`despawn`](Self::despawn) repeatedly
    /// because it can batch some operations.
    ///
    /// # Arguments
    ///
    /// * `entities` - A slice of entities to despawn
    ///
    /// # Returns
    ///
    /// The number of entities successfully despawned. Entities that were already
    /// dead or invalid are not counted.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entities = world.spawn_batch(10);
    /// assert_eq!(world.entity_count(), 10);
    ///
    /// let despawned = world.despawn_batch(&entities);
    /// assert_eq!(despawned, 10);
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    ///
    /// # Performance
    ///
    /// Batch despawning is more efficient when removing many entities, but
    /// the complexity is still O(n * m) where n is the number of entities
    /// and m is the number of component types.
    pub fn despawn_batch(&mut self, entities: &[Entity]) -> usize {
        let mut count = 0;
        for &entity in entities {
            if self.despawn(entity) {
                count += 1;
            }
        }
        count
    }

    // =========================================================================
    // Entity Lifecycle Helpers
    // =========================================================================

    /// Checks if an entity is currently alive in this world.
    ///
    /// An entity is alive if:
    /// - It was allocated by this world's entity allocator
    /// - It has not been despawned
    /// - Its generation matches the current slot generation
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// `true` if the entity is valid and alive in this world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    /// use goud_engine::ecs::Entity;
    ///
    /// let world = World::new();
    ///
    /// // Placeholder entities are never alive
    /// assert!(!world.is_alive(Entity::PLACEHOLDER));
    ///
    /// // Fabricated entities are not alive
    /// let fake = Entity::new(999, 1);
    /// assert!(!world.is_alive(fake));
    /// ```
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Returns the archetype ID for the given entity.
    ///
    /// Returns `None` if the entity is not alive in this world.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    /// use goud_engine::ecs::Entity;
    ///
    /// let world = World::new();
    ///
    /// // Non-existent entity has no archetype
    /// let fake = Entity::new(0, 1);
    /// assert!(world.entity_archetype(fake).is_none());
    /// ```
    #[inline]
    pub fn entity_archetype(&self, entity: Entity) -> Option<ArchetypeId> {
        if self.is_alive(entity) {
            self.entity_archetypes.get(&entity).copied()
        } else {
            None
        }
    }

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

    // =========================================================================
    // Component Insertion
    // =========================================================================

    /// Inserts a component on an entity.
    ///
    /// If the entity already has a component of this type, it is replaced and
    /// the old value is returned. If the entity doesn't have this component,
    /// it is added and the entity transitions to a new archetype.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to insert
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the component to
    /// * `component` - The component value to insert
    ///
    /// # Returns
    ///
    /// - `Some(old_component)` if the entity already had this component
    /// - `None` if this is a new component for the entity
    ///
    /// # Panics
    ///
    /// This method does not panic. If the entity is dead, the operation is a no-op
    /// and returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_empty();
    ///
    /// // First insert returns None
    /// let old = world.insert(entity, Position { x: 1.0, y: 2.0 });
    /// assert!(old.is_none());
    /// assert_eq!(world.get::<Position>(entity), Some(&Position { x: 1.0, y: 2.0 }));
    ///
    /// // Second insert returns old value
    /// let old = world.insert(entity, Position { x: 10.0, y: 20.0 });
    /// assert_eq!(old, Some(Position { x: 1.0, y: 2.0 }));
    /// assert_eq!(world.get::<Position>(entity), Some(&Position { x: 10.0, y: 20.0 }));
    /// ```
    ///
    /// # Archetype Transitions
    ///
    /// When a new component type is added, the entity moves from its current
    /// archetype to a new archetype that includes the new component. This is
    /// handled automatically by the archetype graph.
    ///
    /// # Performance
    ///
    /// - If replacing an existing component: O(1)
    /// - If adding a new component type: O(k) where k is the number of components
    ///   on the entity (archetype transition)
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> Option<T> {
        // Check if entity is alive
        if !self.is_alive(entity) {
            return None;
        }

        let component_id = ComponentId::of::<T>();

        // Get current archetype
        let current_archetype_id = *self.entity_archetypes.get(&entity)?;

        // Check if entity already has this component
        let has_component = self
            .archetypes
            .get(current_archetype_id)
            .map(|arch| arch.has_component(component_id))
            .unwrap_or(false);

        if has_component {
            // Entity already has this component - just replace in storage
            let storage = self.get_or_create_storage_mut::<T>();
            storage.insert(entity, component)
        } else {
            // Entity doesn't have this component - need archetype transition
            // 1. Get the target archetype (with the new component)
            let target_archetype_id = self
                .archetypes
                .get_add_edge(current_archetype_id, component_id);

            // 2. Remove entity from old archetype
            if let Some(old_arch) = self.archetypes.get_mut(current_archetype_id) {
                old_arch.remove_entity(entity);
            }

            // 3. Add entity to new archetype
            if let Some(new_arch) = self.archetypes.get_mut(target_archetype_id) {
                new_arch.add_entity(entity);
            }

            // 4. Update entity -> archetype mapping
            self.entity_archetypes.insert(entity, target_archetype_id);

            // 5. Insert the component into storage
            let storage = self.get_or_create_storage_mut::<T>();
            storage.insert(entity, component);

            // No old value to return
            None
        }
    }

    /// Inserts components for multiple entities in batch.
    ///
    /// This is more efficient than calling [`insert`](Self::insert) repeatedly
    /// because it can batch archetype lookups and transitions.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to insert
    ///
    /// # Arguments
    ///
    /// * `batch` - An iterator of (entity, component) pairs
    ///
    /// # Returns
    ///
    /// The number of components successfully inserted (entities that were alive).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Health(i32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entities = world.spawn_batch(3);
    ///
    /// // Insert health for all entities
    /// let components = vec![
    ///     (entities[0], Health(100)),
    ///     (entities[1], Health(80)),
    ///     (entities[2], Health(50)),
    /// ];
    ///
    /// let count = world.insert_batch(components);
    /// assert_eq!(count, 3);
    ///
    /// assert_eq!(world.get::<Health>(entities[0]), Some(&Health(100)));
    /// assert_eq!(world.get::<Health>(entities[1]), Some(&Health(80)));
    /// assert_eq!(world.get::<Health>(entities[2]), Some(&Health(50)));
    /// ```
    ///
    /// # Performance
    ///
    /// Batch insertion is more efficient because:
    /// - Archetype edge lookups are cached after the first transition
    /// - Storage is accessed once and reused for all insertions
    pub fn insert_batch<T: Component>(
        &mut self,
        batch: impl IntoIterator<Item = (Entity, T)>,
    ) -> usize {
        let mut count = 0;
        for (entity, component) in batch {
            // Check if entity is alive before insertion
            if self.is_alive(entity) {
                self.insert(entity, component);
                count += 1;
            }
        }
        count
    }

    // =========================================================================
    // Component Removal
    // =========================================================================

    /// Removes a component from an entity and returns it.
    ///
    /// If the entity has the component, it is removed, the entity transitions
    /// to a new archetype (without this component), and the old value is returned.
    /// If the entity doesn't have this component, returns `None`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to remove
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from
    ///
    /// # Returns
    ///
    /// - `Some(component)` if the entity had this component
    /// - `None` if the entity doesn't exist, is dead, or doesn't have this component
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
    ///
    /// // Remove returns the component
    /// let removed = world.remove::<Position>(entity);
    /// assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
    ///
    /// // Entity no longer has the component
    /// assert!(!world.has::<Position>(entity));
    ///
    /// // Removing again returns None
    /// let removed_again = world.remove::<Position>(entity);
    /// assert!(removed_again.is_none());
    /// ```
    ///
    /// # Archetype Transitions
    ///
    /// When a component is removed, the entity moves from its current archetype
    /// to a new archetype that doesn't include this component. If this was the
    /// entity's last component, it transitions to the empty archetype.
    ///
    /// # Performance
    ///
    /// This is O(1) for the storage removal plus O(1) archetype transition
    /// (using cached edge lookup). The archetype graph caches remove edges
    /// for efficient repeated transitions.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        // Check if entity is alive
        if !self.is_alive(entity) {
            return None;
        }

        let component_id = ComponentId::of::<T>();

        // Get current archetype
        let current_archetype_id = *self.entity_archetypes.get(&entity)?;

        // Check if entity has this component
        let has_component = self
            .archetypes
            .get(current_archetype_id)
            .map(|arch| arch.has_component(component_id))
            .unwrap_or(false);

        if !has_component {
            // Entity doesn't have this component - nothing to remove
            return None;
        }

        // Remove component from storage first (before archetype transition)
        let removed_component = self.get_storage_option_mut::<T>()?.remove(entity)?;

        // Get the target archetype (without this component)
        // Note: get_remove_edge returns None only if the component wasn't in the source archetype,
        // but we've already verified the entity has this component above
        let target_archetype_id = self
            .archetypes
            .get_remove_edge(current_archetype_id, component_id)
            .expect("Archetype should have remove edge for component it contains");

        // Remove entity from old archetype
        if let Some(old_arch) = self.archetypes.get_mut(current_archetype_id) {
            old_arch.remove_entity(entity);
        }

        // Add entity to new archetype
        if let Some(new_arch) = self.archetypes.get_mut(target_archetype_id) {
            new_arch.add_entity(entity);
        }

        // Update entity -> archetype mapping
        self.entity_archetypes.insert(entity, target_archetype_id);

        Some(removed_component)
    }

    /// Removes a component from an entity and returns it.
    ///
    /// This is an alias for [`remove`](Self::remove). The name `take` follows
    /// Rust's convention for methods that consume/take ownership of data.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Health(i32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn().insert(Health(100)).id();
    ///
    /// // take() is identical to remove()
    /// let health = world.take::<Health>(entity);
    /// assert_eq!(health, Some(Health(100)));
    /// ```
    #[inline]
    pub fn take<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.remove(entity)
    }

    // =========================================================================
    // Direct Access (For Advanced Use)
    // =========================================================================

    /// Returns a reference to the entity allocator.
    ///
    /// This is primarily for internal use and debugging.
    #[inline]
    pub fn entities(&self) -> &EntityAllocator {
        &self.entities
    }

    /// Returns a reference to the archetype graph.
    ///
    /// This is primarily for internal use and debugging.
    #[inline]
    pub fn archetypes(&self) -> &ArchetypeGraph {
        &self.archetypes
    }

    /// Clears all entities and components from the world.
    ///
    /// After calling this:
    /// - All entities are invalid (even if you hold references)
    /// - All component storage is cleared
    /// - Archetypes remain but are empty
    ///
    /// # Use Case
    ///
    /// Useful for level transitions or resetting game state without
    /// recreating the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    /// // ... spawn entities, add components ...
    ///
    /// world.clear();
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    pub fn clear(&mut self) {
        // Clear all entities mapping
        self.entity_archetypes.clear();

        // Clear all component storage
        // Note: We're dropping all storage and recreating empty.
        // This properly cleans up all components regardless of type.
        self.storages.clear();

        // Clear archetype entity lists
        // Collect IDs first to avoid borrow conflict
        let archetype_ids: Vec<ArchetypeId> = self.archetypes.archetype_ids().collect();
        for archetype_id in archetype_ids {
            if let Some(archetype) = self.archetypes.get_mut(archetype_id) {
                archetype.clear_entities();
            }
        }

        // Reset entity allocator by creating a new one
        // This is safe because we've cleared all references
        self.entities = EntityAllocator::new();
    }

    // =========================================================================
    // Resource Management
    // =========================================================================

    /// Inserts a resource into the world.
    ///
    /// Resources are singleton data that exists outside the entity-component
    /// model. Unlike components, each resource type can only have one instance.
    ///
    /// If a resource of this type already exists, it is replaced and the old
    /// value is returned.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type (must implement `Send + Sync + 'static`)
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource value to insert
    ///
    /// # Returns
    ///
    /// - `Some(old_resource)` if a resource of this type was replaced
    /// - `None` if this is a new resource type
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Time { delta: f32, total: f32 }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Time { delta: 0.016, total: 0.0 });
    ///
    /// let time = world.get_resource::<Time>().unwrap();
    /// assert_eq!(time.delta, 0.016);
    /// ```
    #[inline]
    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources.insert(resource)
    }

    /// Removes a resource from the world and returns it.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to remove
    ///
    /// # Returns
    ///
    /// - `Some(resource)` if the resource existed
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Config { debug: bool }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Config { debug: true });
    ///
    /// let config = world.remove_resource::<Config>();
    /// assert!(config.is_some());
    /// assert!(world.get_resource::<Config>().is_none());
    /// ```
    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    /// Returns an immutable reference to a resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// if let Some(score) = world.get_resource::<Score>() {
    ///     println!("Score: {}", score.0);
    /// }
    /// ```
    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get()
    }

    /// Returns a mutable reference to a resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&mut T)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// if let Some(score) = world.get_resource_mut::<Score>() {
    ///     score.0 += 50;
    /// }
    ///
    /// assert_eq!(world.get_resource::<Score>().unwrap().0, 150);
    /// ```
    #[inline]
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }

    /// Returns an immutable [`Res`] wrapper for a resource.
    ///
    /// This is the primary way to access resources in systems. The `Res<T>`
    /// wrapper provides convenient access via `Deref`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(Res<T>)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Time { delta: f32 }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Time { delta: 0.016 });
    ///
    /// let time = world.resource::<Time>().unwrap();
    /// assert_eq!(time.delta, 0.016);
    /// ```
    #[inline]
    pub fn resource<T: Resource>(&self) -> Option<Res<'_, T>> {
        self.resources.get::<T>().map(Res::new)
    }

    /// Returns a mutable [`ResMut`] wrapper for a resource.
    ///
    /// This is the primary way to mutably access resources in systems. The
    /// `ResMut<T>` wrapper provides convenient access via `Deref` and `DerefMut`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(ResMut<T>)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// {
    ///     let mut score = world.resource_mut::<Score>().unwrap();
    ///     score.0 += 50;
    /// }
    ///
    /// assert_eq!(world.get_resource::<Score>().unwrap().0, 150);
    /// ```
    #[inline]
    pub fn resource_mut<T: Resource>(&mut self) -> Option<ResMut<'_, T>> {
        self.resources.get_mut::<T>().map(ResMut::new)
    }

    /// Returns `true` if a resource of the specified type exists.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// assert!(!world.contains_resource::<Score>());
    ///
    /// world.insert_resource(Score(100));
    /// assert!(world.contains_resource::<Score>());
    /// ```
    #[inline]
    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.resources.contains::<T>()
    }

    /// Returns the number of resources in the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    /// struct Time { delta: f32 }
    ///
    /// let mut world = World::new();
    /// assert_eq!(world.resource_count(), 0);
    ///
    /// world.insert_resource(Score(100));
    /// world.insert_resource(Time { delta: 0.016 });
    /// assert_eq!(world.resource_count(), 2);
    /// ```
    #[inline]
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Clears all resources from the world.
    ///
    /// This removes all resources but leaves entities and components intact.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// world.clear_resources();
    /// assert_eq!(world.resource_count(), 0);
    /// ```
    #[inline]
    pub fn clear_resources(&mut self) {
        self.resources.clear();
    }

    // =========================================================================
    // Non-Send Resources
    // =========================================================================

    /// Inserts a non-send resource into the world.
    ///
    /// Non-send resources are resources that cannot be safely sent between threads.
    /// Unlike regular resources, each non-send resource type can only have one instance
    /// and must only be accessed from the main thread.
    ///
    /// If a non-send resource of this type already exists, it is replaced and the old
    /// value is returned.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type (must implement `NonSendResource`)
    ///
    /// # Arguments
    ///
    /// * `resource` - The non-send resource value to insert
    ///
    /// # Returns
    ///
    /// - `Some(old_resource)` if a non-send resource of this type was replaced
    /// - `None` if this is a new non-send resource type
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// let handle = world.get_non_send_resource::<WindowHandle>().unwrap();
    /// assert_eq!(*handle.id, 42);
    /// ```
    #[inline]
    pub fn insert_non_send_resource<T: NonSendResource>(&mut self, resource: T) -> Option<T> {
        self.non_send_resources.insert(resource)
    }

    /// Removes a non-send resource from the world and returns it.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to remove
    ///
    /// # Returns
    ///
    /// - `Some(resource)` if the non-send resource existed
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// let handle = world.remove_non_send_resource::<WindowHandle>();
    /// assert!(handle.is_some());
    /// assert!(world.get_non_send_resource::<WindowHandle>().is_none());
    /// ```
    #[inline]
    pub fn remove_non_send_resource<T: NonSendResource>(&mut self) -> Option<T> {
        self.non_send_resources.remove()
    }

    /// Returns an immutable reference to a non-send resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// if let Some(handle) = world.get_non_send_resource::<WindowHandle>() {
    ///     println!("Window ID: {}", handle.id);
    /// }
    /// ```
    #[inline]
    pub fn get_non_send_resource<T: NonSendResource>(&self) -> Option<&T> {
        self.non_send_resources.get()
    }

    /// Returns a mutable reference to a non-send resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&mut T)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// struct Counter { value: Rc<RefCell<u32>> }
    /// impl NonSendResource for Counter {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(Counter { value: Rc::new(RefCell::new(0)) });
    ///
    /// if let Some(counter) = world.get_non_send_resource_mut::<Counter>() {
    ///     *counter.value.borrow_mut() += 1;
    /// }
    /// ```
    #[inline]
    pub fn get_non_send_resource_mut<T: NonSendResource>(&mut self) -> Option<&mut T> {
        self.non_send_resources.get_mut()
    }

    /// Returns an immutable [`NonSend`] wrapper for a non-send resource.
    ///
    /// This is the primary way to access non-send resources in systems. The `NonSend<T>`
    /// wrapper implements `Deref`, allowing direct access to the resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(NonSend<T>)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// let handle = world.non_send_resource::<WindowHandle>().unwrap();
    /// println!("Window ID: {}", handle.id);
    /// ```
    #[inline]
    pub fn non_send_resource<T: NonSendResource>(&self) -> Option<NonSend<'_, T>> {
        self.non_send_resources.get::<T>().map(NonSend::new)
    }

    /// Returns a mutable [`NonSendMut`] wrapper for a non-send resource.
    ///
    /// This is the primary way to mutably access non-send resources in systems. The
    /// `NonSendMut<T>` wrapper implements `Deref` and `DerefMut`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(NonSendMut<T>)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// struct Counter { value: Rc<RefCell<u32>> }
    /// impl NonSendResource for Counter {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(Counter { value: Rc::new(RefCell::new(0)) });
    ///
    /// {
    ///     let mut counter = world.non_send_resource_mut::<Counter>().unwrap();
    ///     *counter.value.borrow_mut() += 1;
    /// }
    /// ```
    #[inline]
    pub fn non_send_resource_mut<T: NonSendResource>(&mut self) -> Option<NonSendMut<'_, T>> {
        self.non_send_resources.get_mut::<T>().map(NonSendMut::new)
    }

    /// Returns `true` if a non-send resource of the specified type exists.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// assert!(!world.contains_non_send_resource::<WindowHandle>());
    ///
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    /// assert!(world.contains_non_send_resource::<WindowHandle>());
    /// ```
    #[inline]
    pub fn contains_non_send_resource<T: NonSendResource>(&self) -> bool {
        self.non_send_resources.contains::<T>()
    }

    /// Returns the number of non-send resources in the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// struct OpenGLContext { ctx: Rc<u32> }
    /// impl NonSendResource for OpenGLContext {}
    ///
    /// let mut world = World::new();
    /// assert_eq!(world.non_send_resource_count(), 0);
    ///
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(1) });
    /// world.insert_non_send_resource(OpenGLContext { ctx: Rc::new(2) });
    /// assert_eq!(world.non_send_resource_count(), 2);
    /// ```
    #[inline]
    pub fn non_send_resource_count(&self) -> usize {
        self.non_send_resources.len()
    }

    /// Clears all non-send resources from the world.
    ///
    /// This removes all non-send resources but leaves entities, components,
    /// and regular resources intact.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// world.clear_non_send_resources();
    /// assert_eq!(world.non_send_resource_count(), 0);
    /// ```
    #[inline]
    pub fn clear_non_send_resources(&mut self) {
        self.non_send_resources.clear();
    }
}

impl Default for World {
    /// Creates an empty world.
    ///
    /// Equivalent to [`World::new()`].
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// EntityWorldMut - Entity Builder
// =============================================================================

/// A mutable reference to an entity's data within a [`World`].
///
/// `EntityWorldMut` provides a fluent builder API for constructing entities
/// with components. It holds a mutable borrow of the world, allowing chained
/// component insertions.
///
/// # Builder Pattern
///
/// The primary use is via [`World::spawn()`] for fluent entity construction:
///
/// ```ignore
/// let entity = world.spawn()
///     .insert(Position { x: 0.0, y: 0.0 })
///     .insert(Velocity { x: 1.0, y: 0.0 })
///     .id();
/// ```
///
/// # Lifetime
///
/// The builder holds a mutable borrow of the [`World`], so you cannot access
/// the world while an `EntityWorldMut` exists. Call [`id()`](Self::id) to
/// get the entity ID and release the borrow.
///
/// # Thread Safety
///
/// `EntityWorldMut` is not `Send` or `Sync` - it's designed for single-threaded
/// entity construction. For batch spawning, use [`World::spawn_batch()`] (future).
#[derive(Debug)]
pub struct EntityWorldMut<'w> {
    /// The world containing this entity.
    world: &'w mut World,

    /// The entity being built.
    entity: Entity,
}

impl<'w> EntityWorldMut<'w> {
    /// Creates a new `EntityWorldMut` for an entity in the given world.
    ///
    /// # Safety Note
    ///
    /// The entity must already be allocated and registered in the world.
    /// This is an internal constructor - use [`World::spawn()`] instead.
    #[inline]
    pub(crate) fn new(world: &'w mut World, entity: Entity) -> Self {
        Self { world, entity }
    }

    /// Returns the [`Entity`] ID of the entity being built.
    ///
    /// This is commonly used at the end of a builder chain to get the
    /// entity ID for later reference.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let entity = world.spawn().id();
    /// assert!(world.is_alive(entity));
    /// ```
    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// Returns a reference to the [`World`] containing this entity.
    ///
    /// This allows read-only access to world state while building an entity.
    /// For mutable access, you'll need to finish building and drop this
    /// `EntityWorldMut` first.
    #[inline]
    pub fn world(&self) -> &World {
        self.world
    }

    /// Returns a mutable reference to the [`World`] containing this entity.
    ///
    /// # Warning
    ///
    /// Be careful when accessing the world mutably - ensure you don't
    /// invalidate this entity or its archetype in unexpected ways.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    // =========================================================================
    // Component Operations
    // =========================================================================

    /// Inserts a component on this entity.
    ///
    /// If the entity already has a component of this type, it is replaced.
    /// Returns `self` for method chaining.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to insert
    ///
    /// # Arguments
    ///
    /// * `component` - The component value to insert
    ///
    /// # Returns
    ///
    /// `&mut Self` for fluent method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// let mut world = World::new();
    ///
    /// // Fluent builder pattern
    /// let entity = world.spawn()
    ///     .insert(Position { x: 0.0, y: 0.0 })
    ///     .insert(Velocity { x: 1.0, y: 0.0 })
    ///     .id();
    ///
    /// assert!(world.has::<Position>(entity));
    /// assert!(world.has::<Velocity>(entity));
    /// assert_eq!(world.get::<Position>(entity), Some(&Position { x: 0.0, y: 0.0 }));
    /// assert_eq!(world.get::<Velocity>(entity), Some(&Velocity { x: 1.0, y: 0.0 }));
    /// ```
    ///
    /// # Archetype Transitions
    ///
    /// Each `insert` call may trigger an archetype transition if the entity
    /// doesn't already have the component type. Multiple inserts in sequence
    /// will create intermediate archetypes.
    #[inline]
    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.world.insert(self.entity, component);
        self
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

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Player;
    impl Component for Player {}

    // =========================================================================
    // World Construction Tests
    // =========================================================================

    mod construction {
        use super::*;

        #[test]
        fn test_world_new() {
            let world = World::new();

            assert_eq!(world.entity_count(), 0);
            assert_eq!(world.archetype_count(), 1); // Empty archetype
            assert_eq!(world.component_type_count(), 0);
            assert!(world.is_empty());
        }

        #[test]
        fn test_world_default() {
            let world: World = Default::default();

            assert_eq!(world.entity_count(), 0);
            assert_eq!(world.archetype_count(), 1);
        }

        #[test]
        fn test_world_with_capacity() {
            let world = World::with_capacity(10_000, 50);

            assert_eq!(world.entity_count(), 0);
            assert_eq!(world.archetype_count(), 1);
            assert_eq!(world.component_type_count(), 0);
        }

        #[test]
        fn test_world_debug() {
            let world = World::new();
            let debug_str = format!("{:?}", world);

            assert!(debug_str.contains("World"));
            assert!(debug_str.contains("entities"));
            assert!(debug_str.contains("archetypes"));
        }
    }

    // =========================================================================
    // Entity Count Tests
    // =========================================================================

    mod entity_count {
        use super::*;

        #[test]
        fn test_entity_count_starts_at_zero() {
            let world = World::new();
            assert_eq!(world.entity_count(), 0);
        }

        #[test]
        fn test_is_empty_starts_true() {
            let world = World::new();
            assert!(world.is_empty());
        }
    }

    // =========================================================================
    // Archetype Count Tests
    // =========================================================================

    mod archetype_count {
        use super::*;

        #[test]
        fn test_archetype_count_starts_at_one() {
            let world = World::new();
            // Empty archetype always exists
            assert_eq!(world.archetype_count(), 1);
        }

        #[test]
        fn test_empty_archetype_exists() {
            let world = World::new();
            let archetypes = world.archetypes();

            // EMPTY archetype (index 0) should exist
            assert!(archetypes.get(ArchetypeId::EMPTY).is_some());
        }
    }

    // =========================================================================
    // is_alive Tests
    // =========================================================================

    mod is_alive {
        use super::*;

        #[test]
        fn test_placeholder_not_alive() {
            let world = World::new();
            assert!(!world.is_alive(Entity::PLACEHOLDER));
        }

        #[test]
        fn test_fake_entity_not_alive() {
            let world = World::new();

            // Fabricated entity that was never allocated
            let fake = Entity::new(999, 1);
            assert!(!world.is_alive(fake));
        }

        #[test]
        fn test_entity_index_zero_not_alive_when_empty() {
            let world = World::new();

            // Index 0 hasn't been allocated yet
            let fake = Entity::new(0, 1);
            assert!(!world.is_alive(fake));
        }
    }

    // =========================================================================
    // entity_archetype Tests
    // =========================================================================

    mod entity_archetype {
        use super::*;

        #[test]
        fn test_nonexistent_entity_has_no_archetype() {
            let world = World::new();
            let fake = Entity::new(0, 1);

            assert!(world.entity_archetype(fake).is_none());
        }

        #[test]
        fn test_placeholder_has_no_archetype() {
            let world = World::new();
            assert!(world.entity_archetype(Entity::PLACEHOLDER).is_none());
        }
    }

    // =========================================================================
    // Component Type Tests
    // =========================================================================

    mod component_types {
        use super::*;

        #[test]
        fn test_component_type_count_starts_at_zero() {
            let world = World::new();
            assert_eq!(world.component_type_count(), 0);
        }

        #[test]
        fn test_has_component_type_false_initially() {
            let world = World::new();

            assert!(!world.has_component_type::<Position>());
            assert!(!world.has_component_type::<Velocity>());
            assert!(!world.has_component_type::<Name>());
        }
    }

    // =========================================================================
    // Storage Access Tests
    // =========================================================================

    mod storage {
        use super::*;

        #[test]
        fn test_get_storage_returns_none_for_unregistered() {
            let world = World::new();

            assert!(world.get_storage::<Position>().is_none());
        }

        #[test]
        fn test_get_storage_mut_creates_storage() {
            let mut world = World::new();

            // Access storage (will create it)
            let storage = world.get_storage_mut::<Position>();

            // Should be empty but exist
            assert!(storage.is_empty());

            // Now component type is registered
            assert!(world.has_component_type::<Position>());
            assert_eq!(world.component_type_count(), 1);
        }

        #[test]
        fn test_get_storage_mut_returns_same_storage() {
            let mut world = World::new();

            // First access creates storage
            world.get_storage_mut::<Position>();

            // Insert a component
            let entity = Entity::new(0, 1);
            world
                .get_storage_mut::<Position>()
                .insert(entity, Position { x: 1.0, y: 2.0 });

            // Second access returns same storage with data
            let storage = world.get_storage_mut::<Position>();
            assert_eq!(storage.len(), 1);
            assert_eq!(storage.get(entity), Some(&Position { x: 1.0, y: 2.0 }));
        }

        #[test]
        fn test_multiple_component_types_separate_storage() {
            let mut world = World::new();

            // Create storages for different types
            world.get_storage_mut::<Position>();
            world.get_storage_mut::<Velocity>();

            assert_eq!(world.component_type_count(), 2);
            assert!(world.has_component_type::<Position>());
            assert!(world.has_component_type::<Velocity>());
        }

        #[test]
        fn test_get_storage_after_mut_access() {
            let mut world = World::new();

            // Create and populate storage
            let entity = Entity::new(0, 1);
            world
                .get_storage_mut::<Position>()
                .insert(entity, Position { x: 5.0, y: 10.0 });

            // Now immutable access should work
            let storage = world.get_storage::<Position>().unwrap();
            assert_eq!(storage.get(entity), Some(&Position { x: 5.0, y: 10.0 }));
        }
    }

    // =========================================================================
    // Clear Tests
    // =========================================================================

    mod clear {
        use super::*;

        #[test]
        fn test_clear_resets_entity_count() {
            let mut world = World::new();

            // Manually add to entity_archetypes to simulate entities
            // (In a full implementation, spawn() would do this)
            // For now, we just verify clear works on empty world

            world.clear();
            assert_eq!(world.entity_count(), 0);
            assert!(world.is_empty());
        }

        #[test]
        fn test_clear_preserves_archetype_count() {
            let mut world = World::new();

            // Archetypes should still exist after clear
            // (just empty of entities)
            world.clear();
            assert_eq!(world.archetype_count(), 1);
        }
    }

    // =========================================================================
    // Direct Access Tests
    // =========================================================================

    mod direct_access {
        use super::*;

        #[test]
        fn test_entities_accessor() {
            let world = World::new();
            let allocator = world.entities();

            assert_eq!(allocator.len(), 0);
        }

        #[test]
        fn test_archetypes_accessor() {
            let world = World::new();
            let graph = world.archetypes();

            assert_eq!(graph.len(), 1);
            assert!(graph.get(ArchetypeId::EMPTY).is_some());
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        // Note: World is NOT Send because it contains NonSendResources.
        // This is intentional - the World should be owned by the main thread
        // when using non-send resources (window handles, GL contexts, etc.).
        //
        // For thread-safe world access, use:
        // - Arc<RwLock<World>> for shared ownership
        // - The scheduler, which handles thread safety for systems

        #[test]
        fn test_world_is_not_send_due_to_non_send_resources() {
            // This test documents the intentional !Send behavior.
            // World contains NonSendResources which uses NonSendMarker
            // containing *const (), making the whole type !Send.
            fn check_type<T>() {}
            check_type::<World>();
            // The fact that this compiles proves World exists as a type.
            // The !Send behavior is enforced by NonSendMarker.
        }

        // Note: World is NOT Sync by design - concurrent access requires
        // external synchronization (e.g., RwLock) or the scheduler
    }

    // =========================================================================
    // Entity Spawning Tests
    // =========================================================================

    mod spawn {
        use super::*;

        #[test]
        fn test_spawn_empty_creates_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();

            assert!(world.is_alive(entity));
            assert_eq!(world.entity_count(), 1);
        }

        #[test]
        fn test_spawn_empty_adds_to_empty_archetype() {
            let mut world = World::new();

            let entity = world.spawn_empty();

            // Entity should be in the empty archetype
            let archetype_id = world.entity_archetype(entity);
            assert_eq!(archetype_id, Some(ArchetypeId::EMPTY));
        }

        #[test]
        fn test_spawn_empty_multiple_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            assert!(world.is_alive(e1));
            assert!(world.is_alive(e2));
            assert!(world.is_alive(e3));
            assert_eq!(world.entity_count(), 3);

            // All should be in empty archetype
            assert_eq!(world.entity_archetype(e1), Some(ArchetypeId::EMPTY));
            assert_eq!(world.entity_archetype(e2), Some(ArchetypeId::EMPTY));
            assert_eq!(world.entity_archetype(e3), Some(ArchetypeId::EMPTY));
        }

        #[test]
        fn test_spawn_empty_unique_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            // Entities should have different indices
            assert_ne!(e1, e2);
            assert_ne!(e1.index(), e2.index());
        }

        #[test]
        fn test_spawn_returns_builder() {
            let mut world = World::new();

            let entity = world.spawn().id();

            assert!(world.is_alive(entity));
            assert_eq!(world.entity_count(), 1);
        }

        #[test]
        fn test_spawn_builder_id_matches() {
            let mut world = World::new();

            // Get entity ID from builder
            let builder = world.spawn();
            let entity = builder.id();

            // Entity should be alive and valid
            assert!(world.is_alive(entity));
        }

        #[test]
        fn test_spawn_builder_provides_world_access() {
            let mut world = World::new();

            let builder = world.spawn();

            // Should be able to access world read-only
            assert_eq!(builder.world().entity_count(), 1);
        }

        #[test]
        fn test_spawn_batch_empty() {
            let mut world = World::new();

            let entities = world.spawn_batch(0);

            assert!(entities.is_empty());
            assert_eq!(world.entity_count(), 0);
        }

        #[test]
        fn test_spawn_batch_single() {
            let mut world = World::new();

            let entities = world.spawn_batch(1);

            assert_eq!(entities.len(), 1);
            assert_eq!(world.entity_count(), 1);
            assert!(world.is_alive(entities[0]));
        }

        #[test]
        fn test_spawn_batch_multiple() {
            let mut world = World::new();

            let entities = world.spawn_batch(10);

            assert_eq!(entities.len(), 10);
            assert_eq!(world.entity_count(), 10);

            for entity in &entities {
                assert!(world.is_alive(*entity));
                assert_eq!(world.entity_archetype(*entity), Some(ArchetypeId::EMPTY));
            }
        }

        #[test]
        fn test_spawn_batch_large() {
            let mut world = World::new();

            let entities = world.spawn_batch(10_000);

            assert_eq!(entities.len(), 10_000);
            assert_eq!(world.entity_count(), 10_000);

            // Spot check a few
            assert!(world.is_alive(entities[0]));
            assert!(world.is_alive(entities[5000]));
            assert!(world.is_alive(entities[9999]));
        }

        #[test]
        fn test_spawn_batch_unique_entities() {
            let mut world = World::new();

            let entities = world.spawn_batch(100);

            // All entities should be unique
            let unique: std::collections::HashSet<_> = entities.iter().collect();
            assert_eq!(unique.len(), 100);
        }

        #[test]
        fn test_spawn_mixed_individual_and_batch() {
            let mut world = World::new();

            // Spawn some individually
            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            // Spawn a batch
            let batch = world.spawn_batch(5);

            // Spawn more individually
            let e3 = world.spawn_empty();

            assert_eq!(world.entity_count(), 8);
            assert!(world.is_alive(e1));
            assert!(world.is_alive(e2));
            assert!(world.is_alive(e3));
            for entity in &batch {
                assert!(world.is_alive(*entity));
            }
        }

        #[test]
        fn test_spawn_updates_archetype_entity_count() {
            let mut world = World::new();

            world.spawn_batch(5);

            // Empty archetype should have 5 entities
            let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
            assert_eq!(empty_archetype.len(), 5);
        }
    }

    // =========================================================================
    // Despawn Tests
    // =========================================================================

    mod despawn {
        use super::*;

        #[test]
        fn test_despawn_single_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            assert!(world.is_alive(entity));
            assert_eq!(world.entity_count(), 1);

            let despawned = world.despawn(entity);
            assert!(despawned);
            assert!(!world.is_alive(entity));
            assert_eq!(world.entity_count(), 0);
        }

        #[test]
        fn test_despawn_returns_false_for_dead_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.despawn(entity);

            // Despawning again should return false
            let despawned_again = world.despawn(entity);
            assert!(!despawned_again);
        }

        #[test]
        fn test_despawn_returns_false_for_never_allocated() {
            let mut world = World::new();

            // Entity that was never allocated
            let fake = Entity::new(999, 1);
            let despawned = world.despawn(fake);
            assert!(!despawned);
        }

        #[test]
        fn test_despawn_returns_false_for_placeholder() {
            let mut world = World::new();

            let despawned = world.despawn(Entity::PLACEHOLDER);
            assert!(!despawned);
        }

        #[test]
        fn test_despawn_removes_from_archetype() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            // Empty archetype should have 3 entities
            let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
            assert_eq!(empty_archetype.len(), 3);

            // Despawn middle entity
            world.despawn(e2);

            // Empty archetype should now have 2 entities
            let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
            assert_eq!(empty_archetype.len(), 2);

            // e1 and e3 should still be alive
            assert!(world.is_alive(e1));
            assert!(!world.is_alive(e2));
            assert!(world.is_alive(e3));
        }

        #[test]
        fn test_despawn_multiple_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            world.despawn(e1);
            assert!(!world.is_alive(e1));
            assert_eq!(world.entity_count(), 2);

            world.despawn(e3);
            assert!(!world.is_alive(e3));
            assert_eq!(world.entity_count(), 1);

            world.despawn(e2);
            assert!(!world.is_alive(e2));
            assert_eq!(world.entity_count(), 0);
        }

        #[test]
        fn test_despawn_stale_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            let stale = entity; // Copy of the entity

            // Despawn the original
            world.despawn(entity);

            // Spawn a new entity (may reuse the same index with new generation)
            let new_entity = world.spawn_empty();

            // The stale reference should not despawn the new entity
            if stale.index() == new_entity.index() {
                // Same index, different generation
                assert_ne!(stale.generation(), new_entity.generation());
            }

            // Despawning stale entity should fail
            let despawned = world.despawn(stale);
            assert!(!despawned);

            // New entity should still be alive
            assert!(world.is_alive(new_entity));
        }
    }

    // =========================================================================
    // Despawn Batch Tests
    // =========================================================================

    mod despawn_batch {
        use super::*;

        #[test]
        fn test_despawn_batch_empty() {
            let mut world = World::new();

            let despawned = world.despawn_batch(&[]);
            assert_eq!(despawned, 0);
        }

        #[test]
        fn test_despawn_batch_single() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            let despawned = world.despawn_batch(&[entity]);

            assert_eq!(despawned, 1);
            assert!(!world.is_alive(entity));
        }

        #[test]
        fn test_despawn_batch_multiple() {
            let mut world = World::new();

            let entities = world.spawn_batch(10);
            assert_eq!(world.entity_count(), 10);

            let despawned = world.despawn_batch(&entities);
            assert_eq!(despawned, 10);
            assert_eq!(world.entity_count(), 0);

            for entity in &entities {
                assert!(!world.is_alive(*entity));
            }
        }

        #[test]
        fn test_despawn_batch_partial_invalid() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            // Despawn e2 individually
            world.despawn(e2);

            // Batch despawn including already-dead entity
            let despawned = world.despawn_batch(&[e1, e2, e3]);

            // Only e1 and e3 should count as despawned
            assert_eq!(despawned, 2);
            assert_eq!(world.entity_count(), 0);
        }

        #[test]
        fn test_despawn_batch_with_placeholder() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            // Batch despawn with placeholder
            let despawned = world.despawn_batch(&[e1, Entity::PLACEHOLDER, e2]);

            // Placeholder doesn't count
            assert_eq!(despawned, 2);
        }

        #[test]
        fn test_despawn_batch_large() {
            let mut world = World::new();

            let entities = world.spawn_batch(10_000);
            assert_eq!(world.entity_count(), 10_000);

            let despawned = world.despawn_batch(&entities);
            assert_eq!(despawned, 10_000);
            assert_eq!(world.entity_count(), 0);
        }

        #[test]
        fn test_despawn_batch_duplicate_entities() {
            let mut world = World::new();

            let entity = world.spawn_empty();

            // Batch with duplicate - only first despawn should succeed
            let despawned = world.despawn_batch(&[entity, entity, entity]);

            assert_eq!(despawned, 1);
            assert!(!world.is_alive(entity));
        }

        #[test]
        fn test_despawn_batch_preserves_other_entities() {
            let mut world = World::new();

            let keep = world.spawn_batch(5);
            let remove = world.spawn_batch(5);

            assert_eq!(world.entity_count(), 10);

            world.despawn_batch(&remove);

            assert_eq!(world.entity_count(), 5);
            for entity in &keep {
                assert!(world.is_alive(*entity));
            }
            for entity in &remove {
                assert!(!world.is_alive(*entity));
            }
        }
    }

    // =========================================================================
    // EntityWorldMut Tests
    // =========================================================================

    mod entity_world_mut {
        use super::*;

        #[test]
        fn test_entity_world_mut_id() {
            let mut world = World::new();

            let builder = world.spawn();
            let id = builder.id();

            // ID should be valid
            assert!(!id.is_placeholder());
        }

        #[test]
        fn test_entity_world_mut_world_ref() {
            let mut world = World::new();

            let builder = world.spawn();

            // Should have read access to world
            let count = builder.world().entity_count();
            assert_eq!(count, 1);
        }

        #[test]
        fn test_entity_world_mut_world_mut() {
            let mut world = World::new();

            let mut builder = world.spawn();

            // Can access world mutably through builder
            let _world_mut = builder.world_mut();
        }

        #[test]
        fn test_entity_world_mut_debug() {
            let mut world = World::new();

            let builder = world.spawn();
            let debug_str = format!("{:?}", builder);

            assert!(debug_str.contains("EntityWorldMut"));
            assert!(debug_str.contains("entity"));
        }
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_zero_capacity_world() {
            let world = World::with_capacity(0, 0);
            assert_eq!(world.entity_count(), 0);
            assert_eq!(world.component_type_count(), 0);
        }

        #[test]
        fn test_large_capacity_world() {
            let world = World::with_capacity(1_000_000, 1000);
            assert_eq!(world.entity_count(), 0);
        }
    }

    // =========================================================================
    // Component Access Tests (Step 2.4.4)
    // =========================================================================

    mod component_access {
        use super::*;

        // Helper function to insert a component directly via storage
        // (This simulates what insert() will do in Step 2.4.5)
        fn insert_component<T: Component>(world: &mut World, entity: Entity, component: T) {
            world.get_storage_mut::<T>().insert(entity, component);
        }

        // =====================================================================
        // get() Tests
        // =====================================================================

        #[test]
        fn test_get_returns_none_for_dead_entity() {
            let world = World::new();

            // Entity that was never allocated
            let fake = Entity::new(0, 1);
            assert!(world.get::<Position>(fake).is_none());
        }

        #[test]
        fn test_get_returns_none_for_placeholder() {
            let world = World::new();
            assert!(world.get::<Position>(Entity::PLACEHOLDER).is_none());
        }

        #[test]
        fn test_get_returns_none_when_no_storage_exists() {
            let mut world = World::new();

            // Spawn entity but don't add any components
            let entity = world.spawn_empty();

            // No storage for Position exists yet
            assert!(world.get::<Position>(entity).is_none());
        }

        #[test]
        fn test_get_returns_none_when_entity_lacks_component() {
            let mut world = World::new();

            // Spawn two entities
            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            // Add component to e1 only
            insert_component(&mut world, e1, Position { x: 1.0, y: 2.0 });

            // e1 has component
            assert!(world.get::<Position>(e1).is_some());

            // e2 does not have component
            assert!(world.get::<Position>(e2).is_none());
        }

        #[test]
        fn test_get_returns_correct_component() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            let pos = Position { x: 10.0, y: 20.0 };
            insert_component(&mut world, entity, pos);

            let result = world.get::<Position>(entity);
            assert_eq!(result, Some(&Position { x: 10.0, y: 20.0 }));
        }

        #[test]
        fn test_get_returns_correct_component_for_multiple_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            insert_component(&mut world, e1, Position { x: 1.0, y: 1.0 });
            insert_component(&mut world, e2, Position { x: 2.0, y: 2.0 });
            insert_component(&mut world, e3, Position { x: 3.0, y: 3.0 });

            assert_eq!(
                world.get::<Position>(e1),
                Some(&Position { x: 1.0, y: 1.0 })
            );
            assert_eq!(
                world.get::<Position>(e2),
                Some(&Position { x: 2.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Position>(e3),
                Some(&Position { x: 3.0, y: 3.0 })
            );
        }

        #[test]
        fn test_get_different_component_types() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });
            insert_component(&mut world, entity, Velocity { x: 3.0, y: 4.0 });

            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Velocity>(entity),
                Some(&Velocity { x: 3.0, y: 4.0 })
            );
        }

        #[test]
        fn test_get_returns_none_after_despawn() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

            // Component exists before despawn
            assert!(world.get::<Position>(entity).is_some());

            // Despawn the entity
            world.despawn(entity);

            // Component no longer accessible (entity is dead)
            assert!(world.get::<Position>(entity).is_none());
        }

        // =====================================================================
        // get_mut() Tests
        // =====================================================================

        #[test]
        fn test_get_mut_returns_none_for_dead_entity() {
            let mut world = World::new();

            let fake = Entity::new(999, 1);
            assert!(world.get_mut::<Position>(fake).is_none());
        }

        #[test]
        fn test_get_mut_returns_none_for_placeholder() {
            let mut world = World::new();
            assert!(world.get_mut::<Position>(Entity::PLACEHOLDER).is_none());
        }

        #[test]
        fn test_get_mut_returns_none_when_no_storage_exists() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            assert!(world.get_mut::<Velocity>(entity).is_none());
        }

        #[test]
        fn test_get_mut_returns_mutable_reference() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

            // Modify via mutable reference
            if let Some(pos) = world.get_mut::<Position>(entity) {
                pos.x = 100.0;
                pos.y = 200.0;
            }

            // Verify modification persisted
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 100.0, y: 200.0 })
            );
        }

        #[test]
        fn test_get_mut_returns_none_after_despawn() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

            world.despawn(entity);

            assert!(world.get_mut::<Position>(entity).is_none());
        }

        // =====================================================================
        // has() Tests
        // =====================================================================

        #[test]
        fn test_has_returns_false_for_dead_entity() {
            let world = World::new();

            let fake = Entity::new(42, 1);
            assert!(!world.has::<Position>(fake));
        }

        #[test]
        fn test_has_returns_false_for_placeholder() {
            let world = World::new();
            assert!(!world.has::<Position>(Entity::PLACEHOLDER));
        }

        #[test]
        fn test_has_returns_false_when_no_storage_exists() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            assert!(!world.has::<Position>(entity));
        }

        #[test]
        fn test_has_returns_false_when_entity_lacks_component() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            insert_component(&mut world, e1, Position { x: 0.0, y: 0.0 });

            assert!(world.has::<Position>(e1));
            assert!(!world.has::<Position>(e2));
        }

        #[test]
        fn test_has_returns_true_when_entity_has_component() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Player);

            assert!(world.has::<Player>(entity));
        }

        #[test]
        fn test_has_distinguishes_different_component_types() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 0.0, y: 0.0 });

            assert!(world.has::<Position>(entity));
            assert!(!world.has::<Velocity>(entity));
            assert!(!world.has::<Name>(entity));
            assert!(!world.has::<Player>(entity));
        }

        #[test]
        fn test_has_returns_false_after_despawn() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

            assert!(world.has::<Position>(entity));

            world.despawn(entity);

            assert!(!world.has::<Position>(entity));
        }

        // =====================================================================
        // Component Access with Multiple Component Types
        // =====================================================================

        #[test]
        fn test_access_multiple_component_types_on_same_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });
            insert_component(&mut world, entity, Velocity { x: 3.0, y: 4.0 });
            insert_component(&mut world, entity, Name("Test".to_string()));
            insert_component(&mut world, entity, Player);

            // All should be accessible
            assert!(world.has::<Position>(entity));
            assert!(world.has::<Velocity>(entity));
            assert!(world.has::<Name>(entity));
            assert!(world.has::<Player>(entity));

            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Velocity>(entity),
                Some(&Velocity { x: 3.0, y: 4.0 })
            );
            assert_eq!(world.get::<Name>(entity), Some(&Name("Test".to_string())));
            assert_eq!(world.get::<Player>(entity), Some(&Player));
        }

        // =====================================================================
        // Type Safety Tests
        // =====================================================================

        #[test]
        fn test_type_safety_different_types_same_layout() {
            // Two components with same memory layout should not conflict
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct TypeA(f32, f32);
            impl Component for TypeA {}

            #[derive(Debug, Clone, Copy, PartialEq)]
            struct TypeB(f32, f32);
            impl Component for TypeB {}

            let mut world = World::new();
            let entity = world.spawn_empty();

            insert_component(&mut world, entity, TypeA(1.0, 2.0));
            insert_component(&mut world, entity, TypeB(3.0, 4.0));

            // Types should be distinct
            assert_eq!(world.get::<TypeA>(entity), Some(&TypeA(1.0, 2.0)));
            assert_eq!(world.get::<TypeB>(entity), Some(&TypeB(3.0, 4.0)));
        }

        // =====================================================================
        // Stale Entity Reference Tests
        // =====================================================================

        #[test]
        fn test_get_returns_none_for_stale_entity() {
            let mut world = World::new();

            // Spawn and despawn an entity
            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });
            let stale = entity;
            world.despawn(entity);

            // Spawn a new entity at the same index
            let new_entity = world.spawn_empty();
            insert_component(&mut world, new_entity, Position { x: 99.0, y: 99.0 });

            // If same index, generations should differ
            if stale.index() == new_entity.index() {
                // Stale reference should not access new entity's component
                assert!(world.get::<Position>(stale).is_none());
                // New entity should have its own component
                assert_eq!(
                    world.get::<Position>(new_entity),
                    Some(&Position { x: 99.0, y: 99.0 })
                );
            }
        }

        #[test]
        fn test_has_returns_false_for_stale_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            insert_component(&mut world, entity, Player);
            let stale = entity;
            world.despawn(entity);

            let new_entity = world.spawn_empty();
            insert_component(&mut world, new_entity, Player);

            if stale.index() == new_entity.index() {
                assert!(!world.has::<Player>(stale));
                assert!(world.has::<Player>(new_entity));
            }
        }
    }

    // =========================================================================
    // Component Insertion Tests (Step 2.4.5)
    // =========================================================================

    mod insert {
        use super::*;

        // =====================================================================
        // World::insert() Basic Tests
        // =====================================================================

        #[test]
        fn test_insert_first_component_returns_none() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            let old = world.insert(entity, Position { x: 1.0, y: 2.0 });
            assert!(old.is_none());
        }

        #[test]
        fn test_insert_makes_component_accessible() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, Position { x: 1.0, y: 2.0 });

            assert!(world.has::<Position>(entity));
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
        }

        #[test]
        fn test_insert_replace_returns_old_value() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, Position { x: 1.0, y: 2.0 });
            let old = world.insert(entity, Position { x: 10.0, y: 20.0 });

            assert_eq!(old, Some(Position { x: 1.0, y: 2.0 }));
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 10.0, y: 20.0 })
            );
        }

        #[test]
        fn test_insert_on_dead_entity_returns_none() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.despawn(entity);

            let result = world.insert(entity, Position { x: 1.0, y: 2.0 });
            assert!(result.is_none());
        }

        #[test]
        fn test_insert_on_placeholder_returns_none() {
            let mut world = World::new();

            let result = world.insert(Entity::PLACEHOLDER, Position { x: 1.0, y: 2.0 });
            assert!(result.is_none());
        }

        #[test]
        fn test_insert_on_never_allocated_entity_returns_none() {
            let mut world = World::new();

            let fake = Entity::new(999, 1);
            let result = world.insert(fake, Position { x: 1.0, y: 2.0 });
            assert!(result.is_none());
        }

        // =====================================================================
        // Archetype Transition Tests
        // =====================================================================

        #[test]
        fn test_insert_triggers_archetype_transition() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            // Entity starts in empty archetype
            assert_eq!(world.entity_archetype(entity), Some(ArchetypeId::EMPTY));

            // Insert component
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            // Entity should now be in a different archetype
            let archetype_id = world.entity_archetype(entity).unwrap();
            assert_ne!(archetype_id, ArchetypeId::EMPTY);
        }

        #[test]
        fn test_insert_second_component_creates_new_archetype() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, Position { x: 1.0, y: 2.0 });
            let arch_with_pos = world.entity_archetype(entity).unwrap();

            world.insert(entity, Velocity { x: 3.0, y: 4.0 });
            let arch_with_both = world.entity_archetype(entity).unwrap();

            // Should be a different archetype
            assert_ne!(arch_with_pos, arch_with_both);

            // Both components should be accessible
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Velocity>(entity),
                Some(&Velocity { x: 3.0, y: 4.0 })
            );
        }

        #[test]
        fn test_insert_removes_entity_from_old_archetype() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            // Empty archetype should have 1 entity
            let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
            assert_eq!(empty_archetype.len(), 1);

            world.insert(entity, Position { x: 1.0, y: 2.0 });

            // Empty archetype should now be empty
            let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
            assert_eq!(empty_archetype.len(), 0);
        }

        #[test]
        fn test_insert_adds_entity_to_new_archetype() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let archetype_id = world.entity_archetype(entity).unwrap();
            let archetype = world.archetypes().get(archetype_id).unwrap();

            // New archetype should contain the entity
            assert!(archetype.contains_entity(entity));
            assert_eq!(archetype.len(), 1);
        }

        #[test]
        fn test_insert_same_component_does_not_change_archetype() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, Position { x: 1.0, y: 2.0 });
            let arch_before = world.entity_archetype(entity).unwrap();

            world.insert(entity, Position { x: 10.0, y: 20.0 });
            let arch_after = world.entity_archetype(entity).unwrap();

            // Should be same archetype
            assert_eq!(arch_before, arch_after);
        }

        #[test]
        fn test_insert_creates_correct_archetype_count() {
            let mut world = World::new();

            // Start with 1 archetype (empty)
            assert_eq!(world.archetype_count(), 1);

            let entity = world.spawn_empty();

            // Add Position - creates archetype with Position
            world.insert(entity, Position { x: 0.0, y: 0.0 });
            assert_eq!(world.archetype_count(), 2);

            // Add Velocity - creates archetype with Position+Velocity
            world.insert(entity, Velocity { x: 0.0, y: 0.0 });
            assert_eq!(world.archetype_count(), 3);

            // Replace Position - no new archetype
            world.insert(entity, Position { x: 1.0, y: 1.0 });
            assert_eq!(world.archetype_count(), 3);
        }

        // =====================================================================
        // Multiple Entity Tests
        // =====================================================================

        #[test]
        fn test_insert_multiple_entities_same_components() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            world.insert(e1, Position { x: 1.0, y: 1.0 });
            world.insert(e2, Position { x: 2.0, y: 2.0 });
            world.insert(e3, Position { x: 3.0, y: 3.0 });

            // All should have Position
            assert_eq!(
                world.get::<Position>(e1),
                Some(&Position { x: 1.0, y: 1.0 })
            );
            assert_eq!(
                world.get::<Position>(e2),
                Some(&Position { x: 2.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Position>(e3),
                Some(&Position { x: 3.0, y: 3.0 })
            );

            // All should be in same archetype
            assert_eq!(world.entity_archetype(e1), world.entity_archetype(e2));
            assert_eq!(world.entity_archetype(e2), world.entity_archetype(e3));
        }

        #[test]
        fn test_insert_multiple_entities_different_components() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            world.insert(e1, Position { x: 1.0, y: 1.0 });
            world.insert(e2, Velocity { x: 2.0, y: 2.0 });
            world.insert(e3, Player);

            // Each should have its own component
            assert!(world.has::<Position>(e1));
            assert!(!world.has::<Velocity>(e1));

            assert!(world.has::<Velocity>(e2));
            assert!(!world.has::<Position>(e2));

            assert!(world.has::<Player>(e3));
            assert!(!world.has::<Position>(e3));

            // All should be in different archetypes
            assert_ne!(world.entity_archetype(e1), world.entity_archetype(e2));
            assert_ne!(world.entity_archetype(e2), world.entity_archetype(e3));
        }

        // =====================================================================
        // insert_batch() Tests
        // =====================================================================

        #[test]
        fn test_insert_batch_empty() {
            let mut world = World::new();

            let count = world.insert_batch::<Position>(std::iter::empty());
            assert_eq!(count, 0);
        }

        #[test]
        fn test_insert_batch_single() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            let count = world.insert_batch(vec![(entity, Position { x: 1.0, y: 2.0 })]);

            assert_eq!(count, 1);
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
        }

        #[test]
        fn test_insert_batch_multiple() {
            let mut world = World::new();
            let entities = world.spawn_batch(5);

            let batch: Vec<_> = entities
                .iter()
                .enumerate()
                .map(|(i, &e)| {
                    (
                        e,
                        Position {
                            x: i as f32,
                            y: (i * 2) as f32,
                        },
                    )
                })
                .collect();

            let count = world.insert_batch(batch);

            assert_eq!(count, 5);
            for (i, entity) in entities.iter().enumerate() {
                assert_eq!(
                    world.get::<Position>(*entity),
                    Some(&Position {
                        x: i as f32,
                        y: (i * 2) as f32
                    })
                );
            }
        }

        #[test]
        fn test_insert_batch_skips_dead_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();
            let e3 = world.spawn_empty();

            // Despawn e2
            world.despawn(e2);

            let batch = vec![
                (e1, Position { x: 1.0, y: 1.0 }),
                (e2, Position { x: 2.0, y: 2.0 }), // dead
                (e3, Position { x: 3.0, y: 3.0 }),
            ];

            let count = world.insert_batch(batch);

            assert_eq!(count, 2);
            assert!(world.has::<Position>(e1));
            assert!(!world.has::<Position>(e2)); // not inserted (dead)
            assert!(world.has::<Position>(e3));
        }

        #[test]
        fn test_insert_batch_with_placeholder() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            let batch = vec![
                (entity, Position { x: 1.0, y: 1.0 }),
                (Entity::PLACEHOLDER, Position { x: 0.0, y: 0.0 }),
            ];

            let count = world.insert_batch(batch);

            assert_eq!(count, 1);
            assert!(world.has::<Position>(entity));
        }

        // =====================================================================
        // EntityWorldMut::insert() Tests
        // =====================================================================

        #[test]
        fn test_entity_builder_insert_single() {
            let mut world = World::new();

            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            assert!(world.has::<Position>(entity));
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
        }

        #[test]
        fn test_entity_builder_insert_multiple() {
            let mut world = World::new();

            let entity = world
                .spawn()
                .insert(Position { x: 1.0, y: 2.0 })
                .insert(Velocity { x: 3.0, y: 4.0 })
                .insert(Player)
                .id();

            assert!(world.has::<Position>(entity));
            assert!(world.has::<Velocity>(entity));
            assert!(world.has::<Player>(entity));

            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 1.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Velocity>(entity),
                Some(&Velocity { x: 3.0, y: 4.0 })
            );
            assert_eq!(world.get::<Player>(entity), Some(&Player));
        }

        #[test]
        fn test_entity_builder_insert_replace() {
            let mut world = World::new();

            let entity = world
                .spawn()
                .insert(Position { x: 1.0, y: 2.0 })
                .insert(Position { x: 10.0, y: 20.0 }) // Replace
                .id();

            // Should have the second value
            assert_eq!(
                world.get::<Position>(entity),
                Some(&Position { x: 10.0, y: 20.0 })
            );
        }

        #[test]
        fn test_entity_builder_chaining_returns_self() {
            let mut world = World::new();

            // Verify chaining works by using mutable borrows
            let mut builder = world.spawn();
            let entity_id = builder.id();

            // Chain returns &mut Self for fluent API
            builder
                .insert(Position { x: 0.0, y: 0.0 })
                .insert(Velocity { x: 1.0, y: 1.0 });

            // Need to drop builder to access world again
            drop(builder);

            assert!(world.has::<Position>(entity_id));
            assert!(world.has::<Velocity>(entity_id));
        }

        // =====================================================================
        // Component Type Registration Tests
        // =====================================================================

        #[test]
        fn test_insert_registers_component_type() {
            let mut world = World::new();

            assert!(!world.has_component_type::<Position>());

            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 0.0, y: 0.0 });

            assert!(world.has_component_type::<Position>());
        }

        #[test]
        fn test_insert_multiple_types_registers_all() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 0.0, y: 0.0 });
            world.insert(entity, Velocity { x: 0.0, y: 0.0 });
            world.insert(entity, Player);

            assert_eq!(world.component_type_count(), 3);
            assert!(world.has_component_type::<Position>());
            assert!(world.has_component_type::<Velocity>());
            assert!(world.has_component_type::<Player>());
        }

        // =====================================================================
        // Edge Cases
        // =====================================================================

        #[test]
        fn test_insert_after_despawn_and_respawn() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            // Spawn new entity (might reuse slot)
            let new_entity = world.spawn_empty();
            world.insert(new_entity, Position { x: 10.0, y: 20.0 });

            // New entity should have its component
            assert_eq!(
                world.get::<Position>(new_entity),
                Some(&Position { x: 10.0, y: 20.0 })
            );

            // Old entity (stale) should not
            if entity.index() == new_entity.index() {
                assert!(world.get::<Position>(entity).is_none());
            }
        }

        #[test]
        fn test_insert_large_component() {
            #[derive(Debug, Clone, PartialEq)]
            struct LargeComponent {
                data: [u8; 1024],
            }
            impl Component for LargeComponent {}

            let mut world = World::new();
            let entity = world.spawn_empty();

            let large = LargeComponent { data: [42; 1024] };
            world.insert(entity, large.clone());

            assert_eq!(world.get::<LargeComponent>(entity), Some(&large));
        }

        #[test]
        fn test_insert_string_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, Name("Hello, World!".to_string()));

            assert_eq!(
                world.get::<Name>(entity),
                Some(&Name("Hello, World!".to_string()))
            );
        }

        #[test]
        fn test_insert_stress_many_entities() {
            let mut world = World::new();

            let entities = world.spawn_batch(10_000);

            for (i, &entity) in entities.iter().enumerate() {
                world.insert(
                    entity,
                    Position {
                        x: i as f32,
                        y: (i * 2) as f32,
                    },
                );
            }

            // Spot check
            assert_eq!(
                world.get::<Position>(entities[0]),
                Some(&Position { x: 0.0, y: 0.0 })
            );
            assert_eq!(
                world.get::<Position>(entities[5000]),
                Some(&Position {
                    x: 5000.0,
                    y: 10000.0
                })
            );
            assert_eq!(
                world.get::<Position>(entities[9999]),
                Some(&Position {
                    x: 9999.0,
                    y: 19998.0
                })
            );
        }

        #[test]
        fn test_insert_stress_many_component_types() {
            // Test with 10 different component types on single entity
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C1(u32);
            impl Component for C1 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C2(u32);
            impl Component for C2 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C3(u32);
            impl Component for C3 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C4(u32);
            impl Component for C4 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C5(u32);
            impl Component for C5 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C6(u32);
            impl Component for C6 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C7(u32);
            impl Component for C7 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C8(u32);
            impl Component for C8 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C9(u32);
            impl Component for C9 {}
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct C10(u32);
            impl Component for C10 {}

            let mut world = World::new();
            let entity = world.spawn_empty();

            world.insert(entity, C1(1));
            world.insert(entity, C2(2));
            world.insert(entity, C3(3));
            world.insert(entity, C4(4));
            world.insert(entity, C5(5));
            world.insert(entity, C6(6));
            world.insert(entity, C7(7));
            world.insert(entity, C8(8));
            world.insert(entity, C9(9));
            world.insert(entity, C10(10));

            assert_eq!(world.get::<C1>(entity), Some(&C1(1)));
            assert_eq!(world.get::<C5>(entity), Some(&C5(5)));
            assert_eq!(world.get::<C10>(entity), Some(&C10(10)));
            assert_eq!(world.component_type_count(), 10);
        }
    }

    // =========================================================================
    // Component Removal Tests (Step 2.4.6)
    // =========================================================================

    mod remove {
        use super::*;

        // =====================================================================
        // Basic Removal
        // =====================================================================

        #[test]
        fn test_remove_returns_component() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            let removed = world.remove::<Position>(entity);
            assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
        }

        #[test]
        fn test_remove_entity_no_longer_has_component() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            world.remove::<Position>(entity);

            assert!(!world.has::<Position>(entity));
            assert!(world.get::<Position>(entity).is_none());
        }

        #[test]
        fn test_remove_nonexistent_component_returns_none() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            let removed = world.remove::<Position>(entity);
            assert!(removed.is_none());
        }

        #[test]
        fn test_remove_twice_returns_none_second_time() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            let first = world.remove::<Position>(entity);
            let second = world.remove::<Position>(entity);

            assert_eq!(first, Some(Position { x: 1.0, y: 2.0 }));
            assert!(second.is_none());
        }

        #[test]
        fn test_remove_dead_entity_returns_none() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
            world.despawn(entity);

            let removed = world.remove::<Position>(entity);
            assert!(removed.is_none());
        }

        #[test]
        fn test_remove_placeholder_returns_none() {
            let mut world = World::new();

            let removed = world.remove::<Position>(Entity::PLACEHOLDER);
            assert!(removed.is_none());
        }

        #[test]
        fn test_remove_never_allocated_entity_returns_none() {
            let mut world = World::new();
            let fake = Entity::new(999, 1);

            let removed = world.remove::<Position>(fake);
            assert!(removed.is_none());
        }

        // =====================================================================
        // Archetype Transitions
        // =====================================================================

        #[test]
        fn test_remove_triggers_archetype_transition() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            // Entity should be in archetype with Position
            let archetype_before = world.entity_archetype(entity).unwrap();
            assert!(!archetype_before.is_empty()); // Not in empty archetype

            // Remove the component
            world.remove::<Position>(entity);

            // Entity should be in empty archetype
            let archetype_after = world.entity_archetype(entity).unwrap();
            assert!(archetype_after.is_empty()); // Now in empty archetype
        }

        #[test]
        fn test_remove_to_empty_archetype() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            world.remove::<Position>(entity);

            // Entity should still be alive
            assert!(world.is_alive(entity));

            // Entity should be in empty archetype
            let archetype_id = world.entity_archetype(entity).unwrap();
            assert_eq!(archetype_id, ArchetypeId::EMPTY);
        }

        #[test]
        fn test_remove_one_of_multiple_components() {
            let mut world = World::new();
            let entity = world
                .spawn()
                .insert(Position { x: 1.0, y: 2.0 })
                .insert(Velocity { x: 3.0, y: 4.0 })
                .id();

            // Remove only Position
            world.remove::<Position>(entity);

            // Should still have Velocity
            assert!(!world.has::<Position>(entity));
            assert!(world.has::<Velocity>(entity));
            assert_eq!(
                world.get::<Velocity>(entity),
                Some(&Velocity { x: 3.0, y: 4.0 })
            );
        }

        #[test]
        fn test_remove_creates_correct_target_archetype() {
            let mut world = World::new();

            // Create entity with Position + Velocity
            let entity1 = world
                .spawn()
                .insert(Position { x: 0.0, y: 0.0 })
                .insert(Velocity { x: 0.0, y: 0.0 })
                .id();

            // Create another entity with just Velocity
            let entity2 = world.spawn().insert(Velocity { x: 1.0, y: 1.0 }).id();

            // Get archetype for entity with just Velocity
            let velocity_archetype = world.entity_archetype(entity2).unwrap();

            // Remove Position from entity1
            world.remove::<Position>(entity1);

            // Entity1 should now be in same archetype as entity2 (just Velocity)
            assert_eq!(world.entity_archetype(entity1), Some(velocity_archetype));
        }

        #[test]
        fn test_remove_removes_from_old_archetype() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            let archetype_before = world.entity_archetype(entity).unwrap();

            // Verify entity is in old archetype
            {
                let arch = world.archetypes().get(archetype_before).unwrap();
                assert!(arch.contains_entity(entity));
            }

            // Remove component
            world.remove::<Position>(entity);

            // Verify entity is NOT in old archetype anymore
            {
                let arch = world.archetypes().get(archetype_before).unwrap();
                assert!(!arch.contains_entity(entity));
            }
        }

        #[test]
        fn test_remove_adds_to_new_archetype() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

            world.remove::<Position>(entity);

            // Verify entity is in empty archetype
            let arch = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
            assert!(arch.contains_entity(entity));
        }

        // =====================================================================
        // Take (alias for remove)
        // =====================================================================

        #[test]
        fn test_take_returns_component() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 5.0, y: 10.0 }).id();

            let taken = world.take::<Position>(entity);
            assert_eq!(taken, Some(Position { x: 5.0, y: 10.0 }));
        }

        #[test]
        fn test_take_removes_component() {
            let mut world = World::new();
            let entity = world.spawn().insert(Velocity { x: 1.0, y: 2.0 }).id();

            world.take::<Velocity>(entity);

            assert!(!world.has::<Velocity>(entity));
        }

        #[test]
        fn test_take_dead_entity_returns_none() {
            let mut world = World::new();
            let entity = world.spawn().insert(Position { x: 0.0, y: 0.0 }).id();
            world.despawn(entity);

            assert!(world.take::<Position>(entity).is_none());
        }

        // =====================================================================
        // Edge Cases
        // =====================================================================

        #[test]
        fn test_remove_after_despawn_and_respawn() {
            let mut world = World::new();

            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
            let old_entity = entity;
            world.despawn(entity);

            // Spawn new entity (might reuse slot)
            let new_entity = world.spawn().insert(Position { x: 10.0, y: 20.0 }).id();

            // New entity's component can be removed
            let removed = world.remove::<Position>(new_entity);
            assert_eq!(removed, Some(Position { x: 10.0, y: 20.0 }));

            // Old entity (stale) cannot have components removed
            if old_entity.index() == new_entity.index() {
                assert!(world.remove::<Position>(old_entity).is_none());
            }
        }

        #[test]
        fn test_remove_string_component() {
            let mut world = World::new();
            let entity = world.spawn().insert(Name("Test Name".to_string())).id();

            let removed = world.remove::<Name>(entity);
            assert_eq!(removed, Some(Name("Test Name".to_string())));
            assert!(!world.has::<Name>(entity));
        }

        #[test]
        fn test_remove_large_component() {
            #[derive(Debug, Clone, PartialEq)]
            struct LargeComponent {
                data: [u8; 1024],
            }
            impl Component for LargeComponent {}

            let mut world = World::new();
            let entity = world.spawn_empty();

            let large = LargeComponent { data: [42; 1024] };
            world.insert(entity, large.clone());

            let removed = world.remove::<LargeComponent>(entity);
            assert_eq!(removed, Some(large));
        }

        #[test]
        fn test_remove_stale_entity() {
            let mut world = World::new();

            let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
            let stale = entity;

            world.despawn(entity);

            // Spawn enough entities to potentially reuse the slot
            let _new_entity = world.spawn().insert(Position { x: 99.0, y: 99.0 }).id();

            // Stale entity should return None
            assert!(world.remove::<Position>(stale).is_none());
        }

        // =====================================================================
        // Stress Tests
        // =====================================================================

        #[test]
        fn test_remove_stress_many_entities() {
            let mut world = World::new();

            // Spawn entities with Position
            let entities: Vec<Entity> = (0..1000)
                .map(|i| {
                    world
                        .spawn()
                        .insert(Position {
                            x: i as f32,
                            y: (i * 2) as f32,
                        })
                        .id()
                })
                .collect();

            // Remove Position from all entities
            for (i, &entity) in entities.iter().enumerate() {
                let removed = world.remove::<Position>(entity);
                assert_eq!(
                    removed,
                    Some(Position {
                        x: i as f32,
                        y: (i * 2) as f32
                    })
                );
            }

            // All entities should be alive but without Position
            for &entity in &entities {
                assert!(world.is_alive(entity));
                assert!(!world.has::<Position>(entity));
                assert!(world.entity_archetype(entity) == Some(ArchetypeId::EMPTY));
            }
        }

        #[test]
        fn test_remove_add_cycle() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            // Add and remove the same component multiple times
            for i in 0..100 {
                world.insert(
                    entity,
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                );
                assert!(world.has::<Position>(entity));

                let removed = world.remove::<Position>(entity);
                assert_eq!(
                    removed,
                    Some(Position {
                        x: i as f32,
                        y: 0.0
                    })
                );
                assert!(!world.has::<Position>(entity));
            }

            // Entity should be alive and in empty archetype
            assert!(world.is_alive(entity));
            assert_eq!(world.entity_archetype(entity), Some(ArchetypeId::EMPTY));
        }

        #[test]
        fn test_remove_preserves_other_entities_components() {
            let mut world = World::new();

            // Create two entities with the same components
            let entity1 = world
                .spawn()
                .insert(Position { x: 1.0, y: 1.0 })
                .insert(Velocity { x: 1.0, y: 1.0 })
                .id();

            let entity2 = world
                .spawn()
                .insert(Position { x: 2.0, y: 2.0 })
                .insert(Velocity { x: 2.0, y: 2.0 })
                .id();

            // Remove Position from entity1 only
            world.remove::<Position>(entity1);

            // Entity1 should no longer have Position
            assert!(!world.has::<Position>(entity1));
            assert!(world.has::<Velocity>(entity1));

            // Entity2 should still have both components unchanged
            assert!(world.has::<Position>(entity2));
            assert!(world.has::<Velocity>(entity2));
            assert_eq!(
                world.get::<Position>(entity2),
                Some(&Position { x: 2.0, y: 2.0 })
            );
            assert_eq!(
                world.get::<Velocity>(entity2),
                Some(&Velocity { x: 2.0, y: 2.0 })
            );
        }

        #[test]
        fn test_remove_different_types_same_entity() {
            let mut world = World::new();

            let entity = world
                .spawn()
                .insert(Position { x: 1.0, y: 2.0 })
                .insert(Velocity { x: 3.0, y: 4.0 })
                .insert(Player)
                .id();

            // Remove each component type in sequence
            let pos = world.remove::<Position>(entity);
            assert_eq!(pos, Some(Position { x: 1.0, y: 2.0 }));
            assert!(world.has::<Velocity>(entity));
            assert!(world.has::<Player>(entity));

            let vel = world.remove::<Velocity>(entity);
            assert_eq!(vel, Some(Velocity { x: 3.0, y: 4.0 }));
            assert!(world.has::<Player>(entity));

            let player = world.remove::<Player>(entity);
            assert!(player.is_some()); // Player is a unit struct

            // Entity should be in empty archetype
            assert!(world.is_alive(entity));
            assert_eq!(world.entity_archetype(entity), Some(ArchetypeId::EMPTY));
        }
    }
}
