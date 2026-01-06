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

use std::collections::{BTreeSet, HashMap};
use std::fmt;

use super::component::ComponentId;
use super::entity::Entity;

/// Unique identifier for an archetype.
///
/// Archetypes group entities that have the exact same set of components.
/// The `ArchetypeId` is used to efficiently look up and manage archetypes.
///
/// # Invariants
///
/// - `ArchetypeId(0)` is always the EMPTY archetype (no components)
/// - IDs are assigned sequentially as new archetypes are discovered
/// - IDs are stable within a single run but may differ between runs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    /// The empty archetype - contains entities with no components.
    ///
    /// This archetype always exists at index 0 in the archetype graph.
    /// Newly spawned entities without components start here.
    pub const EMPTY: Self = Self(0);

    /// Creates a new `ArchetypeId` with the given index.
    ///
    /// # Arguments
    ///
    /// * `id` - The numeric index for this archetype
    ///
    /// # Note
    ///
    /// This is typically called internally by the archetype graph.
    /// Users should not need to create archetype IDs manually.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the numeric index of this archetype.
    ///
    /// This can be used for indexing into archetype storage arrays.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.0
    }

    /// Returns whether this is the empty archetype (no components).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "ArchetypeId(EMPTY)")
        } else {
            write!(f, "ArchetypeId({})", self.0)
        }
    }
}

impl fmt::Display for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ArchetypeId {
    /// Returns the EMPTY archetype as the default.
    fn default() -> Self {
        Self::EMPTY
    }
}

impl From<u32> for ArchetypeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<ArchetypeId> for u32 {
    fn from(id: ArchetypeId) -> Self {
        id.0
    }
}

impl From<ArchetypeId> for usize {
    fn from(id: ArchetypeId) -> Self {
        id.0 as usize
    }
}

// =============================================================================
// Archetype
// =============================================================================

/// An archetype groups entities that have the exact same set of components.
///
/// Archetypes are a key optimization in ECS architecture. By grouping entities
/// with identical component sets together, the engine can:
///
/// - Iterate over components more cache-efficiently (contiguous memory)
/// - Quickly determine which entities match a query (by archetype, not entity)
/// - Manage component storage more efficiently
///
/// # Structure
///
/// Each archetype contains:
/// - A unique [`ArchetypeId`] for identification
/// - A sorted set of [`ComponentId`]s defining which components entities have
/// - A list of [`Entity`] references that belong to this archetype
///
/// # Component Set
///
/// The component set is stored as a `BTreeSet` for several reasons:
/// - Consistent ordering enables reliable hashing/comparison
/// - O(log n) membership tests
/// - Natural iteration order for debugging
///
/// # Entity Storage
///
/// Entities are stored in a dense `Vec` for cache-friendly iteration.
/// The order is not guaranteed and may change when entities are removed
/// (swap-remove pattern).
///
/// # Example
///
/// ```
/// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
/// use goud_engine::ecs::component::{Component, ComponentId};
/// use std::collections::BTreeSet;
///
/// // Define some components
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// // Create a component set
/// let mut components = BTreeSet::new();
/// components.insert(ComponentId::of::<Position>());
/// components.insert(ComponentId::of::<Velocity>());
///
/// // Create an archetype
/// let archetype = Archetype::new(ArchetypeId::new(1), components);
///
/// assert_eq!(archetype.id().index(), 1);
/// assert_eq!(archetype.component_count(), 2);
/// assert!(archetype.has_component(ComponentId::of::<Position>()));
/// assert!(archetype.has_component(ComponentId::of::<Velocity>()));
/// ```
#[derive(Debug, Clone)]
pub struct Archetype {
    /// Unique identifier for this archetype.
    id: ArchetypeId,

    /// The set of component types that entities in this archetype have.
    ///
    /// Using BTreeSet ensures consistent ordering, which is important for:
    /// - Reliable hashing of component sets
    /// - Deterministic behavior across runs
    /// - Debugging and inspection
    components: BTreeSet<ComponentId>,

    /// Entities that belong to this archetype.
    ///
    /// This is a dense array - when entities are removed, the last entity
    /// is swapped into the removed slot to maintain density.
    entities: Vec<Entity>,

    /// Maps entities to their index in the `entities` vector.
    ///
    /// This enables O(1) lookup for entity membership and removal.
    /// Must be kept in sync with `entities` vector.
    entity_indices: HashMap<Entity, usize>,
}

