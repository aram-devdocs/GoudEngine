//! [`Archetype`] type — groups entities with identical component sets.

use std::collections::{BTreeSet, HashMap};

use super::super::component::ComponentId;
use super::super::entity::Entity;
use super::archetype_id::ArchetypeId;

/// An archetype groups entities that have the exact same set of components.
///
/// Archetypes are a key optimization in ECS architecture. By grouping entities
/// with identical component sets together, the engine can iterate components
/// cache-efficiently and quickly match queries by archetype rather than entity.
///
/// # Structure
///
/// Each archetype contains:
/// - A unique [`ArchetypeId`] for identification
/// - A sorted set of [`ComponentId`]s defining which components entities have
/// - A dense [`Vec`] of [`Entity`] references for cache-friendly iteration
/// - A [`HashMap`] mapping entities to their dense-array index for O(1) lookup
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
/// let mut components = BTreeSet::new();
/// components.insert(ComponentId::of::<Position>());
/// let archetype = Archetype::new(ArchetypeId::new(1), components);
///
/// assert_eq!(archetype.id().index(), 1);
/// assert_eq!(archetype.component_count(), 1);
/// assert!(archetype.has_component(ComponentId::of::<Position>()));
/// ```
#[derive(Debug, Clone)]
pub struct Archetype {
    /// Unique identifier for this archetype.
    id: ArchetypeId,
    /// The sorted set of component types that entities in this archetype have.
    components: BTreeSet<ComponentId>,
    /// Dense entity array for cache-friendly iteration. Swap-remove on deletion.
    entities: Vec<Entity>,
    /// Maps entities to their index in the `entities` vector (O(1) lookup/removal).
    entity_indices: HashMap<Entity, usize>,
}

impl Archetype {
    /// Creates a new archetype with the given ID and component set.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let archetype = Archetype::new(ArchetypeId::new(5), BTreeSet::new());
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
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use std::collections::BTreeSet;
    ///
    /// let archetype = Archetype::with_capacity(ArchetypeId::EMPTY, BTreeSet::new(), 1000);
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
    #[inline]
    pub const fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Returns a reference to the set of component types in this archetype.
    #[inline]
    pub fn components(&self) -> &BTreeSet<ComponentId> {
        &self.components
    }

    /// Returns `true` if entities in this archetype have the specified component.
    #[inline]
    pub fn has_component(&self, id: ComponentId) -> bool {
        self.components.contains(&id)
    }

    /// Returns a slice of all entities in this archetype.
    ///
    /// Order is not guaranteed — swap-remove may change positions on deletion.
    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Returns the number of entities in this archetype.
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if this archetype contains no entities.
    ///
    /// Note: this differs from the EMPTY archetype concept (no components).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns the number of component types in this archetype.
    #[inline]
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Returns `true` if this archetype has no component types.
    ///
    /// Distinct from [`is_empty`](Self::is_empty), which checks for no entities.
    #[inline]
    pub fn has_no_components(&self) -> bool {
        self.components.is_empty()
    }

    /// Returns `true` if this archetype has ALL of the specified component types.
    ///
    /// Useful for query matching — a query for `(&A, &B)` matches any archetype
    /// that has both A and B (and possibly more).
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
    /// let archetype = Archetype::new(ArchetypeId::new(1), components);
    ///
    /// assert!(archetype.has_all(&[ComponentId::of::<A>(), ComponentId::of::<B>()]));
    /// assert!(archetype.has_all(&[])); // empty slice is vacuously true
    /// ```
    #[inline]
    pub fn has_all<'a>(&self, component_ids: impl IntoIterator<Item = &'a ComponentId>) -> bool {
        component_ids
            .into_iter()
            .all(|id| self.components.contains(id))
    }

    /// Returns `true` if this archetype has NONE of the specified component types.
    ///
    /// Useful for query exclusion filters.
    #[inline]
    pub fn has_none<'a>(&self, component_ids: impl IntoIterator<Item = &'a ComponentId>) -> bool {
        component_ids
            .into_iter()
            .all(|id| !self.components.contains(id))
    }

    // =========================================================================
    // Entity Management
    // =========================================================================

    /// Adds an entity to this archetype, returning its dense-array index.
    ///
    /// If the entity already exists, returns its current index (idempotent).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// let e1 = Entity::new(0, 1);
    /// let e2 = Entity::new(1, 1);
    ///
    /// assert_eq!(archetype.add_entity(e1), 0);
    /// assert_eq!(archetype.add_entity(e2), 1);
    /// assert_eq!(archetype.len(), 2);
    /// ```
    #[inline]
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        if let Some(&index) = self.entity_indices.get(&entity) {
            return index;
        }
        let index = self.entities.len();
        self.entities.push(entity);
        self.entity_indices.insert(entity, index);
        index
    }

    /// Removes an entity using swap-remove semantics (O(1), does not preserve order).
    ///
    /// # Returns
    ///
    /// - `Some((removed_index, Some(swapped)))` — entity removed, another was moved
    /// - `Some((removed_index, None))` — entity removed, it was the last one
    /// - `None` — entity not found in this archetype
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{Archetype, ArchetypeId};
    /// use goud_engine::ecs::entity::Entity;
    /// use std::collections::BTreeSet;
    ///
    /// let mut archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
    /// let e1 = Entity::new(0, 1);
    /// let e2 = Entity::new(1, 1);
    /// let e3 = Entity::new(2, 1);
    /// archetype.add_entity(e1);
    /// archetype.add_entity(e2);
    /// archetype.add_entity(e3);
    ///
    /// // Remove e1; e3 is swapped into index 0
    /// let (idx, swapped) = archetype.remove_entity(e1).unwrap();
    /// assert_eq!(idx, 0);
    /// assert_eq!(swapped, Some(e3));
    /// assert_eq!(archetype.entity_index(e3), Some(0));
    /// ```
    pub fn remove_entity(&mut self, entity: Entity) -> Option<(usize, Option<Entity>)> {
        let index = self.entity_indices.remove(&entity)?;
        let last_index = self.entities.len() - 1;

        if index == last_index {
            self.entities.pop();
            Some((index, None))
        } else {
            let swapped_entity = self.entities[last_index];
            self.entities.swap_remove(index);
            self.entity_indices.insert(swapped_entity, index);
            Some((index, Some(swapped_entity)))
        }
    }

    /// Returns `true` if the entity belongs to this archetype (O(1)).
    #[inline]
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entity_indices.contains_key(&entity)
    }

    /// Returns the dense-array index of an entity, or `None` if not present.
    #[inline]
    pub fn entity_index(&self, entity: Entity) -> Option<usize> {
        self.entity_indices.get(&entity).copied()
    }

    /// Clears all entities while preserving the archetype's component configuration.
    #[inline]
    pub fn clear_entities(&mut self) {
        self.entities.clear();
        self.entity_indices.clear();
    }

    /// Reserves capacity for at least `additional` more entities.
    #[inline]
    pub fn reserve_entities(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.entity_indices.reserve(additional);
    }
}

impl Default for Archetype {
    /// Creates the empty archetype (no components, no entities).
    fn default() -> Self {
        Self::new(ArchetypeId::EMPTY, BTreeSet::new())
    }
}
