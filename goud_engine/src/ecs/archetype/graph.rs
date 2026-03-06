//! [`ArchetypeGraph`] — registry for all archetypes and their transitions.

use std::collections::{BTreeSet, HashMap};

use super::super::component::ComponentId;
use super::archetype_id::ArchetypeId;
use super::storage::Archetype;

/// Manages all archetypes and their transition relationships.
///
/// The `ArchetypeGraph` is the central registry for archetypes in the ECS.
/// It tracks:
/// - All archetypes (dense `Vec` indexed by `ArchetypeId`)
/// - A component-set-to-ID index for O(1) archetype lookup
/// - Cached add/remove transition edges for efficient component changes
///
/// The empty archetype (ID = 0, no components) is always present at index 0.
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
/// assert!(graph.get(ArchetypeId::EMPTY).is_some());
///
/// let mut components = BTreeSet::new();
/// components.insert(ComponentId::of::<Position>());
/// let id = graph.find_or_create(components.clone());
/// assert_eq!(graph.find_or_create(components), id); // same set -> same ID
/// ```
#[derive(Debug)]
pub struct ArchetypeGraph {
    /// All archetypes; index == `ArchetypeId.index()`. Index 0 is always the empty archetype.
    archetypes: Vec<Archetype>,
    /// Maps component sets to their archetype ID for O(1) lookup.
    component_index: HashMap<BTreeSet<ComponentId>, ArchetypeId>,
    /// Cached add transitions: `(from, component) -> target`.
    edges: HashMap<(ArchetypeId, ComponentId), ArchetypeId>,
    /// Cached remove transitions: `(from, component) -> target`.
    remove_edges: HashMap<(ArchetypeId, ComponentId), ArchetypeId>,
}

impl ArchetypeGraph {
    /// Creates a new archetype graph containing only the empty archetype.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    ///
    /// let graph = ArchetypeGraph::new();
    /// assert!(graph.get(ArchetypeId::EMPTY).is_some());
    /// assert_eq!(graph.len(), 1);
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