impl Archetype {
    /// Creates a new archetype with the given ID and component set.
    ///
    /// The archetype starts with no entities. Use [`add_entity`](Self::add_entity)
    /// (in future steps) to populate it.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for this archetype
    /// * `components` - The set of component types for entities in this archetype
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<Health>());
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(5), components);
    /// assert_eq!(archetype.id().index(), 5);
    /// ```
    #[inline]
    pub fn new(id: ArchetypeId, components: BTreeSet<ComponentId>) -> Self {
        Self {
            id,
            components,
            entities: Vec::new(),
            entity_indices: HashMap::new(),
        }
    }

    /// Creates a new archetype with pre-allocated entity capacity.
    ///
    /// Use this when you know approximately how many entities will be in
    /// this archetype to avoid reallocations.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for this archetype
    /// * `components` - The set of component types
    /// * `entity_capacity` - Initial capacity for the entity vector
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// // Create an empty archetype with capacity for 1000 entities
    /// let archetype = Archetype::with_capacity(
    ///     ArchetypeId::EMPTY,
    ///     BTreeSet::new(),
    ///     1000
    /// );
    /// assert!(archetype.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(
        id: ArchetypeId,
        components: BTreeSet<ComponentId>,
        entity_capacity: usize,
    ) -> Self {
        Self {
            id,
            components,
            entities: Vec::with_capacity(entity_capacity),
            entity_indices: HashMap::with_capacity(entity_capacity),
        }
    }

    /// Returns the unique identifier for this archetype.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(42), BTreeSet::new());
    /// assert_eq!(archetype.id().index(), 42);
    /// ```
    #[inline]
    pub const fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Returns a reference to the set of component types in this archetype.
    ///
    /// The set is sorted by `ComponentId` order, which is based on `TypeId`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct A; impl Component for A {}
    /// struct B; impl Component for B {}
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<A>());
    /// components.insert(ComponentId::of::<B>());
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(1), components.clone());
    /// assert_eq!(archetype.components(), &components);
    /// ```
    #[inline]
    pub fn components(&self) -> &BTreeSet<ComponentId> {
        &self.components
    }

    /// Checks if this archetype contains a specific component type.
    ///
    /// # Arguments
    ///
    /// * `id` - The component ID to check for
    ///
    /// # Returns
    ///
    /// `true` if entities in this archetype have the specified component.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<Position>());
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(1), components);
    ///
    /// assert!(archetype.has_component(ComponentId::of::<Position>()));
    /// assert!(!archetype.has_component(ComponentId::of::<Velocity>()));
    /// ```
    #[inline]
    pub fn has_component(&self, id: ComponentId) -> bool {
        self.components.contains(&id)
    }

    /// Returns a slice of all entities in this archetype.
    ///
    /// The order is not guaranteed - entities may be in any order and the
    /// order may change when entities are removed (due to swap-remove).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// assert!(archetype.entities().is_empty());
    /// ```
    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Returns the number of entities in this archetype.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// assert_eq!(archetype.len(), 0);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if this archetype contains no entities.
    ///
    /// Note that an empty archetype is still valid - it just has no entities
    /// currently assigned to it. This is different from the EMPTY archetype
    /// (which has no components).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(1), BTreeSet::new());
    /// assert!(archetype.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns the number of component types in this archetype.
    ///
    /// This is `0` for the empty archetype and increases with each
    /// unique component type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct A; impl Component for A {}
    /// struct B; impl Component for B {}
    ///
    /// let empty = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// assert_eq!(empty.component_count(), 0);
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<A>());
    /// components.insert(ComponentId::of::<B>());
    ///
    /// let with_components = Archetype::new(ArchetypeId::new(1), components);
    /// assert_eq!(with_components.component_count(), 2);
    /// ```
    #[inline]
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Checks if this archetype has no components (the empty archetype pattern).
    ///
    /// This is different from [`is_empty`](Self::is_empty) which checks for
    /// no entities. An archetype can have no components but still have entities
    /// (entities with no components exist in the EMPTY archetype).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// let empty = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// assert!(empty.has_no_components());
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<Health>());
    /// let with_health = Archetype::new(ArchetypeId::new(1), components);
    /// assert!(!with_health.has_no_components());
    /// ```
    #[inline]
    pub fn has_no_components(&self) -> bool {
        self.components.is_empty()
    }

    /// Checks if this archetype contains all the specified component types.
    ///
    /// This is useful for query matching - a query for `(&A, &B)` matches
    /// any archetype that has both A and B (and possibly more).
    ///
    /// # Arguments
    ///
    /// * `component_ids` - An iterator of component IDs to check
    ///
    /// # Returns
    ///
    /// `true` if this archetype has ALL of the specified components.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct A; impl Component for A {}
    /// struct B; impl Component for B {}
    /// struct C; impl Component for C {}
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<A>());
    /// components.insert(ComponentId::of::<B>());
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(1), components);
    ///
    /// // Has both A and B
    /// assert!(archetype.has_all(&[ComponentId::of::<A>(), ComponentId::of::<B>()]));
    ///
    /// // Has A alone
    /// assert!(archetype.has_all(&[ComponentId::of::<A>()]));
    ///
    /// // Does not have C
    /// assert!(!archetype.has_all(&[ComponentId::of::<C>()]));
    ///
    /// // Does not have all of A, B, C
    /// assert!(!archetype.has_all(&[ComponentId::of::<A>(), ComponentId::of::<C>()]));
    /// ```
    #[inline]
    pub fn has_all<'a>(&self, component_ids: impl IntoIterator<Item = &'a ComponentId>) -> bool {
        component_ids
            .into_iter()
            .all(|id| self.components.contains(id))
    }

    /// Checks if this archetype contains none of the specified component types.
    ///
    /// This is useful for query exclusion filters - a query that excludes
    /// certain components should skip archetypes that have any of them.
    ///
    /// # Arguments
    ///
    /// * `component_ids` - An iterator of component IDs to check
    ///
    /// # Returns
    ///
    /// `true` if this archetype has NONE of the specified components.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct A; impl Component for A {}
    /// struct B; impl Component for B {}
    /// struct C; impl Component for C {}
    ///
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<A>());
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(1), components);
    ///
    /// // Has none of B and C
    /// assert!(archetype.has_none(&[ComponentId::of::<B>(), ComponentId::of::<C>()]));
    ///
    /// // Does have A, so this returns false
    /// assert!(!archetype.has_none(&[ComponentId::of::<A>()]));
    /// assert!(!archetype.has_none(&[ComponentId::of::<A>(), ComponentId::of::<B>()]));
    /// ```
    #[inline]
    pub fn has_none<'a>(&self, component_ids: impl IntoIterator<Item = &'a ComponentId>) -> bool {
        component_ids
            .into_iter()
            .all(|id| !self.components.contains(id))
    }

    // =========================================================================
    // Entity Management
    // =========================================================================

    /// Adds an entity to this archetype.
    ///
    /// Returns the index at which the entity was inserted. This index can be
    /// used for dense component storage arrays that parallel the entity array.
    ///
    /// If the entity already exists in this archetype, this is a no-op and
    /// returns the existing index (idempotent behavior).
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add to this archetype
    ///
    /// # Returns
    ///
    /// The index of the entity in the dense `entities` array.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    ///
    /// let e1 = Entity::new(0, 1);
    /// let e2 = Entity::new(1, 1);
    ///
    /// let idx1 = archetype.add_entity(e1);
    /// let idx2 = archetype.add_entity(e2);
    ///
    /// assert_eq!(idx1, 0);
    /// assert_eq!(idx2, 1);
    /// assert_eq!(archetype.len(), 2);
    /// assert!(archetype.contains_entity(e1));
    /// assert!(archetype.contains_entity(e2));
    /// ```
    #[inline]
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        // If entity already exists, return its current index (idempotent)
        if let Some(&index) = self.entity_indices.get(&entity) {
            return index;
        }

        let index = self.entities.len();
        self.entities.push(entity);
        self.entity_indices.insert(entity, index);
        index
    }

    /// Removes an entity from this archetype.
    ///
    /// Uses swap-remove semantics: the last entity in the array is moved to
    /// fill the gap left by the removed entity. This maintains dense packing
    /// and is O(1), but does not preserve order.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove
    ///
    /// # Returns
    ///
    /// - `Some((removed_index, swapped_entity))` if the entity was found and removed.
    ///   `swapped_entity` is `Some(entity)` if another entity was moved to fill the gap,
    ///   or `None` if the removed entity was the last one.
    /// - `None` if the entity was not found in this archetype.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    ///
    /// let e1 = Entity::new(0, 1);
    /// let e2 = Entity::new(1, 1);
    /// let e3 = Entity::new(2, 1);
    ///
    /// archetype.add_entity(e1);
    /// archetype.add_entity(e2);
    /// archetype.add_entity(e3);
    ///
    /// // Remove e1 (at index 0) - e3 will be swapped into its place
    /// let result = archetype.remove_entity(e1);
    /// assert!(result.is_some());
    /// let (removed_idx, swapped) = result.unwrap();
    /// assert_eq!(removed_idx, 0);
    /// assert_eq!(swapped, Some(e3)); // e3 was moved to index 0
    ///
    /// assert_eq!(archetype.len(), 2);
    /// assert!(!archetype.contains_entity(e1));
    /// assert!(archetype.contains_entity(e2));
    /// assert!(archetype.contains_entity(e3));
    ///
    /// // e3 is now at index 0
    /// assert_eq!(archetype.entity_index(e3), Some(0));
    /// ```
    pub fn remove_entity(&mut self, entity: Entity) -> Option<(usize, Option<Entity>)> {
        // Get and remove the entity's index
        let index = self.entity_indices.remove(&entity)?;

        // Swap-remove from entities vector
        let last_index = self.entities.len() - 1;

        if index == last_index {
            // Removing the last entity - no swap needed
            self.entities.pop();
            Some((index, None))
        } else {
            // Swap the last entity into the removed slot
            let swapped_entity = self.entities[last_index];
            self.entities.swap_remove(index);

            // Update the swapped entity's index in the map
            self.entity_indices.insert(swapped_entity, index);

            Some((index, Some(swapped_entity)))
        }
    }

    /// Checks if an entity belongs to this archetype.
    ///
    /// This is an O(1) operation using the entity index map.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// `true` if the entity is in this archetype, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    ///
    /// let e1 = Entity::new(0, 1);
    /// let e2 = Entity::new(1, 1);
    ///
    /// archetype.add_entity(e1);
    ///
    /// assert!(archetype.contains_entity(e1));
    /// assert!(!archetype.contains_entity(e2));
    /// ```
    #[inline]
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entity_indices.contains_key(&entity)
    }

    /// Returns the dense array index of an entity within this archetype.
    ///
    /// This index corresponds to the position in the `entities()` slice and
    /// should be used to index into parallel component storage arrays.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Returns
    ///
    /// `Some(index)` if the entity is in this archetype, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    ///
    /// let e1 = Entity::new(0, 1);
    /// let e2 = Entity::new(1, 1);
    ///
    /// archetype.add_entity(e1);
    /// archetype.add_entity(e2);
    ///
    /// assert_eq!(archetype.entity_index(e1), Some(0));
    /// assert_eq!(archetype.entity_index(e2), Some(1));
    ///
    /// let e3 = Entity::new(2, 1);
    /// assert_eq!(archetype.entity_index(e3), None);
    /// ```
    #[inline]
    pub fn entity_index(&self, entity: Entity) -> Option<usize> {
        self.entity_indices.get(&entity).copied()
    }

    /// Clears all entities from this archetype.
    ///
    /// This removes all entities but preserves the archetype's component
    /// configuration. Useful for resetting state or clearing during cleanup.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    ///
    /// archetype.add_entity(Entity::new(0, 1));
    /// archetype.add_entity(Entity::new(1, 1));
    /// assert_eq!(archetype.len(), 2);
    ///
    /// archetype.clear_entities();
    /// assert!(archetype.is_empty());
    /// assert_eq!(archetype.len(), 0);
    /// ```
    #[inline]
    pub fn clear_entities(&mut self) {
        self.entities.clear();
        self.entity_indices.clear();
    }

    /// Reserves capacity for at least `additional` more entities.
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional entities to reserve space for
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// archetype.reserve_entities(1000);
    /// // Now adding up to 1000 entities won't cause reallocations
    /// ```
    #[inline]
    pub fn reserve_entities(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.entity_indices.reserve(additional);
    }
}

impl Default for Archetype {
    /// Creates the empty archetype (no components, no entities).
    ///
    /// Equivalent to `Archetype::new(ArchetypeId::EMPTY, BTreeSet::new())`.
    fn default() -> Self {
        Self::new(ArchetypeId::EMPTY, BTreeSet::new())
    }
}

// =============================================================================
// ArchetypeGraph
// =============================================================================

/// Manages all archetypes and their transition relationships.
///
/// The `ArchetypeGraph` is the central registry for archetypes in the ECS.
/// It tracks:
/// - All archetypes that have been discovered/created
/// - The relationships between archetypes (component add/remove transitions)
/// - Efficient lookup from component sets to archetype IDs
///
/// # Architecture
///
/// The graph maintains several key data structures:
///
/// - **archetypes**: A `Vec<Archetype>` where the index matches `ArchetypeId.index()`
/// - **component_index**: A `HashMap` mapping component sets to archetype IDs for
///   O(1) archetype lookup by components
/// - **edges**: Cached transitions for adding components to archetypes
/// - **remove_edges**: Cached transitions for removing components from archetypes
///
/// # Empty Archetype
///
/// The graph always contains the empty archetype at index 0. This is where
/// newly spawned entities without components initially reside.
///
/// # Archetype Discovery
///
/// New archetypes are created lazily when entities transition between component
/// configurations. The graph caches these transitions as "edges" for efficient
/// subsequent lookups.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::archetype::ArchetypeGraph;
/// use goud_engine::ecs::component::{Component, ComponentId};
/// use std::collections::BTreeSet;
///
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// let mut graph = ArchetypeGraph::new();
///
/// // Empty archetype always exists
/// assert!(graph.get(goud_engine::ecs::archetype::ArchetypeId::EMPTY).is_some());
///
/// // Find or create an archetype with Position
/// let mut components = BTreeSet::new();
/// components.insert(ComponentId::of::<Position>());
/// let pos_archetype = graph.find_or_create(components.clone());
///
/// // Same component set returns same archetype
/// let same = graph.find_or_create(components);
/// assert_eq!(pos_archetype, same);
///
/// // Different component set creates new archetype
/// let mut both = BTreeSet::new();
/// both.insert(ComponentId::of::<Position>());
/// both.insert(ComponentId::of::<Velocity>());
/// let both_archetype = graph.find_or_create(both);
/// assert_ne!(pos_archetype, both_archetype);
/// ```
#[derive(Debug)]
pub struct ArchetypeGraph {
    /// All archetypes in the graph.
    ///
    /// The index in this vector corresponds to `ArchetypeId.index()`.
    /// Index 0 is always the empty archetype.
    archetypes: Vec<Archetype>,

