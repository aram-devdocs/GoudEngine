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

use std::collections::HashMap;

use super::archetype::{ArchetypeGraph, ArchetypeId};
use super::component::ComponentId;
use super::entity::{Entity, EntityAllocator};
use super::resource::{NonSendResources, Resources};

mod clone_entity;
mod component_access;
mod component_mutation;
mod entity_ops;
mod entity_world_mut;
mod non_send_resources;
mod resources;
mod storage_entry;

#[cfg(test)]
mod tests;

pub use entity_world_mut::EntityWorldMut;

use storage_entry::ComponentStorageEntry;

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

    /// Whether built-in components have been registered as cloneable.
    builtins_registered: bool,

    /// Current change tick, incremented at system boundaries.
    change_tick: u32,

    /// The change tick value from the end of the previous system run.
    /// Used by `Changed<T>` and `Added<T>` filters to detect new changes.
    last_change_tick: u32,
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
            builtins_registered: false,
            change_tick: 0,
            last_change_tick: 0,
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
            builtins_registered: false,
            change_tick: 0,
            last_change_tick: 0,
        }
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

        // Reset change detection ticks
        self.change_tick = 0;
        self.last_change_tick = 0;
    }

    // =========================================================================
    // Change Detection Ticks
    // =========================================================================

    /// Returns the current change tick.
    #[inline]
    pub fn change_tick(&self) -> u32 {
        self.change_tick
    }

    /// Returns the change tick from the end of the previous system run.
    #[inline]
    pub fn last_change_tick(&self) -> u32 {
        self.last_change_tick
    }

    /// Increments the change tick and returns the new value.
    #[inline]
    pub fn increment_change_tick(&mut self) -> u32 {
        self.change_tick += 1;
        self.change_tick
    }

    /// Sets the last change tick (typically called at the end of a system).
    #[inline]
    pub fn set_last_change_tick(&mut self, tick: u32) {
        self.last_change_tick = tick;
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