    /// Returns a reference to the archetype with the given ID, or `None` if out of range.
    #[inline]
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.index() as usize)
    }

    /// Returns a mutable reference to the archetype with the given ID, or `None` if out of range.
    #[inline]
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.index() as usize)
    }

    /// Finds an existing archetype with the given component set, or creates a new one.
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
    /// let mut components = BTreeSet::new();
    /// components.insert(ComponentId::of::<Position>());
    ///
    /// let id1 = graph.find_or_create(components.clone());
    /// let id2 = graph.find_or_create(components);
    /// assert_eq!(id1, id2); // same set -> same ID
    ///
    /// assert_eq!(graph.find_or_create(BTreeSet::new()), ArchetypeId::EMPTY);
    /// ```
    pub fn find_or_create(&mut self, components: BTreeSet<ComponentId>) -> ArchetypeId {
        if let Some(&id) = self.component_index.get(&components) {
            return id;
        }

        let id = ArchetypeId::new(self.archetypes.len() as u32);
        let archetype = Archetype::new(id, components.clone());
        self.archetypes.push(archetype);
        self.component_index.insert(components, id);
        id
    }

    /// Returns the number of archetypes in the graph (always >= 1).
    #[inline]
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    /// Returns `false` in all valid states (the empty archetype always exists).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }

    /// Returns an iterator over all archetypes in ID order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    /// Returns an iterator over all archetype IDs in ID order.
    #[inline]
    pub fn archetype_ids(&self) -> impl Iterator<Item = ArchetypeId> + '_ {
        (0..self.archetypes.len()).map(|i| ArchetypeId::new(i as u32))
    }

    /// Returns `true` if an archetype with the given ID exists.
    #[inline]
    pub fn contains(&self, id: ArchetypeId) -> bool {
        (id.index() as usize) < self.archetypes.len()
    }

    /// Finds the archetype ID for a given component set without creating one.
    ///
    /// Returns `None` if no archetype with exactly those components exists.
    #[inline]
    pub fn find(&self, components: &BTreeSet<ComponentId>) -> Option<ArchetypeId> {
        self.component_index.get(components).copied()
    }

    /// Returns the total number of entities across all archetypes (computed).
    pub fn entity_count(&self) -> usize {
        self.archetypes.iter().map(|a| a.len()).sum()
    }

    /// Returns the number of cached add-transition edges.
    #[inline]
    pub fn add_edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns the number of cached remove-transition edges.
    #[inline]
    pub fn remove_edge_count(&self) -> usize {
        self.remove_edges.len()
    }

    /// Clears all cached transition edges without affecting archetypes or entities.
    ///
    /// Edges are rebuilt lazily as transitions are requested.
    pub fn clear_edge_cache(&mut self) {
        self.edges.clear();
        self.remove_edges.clear();
    }

    // =========================================================================
    // Archetype Transitions
    // =========================================================================

    /// Gets or creates the target archetype for adding `component` to `from`.
    ///
    /// The edge is cached for efficient subsequent lookups. If `from` already
    /// has `component`, returns `from` (no-op transition).
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
    /// let mut graph = ArchetypeGraph::new();
    /// let pos = ComponentId::of::<Position>();
    /// let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos);
    ///
    /// assert_ne!(pos_arch, ArchetypeId::EMPTY);
    /// assert!(graph.get(pos_arch).unwrap().has_component(pos));
    ///
    /// // Adding same component is a no-op
    /// assert_eq!(graph.get_add_edge(pos_arch, pos), pos_arch);
    /// ```
    pub fn get_add_edge(&mut self, from: ArchetypeId, component: ComponentId) -> ArchetypeId {
        let key = (from, component);
        if let Some(&target) = self.edges.get(&key) {
            return target;
        }

        let from_archetype = self
            .archetypes
            .get(from.index() as usize)
            .expect("source archetype must exist");

        if from_archetype.has_component(component) {
            self.edges.insert(key, from);
            return from;
        }

        let mut new_components = from_archetype.components().clone();
        new_components.insert(component);
        let target = self.find_or_create(new_components);
        self.edges.insert(key, target);
        target
    }

    /// Gets or creates the target archetype for removing `component` from `from`.
    ///
    /// The edge is cached for efficient subsequent lookups.
    ///
    /// # Returns
    ///
    /// - `Some(target)` — the archetype after removing the component
    /// - `None` — `from` does not have `component`
    ///
    /// # Panics
    ///
    /// Panics if `from` does not exist in the graph.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::archetype::{ArchetypeGraph, ArchetypeId};
    /// use goud_engine::ecs::component::{Component, ComponentId};
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut graph = ArchetypeGraph::new();
    /// let pos = ComponentId::of::<Position>();
    /// let pos_arch = graph.get_add_edge(ArchetypeId::EMPTY, pos);
    ///
    /// // Remove Position -> back to empty
    /// assert_eq!(graph.get_remove_edge(pos_arch, pos), Some(ArchetypeId::EMPTY));
    ///
    /// // Component not present -> None
    /// assert_eq!(graph.get_remove_edge(ArchetypeId::EMPTY, pos), None);
    /// ```
    pub fn get_remove_edge(
        &mut self,
        from: ArchetypeId,
        component: ComponentId,
    ) -> Option<ArchetypeId> {
        let key = (from, component);
        if let Some(&target) = self.remove_edges.get(&key) {
            return Some(target);
        }

        let from_archetype = self
            .archetypes
            .get(from.index() as usize)
            .expect("source archetype must exist");

        if !from_archetype.has_component(component) {
            return None;
        }

        let mut new_components = from_archetype.components().clone();
        new_components.remove(&component);
        let target = self.find_or_create(new_components);
        self.remove_edges.insert(key, target);
        Some(target)
    }
}

impl Default for ArchetypeGraph {
    fn default() -> Self {
        Self::new()
    }
}