    /// Maps component sets to their archetype ID.
    ///
    /// This enables O(1) lookup of an archetype by its component composition.
    /// The `BTreeSet` key ensures consistent ordering for reliable hashing.
    component_index: HashMap<BTreeSet<ComponentId>, ArchetypeId>,

    /// Cached add transitions: (from_archetype, component_to_add) -> target_archetype
    ///
    /// When an entity in archetype A adds component C, this tells us which
    /// archetype it transitions to. Cached for performance since transitions
    /// are common operations.
    edges: HashMap<(ArchetypeId, ComponentId), ArchetypeId>,

    /// Cached remove transitions: (from_archetype, component_to_remove) -> target_archetype
    ///
    /// When an entity in archetype A removes component C, this tells us which
    /// archetype it transitions to. Only valid if archetype A actually has
    /// component C.
    remove_edges: HashMap<(ArchetypeId, ComponentId), ArchetypeId>,
}

impl ArchetypeGraph {
    /// Creates a new archetype graph with the empty archetype.
    ///
    /// The empty archetype (ID = 0, no components) is automatically created
    /// and will always exist. All newly spawned entities without components
    /// belong to this archetype.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    ///
    /// let graph = ArchetypeGraph::new();
    ///
    /// // Empty archetype exists
    /// assert!(graph.get(ArchetypeId::EMPTY).is_some());
    /// assert_eq!(graph.len(), 1);
    ///
    /// // The empty archetype has no components
    /// let empty = graph.get(ArchetypeId::EMPTY).unwrap();
    /// assert!(empty.has_no_components());
    /// ```
    pub fn new() -> Self {
        let empty_archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        let mut component_index = HashMap::new();
        component_index.insert(BTreeSet::new(), ArchetypeId::EMPTY);

        Self {
            archetypes: vec![empty_archetype],
            component_index,
            edges: HashMap::new(),
            remove_edges: HashMap::new(),
        }
    }

    /// Returns a reference to the archetype with the given ID.
    ///
    /// Returns `None` if no archetype with that ID exists (ID is out of bounds).
    ///
    /// # Arguments
    ///
    /// * `id` - The archetype ID to look up
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    ///
    /// let graph = ArchetypeGraph::new();
    ///
    /// // Empty archetype exists
    /// assert!(graph.get(ArchetypeId::EMPTY).is_some());
    ///
    /// // Non-existent archetype returns None
    /// assert!(graph.get(ArchetypeId::new(999)).is_none());
    /// ```
    #[inline]
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.index() as usize)
    }

    /// Returns a mutable reference to the archetype with the given ID.
    ///
    /// Returns `None` if no archetype with that ID exists (ID is out of bounds).
    ///
    /// # Arguments
    ///
    /// * `id` - The archetype ID to look up
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let mut graph = ArchetypeGraph::new();
    ///
    /// // Get mutable access to add an entity
    /// let empty = graph.get_mut(ArchetypeId::EMPTY).unwrap();
    /// let entity = Entity::new(0, 1);
    /// empty.add_entity(entity);
    ///
    /// // Verify the entity was added
    /// assert!(graph.get(ArchetypeId::EMPTY).unwrap().contains_entity(entity));
    /// ```
    #[inline]
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.index() as usize)
    }

    /// Finds an existing archetype with the given component set, or creates a new one.
    ///
    /// This is the primary way to obtain an archetype for a specific component
    /// configuration. If an archetype with exactly the specified components
    /// already exists, its ID is returned. Otherwise, a new archetype is created.
    ///
    /// # Arguments
    ///
    /// * `components` - The set of component types for the archetype
    ///
    /// # Returns
    ///
    /// The `ArchetypeId` of the archetype with the specified components.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut graph = ArchetypeGraph::new();
    ///
    /// // Create archetype with Position
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<Position>());
    ///
    /// let id1 = graph.find_or_create(components.clone());
    /// assert_ne!(id1, ArchetypeId::EMPTY); // Not empty archetype
    ///
    /// // Same components returns same ID
    /// let id2 = graph.find_or_create(components);
    /// assert_eq!(id1, id2);
    ///
    /// // Empty component set returns empty archetype
    /// let empty_id = graph.find_or_create(BTreeSet::new());
    /// assert_eq!(empty_id, ArchetypeId::EMPTY);
    /// ```
    pub fn find_or_create(&mut self, components: BTreeSet<ComponentId>) -> ArchetypeId {
        // Check if archetype already exists
        if let Some(&id) = self.component_index.get(&components) {
            return id;
        }

        // Create new archetype
        let id = ArchetypeId::new(self.archetypes.len() as u32);
        let archetype = Archetype::new(id, components.clone());

        self.archetypes.push(archetype);
        self.component_index.insert(components, id);

        id
    }

    /// Returns the number of archetypes in the graph.
    ///
    /// This always returns at least 1, since the empty archetype always exists.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::ArchetypeGraph;
    ///
    /// let graph = ArchetypeGraph::new();
    /// assert_eq!(graph.len(), 1); // Empty archetype
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    /// Returns whether the graph contains only the empty archetype.
    ///
    /// Note: This will always return `false` for a newly created graph since
    /// the empty archetype counts as one archetype. A "truly empty" graph is
    /// not possible.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::ArchetypeGraph;
    ///
    /// let graph = ArchetypeGraph::new();
    /// assert!(!graph.is_empty()); // Always has empty archetype
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty() // Always false in valid state
    }

    /// Returns an iterator over all archetypes in the graph.
    ///
    /// The iterator yields archetypes in ID order, with the empty archetype first.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    ///
    /// let graph = ArchetypeGraph::new();
    ///
    /// let archetypes: Vec<_> = graph.iter().collect();
    /// assert_eq!(archetypes.len(), 1);
    /// assert_eq!(archetypes[0].id(), ArchetypeId::EMPTY);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    /// Returns an iterator over all archetype IDs in the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    ///
    /// let graph = ArchetypeGraph::new();
    ///
    /// let ids: Vec<_> = graph.archetype_ids().collect();
    /// assert_eq!(ids, vec![ArchetypeId::EMPTY]);
    /// ```
    #[inline]
    pub fn archetype_ids(&self) -> impl Iterator<Item = ArchetypeId> + '_ {
        (0..self.archetypes.len()).map(|i| ArchetypeId::new(i as u32))
    }

    /// Checks if an archetype with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - The archetype ID to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    ///
    /// let graph = ArchetypeGraph::new();
    ///
    /// assert!(graph.contains(ArchetypeId::EMPTY));
    /// assert!(!graph.contains(ArchetypeId::new(100)));
    /// ```
    #[inline]
    pub fn contains(&self, id: ArchetypeId) -> bool {
        (id.index() as usize) < self.archetypes.len()
    }

    /// Finds the archetype ID for a given component set, if it exists.
    ///
    /// Unlike `find_or_create`, this does not create new archetypes.
    ///
    /// # Arguments
    ///
    /// * `components` - The component set to look up
    ///
    /// # Returns
    ///
    /// `Some(id)` if an archetype with exactly these components exists,
    /// `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut graph = ArchetypeGraph::new();
    ///
    /// // Empty set maps to empty archetype
    /// assert_eq!(graph.find(&BTreeSet::new()), Some(ArchetypeId::EMPTY));
    ///
    /// // Unknown component set returns None
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<Position>());
    /// assert_eq!(graph.find(&components), None);
    ///
    /// // After creating, it can be found
    /// graph.find_or_create(components.clone());
    /// assert!(graph.find(&components).is_some());
    /// ```
    #[inline]
    pub fn find(&self, components: &BTreeSet<ComponentId>) -> Option<ArchetypeId> {
        self.component_index.get(components).copied()
    }

    /// Returns the total number of entities across all archetypes.
    ///
    /// This is a computed value that iterates through all archetypes.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    ///
    /// let mut graph = ArchetypeGraph::new();
    ///
    /// assert_eq!(graph.entity_count(), 0);
    ///
    /// // Add some entities to the empty archetype
    /// let empty = graph.get_mut(ArchetypeId::EMPTY).unwrap();
    /// empty.add_entity(Entity::new(0, 1));
    /// empty.add_entity(Entity::new(1, 1));
    ///
    /// assert_eq!(graph.entity_count(), 2);
    /// ```
    pub fn entity_count(&self) -> usize {
        self.archetypes.iter().map(|a| a.len()).sum()
    }

    /// Returns the number of cached add edges.
    ///
    /// This is mainly useful for debugging and testing.
    #[inline]
    pub fn add_edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns the number of cached remove edges.
    ///
    /// This is mainly useful for debugging and testing.
    #[inline]
    pub fn remove_edge_count(&self) -> usize {
        self.remove_edges.len()
    }

    /// Clears all cached transition edges.
    ///
    /// This does not affect archetypes or their entities - only the cached
    /// transitions are cleared. Edges will be rebuilt lazily as transitions
    /// are requested.
    ///
    /// This is mainly useful for testing or when memory pressure requires
    /// clearing caches.
    pub fn clear_edge_cache(&mut self) {
        self.edges.clear();
        self.remove_edges.clear();
    }

    // =========================================================================
    // Archetype Transitions
    // =========================================================================

    /// Gets or creates the target archetype for adding a component.
    ///
    /// When an entity in archetype `from` adds component `component`, this returns
    /// the archetype ID the entity should transition to. The edge is cached for
    /// efficient subsequent lookups.
    ///
    /// # Arguments
    ///
    /// * `from` - The source archetype ID
    /// * `component` - The component being added
    ///
    /// # Returns
    ///
    /// The target `ArchetypeId` after adding the component. If the archetype
    /// already has the component, returns `from` (no transition needed).
    ///
    /// # Panics
    ///
    /// Panics if `from` does not exist in the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// let mut graph = ArchetypeGraph::new();
    ///
    /// // Start with empty archetype, add Position
    /// let pos_component = ComponentId::of::<Position>();
    /// let pos_archetype = graph.get_add_edge(ArchetypeId::EMPTY, pos_component);
    ///
    /// // pos_archetype is NOT empty
    /// assert_ne!(pos_archetype, ArchetypeId::EMPTY);
    ///
    /// // Adding same component again returns same archetype (no-op)
    /// let same = graph.get_add_edge(pos_archetype, pos_component);
    /// assert_eq!(same, pos_archetype);
    ///
    /// // Adding Velocity creates new archetype
    /// let vel_component = ComponentId::of::<Velocity>();
    /// let both_archetype = graph.get_add_edge(pos_archetype, vel_component);
    /// assert_ne!(both_archetype, pos_archetype);
    ///
    /// // The new archetype has both components
    /// let arch = graph.get(both_archetype).unwrap();
    /// assert!(arch.has_component(pos_component));
    /// assert!(arch.has_component(vel_component));
    /// ```
    pub fn get_add_edge(&mut self, from: ArchetypeId, component: ComponentId) -> ArchetypeId {
        // Check cached edge first
        let key = (from, component);
        if let Some(&target) = self.edges.get(&key) {
            return target;
        }

        // Get source archetype (panic if doesn't exist - programming error)
        let from_archetype = self
            .archetypes
            .get(from.index() as usize)
            .expect("source archetype must exist");

        // If archetype already has this component, edge points to self (no-op)
        if from_archetype.has_component(component) {
            self.edges.insert(key, from);
            return from;
        }

        // Create new component set with the added component
        let mut new_components = from_archetype.components().clone();
        new_components.insert(component);

        // Find or create the target archetype
        let target = self.find_or_create(new_components);

        // Cache the edge
        self.edges.insert(key, target);

        target
    }

    /// Gets or creates the target archetype for removing a component.
    ///
    /// When an entity in archetype `from` removes component `component`, this returns
    /// the archetype ID the entity should transition to. The edge is cached for
    /// efficient subsequent lookups.
    ///
    /// # Arguments
    ///
    /// * `from` - The source archetype ID
    /// * `component` - The component being removed
    ///
    /// # Returns
    ///
    /// - `Some(target)` - The target archetype after removing the component
    /// - `None` - If the archetype doesn't have the component (nothing to remove)
    ///
    /// # Panics
    ///
    /// Panics if `from` does not exist in the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    /// use std::collections::BTreeSet;
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// let mut graph = ArchetypeGraph::new();
    ///
    /// // Create archetype with Position and Velocity
    /// let pos_id = ComponentId::of::<Position>();
    /// let vel_id = ComponentId::of::<Velocity>();
    /// let mut components = BTreeSet::new();
    /// components.insert(pos_id);
    /// components.insert(vel_id);
    /// let both_archetype = graph.find_or_create(components);
    ///
    /// // Remove Velocity - should get archetype with only Position
    /// let pos_only = graph.get_remove_edge(both_archetype, vel_id);
    /// assert!(pos_only.is_some());
    /// let pos_only = pos_only.unwrap();
    ///
    /// let arch = graph.get(pos_only).unwrap();
    /// assert!(arch.has_component(pos_id));
    /// assert!(!arch.has_component(vel_id));
    ///
    /// // Remove Position from pos_only - should get empty archetype
    /// let empty = graph.get_remove_edge(pos_only, pos_id);
    /// assert_eq!(empty, Some(ArchetypeId::EMPTY));
    ///
    /// // Try to remove component that doesn't exist - returns None
    /// let health_id = ComponentId::of::<u32>(); // some other component
    /// let none = graph.get_remove_edge(pos_only, health_id);
    /// assert!(none.is_none());
    /// ```
    pub fn get_remove_edge(
        &mut self,
        from: ArchetypeId,
        component: ComponentId,
    ) -> Option<ArchetypeId> {
        // Check cached edge first
        let key = (from, component);
        if let Some(&target) = self.remove_edges.get(&key) {
            return Some(target);
        }

        // Get source archetype (panic if doesn't exist - programming error)
        let from_archetype = self
            .archetypes
            .get(from.index() as usize)
            .expect("source archetype must exist");

        // If archetype doesn't have this component, can't remove it
        if !from_archetype.has_component(component) {
            return None;
        }

        // Create new component set without the removed component
        let mut new_components = from_archetype.components().clone();
        new_components.remove(&component);

        // Find or create the target archetype
        let target = self.find_or_create(new_components);

        // Cache the edge
        self.remove_edges.insert(key, target);

        Some(target)
    }
}

impl Default for ArchetypeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    // ==================== ArchetypeId Structure Tests ====================

    #[test]
    fn test_archetype_id_empty_constant() {
        let empty = ArchetypeId::EMPTY;
        assert_eq!(empty.index(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_archetype_id_new() {
        let id = ArchetypeId::new(42);
        assert_eq!(id.index(), 42);
        assert!(!id.is_empty());
    }

    #[test]
    fn test_archetype_id_new_zero() {
        let id = ArchetypeId::new(0);
        assert_eq!(id.index(), 0);
        assert!(id.is_empty());
        assert_eq!(id, ArchetypeId::EMPTY);
    }

    #[test]
    fn test_archetype_id_max_value() {
        let id = ArchetypeId::new(u32::MAX);
        assert_eq!(id.index(), u32::MAX);
        assert!(!id.is_empty());
    }

    // ==================== Trait Implementation Tests ====================

    #[test]
    fn test_archetype_id_clone() {
        let id1 = ArchetypeId::new(5);
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_archetype_id_copy() {
        let id1 = ArchetypeId::new(5);
        let id2 = id1; // Copy
        assert_eq!(id1, id2);
        // id1 is still valid (Copy semantics)
        assert_eq!(id1.index(), 5);
    }

    #[test]
    fn test_archetype_id_equality() {
        let id1 = ArchetypeId::new(10);
        let id2 = ArchetypeId::new(10);
        let id3 = ArchetypeId::new(20);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_archetype_id_hash() {
        let mut set = HashSet::new();

        let id1 = ArchetypeId::new(1);
        let id2 = ArchetypeId::new(2);
        let id1_dup = ArchetypeId::new(1);

        set.insert(id1);
        set.insert(id2);
        set.insert(id1_dup); // Should not increase size

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }

    #[test]
    fn test_archetype_id_as_hashmap_key() {
        let mut map: HashMap<ArchetypeId, &str> = HashMap::new();

        map.insert(ArchetypeId::EMPTY, "empty");
        map.insert(ArchetypeId::new(1), "first");
        map.insert(ArchetypeId::new(2), "second");

        assert_eq!(map.get(&ArchetypeId::EMPTY), Some(&"empty"));
        assert_eq!(map.get(&ArchetypeId::new(1)), Some(&"first"));
        assert_eq!(map.get(&ArchetypeId::new(2)), Some(&"second"));
        assert_eq!(map.get(&ArchetypeId::new(99)), None);
    }

    #[test]
    fn test_archetype_id_debug_empty() {
        let empty = ArchetypeId::EMPTY;
        let debug_str = format!("{:?}", empty);
        assert_eq!(debug_str, "ArchetypeId(EMPTY)");
    }

    #[test]
    fn test_archetype_id_debug_non_empty() {
        let id = ArchetypeId::new(42);
        let debug_str = format!("{:?}", id);
        assert_eq!(debug_str, "ArchetypeId(42)");
    }

    #[test]
    fn test_archetype_id_display() {
        let empty = ArchetypeId::EMPTY;
        let id = ArchetypeId::new(42);

        assert_eq!(format!("{}", empty), "0");
        assert_eq!(format!("{}", id), "42");
    }

    #[test]
    fn test_archetype_id_default() {
        let default = ArchetypeId::default();
        assert_eq!(default, ArchetypeId::EMPTY);
        assert!(default.is_empty());
    }

    // ==================== Conversion Tests ====================

    #[test]
    fn test_archetype_id_from_u32() {
        let id: ArchetypeId = 100u32.into();
        assert_eq!(id.index(), 100);
    }

    #[test]
    fn test_archetype_id_into_u32() {
        let id = ArchetypeId::new(100);
        let val: u32 = id.into();
        assert_eq!(val, 100);
    }

    #[test]
    fn test_archetype_id_into_usize() {
        let id = ArchetypeId::new(42);
        let idx: usize = id.into();
        assert_eq!(idx, 42);
    }

    // ==================== Size and Layout Tests ====================

    #[test]
    fn test_archetype_id_size() {
        // ArchetypeId should be exactly 4 bytes (u32)
        assert_eq!(std::mem::size_of::<ArchetypeId>(), 4);
    }

    #[test]
    fn test_archetype_id_alignment() {
        // Should align to 4 bytes (u32 alignment)
        assert_eq!(std::mem::align_of::<ArchetypeId>(), 4);
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_archetype_id_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ArchetypeId>();
    }

    #[test]
    fn test_archetype_id_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ArchetypeId>();
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_archetype_id_sequential_creation() {
        // Simulate how archetype graph would create IDs
        let ids: Vec<ArchetypeId> = (0..100).map(ArchetypeId::new).collect();

        for (i, id) in ids.iter().enumerate() {
            assert_eq!(id.index() as usize, i);
        }
    }

    #[test]
    fn test_archetype_id_const_new() {
        // Verify const fn works at compile time
        const ID: ArchetypeId = ArchetypeId::new(42);
        assert_eq!(ID.index(), 42);
    }

    #[test]
    fn test_archetype_id_const_empty() {
        // Verify EMPTY constant is const
        const EMPTY: ArchetypeId = ArchetypeId::EMPTY;
        assert!(EMPTY.is_empty());
    }

    // ==================== Archetype Structure Tests ====================

    // Test component types
    use super::super::component::{Component, ComponentId};

    #[derive(Debug, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, Copy)]
    struct Health(f32);
    impl Component for Health {}

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    // Helper to create a component set
    fn make_components(ids: &[ComponentId]) -> BTreeSet<ComponentId> {
        ids.iter().cloned().collect()
    }

    #[test]
    fn test_archetype_new_empty() {
        let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        assert_eq!(archetype.id(), ArchetypeId::EMPTY);
        assert!(archetype.components().is_empty());
        assert!(archetype.entities().is_empty());
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
        assert_eq!(archetype.component_count(), 0);
        assert!(archetype.has_no_components());
    }

    #[test]
    fn test_archetype_new_with_components() {
        let components =
            make_components(&[ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]);

        let archetype = Archetype::new(ArchetypeId::new(1), components.clone());

        assert_eq!(archetype.id().index(), 1);
        assert_eq!(archetype.components(), &components);
        assert!(archetype.entities().is_empty());
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
        assert_eq!(archetype.component_count(), 2);
        assert!(!archetype.has_no_components());
    }

    #[test]
    fn test_archetype_with_capacity() {
        let archetype = Archetype::with_capacity(ArchetypeId::new(5), BTreeSet::new(), 1000);

        assert_eq!(archetype.id().index(), 5);
        assert!(archetype.is_empty());
        assert_eq!(archetype.len(), 0);
    }

    #[test]
    fn test_archetype_id_accessor() {
        let archetype = Archetype::new(ArchetypeId::new(42), BTreeSet::new());
        assert_eq!(archetype.id(), ArchetypeId::new(42));
        assert_eq!(archetype.id().index(), 42);
    }

    #[test]
    fn test_archetype_components_accessor() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let components = make_components(&[pos_id, vel_id, health_id]);
        let archetype = Archetype::new(ArchetypeId::new(1), components.clone());

        assert_eq!(archetype.components().len(), 3);
        assert!(archetype.components().contains(&pos_id));
        assert!(archetype.components().contains(&vel_id));
        assert!(archetype.components().contains(&health_id));
    }

    #[test]
    fn test_archetype_has_component() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let components = make_components(&[pos_id, vel_id]);
        let archetype = Archetype::new(ArchetypeId::new(1), components);

        assert!(archetype.has_component(pos_id));
        assert!(archetype.has_component(vel_id));
        assert!(!archetype.has_component(health_id));
    }

    #[test]
    fn test_archetype_entities_empty() {
        let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        assert!(archetype.entities().is_empty());
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
    }

    #[test]
    fn test_archetype_component_count() {
        // 0 components
        let empty = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        assert_eq!(empty.component_count(), 0);

        // 1 component
        let one = Archetype::new(
            ArchetypeId::new(1),
            make_components(&[ComponentId::of::<Position>()]),
        );
        assert_eq!(one.component_count(), 1);

        // 4 components
        let four = Archetype::new(
            ArchetypeId::new(2),
            make_components(&[
                ComponentId::of::<Position>(),
                ComponentId::of::<Velocity>(),
                ComponentId::of::<Health>(),
                ComponentId::of::<Player>(),
            ]),
        );
        assert_eq!(four.component_count(), 4);
    }

    #[test]
    fn test_archetype_has_no_components() {
        let empty = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
        assert!(empty.has_no_components());

        let with_one = Archetype::new(
            ArchetypeId::new(1),
            make_components(&[ComponentId::of::<Position>()]),
        );
        assert!(!with_one.has_no_components());
    }

    #[test]
    fn test_archetype_has_all() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let components = make_components(&[pos_id, vel_id]);
        let archetype = Archetype::new(ArchetypeId::new(1), components);

        // Has all of these
        assert!(archetype.has_all(&[pos_id]));
        assert!(archetype.has_all(&[vel_id]));
        assert!(archetype.has_all(&[pos_id, vel_id]));

        // Empty slice - trivially true
        assert!(archetype.has_all(&[]));

        // Does not have health
        assert!(!archetype.has_all(&[health_id]));
        assert!(!archetype.has_all(&[pos_id, health_id]));
        assert!(!archetype.has_all(&[pos_id, vel_id, health_id]));
    }

    #[test]
    fn test_archetype_has_none() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();
        let player_id = ComponentId::of::<Player>();

        let components = make_components(&[pos_id, vel_id]);
        let archetype = Archetype::new(ArchetypeId::new(1), components);

        // Has none of these (not in archetype)
        assert!(archetype.has_none(&[health_id]));
        assert!(archetype.has_none(&[player_id]));
        assert!(archetype.has_none(&[health_id, player_id]));

        // Empty slice - trivially true
        assert!(archetype.has_none(&[]));

        // Does have position
        assert!(!archetype.has_none(&[pos_id]));
        assert!(!archetype.has_none(&[pos_id, health_id])); // pos is present
        assert!(!archetype.has_none(&[pos_id, vel_id]));
    }

    #[test]
    fn test_archetype_default() {
        let default_archetype = Archetype::default();

        assert_eq!(default_archetype.id(), ArchetypeId::EMPTY);
        assert!(default_archetype.components().is_empty());
        assert!(default_archetype.entities().is_empty());
        assert!(default_archetype.is_empty());
        assert!(default_archetype.has_no_components());
    }

    #[test]
    fn test_archetype_clone() {
        let components =
            make_components(&[ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]);
        let archetype = Archetype::new(ArchetypeId::new(5), components.clone());

        let cloned = archetype.clone();

        assert_eq!(cloned.id(), archetype.id());
        assert_eq!(cloned.components(), archetype.components());
        assert_eq!(cloned.len(), archetype.len());
    }

    #[test]
    fn test_archetype_debug() {
        let components = make_components(&[ComponentId::of::<Position>()]);
        let archetype = Archetype::new(ArchetypeId::new(1), components);

        let debug_str = format!("{:?}", archetype);

        // Should contain Archetype and relevant info
        assert!(debug_str.contains("Archetype"));
        assert!(debug_str.contains("id"));
        assert!(debug_str.contains("components"));
        assert!(debug_str.contains("entities"));
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_archetype_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Archetype>();
    }

    #[test]
    fn test_archetype_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Archetype>();
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_archetype_many_components() {
        // Define a bunch of marker components for this test
        macro_rules! define_marker_components {
            ($($name:ident),*) => {
                $(
                    #[derive(Debug)]
                    struct $name;
                    impl Component for $name {}
                )*
            };
        }

        define_marker_components!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);

        let components = make_components(&[
            ComponentId::of::<C1>(),
            ComponentId::of::<C2>(),
            ComponentId::of::<C3>(),
            ComponentId::of::<C4>(),
            ComponentId::of::<C5>(),
            ComponentId::of::<C6>(),
            ComponentId::of::<C7>(),
            ComponentId::of::<C8>(),
            ComponentId::of::<C9>(),
            ComponentId::of::<C10>(),
        ]);

        let archetype = Archetype::new(ArchetypeId::new(1), components);
        assert_eq!(archetype.component_count(), 10);

        // Check has_all with subset
        assert!(archetype.has_all(&[ComponentId::of::<C1>(), ComponentId::of::<C5>()]));

        // Check has_component for each
        assert!(archetype.has_component(ComponentId::of::<C1>()));
        assert!(archetype.has_component(ComponentId::of::<C10>()));
    }

    #[test]
    fn test_archetype_component_set_is_sorted() {
        // BTreeSet should maintain consistent order
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        // Create with different insertion orders
        let mut set1 = BTreeSet::new();
        set1.insert(pos_id);
        set1.insert(vel_id);
        set1.insert(health_id);

        let mut set2 = BTreeSet::new();
        set2.insert(health_id);
        set2.insert(pos_id);
        set2.insert(vel_id);

        let mut set3 = BTreeSet::new();
        set3.insert(vel_id);
        set3.insert(health_id);
        set3.insert(pos_id);

        // All sets should be equal (same order)
        assert_eq!(set1, set2);
        assert_eq!(set2, set3);

        // Archetypes with these sets should have identical component views
        let arch1 = Archetype::new(ArchetypeId::new(1), set1);
        let arch2 = Archetype::new(ArchetypeId::new(2), set2);
        let arch3 = Archetype::new(ArchetypeId::new(3), set3);

        assert_eq!(arch1.components(), arch2.components());
        assert_eq!(arch2.components(), arch3.components());
    }

    #[test]
    fn test_archetype_empty_has_all_trivially_true() {
        let archetype = Archetype::default();

        // has_all with empty slice is true (vacuous truth)
        assert!(archetype.has_all(&[]));

        // has_all with any component is false
        assert!(!archetype.has_all(&[ComponentId::of::<Position>()]));
    }

    #[test]
    fn test_archetype_empty_has_none_trivially_true() {
        let archetype = Archetype::default();

        // has_none with empty slice is true (vacuous truth)
        assert!(archetype.has_none(&[]));

        // has_none with any component is true (empty has nothing)
        assert!(archetype.has_none(&[ComponentId::of::<Position>()]));
        assert!(archetype.has_none(&[ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]));
    }

    // ==================== Entity Management Tests ====================

    #[test]
    fn test_add_entity_basic() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        let idx1 = archetype.add_entity(e1);
        let idx2 = archetype.add_entity(e2);
        let idx3 = archetype.add_entity(e3);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(archetype.len(), 3);
    }

    #[test]
    fn test_add_entity_idempotent() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);

        let idx1 = archetype.add_entity(e1);
        let idx2 = archetype.add_entity(e1); // Add same entity again
        let idx3 = archetype.add_entity(e1); // And again

        // Should return same index each time
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0);
        assert_eq!(idx3, 0);

        // Should only have one entity
        assert_eq!(archetype.len(), 1);
    }

    #[test]
    fn test_contains_entity() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        assert!(archetype.contains_entity(e1));
        assert!(archetype.contains_entity(e2));
        assert!(!archetype.contains_entity(e3));
    }

    #[test]
    fn test_entity_index() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        assert_eq!(archetype.entity_index(e1), Some(0));
        assert_eq!(archetype.entity_index(e2), Some(1));
        assert_eq!(archetype.entity_index(e3), None);
    }

    #[test]
    fn test_remove_entity_last() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        // Remove the last entity - no swap needed
        let result = archetype.remove_entity(e2);

        assert!(result.is_some());
        let (removed_idx, swapped) = result.unwrap();
        assert_eq!(removed_idx, 1);
        assert!(swapped.is_none()); // No swap when removing last

        assert_eq!(archetype.len(), 1);
        assert!(archetype.contains_entity(e1));
        assert!(!archetype.contains_entity(e2));
    }

    #[test]
    fn test_remove_entity_swap_remove() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);
        archetype.add_entity(e3);

        // Verify initial indices
        assert_eq!(archetype.entity_index(e1), Some(0));
        assert_eq!(archetype.entity_index(e2), Some(1));
        assert_eq!(archetype.entity_index(e3), Some(2));

        // Remove e1 (at index 0) - e3 should be swapped into its place
        let result = archetype.remove_entity(e1);

        assert!(result.is_some());
        let (removed_idx, swapped) = result.unwrap();
        assert_eq!(removed_idx, 0);
        assert_eq!(swapped, Some(e3)); // e3 was moved to index 0

        assert_eq!(archetype.len(), 2);
        assert!(!archetype.contains_entity(e1));
        assert!(archetype.contains_entity(e2));
        assert!(archetype.contains_entity(e3));

        // Check that e3's index was updated
        assert_eq!(archetype.entity_index(e3), Some(0));
        assert_eq!(archetype.entity_index(e2), Some(1));

        // Verify entities() slice is consistent
        let entities = archetype.entities();
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0], e3); // e3 is now at index 0
        assert_eq!(entities[1], e2);
    }

    #[test]
    fn test_remove_entity_not_found() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        archetype.add_entity(e1);

        // Try to remove entity that's not in archetype
        let result = archetype.remove_entity(e2);

        assert!(result.is_none());
        assert_eq!(archetype.len(), 1);
        assert!(archetype.contains_entity(e1));
    }

    #[test]
    fn test_remove_entity_single() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);

        archetype.add_entity(e1);

        let result = archetype.remove_entity(e1);

        assert!(result.is_some());
        let (removed_idx, swapped) = result.unwrap();
        assert_eq!(removed_idx, 0);
        assert!(swapped.is_none());

        assert!(archetype.is_empty());
        assert!(!archetype.contains_entity(e1));
    }

    #[test]
    fn test_remove_entity_middle() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);
        let e4 = Entity::new(3, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);
        archetype.add_entity(e3);
        archetype.add_entity(e4);

        // Remove e2 (at index 1) - e4 should be swapped into its place
        let result = archetype.remove_entity(e2);

        assert!(result.is_some());
        let (removed_idx, swapped) = result.unwrap();
        assert_eq!(removed_idx, 1);
        assert_eq!(swapped, Some(e4));

        // Verify new state
        assert_eq!(archetype.len(), 3);
        assert_eq!(archetype.entity_index(e1), Some(0));
        assert_eq!(archetype.entity_index(e4), Some(1)); // e4 moved to index 1
        assert_eq!(archetype.entity_index(e3), Some(2));
        assert_eq!(archetype.entity_index(e2), None); // e2 removed
    }

    #[test]
    fn test_clear_entities() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);
        archetype.add_entity(e3);

        assert_eq!(archetype.len(), 3);

        archetype.clear_entities();

        assert!(archetype.is_empty());
        assert_eq!(archetype.len(), 0);
        assert!(!archetype.contains_entity(e1));
        assert!(!archetype.contains_entity(e2));
        assert!(!archetype.contains_entity(e3));
    }

    #[test]
    fn test_reserve_entities() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        archetype.reserve_entities(1000);

        // Should not change size, but capacity is increased (implementation detail)
        assert!(archetype.is_empty());

        // Add entities and verify no issues
        for i in 0..1000 {
            archetype.add_entity(Entity::new(i, 1));
        }

        assert_eq!(archetype.len(), 1000);
    }

    #[test]
    fn test_entity_management_with_components() {
        // Verify entity management works with component-bearing archetypes
        let components =
            make_components(&[ComponentId::of::<Position>(), ComponentId::of::<Velocity>()]);
        let mut archetype = Archetype::new(ArchetypeId::new(1), components);

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        assert!(archetype.contains_entity(e1));
        assert!(archetype.contains_entity(e2));
        assert_eq!(archetype.len(), 2);
        assert_eq!(archetype.component_count(), 2);

        archetype.remove_entity(e1);

        assert!(!archetype.contains_entity(e1));
        assert!(archetype.contains_entity(e2));
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.component_count(), 2); // Components unchanged
    }

    #[test]
    fn test_entity_management_stress() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        // Add many entities
        let entities: Vec<Entity> = (0..1000).map(|i| Entity::new(i, 1)).collect();

        for &entity in &entities {
            archetype.add_entity(entity);
        }

        assert_eq!(archetype.len(), 1000);

        // Verify all are present
        for &entity in &entities {
            assert!(archetype.contains_entity(entity));
        }

        // Remove every other entity
        for (i, &entity) in entities.iter().enumerate() {
            if i % 2 == 0 {
                let result = archetype.remove_entity(entity);
                assert!(result.is_some());
            }
        }

        assert_eq!(archetype.len(), 500);

        // Verify remaining entities
        for (i, &entity) in entities.iter().enumerate() {
            if i % 2 == 0 {
                assert!(!archetype.contains_entity(entity));
            } else {
                assert!(archetype.contains_entity(entity));
            }
        }
    }

    #[test]
    fn test_entity_index_consistency() {
        let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);
        let e4 = Entity::new(3, 1);
        let e5 = Entity::new(4, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);
        archetype.add_entity(e3);
        archetype.add_entity(e4);
        archetype.add_entity(e5);

        // After each removal, verify that entity_index is consistent with entities()
        archetype.remove_entity(e2);
        verify_index_consistency(&archetype);

        archetype.remove_entity(e4);
        verify_index_consistency(&archetype);

        archetype.remove_entity(e1);
        verify_index_consistency(&archetype);
    }

    // Helper to verify entity indices are consistent with the entities vector
    fn verify_index_consistency(archetype: &Archetype) {
        for (actual_idx, &entity) in archetype.entities().iter().enumerate() {
            let stored_idx = archetype.entity_index(entity);
            assert_eq!(
                stored_idx,
                Some(actual_idx),
                "Entity {:?} should be at index {}, but entity_index returned {:?}",
                entity,
                actual_idx,
                stored_idx
            );
        }
    }

    #[test]
    fn test_clone_with_entities() {
        let mut archetype = Archetype::new(ArchetypeId::new(1), BTreeSet::new());

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        archetype.add_entity(e1);
        archetype.add_entity(e2);

        let cloned = archetype.clone();

        // Cloned should have same entities
        assert_eq!(cloned.len(), 2);
        assert!(cloned.contains_entity(e1));
        assert!(cloned.contains_entity(e2));
        assert_eq!(cloned.entity_index(e1), Some(0));
        assert_eq!(cloned.entity_index(e2), Some(1));

        // Modifications to original shouldn't affect clone
        // (This test just verifies independent clones - Rust ownership handles this)
    }

    #[test]
    fn test_debug_with_entities() {
        let mut archetype = Archetype::new(ArchetypeId::new(1), BTreeSet::new());

        let e1 = Entity::new(0, 1);
        archetype.add_entity(e1);

        let debug_str = format!("{:?}", archetype);

        // Debug should include entity_indices
        assert!(debug_str.contains("entity_indices"));
    }

    // ==================== ArchetypeGraph Tests ====================

    #[test]
    fn test_graph_new() {
        let graph = ArchetypeGraph::new();

        // Should have exactly one archetype (empty)
        assert_eq!(graph.len(), 1);
        assert!(!graph.is_empty()); // Always has empty archetype

        // Empty archetype should exist
        let empty = graph.get(ArchetypeId::EMPTY);
        assert!(empty.is_some());

        let empty = empty.unwrap();
        assert_eq!(empty.id(), ArchetypeId::EMPTY);
        assert!(empty.has_no_components());
        assert!(empty.is_empty());
    }

    #[test]
    fn test_graph_default() {
        let graph = ArchetypeGraph::default();

        assert_eq!(graph.len(), 1);
        assert!(graph.get(ArchetypeId::EMPTY).is_some());
    }

    #[test]
    fn test_graph_get_empty_archetype() {
        let graph = ArchetypeGraph::new();

        let empty = graph.get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty.id(), ArchetypeId::EMPTY);
        assert!(empty.components().is_empty());
    }

    #[test]
    fn test_graph_get_nonexistent() {
        let graph = ArchetypeGraph::new();

        // Non-existent archetypes return None
        assert!(graph.get(ArchetypeId::new(1)).is_none());
        assert!(graph.get(ArchetypeId::new(100)).is_none());
        assert!(graph.get(ArchetypeId::new(u32::MAX)).is_none());
    }

    #[test]
    fn test_graph_get_mut() {
        let mut graph = ArchetypeGraph::new();

        // Modify empty archetype
        let empty = graph.get_mut(ArchetypeId::EMPTY).unwrap();
        let entity = Entity::new(0, 1);
        empty.add_entity(entity);

        // Verify change persisted
        assert!(graph
            .get(ArchetypeId::EMPTY)
            .unwrap()
            .contains_entity(entity));
    }

    #[test]
    fn test_graph_get_mut_nonexistent() {
        let mut graph = ArchetypeGraph::new();

        assert!(graph.get_mut(ArchetypeId::new(1)).is_none());
        assert!(graph.get_mut(ArchetypeId::new(999)).is_none());
    }

    #[test]
    fn test_graph_find_or_create_empty() {
        let mut graph = ArchetypeGraph::new();

        // Finding empty component set should return empty archetype
        let id = graph.find_or_create(BTreeSet::new());
        assert_eq!(id, ArchetypeId::EMPTY);
        assert_eq!(graph.len(), 1); // No new archetype created
    }

    #[test]
    fn test_graph_find_or_create_new() {
        let mut graph = ArchetypeGraph::new();

        // Create archetype with one component
        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());

        let id = graph.find_or_create(components);

        assert_ne!(id, ArchetypeId::EMPTY);
        assert_eq!(id.index(), 1); // Second archetype
        assert_eq!(graph.len(), 2);

        // Verify the archetype has correct components
        let archetype = graph.get(id).unwrap();
        assert!(archetype.has_component(ComponentId::of::<Position>()));
        assert!(!archetype.has_component(ComponentId::of::<Velocity>()));
    }

    #[test]
    fn test_graph_find_or_create_existing() {
        let mut graph = ArchetypeGraph::new();

        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());

        // Create the archetype
        let id1 = graph.find_or_create(components.clone());
        assert_eq!(graph.len(), 2);

        // Finding same component set should return same ID
        let id2 = graph.find_or_create(components.clone());
        assert_eq!(id1, id2);
        assert_eq!(graph.len(), 2); // No new archetype

        // And again
        let id3 = graph.find_or_create(components);
        assert_eq!(id1, id3);
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn test_graph_find_or_create_multiple() {
        let mut graph = ArchetypeGraph::new();

        // Create archetype with Position
        let mut pos_components = BTreeSet::new();
        pos_components.insert(ComponentId::of::<Position>());
        let pos_id = graph.find_or_create(pos_components);

        // Create archetype with Velocity
        let mut vel_components = BTreeSet::new();
        vel_components.insert(ComponentId::of::<Velocity>());
        let vel_id = graph.find_or_create(vel_components);

        // Create archetype with both
        let mut both_components = BTreeSet::new();
        both_components.insert(ComponentId::of::<Position>());
        both_components.insert(ComponentId::of::<Velocity>());
        let both_id = graph.find_or_create(both_components);

        // All should be different
        assert_ne!(pos_id, vel_id);
        assert_ne!(pos_id, both_id);
        assert_ne!(vel_id, both_id);
        assert_ne!(pos_id, ArchetypeId::EMPTY);
        assert_ne!(vel_id, ArchetypeId::EMPTY);
        assert_ne!(both_id, ArchetypeId::EMPTY);

        // Total of 4 archetypes (empty + 3 created)
        assert_eq!(graph.len(), 4);
    }

    #[test]
    fn test_graph_find() {
        let mut graph = ArchetypeGraph::new();

        // Empty set always findable
        assert_eq!(graph.find(&BTreeSet::new()), Some(ArchetypeId::EMPTY));

        // Unknown set not found
        let mut pos_components = BTreeSet::new();
        pos_components.insert(ComponentId::of::<Position>());
        assert_eq!(graph.find(&pos_components), None);

        // After creating, it's found
        let id = graph.find_or_create(pos_components.clone());
        assert_eq!(graph.find(&pos_components), Some(id));
    }

    #[test]
    fn test_graph_contains() {
        let mut graph = ArchetypeGraph::new();

        assert!(graph.contains(ArchetypeId::EMPTY));
        assert!(!graph.contains(ArchetypeId::new(1)));
        assert!(!graph.contains(ArchetypeId::new(100)));

        // Create an archetype
        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());
        let id = graph.find_or_create(components);

        assert!(graph.contains(id));
        assert!(!graph.contains(ArchetypeId::new(2)));
    }

    #[test]
    fn test_graph_iter() {
        let mut graph = ArchetypeGraph::new();

        // Create some archetypes
        let mut pos_components = BTreeSet::new();
        pos_components.insert(ComponentId::of::<Position>());
        graph.find_or_create(pos_components);

        let mut vel_components = BTreeSet::new();
        vel_components.insert(ComponentId::of::<Velocity>());
        graph.find_or_create(vel_components);

        // Collect archetypes via iterator
        let archetypes: Vec<_> = graph.iter().collect();
        assert_eq!(archetypes.len(), 3);

        // First should be empty archetype
        assert_eq!(archetypes[0].id(), ArchetypeId::EMPTY);

        // Others have components
        assert!(archetypes[1].component_count() > 0 || archetypes[2].component_count() > 0);
    }

    #[test]
    fn test_graph_archetype_ids() {
        let mut graph = ArchetypeGraph::new();

        // Create some archetypes
        let mut pos_components = BTreeSet::new();
        pos_components.insert(ComponentId::of::<Position>());
        let pos_id = graph.find_or_create(pos_components);

        let ids: Vec<_> = graph.archetype_ids().collect();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], ArchetypeId::EMPTY);
        assert_eq!(ids[1], pos_id);
    }

    #[test]
    fn test_graph_entity_count() {
        let mut graph = ArchetypeGraph::new();

        assert_eq!(graph.entity_count(), 0);

        // Add entities to empty archetype
        let empty = graph.get_mut(ArchetypeId::EMPTY).unwrap();
        empty.add_entity(Entity::new(0, 1));
        empty.add_entity(Entity::new(1, 1));

        assert_eq!(graph.entity_count(), 2);

        // Create another archetype with entities
        let mut components = BTreeSet::new();
        components.insert(ComponentId::of::<Position>());
        let pos_id = graph.find_or_create(components);

        let pos_arch = graph.get_mut(pos_id).unwrap();
        pos_arch.add_entity(Entity::new(2, 1));
        pos_arch.add_entity(Entity::new(3, 1));
        pos_arch.add_entity(Entity::new(4, 1));

        assert_eq!(graph.entity_count(), 5); // 2 + 3
    }

    #[test]
    fn test_graph_edge_count_initial() {
        let graph = ArchetypeGraph::new();

        // No edges initially
        assert_eq!(graph.add_edge_count(), 0);
        assert_eq!(graph.remove_edge_count(), 0);
    }

    #[test]
    fn test_graph_clear_edge_cache() {
        let mut graph = ArchetypeGraph::new();

        // Note: edges are populated by get_add_edge/get_remove_edge (Step 2.3.5)
        // For now, just test that clear doesn't crash
        graph.clear_edge_cache();

        assert_eq!(graph.add_edge_count(), 0);
        assert_eq!(graph.remove_edge_count(), 0);
    }

    #[test]
    fn test_graph_debug() {
        let graph = ArchetypeGraph::new();

        let debug_str = format!("{:?}", graph);

        assert!(debug_str.contains("ArchetypeGraph"));
        assert!(debug_str.contains("archetypes"));
        assert!(debug_str.contains("component_index"));
        assert!(debug_str.contains("edges"));
    }

    #[test]
    fn test_graph_many_archetypes() {
        let mut graph = ArchetypeGraph::new();

        // Define many marker components
        macro_rules! define_graph_marker_components {
            ($($name:ident),*) => {
                $(
                    #[derive(Debug)]
                    struct $name;
                    impl Component for $name {}
                )*
            };
        }

        define_graph_marker_components!(G1, G2, G3, G4, G5, G6, G7, G8);

        // Create archetypes with different combinations
        let component_ids = [
            ComponentId::of::<G1>(),
            ComponentId::of::<G2>(),
            ComponentId::of::<G3>(),
            ComponentId::of::<G4>(),
            ComponentId::of::<G5>(),
            ComponentId::of::<G6>(),
            ComponentId::of::<G7>(),
            ComponentId::of::<G8>(),
        ];

        // Create archetypes with 1, 2, 3, ... components
        let mut created_ids = Vec::new();
        for i in 1..=8 {
            let mut components = BTreeSet::new();
            for id in &component_ids[0..i] {
                components.insert(*id);
            }
            let arch_id = graph.find_or_create(components);
            created_ids.push(arch_id);
        }

        // Should have 9 archetypes (empty + 8)
        assert_eq!(graph.len(), 9);

        // All IDs should be unique
        for (i, id1) in created_ids.iter().enumerate() {
            for (j, id2) in created_ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id1, id2);
                }
            }
        }
    }

    #[test]
    fn test_graph_component_order_independence() {
        let mut graph = ArchetypeGraph::new();

        // Create component sets with different insertion orders
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        let mut set1 = BTreeSet::new();
        set1.insert(pos_id);
        set1.insert(vel_id);
        set1.insert(health_id);

        let mut set2 = BTreeSet::new();
        set2.insert(health_id);
        set2.insert(pos_id);
        set2.insert(vel_id);

        let id1 = graph.find_or_create(set1);
        let id2 = graph.find_or_create(set2);

        // Same components, regardless of insertion order
        assert_eq!(id1, id2);
        assert_eq!(graph.len(), 2); // empty + one archetype
    }

    #[test]
    fn test_graph_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ArchetypeGraph>();
        assert_sync::<ArchetypeGraph>();
    }

    #[test]
    fn test_graph_stress_find_or_create() {
        let mut graph = ArchetypeGraph::new();

        // Create many archetypes with different combinations
        // Using Position, Velocity, Health, Player as base components
        let pos = ComponentId::of::<Position>();
        let vel = ComponentId::of::<Velocity>();
        let health = ComponentId::of::<Health>();
        let player = ComponentId::of::<Player>();

        let base_ids = [pos, vel, health, player];

        // Create all 2^4 = 16 possible combinations
        for mask in 0u32..16 {
            let mut components = BTreeSet::new();
            for (i, id) in base_ids.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    components.insert(*id);
                }
            }
            graph.find_or_create(components);
        }

        assert_eq!(graph.len(), 16); // All 16 combinations including empty

        // Verify all can be found again
        for mask in 0u32..16 {
            let mut components = BTreeSet::new();
            for (i, id) in base_ids.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    components.insert(*id);
                }
            }
            assert!(graph.find(&components).is_some());
        }
    }

    // ==================== Archetype Transition Tests ====================

    #[test]
    fn test_get_add_edge_from_empty() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        // Add Position to empty archetype
        let target = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // Target should be a new archetype (not empty)
        assert_ne!(target, ArchetypeId::EMPTY);

        // Target archetype should have Position
        let target_arch = graph.get(target).unwrap();
        assert!(target_arch.has_component(pos_id));
        assert_eq!(target_arch.component_count(), 1);

        // Edge should be cached
        assert_eq!(graph.add_edge_count(), 1);
    }

    #[test]
    fn test_get_add_edge_existing_component() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        // Create archetype with Position
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // Adding Position again should return same archetype (no-op)
        let same = graph.get_add_edge(pos_arch, pos_id);
        assert_eq!(same, pos_arch);

        // Should have cached this edge too
        assert_eq!(graph.add_edge_count(), 2);
    }

    #[test]
    fn test_get_add_edge_multiple_components() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        // empty -> Position
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // Position -> Position+Velocity
        let both_arch = graph.get_add_edge(pos_arch, vel_id);

        // Verify both archetype has both components
        let both = graph.get(both_arch).unwrap();
        assert!(both.has_component(pos_id));
        assert!(both.has_component(vel_id));
        assert_eq!(both.component_count(), 2);

        // Should have 3 archetypes: empty, Position, Position+Velocity
        assert_eq!(graph.len(), 3);
    }

    #[test]
    fn test_get_add_edge_caching() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        // First call creates edge
        let target1 = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(graph.add_edge_count(), 1);

        // Second call should use cache
        let target2 = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(target1, target2);
        assert_eq!(graph.add_edge_count(), 1); // Still 1, used cache
    }

    #[test]
    fn test_get_remove_edge_basic() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        // Create archetype with Position
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // Remove Position -> should go back to empty
        let target = graph.get_remove_edge(pos_arch, pos_id);
        assert_eq!(target, Some(ArchetypeId::EMPTY));

        // Edge should be cached
        assert_eq!(graph.remove_edge_count(), 1);
    }

    #[test]
    fn test_get_remove_edge_component_not_present() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        // Create archetype with only Position
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // Try to remove Velocity (not present) -> None
        let result = graph.get_remove_edge(pos_arch, vel_id);
        assert_eq!(result, None);

        // No edge should be cached for this
        assert_eq!(graph.remove_edge_count(), 0);
    }

    #[test]
    fn test_get_remove_edge_from_empty() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        // Try to remove from empty archetype -> None
        let result = graph.get_remove_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_remove_edge_to_existing_archetype() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        // Create archetype with Position
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // Create archetype with Position+Velocity
        let both_arch = graph.get_add_edge(pos_arch, vel_id);

        // Remove Velocity from both -> should return existing Position archetype
        let result = graph.get_remove_edge(both_arch, vel_id);
        assert_eq!(result, Some(pos_arch));
    }

    #[test]
    fn test_get_remove_edge_caching() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();

        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);

        // First call creates edge
        let target1 = graph.get_remove_edge(pos_arch, pos_id);
        assert_eq!(graph.remove_edge_count(), 1);

        // Second call should use cache
        let target2 = graph.get_remove_edge(pos_arch, pos_id);
        assert_eq!(target1, target2);
        assert_eq!(graph.remove_edge_count(), 1); // Still 1, used cache
    }

    #[test]
    fn test_transition_roundtrip() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        // Start at empty
        let mut current = ArchetypeId::EMPTY;

        // Add Position
        current = graph.get_add_edge(current, pos_id);
        assert!(graph.get(current).unwrap().has_component(pos_id));

        // Add Velocity
        current = graph.get_add_edge(current, vel_id);
        let arch = graph.get(current).unwrap();
        assert!(arch.has_component(pos_id));
        assert!(arch.has_component(vel_id));

        // Add Health
        current = graph.get_add_edge(current, health_id);
        let arch = graph.get(current).unwrap();
        assert_eq!(arch.component_count(), 3);

        // Now remove in different order
        // Remove Velocity
        current = graph.get_remove_edge(current, vel_id).unwrap();
        let arch = graph.get(current).unwrap();
        assert!(arch.has_component(pos_id));
        assert!(!arch.has_component(vel_id));
        assert!(arch.has_component(health_id));
        assert_eq!(arch.component_count(), 2);

        // Remove Position
        current = graph.get_remove_edge(current, pos_id).unwrap();
        let arch = graph.get(current).unwrap();
        assert!(!arch.has_component(pos_id));
        assert!(arch.has_component(health_id));
        assert_eq!(arch.component_count(), 1);

        // Remove Health - back to empty
        current = graph.get_remove_edge(current, health_id).unwrap();
        assert_eq!(current, ArchetypeId::EMPTY);
    }

    #[test]
    fn test_transition_creates_correct_archetypes() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        // Two paths to same destination: empty -> A+B
        // Path 1: empty -> A -> A+B
        let via_a = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        let via_a_then_b = graph.get_add_edge(via_a, vel_id);

        // Path 2: empty -> B -> A+B
        let via_b = graph.get_add_edge(ArchetypeId::EMPTY, vel_id);
        let via_b_then_a = graph.get_add_edge(via_b, pos_id);

        // Both paths should reach the same archetype
        assert_eq!(via_a_then_b, via_b_then_a);

        // Should have created 4 archetypes: empty, A, B, A+B
        assert_eq!(graph.len(), 4);
    }

    #[test]
    fn test_transition_edge_count_after_clear() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();

        // Create some edges
        let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        graph.get_add_edge(pos_arch, vel_id);
        graph.get_remove_edge(pos_arch, pos_id);

        assert!(graph.add_edge_count() > 0);
        assert!(graph.remove_edge_count() > 0);

        // Clear cache
        graph.clear_edge_cache();
        assert_eq!(graph.add_edge_count(), 0);
        assert_eq!(graph.remove_edge_count(), 0);

        // Edges can be recreated
        let pos_arch2 = graph.get_add_edge(ArchetypeId::EMPTY, pos_id);
        assert_eq!(pos_arch, pos_arch2); // Same archetype
        assert_eq!(graph.add_edge_count(), 1);
    }

    #[test]
    fn test_transition_stress() {
        let mut graph = ArchetypeGraph::new();
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();
        let player_id = ComponentId::of::<Player>();

        let components = [pos_id, vel_id, health_id, player_id];

        // Perform many transitions
        let mut current = ArchetypeId::EMPTY;

        // Add all components
        for comp in &components {
            current = graph.get_add_edge(current, *comp);
        }
        assert_eq!(graph.get(current).unwrap().component_count(), 4);

        // Remove all components
        for comp in &components {
            if let Some(next) = graph.get_remove_edge(current, *comp) {
                current = next;
            }
        }
        assert_eq!(current, ArchetypeId::EMPTY);

        // Many edges should be cached now
        assert!(graph.add_edge_count() >= 4);
        assert!(graph.remove_edge_count() >= 4);
    }
}
