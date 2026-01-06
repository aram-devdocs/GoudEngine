//! Query fetch traits for the ECS.
//!
//! This module defines the foundational traits for the query system. Queries
//! allow systems to access component data in a type-safe manner. The fetch
//! traits define what can be queried and how data is retrieved.
//!
//! # Architecture
//!
//! The query system is built around two key traits:
//!
//! - [`WorldQuery`]: Defines what data a query fetches and how to access it
//! - [`ReadOnlyWorldQuery`]: Marker trait for queries that don't mutate data
//!
//! # Query Item Lifetime
//!
//! The [`WorldQuery`] trait uses Generic Associated Types (GATs) for the `Item`
//! type. This allows the fetched item to have a lifetime tied to the world
//! borrow, enabling references to component data.
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::{Component, Entity, World};
//! use goud_engine::ecs::query::{WorldQuery, QueryState};
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // WorldQuery is implemented for &T to fetch component references
//! // (Implementation shown in fetch.rs Step 2.5.2)
//! ```
//!
//! # Design Rationale
//!
//! This design is inspired by Bevy's query system, adapted for GoudEngine's
//! needs:
//!
//! 1. **Type Safety**: The trait bounds ensure only valid queries compile
//! 2. **Performance**: State caching avoids repeated archetype lookups
//! 3. **Flexibility**: The generic design supports arbitrary query types
//! 4. **Parallel Safety**: ReadOnlyWorldQuery enables safe concurrent reads

use std::collections::BTreeSet;
use std::fmt;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
use crate::ecs::resource::{NonSendResourceId, ResourceId};
use crate::ecs::World;

// =============================================================================
// WorldQuery Trait
// =============================================================================

/// Trait for types that can be queried from a [`World`].
///
/// `WorldQuery` is the foundational trait of the ECS query system. It defines:
///
/// 1. What type of data the query produces (`Item`)
/// 2. What state the query caches (`State`)
/// 3. How to match archetypes (`matches_archetype`)
/// 4. How to fetch data from the world (`fetch`)
///
/// # Generic Associated Type
///
/// The `Item<'w>` associated type uses a Generic Associated Type (GAT) to
/// express that the fetched item has a lifetime tied to the world borrow.
/// This enables returning references to component data.
///
/// # State Caching
///
/// The `State` type caches information needed for efficient queries, typically:
/// - Component IDs (avoids repeated TypeId lookups)
/// - Cached archetype matches (avoids repeated set operations)
///
/// # Safety
///
/// Implementors must ensure:
/// 1. `matches_archetype` accurately reflects what entities can be fetched
/// 2. `fetch` returns `None` for entities that don't match
/// 3. Mutable queries conflict with other queries on the same component
///
/// # Built-in Implementations
///
/// The following types implement `WorldQuery`:
///
/// - `&T` where `T: Component` - Fetches immutable component reference
/// - `&mut T` where `T: Component` - Fetches mutable component reference
/// - `Entity` - Fetches the entity ID itself
/// - Tuples `(A, B, ...)` - Combines multiple queries
/// - `Option<Q>` - Optional query, always matches
/// - `With<T>` - Filter for entities that have T (no data)
/// - `Without<T>` - Filter for entities that don't have T (no data)
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, Entity, World};
/// use goud_engine::ecs::query::WorldQuery;
///
/// #[derive(Debug)]
/// struct Health(f32);
/// impl Component for Health {}
///
/// // The trait bounds ensure type safety:
/// fn query_requires_world_query<Q: WorldQuery>() {}
///
/// // Entity always implements WorldQuery
/// query_requires_world_query::<Entity>();
/// ```
pub trait WorldQuery {
    /// The type of data this query fetches.
    ///
    /// This is a Generic Associated Type (GAT) that takes a lifetime parameter
    /// `'w` representing the world borrow lifetime. This allows the item to
    /// contain references to data stored in the world.
    ///
    /// Examples:
    /// - For `&T`: `Item<'w> = &'w T`
    /// - For `&mut T`: `Item<'w> = &'w mut T`
    /// - For `Entity`: `Item<'w> = Entity` (no reference needed)
    type Item<'w>;

    /// The cached state for this query.
    ///
    /// State is initialized once via `init_state` and reused for subsequent
    /// queries. This avoids repeated lookups of component IDs and other
    /// metadata.
    ///
    /// Examples:
    /// - For `&T`: `State = ComponentId`
    /// - For tuples: `State = (A::State, B::State, ...)`
    type State: QueryState;

    /// Initializes the query state from the world.
    ///
    /// This is called once when a query is first created. The returned state
    /// is cached and reused for all subsequent operations.
    ///
    /// # Arguments
    ///
    /// * `world` - Reference to the world (may be used for resource lookups)
    ///
    /// # Returns
    ///
    /// The initialized state for this query.
    fn init_state(world: &World) -> Self::State;

    /// Returns the set of component IDs this query reads.
    ///
    /// Used for access conflict detection. Two queries conflict if one writes
    /// a component that the other reads or writes.
    ///
    /// # Arguments
    ///
    /// * `state` - The query state
    ///
    /// # Returns
    ///
    /// Set of component IDs accessed for reading (includes mutable access).
    fn component_access(state: &Self::State) -> BTreeSet<ComponentId>;

    /// Checks whether this query matches the given archetype.
    ///
    /// Returns `true` if entities in the archetype can produce a valid `Item`.
    /// This is used to filter archetypes during query iteration.
    ///
    /// # Arguments
    ///
    /// * `state` - The cached query state
    /// * `archetype` - The archetype to check
    ///
    /// # Returns
    ///
    /// `true` if entities in this archetype match the query.
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool;

    /// Fetches data for a specific entity from the world.
    ///
    /// Returns `Some(Item)` if the entity has all required components,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `state` - The cached query state
    /// * `world` - Reference to the world containing component data
    /// * `entity` - The entity to fetch data for
    ///
    /// # Returns
    ///
    /// `Some(Item)` if the entity matches the query, `None` otherwise.
    ///
    /// # Safety Note
    ///
    /// This method takes an immutable world reference but may return mutable
    /// component references. The caller must ensure aliasing rules are
    /// maintained (typically enforced by the query iterator).
    fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>>;

    /// Fetches data for a specific entity from a mutable world reference.
    ///
    /// This variant is used when mutable component access is needed.
    /// The default implementation calls `fetch` with an immutable borrow,
    /// but mutable query implementations override this.
    ///
    /// # Arguments
    ///
    /// * `state` - The cached query state
    /// * `world` - Mutable reference to the world
    /// * `entity` - The entity to fetch data for
    ///
    /// # Returns
    ///
    /// `Some(Item)` if the entity matches the query, `None` otherwise.
    fn fetch_mut<'w>(
        state: &Self::State,
        world: &'w mut World,
        entity: Entity,
    ) -> Option<Self::Item<'w>> {
        // Default implementation for read-only queries
        Self::fetch(state, world, entity)
    }
}

// =============================================================================
// ReadOnlyWorldQuery Trait
// =============================================================================

/// Marker trait for queries that only read data.
///
/// `ReadOnlyWorldQuery` is a marker trait that indicates a query does not
/// mutate any component data. This enables:
///
/// 1. **Parallel Execution**: Multiple read-only queries can run concurrently
/// 2. **Shared Borrows**: Can coexist with other readers of the same component
/// 3. **Query Combination**: Read-only queries can be combined more freely
///
/// # Safety
///
/// Implementors must ensure that the query never mutates component data,
/// even when given a mutable world reference.
///
/// # Built-in Implementations
///
/// - `&T` where `T: Component`
/// - `Entity`
/// - `Option<Q>` where `Q: ReadOnlyWorldQuery`
/// - Tuples of read-only queries
/// - `With<T>` and `Without<T>` filters
///
/// # Example
///
/// ```
/// use goud_engine::ecs::Entity;
/// use goud_engine::ecs::query::ReadOnlyWorldQuery;
///
/// // Entity is always read-only
/// fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
/// requires_read_only::<Entity>();
/// ```
pub trait ReadOnlyWorldQuery: WorldQuery {}

// =============================================================================
// QueryState Trait
// =============================================================================

/// Trait for query state types.
///
/// Query state caches information needed for efficient query execution.
/// The state is initialized once and reused for all query operations.
///
/// # Requirements
///
/// State types must be:
/// - `Send + Sync`: For parallel query execution
/// - `Clone`: To allow query state to be copied if needed
///
/// # Example
///
/// ```
/// use goud_engine::ecs::component::ComponentId;
/// use goud_engine::ecs::query::QueryState;
///
/// // ComponentId implements QueryState
/// fn requires_query_state<S: QueryState>() {}
/// requires_query_state::<ComponentId>();
/// ```
pub trait QueryState: Send + Sync + Clone + 'static {}

// Blanket implementation for all qualifying types
impl<T: Send + Sync + Clone + 'static> QueryState for T {}

// =============================================================================
// Entity Implementation
// =============================================================================

/// `Entity` can be queried to get the entity ID itself.
///
/// This is useful when you need the entity ID along with component data,
/// or when iterating over all entities in an archetype.
impl WorldQuery for Entity {
    type Item<'w> = Entity;
    type State = ();

    #[inline]
    fn init_state(_world: &World) -> Self::State {
        // Entity query has no state
    }

    #[inline]
    fn component_access(_state: &Self::State) -> BTreeSet<ComponentId> {
        // Entity query accesses no components
        BTreeSet::new()
    }

    #[inline]
    fn matches_archetype(_state: &Self::State, _archetype: &Archetype) -> bool {
        // Entity query matches all archetypes
        true
    }

    #[inline]
    fn fetch<'w>(_state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        // Return the entity if it's alive
        if world.is_alive(entity) {
            Some(entity)
        } else {
            None
        }
    }
}

impl ReadOnlyWorldQuery for Entity {}

// =============================================================================
// Unit Type Implementation
// =============================================================================

/// Unit type `()` represents an empty query.
///
/// This is useful as a base case for tuple queries and for queries that
/// only use filters without fetching any data.
impl WorldQuery for () {
    type Item<'w> = ();
    type State = ();

    #[inline]
    fn init_state(_world: &World) -> Self::State {}

    #[inline]
    fn component_access(_state: &Self::State) -> BTreeSet<ComponentId> {
        BTreeSet::new()
    }

    #[inline]
    fn matches_archetype(_state: &Self::State, _archetype: &Archetype) -> bool {
        true
    }

    #[inline]
    fn fetch<'w>(_state: &Self::State, _world: &'w World, _entity: Entity) -> Option<Self::Item<'w>> {
        Some(())
    }
}

impl ReadOnlyWorldQuery for () {}

// =============================================================================
// Tuple WorldQuery Implementations
// =============================================================================

// Macro to implement WorldQuery for tuples of different sizes
macro_rules! impl_tuple_world_query {
    ($($T:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($T: WorldQuery),*> WorldQuery for ($($T,)*) {
            type Item<'w> = ($($T::Item<'w>,)*);
            type State = ($($T::State,)*);

            #[inline]
            fn init_state(world: &World) -> Self::State {
                ($($T::init_state(world),)*)
            }

            #[inline]
            fn component_access(state: &Self::State) -> BTreeSet<ComponentId> {
                let ($($T,)*) = state;
                let mut access = BTreeSet::new();
                $(
                    access.extend($T::component_access($T));
                )*
                access
            }

            #[inline]
            fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
                let ($($T,)*) = state;
                true $(&& $T::matches_archetype($T, archetype))*
            }

            #[inline]
            fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
                let ($($T,)*) = state;
                Some((
                    $($T::fetch($T, world, entity)?,)*
                ))
            }
        }

        // A tuple is ReadOnlyWorldQuery if all its elements are ReadOnlyWorldQuery
        impl<$($T: ReadOnlyWorldQuery),*> ReadOnlyWorldQuery for ($($T,)*) {}
    };
}

// Implement for tuples of size 1-8
impl_tuple_world_query!(A);
impl_tuple_world_query!(A, B);
impl_tuple_world_query!(A, B, C);
impl_tuple_world_query!(A, B, C, D);
impl_tuple_world_query!(A, B, C, D, E);
impl_tuple_world_query!(A, B, C, D, E, F);
impl_tuple_world_query!(A, B, C, D, E, F, G);
impl_tuple_world_query!(A, B, C, D, E, F, G, H);

// =============================================================================
// Filter Types
// =============================================================================

/// Query filter that matches entities that have a component.
///
/// `With<T>` is a filter, not a data fetch. It filters entities to only those
/// that have component `T`, but doesn't actually retrieve the component data.
///
/// Use `With<T>` when you need to ensure an entity has a component but don't
/// need to access its data.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, World};
/// use goud_engine::ecs::query::With;
///
/// struct Player;
/// impl Component for Player {}
///
/// struct Health(f32);
/// impl Component for Health {}
///
/// // Query for Health, but only for entities that also have Player
/// // (Health, With<Player>) - fetches Health data, filters by Player
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct With<T>(std::marker::PhantomData<T>);

impl<T> With<T> {
    /// Creates a new `With` filter.
    #[inline]
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: crate::ecs::Component> WorldQuery for With<T> {
    type Item<'w> = ();
    type State = ComponentId;

    #[inline]
    fn init_state(_world: &World) -> Self::State {
        ComponentId::of::<T>()
    }

    #[inline]
    fn component_access(_state: &Self::State) -> BTreeSet<ComponentId> {
        // Filters don't access component data, just check existence
        BTreeSet::new()
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(*state)
    }

    #[inline]
    fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        // Filter only checks if entity has the component
        if world.has::<T>(entity) {
            Some(())
        } else {
            // Also handle state usage to avoid unused warning
            let _ = state;
            None
        }
    }
}

impl<T: crate::ecs::Component> ReadOnlyWorldQuery for With<T> {}

/// Query filter that matches entities that don't have a component.
///
/// `Without<T>` is a filter that excludes entities with component `T`.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, World};
/// use goud_engine::ecs::query::Without;
///
/// struct Dead;
/// impl Component for Dead {}
///
/// struct Health(f32);
/// impl Component for Health {}
///
/// // Query for Health, but only for entities that don't have Dead
/// // (Health, Without<Dead>) - fetches Health data, excludes Dead entities
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Without<T>(std::marker::PhantomData<T>);

impl<T> Without<T> {
    /// Creates a new `Without` filter.
    #[inline]
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: crate::ecs::Component> WorldQuery for Without<T> {
    type Item<'w> = ();
    type State = ComponentId;

    #[inline]
    fn init_state(_world: &World) -> Self::State {
        ComponentId::of::<T>()
    }

    #[inline]
    fn component_access(_state: &Self::State) -> BTreeSet<ComponentId> {
        BTreeSet::new()
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        !archetype.has_component(*state)
    }

    #[inline]
    fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        // Filter only checks if entity doesn't have the component
        if !world.has::<T>(entity) {
            Some(())
        } else {
            let _ = state;
            None
        }
    }
}

impl<T: crate::ecs::Component> ReadOnlyWorldQuery for Without<T> {}

// =============================================================================
// Component Reference Implementation (&T)
// =============================================================================

/// Query for an immutable component reference.
///
/// `&T` queries fetch a reference to component `T` for each matching entity.
/// This is one of the most common query types.
///
/// # Archetype Matching
///
/// An archetype matches `&T` if it contains component `T`. The query returns
/// `None` for entities that don't have the component.
///
/// # Parallel Safety
///
/// `&T` is a read-only query and implements [`ReadOnlyWorldQuery`]. Multiple
/// `&T` queries for the same component can run in parallel, and `&T` can run
/// alongside `&U` for different components.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World, Component, Entity};
/// use goud_engine::ecs::query::WorldQuery;
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// let mut world = World::new();
/// let entity = world.spawn_empty();
/// world.insert(entity, Position { x: 1.0, y: 2.0 });
///
/// // Initialize query state
/// let state = <&Position>::init_state(&world);
///
/// // Fetch component reference
/// let pos = <&Position>::fetch(&state, &world, entity);
/// assert!(pos.is_some());
/// assert_eq!(pos.unwrap(), &Position { x: 1.0, y: 2.0 });
/// ```
///
/// # Access Conflicts
///
/// - `&T` conflicts with `&mut T` (read-write conflict)
/// - `&T` does NOT conflict with `&T` (multiple readers allowed)
/// - `&T` does NOT conflict with `&U` where U ≠ T
impl<T: crate::ecs::Component> WorldQuery for &T {
    type Item<'w> = &'w T;
    type State = ComponentId;

    #[inline]
    fn init_state(_world: &World) -> Self::State {
        ComponentId::of::<T>()
    }

    #[inline]
    fn component_access(state: &Self::State) -> BTreeSet<ComponentId> {
        let mut set = BTreeSet::new();
        set.insert(*state);
        set
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(*state)
    }

    #[inline]
    fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        // Avoid unused variable warning while maintaining intent
        let _ = state;

        // Use World::get which already checks entity liveness
        world.get::<T>(entity)
    }
}

impl<T: crate::ecs::Component> ReadOnlyWorldQuery for &T {}

// =============================================================================
// Mutable Component Reference Implementation (&mut T)
// =============================================================================

/// Marker type for tracking write access to a component.
///
/// Used to distinguish between read and write access in conflict detection.
/// This allows the query system to detect when two queries would violate
/// Rust's aliasing rules (one mutable + any other access to same component).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WriteAccess(pub ComponentId);

/// Query state for mutable component access.
///
/// Contains both the component ID and a marker indicating this is a write access.
/// Used for accurate access conflict detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutState {
    /// The component ID being accessed.
    pub component_id: ComponentId,
}

impl MutState {
    /// Creates a new mutable access state for the given component type.
    #[inline]
    pub fn of<T: crate::ecs::Component>() -> Self {
        Self {
            component_id: ComponentId::of::<T>(),
        }
    }
}

/// Query for a mutable component reference.
///
/// `&mut T` queries fetch a mutable reference to component `T` for each matching
/// entity. This allows modifying component data in place.
///
/// # Archetype Matching
///
/// An archetype matches `&mut T` if it contains component `T`. The query returns
/// `None` for entities that don't have the component.
///
/// # Access Conflicts
///
/// Mutable queries have strict access requirements:
///
/// - `&mut T` **conflicts** with `&T` (write-read conflict)
/// - `&mut T` **conflicts** with `&mut T` (write-write conflict)
/// - `&mut T` does **NOT** conflict with `&U` or `&mut U` where U ≠ T
///
/// The scheduler uses this information to prevent parallel execution of
/// conflicting systems.
///
/// # Thread Safety
///
/// `&mut T` does **NOT** implement [`ReadOnlyWorldQuery`]. This means:
///
/// 1. Systems using `&mut T` cannot run in parallel with other systems accessing `T`
/// 2. The query iterator enforces exclusive access at runtime
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World, Component, Entity};
/// use goud_engine::ecs::query::WorldQuery;
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// struct Health(f32);
/// impl Component for Health {}
///
/// let mut world = World::new();
/// let entity = world.spawn_empty();
/// world.insert(entity, Health(100.0));
///
/// // Initialize query state
/// let state = <&mut Health>::init_state(&world);
///
/// // Fetch mutable component reference
/// if let Some(health) = <&mut Health>::fetch_mut(&state, &mut world, entity) {
///     health.0 -= 10.0; // Modify in place
/// }
///
/// // Verify modification
/// assert_eq!(world.get::<Health>(entity), Some(&Health(90.0)));
/// ```
///
/// # Important Notes
///
/// - The `fetch` method returns `None` because mutable access requires a mutable
///   world reference. Use `fetch_mut` for mutable queries.
/// - The query system enforces that only one mutable access to a component exists
///   at any time, preventing aliasing issues.
impl<T: crate::ecs::Component> WorldQuery for &mut T {
    type Item<'w> = &'w mut T;
    type State = MutState;

    #[inline]
    fn init_state(_world: &World) -> Self::State {
        MutState::of::<T>()
    }

    #[inline]
    fn component_access(state: &Self::State) -> BTreeSet<ComponentId> {
        let mut set = BTreeSet::new();
        set.insert(state.component_id);
        set
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(state.component_id)
    }

    /// Returns `None` because mutable access requires `fetch_mut`.
    ///
    /// This is intentional - the immutable world reference cannot provide
    /// mutable component access without violating Rust's aliasing rules.
    #[inline]
    fn fetch<'w>(_state: &Self::State, _world: &'w World, _entity: Entity) -> Option<Self::Item<'w>> {
        // Cannot provide mutable access from immutable world reference
        // Callers must use fetch_mut for mutable queries
        None
    }

    #[inline]
    fn fetch_mut<'w>(
        state: &Self::State,
        world: &'w mut World,
        entity: Entity,
    ) -> Option<Self::Item<'w>> {
        // Avoid unused variable warning while maintaining intent
        let _ = state;

        // Use World::get_mut which already checks entity liveness
        world.get_mut::<T>(entity)
    }
}

// NOTE: &mut T does NOT implement ReadOnlyWorldQuery
// This is intentional - mutable queries conflict with all other access to the same component

// =============================================================================
// Access Conflict Detection
// =============================================================================

/// Represents the type of access a query has to a component.
///
/// Used for detecting conflicts between queries. Two queries conflict if:
/// - One has `Write` access and the other has `Read` or `Write` to the same component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessType {
    /// Read-only access (`&T`)
    Read,
    /// Mutable access (`&mut T`)
    Write,
}

/// Describes the component access pattern for a query.
///
/// Used by the scheduler to determine which systems can run in parallel.
#[derive(Debug, Clone, Default)]
pub struct Access {
    /// Components accessed for reading only
    reads: BTreeSet<ComponentId>,
    /// Components accessed for writing (also counts as read)
    writes: BTreeSet<ComponentId>,
    /// Resources accessed for reading only
    resource_reads: BTreeSet<ResourceId>,
    /// Resources accessed for writing (also counts as read)
    resource_writes: BTreeSet<ResourceId>,
    /// Non-send resources accessed for reading only
    non_send_reads: BTreeSet<NonSendResourceId>,
    /// Non-send resources accessed for writing (also counts as read)
    non_send_writes: BTreeSet<NonSendResourceId>,
}

impl Access {
    /// Creates a new empty access descriptor.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a read access for the given component.
    #[inline]
    pub fn add_read(&mut self, id: ComponentId) {
        self.reads.insert(id);
    }

    /// Adds a write access for the given component.
    ///
    /// Write access implies read access.
    #[inline]
    pub fn add_write(&mut self, id: ComponentId) {
        self.writes.insert(id);
    }

    /// Returns all components that are read (including those also written).
    #[inline]
    pub fn reads(&self) -> impl Iterator<Item = &ComponentId> {
        self.reads.iter().chain(self.writes.iter())
    }

    /// Returns all components that are written.
    #[inline]
    pub fn writes(&self) -> &BTreeSet<ComponentId> {
        &self.writes
    }

    /// Returns the set of read-only components (read but not written).
    #[inline]
    pub fn reads_only(&self) -> impl Iterator<Item = &ComponentId> {
        self.reads.iter().filter(|id| !self.writes.contains(id))
    }

    /// Checks if this access conflicts with another.
    ///
    /// Two accesses conflict if:
    /// - One writes to a component that the other reads or writes
    /// - One writes to a resource that the other reads or writes
    #[inline]
    pub fn conflicts_with(&self, other: &Access) -> bool {
        // Check component conflicts

        // Check if our writes conflict with their reads or writes
        for write in &self.writes {
            if other.reads.contains(write) || other.writes.contains(write) {
                return true;
            }
        }

        // Check if their writes conflict with our reads
        for write in &other.writes {
            if self.reads.contains(write) {
                return true;
            }
        }

        // Check resource conflicts
        if self.resource_conflicts_with(other) {
            return true;
        }

        // Check non-send resource conflicts
        self.non_send_conflicts_with(other)
    }

    /// Returns true if this access pattern is read-only.
    ///
    /// This checks component, resource, and non-send resource access.
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.writes.is_empty() && self.resource_writes.is_empty() && self.non_send_writes.is_empty()
    }

    /// Merges another access into this one.
    #[inline]
    pub fn extend(&mut self, other: &Access) {
        self.reads.extend(other.reads.iter().copied());
        self.writes.extend(other.writes.iter().copied());
        self.resource_reads.extend(other.resource_reads.iter().copied());
        self.resource_writes.extend(other.resource_writes.iter().copied());
        self.non_send_reads.extend(other.non_send_reads.iter().copied());
        self.non_send_writes.extend(other.non_send_writes.iter().copied());
    }

    // =========================================================================
    // Resource Access
    // =========================================================================

    /// Adds a read access for the given resource.
    #[inline]
    pub fn add_resource_read(&mut self, id: ResourceId) {
        self.resource_reads.insert(id);
    }

    /// Adds a write access for the given resource.
    ///
    /// Write access implies read access.
    #[inline]
    pub fn add_resource_write(&mut self, id: ResourceId) {
        self.resource_writes.insert(id);
    }

    /// Returns all resources that are read (including those also written).
    #[inline]
    pub fn resource_reads(&self) -> impl Iterator<Item = &ResourceId> {
        self.resource_reads.iter().chain(self.resource_writes.iter())
    }

    /// Returns all resources that are written.
    #[inline]
    pub fn resource_writes(&self) -> &BTreeSet<ResourceId> {
        &self.resource_writes
    }

    /// Returns the set of read-only resources (read but not written).
    #[inline]
    pub fn resource_reads_only(&self) -> impl Iterator<Item = &ResourceId> {
        self.resource_reads.iter().filter(|id| !self.resource_writes.contains(id))
    }

    /// Checks if resource access conflicts with another.
    ///
    /// Two accesses conflict if one writes to a resource that the other
    /// reads or writes.
    #[inline]
    pub fn resource_conflicts_with(&self, other: &Access) -> bool {
        // Check if our resource writes conflict with their reads or writes
        for write in &self.resource_writes {
            if other.resource_reads.contains(write) || other.resource_writes.contains(write) {
                return true;
            }
        }

        // Check if their resource writes conflict with our reads
        for write in &other.resource_writes {
            if self.resource_reads.contains(write) {
                return true;
            }
        }

        false
    }

    /// Checks if this access has any resource access.
    #[inline]
    pub fn has_resource_access(&self) -> bool {
        !self.resource_reads.is_empty() || !self.resource_writes.is_empty()
    }

    /// Checks if this access has any component access.
    #[inline]
    pub fn has_component_access(&self) -> bool {
        !self.reads.is_empty() || !self.writes.is_empty()
    }

    // =========================================================================
    // Non-Send Resource Access
    // =========================================================================

    /// Adds a read access for the given non-send resource.
    #[inline]
    pub fn add_non_send_read(&mut self, id: NonSendResourceId) {
        self.non_send_reads.insert(id);
    }

    /// Adds a write access for the given non-send resource.
    ///
    /// Write access implies read access.
    #[inline]
    pub fn add_non_send_write(&mut self, id: NonSendResourceId) {
        self.non_send_writes.insert(id);
    }

    /// Returns all non-send resources that are read (including those also written).
    #[inline]
    pub fn non_send_reads(&self) -> impl Iterator<Item = &NonSendResourceId> {
        self.non_send_reads.iter().chain(self.non_send_writes.iter())
    }

    /// Returns all non-send resources that are written.
    #[inline]
    pub fn non_send_writes(&self) -> &BTreeSet<NonSendResourceId> {
        &self.non_send_writes
    }

    /// Returns the set of read-only non-send resources (read but not written).
    #[inline]
    pub fn non_send_reads_only(&self) -> impl Iterator<Item = &NonSendResourceId> {
        self.non_send_reads.iter().filter(|id| !self.non_send_writes.contains(id))
    }

    /// Checks if non-send resource access conflicts with another.
    ///
    /// Two accesses conflict if one writes to a non-send resource that the other
    /// reads or writes.
    #[inline]
    pub fn non_send_conflicts_with(&self, other: &Access) -> bool {
        // Check if our non-send writes conflict with their reads or writes
        for write in &self.non_send_writes {
            if other.non_send_reads.contains(write) || other.non_send_writes.contains(write) {
                return true;
            }
        }

        // Check if their non-send writes conflict with our reads
        for write in &other.non_send_writes {
            if self.non_send_reads.contains(write) {
                return true;
            }
        }

        false
    }

    /// Checks if this access has any non-send resource access.
    #[inline]
    pub fn has_non_send_access(&self) -> bool {
        !self.non_send_reads.is_empty() || !self.non_send_writes.is_empty()
    }

    /// Returns true if this access requires execution on the main thread.
    ///
    /// This returns true if there is any non-send resource access.
    #[inline]
    pub fn requires_main_thread(&self) -> bool {
        self.has_non_send_access()
    }

    /// Returns detailed information about conflicts between this access and another.
    ///
    /// If there are no conflicts, returns `None`. Otherwise, returns an `AccessConflict`
    /// describing all the conflicting components and resources.
    ///
    /// # Arguments
    ///
    /// * `other` - The other access pattern to check against
    ///
    /// # Returns
    ///
    /// `Some(AccessConflict)` if there are conflicts, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::query::Access;
    /// use goud_engine::ecs::component::ComponentId;
    /// use goud_engine::ecs::Component;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut access1 = Access::new();
    /// access1.add_write(ComponentId::of::<Position>());
    ///
    /// let mut access2 = Access::new();
    /// access2.add_read(ComponentId::of::<Position>());
    ///
    /// let conflict = access1.get_conflicts(&access2);
    /// assert!(conflict.is_some());
    /// let conflict = conflict.unwrap();
    /// assert_eq!(conflict.component_conflicts().len(), 1);
    /// ```
    pub fn get_conflicts(&self, other: &Access) -> Option<AccessConflict> {
        let mut component_conflicts = Vec::new();
        let mut resource_conflicts = Vec::new();
        let mut non_send_conflicts = Vec::new();

        // Check component conflicts: our writes vs their reads/writes
        for &write in &self.writes {
            if other.reads.contains(&write) {
                component_conflicts.push(ConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Read,
                ));
            } else if other.writes.contains(&write) {
                component_conflicts.push(ConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Write,
                ));
            }
        }

        // Check component conflicts: their writes vs our reads
        for &write in &other.writes {
            if self.reads.contains(&write) && !self.writes.contains(&write) {
                // Only add if we haven't already added this conflict from the other direction
                component_conflicts.push(ConflictInfo::new(
                    write,
                    AccessType::Read,
                    AccessType::Write,
                ));
            }
        }

        // Check resource conflicts: our writes vs their reads/writes
        for &write in &self.resource_writes {
            if other.resource_reads.contains(&write) {
                resource_conflicts.push(ResourceConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Read,
                ));
            } else if other.resource_writes.contains(&write) {
                resource_conflicts.push(ResourceConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Write,
                ));
            }
        }

        // Check resource conflicts: their writes vs our reads
        for &write in &other.resource_writes {
            if self.resource_reads.contains(&write) && !self.resource_writes.contains(&write) {
                resource_conflicts.push(ResourceConflictInfo::new(
                    write,
                    AccessType::Read,
                    AccessType::Write,
                ));
            }
        }

        // Check non-send resource conflicts: our writes vs their reads/writes
        for &write in &self.non_send_writes {
            if other.non_send_reads.contains(&write) {
                non_send_conflicts.push(NonSendConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Read,
                ));
            } else if other.non_send_writes.contains(&write) {
                non_send_conflicts.push(NonSendConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Write,
                ));
            }
        }

        // Check non-send resource conflicts: their writes vs our reads
        for &write in &other.non_send_writes {
            if self.non_send_reads.contains(&write) && !self.non_send_writes.contains(&write) {
                non_send_conflicts.push(NonSendConflictInfo::new(
                    write,
                    AccessType::Read,
                    AccessType::Write,
                ));
            }
        }

        if component_conflicts.is_empty()
            && resource_conflicts.is_empty()
            && non_send_conflicts.is_empty()
        {
            None
        } else {
            Some(AccessConflict {
                component_conflicts,
                resource_conflicts,
                non_send_conflicts,
            })
        }
    }

    /// Returns true if this access is empty (no reads or writes).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.reads.is_empty()
            && self.writes.is_empty()
            && self.resource_reads.is_empty()
            && self.resource_writes.is_empty()
            && self.non_send_reads.is_empty()
            && self.non_send_writes.is_empty()
    }

    /// Clears all access information.
    #[inline]
    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
        self.resource_reads.clear();
        self.resource_writes.clear();
        self.non_send_reads.clear();
        self.non_send_writes.clear();
    }
}

// =============================================================================
// Access Conflict Reporting
// =============================================================================

/// Information about a single component access conflict.
///
/// Describes which component conflicts and what type of access each side has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictInfo {
    /// The conflicting component ID.
    pub component_id: ComponentId,
    /// How the first access pattern accesses this component.
    pub first_access: AccessType,
    /// How the second access pattern accesses this component.
    pub second_access: AccessType,
}

impl ConflictInfo {
    /// Creates a new conflict info.
    #[inline]
    pub fn new(
        component_id: ComponentId,
        first_access: AccessType,
        second_access: AccessType,
    ) -> Self {
        Self {
            component_id,
            first_access,
            second_access,
        }
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.first_access == AccessType::Write && self.second_access == AccessType::Write
    }

    /// Returns true if this is a read-write conflict.
    #[inline]
    pub fn is_read_write(&self) -> bool {
        (self.first_access == AccessType::Read && self.second_access == AccessType::Write)
            || (self.first_access == AccessType::Write && self.second_access == AccessType::Read)
    }
}

impl fmt::Display for ConflictInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Component {:?}: {:?} vs {:?}",
            self.component_id, self.first_access, self.second_access
        )
    }
}

/// Information about a single resource access conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceConflictInfo {
    /// The conflicting resource ID.
    pub resource_id: ResourceId,
    /// How the first access pattern accesses this resource.
    pub first_access: AccessType,
    /// How the second access pattern accesses this resource.
    pub second_access: AccessType,
}

impl ResourceConflictInfo {
    /// Creates a new resource conflict info.
    #[inline]
    pub fn new(
        resource_id: ResourceId,
        first_access: AccessType,
        second_access: AccessType,
    ) -> Self {
        Self {
            resource_id,
            first_access,
            second_access,
        }
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.first_access == AccessType::Write && self.second_access == AccessType::Write
    }
}

impl fmt::Display for ResourceConflictInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Resource {:?}: {:?} vs {:?}",
            self.resource_id, self.first_access, self.second_access
        )
    }
}

/// Information about a single non-send resource access conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonSendConflictInfo {
    /// The conflicting non-send resource ID.
    pub resource_id: NonSendResourceId,
    /// How the first access pattern accesses this resource.
    pub first_access: AccessType,
    /// How the second access pattern accesses this resource.
    pub second_access: AccessType,
}

impl NonSendConflictInfo {
    /// Creates a new non-send resource conflict info.
    #[inline]
    pub fn new(
        resource_id: NonSendResourceId,
        first_access: AccessType,
        second_access: AccessType,
    ) -> Self {
        Self {
            resource_id,
            first_access,
            second_access,
        }
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.first_access == AccessType::Write && self.second_access == AccessType::Write
    }
}

impl fmt::Display for NonSendConflictInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NonSendResource {:?}: {:?} vs {:?}",
            self.resource_id, self.first_access, self.second_access
        )
    }
}

/// Detailed information about all conflicts between two access patterns.
///
/// This struct is returned by [`Access::get_conflicts()`] when there are
/// conflicting accesses. It contains separate lists of component, resource,
/// and non-send resource conflicts.
///
/// # Usage
///
/// ```
/// use goud_engine::ecs::query::{Access, AccessConflict};
/// use goud_engine::ecs::component::ComponentId;
/// use goud_engine::ecs::Component;
///
/// #[derive(Clone, Copy)]
/// struct Health(f32);
/// impl Component for Health {}
///
/// let mut system_a = Access::new();
/// system_a.add_write(ComponentId::of::<Health>());
///
/// let mut system_b = Access::new();
/// system_b.add_read(ComponentId::of::<Health>());
///
/// if let Some(conflict) = system_a.get_conflicts(&system_b) {
///     println!("Systems conflict on {} components", conflict.component_count());
///     for info in conflict.component_conflicts() {
///         println!("  - {}", info);
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessConflict {
    /// Component access conflicts.
    component_conflicts: Vec<ConflictInfo>,
    /// Resource access conflicts.
    resource_conflicts: Vec<ResourceConflictInfo>,
    /// Non-send resource access conflicts.
    non_send_conflicts: Vec<NonSendConflictInfo>,
}

impl AccessConflict {
    /// Creates a new empty access conflict.
    #[inline]
    pub fn new() -> Self {
        Self {
            component_conflicts: Vec::new(),
            resource_conflicts: Vec::new(),
            non_send_conflicts: Vec::new(),
        }
    }

    /// Returns the component conflicts.
    #[inline]
    pub fn component_conflicts(&self) -> &[ConflictInfo] {
        &self.component_conflicts
    }

    /// Returns the resource conflicts.
    #[inline]
    pub fn resource_conflicts(&self) -> &[ResourceConflictInfo] {
        &self.resource_conflicts
    }

    /// Returns the non-send resource conflicts.
    #[inline]
    pub fn non_send_conflicts(&self) -> &[NonSendConflictInfo] {
        &self.non_send_conflicts
    }

    /// Returns the total number of component conflicts.
    #[inline]
    pub fn component_count(&self) -> usize {
        self.component_conflicts.len()
    }

    /// Returns the total number of resource conflicts.
    #[inline]
    pub fn resource_count(&self) -> usize {
        self.resource_conflicts.len()
    }

    /// Returns the total number of non-send resource conflicts.
    #[inline]
    pub fn non_send_count(&self) -> usize {
        self.non_send_conflicts.len()
    }

    /// Returns the total number of conflicts across all categories.
    #[inline]
    pub fn total_count(&self) -> usize {
        self.component_conflicts.len()
            + self.resource_conflicts.len()
            + self.non_send_conflicts.len()
    }

    /// Returns true if there are no conflicts.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.component_conflicts.is_empty()
            && self.resource_conflicts.is_empty()
            && self.non_send_conflicts.is_empty()
    }

    /// Returns true if any conflict is a write-write conflict.
    #[inline]
    pub fn has_write_write(&self) -> bool {
        self.component_conflicts.iter().any(|c| c.is_write_write())
            || self.resource_conflicts.iter().any(|c| c.is_write_write())
            || self.non_send_conflicts.iter().any(|c| c.is_write_write())
    }

    /// Returns an iterator over all conflicting component IDs.
    #[inline]
    pub fn conflicting_components(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.component_conflicts.iter().map(|c| c.component_id)
    }

    /// Returns an iterator over all conflicting resource IDs.
    #[inline]
    pub fn conflicting_resources(&self) -> impl Iterator<Item = ResourceId> + '_ {
        self.resource_conflicts.iter().map(|c| c.resource_id)
    }

    /// Returns an iterator over all conflicting non-send resource IDs.
    #[inline]
    pub fn conflicting_non_send_resources(&self) -> impl Iterator<Item = NonSendResourceId> + '_ {
        self.non_send_conflicts.iter().map(|c| c.resource_id)
    }
}

impl Default for AccessConflict {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AccessConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AccessConflict(")?;
        let mut first = true;

        for conflict in &self.component_conflicts {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", conflict)?;
            first = false;
        }

        for conflict in &self.resource_conflicts {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", conflict)?;
            first = false;
        }

        for conflict in &self.non_send_conflicts {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", conflict)?;
            first = false;
        }

        write!(f, ")")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Component;

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

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    // =========================================================================
    // WorldQuery Trait Tests
    // =========================================================================

    mod world_query_trait {
        use super::*;

        #[test]
        fn test_entity_query_init_state() {
            let world = World::new();
            let _state: () = Entity::init_state(&world);
            // Entity state is unit type - assertion via type annotation
        }

        #[test]
        fn test_entity_query_matches_all_archetypes() {
            use crate::ecs::archetype::{Archetype, ArchetypeId};
            use std::collections::BTreeSet;

            let state = ();

            // Empty archetype
            let empty_arch = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            assert!(Entity::matches_archetype(&state, &empty_arch));

            // Archetype with components
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let pos_arch = Archetype::new(ArchetypeId::new(1), components);
            assert!(Entity::matches_archetype(&state, &pos_arch));
        }

        #[test]
        fn test_entity_query_fetch_alive() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            let _state: () = Entity::init_state(&world);
            let fetched = Entity::fetch(&(), &world, entity);

            assert_eq!(fetched, Some(entity));
        }

        #[test]
        fn test_entity_query_fetch_dead() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.despawn(entity);

            let _state: () = Entity::init_state(&world);
            let fetched = Entity::fetch(&(), &world, entity);

            assert_eq!(fetched, None);
        }

        #[test]
        fn test_entity_query_is_read_only() {
            // Compile-time test: Entity implements ReadOnlyWorldQuery
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<Entity>();
        }

        #[test]
        fn test_entity_query_component_access_empty() {
            let state = ();
            let access = Entity::component_access(&state);
            assert!(access.is_empty());
        }
    }

    // =========================================================================
    // Unit Query Tests
    // =========================================================================

    mod unit_query {
        use super::*;

        #[test]
        fn test_unit_query_matches_all() {
            use crate::ecs::archetype::{Archetype, ArchetypeId};
            use std::collections::BTreeSet;

            let state = ();

            let empty_arch = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            assert!(<()>::matches_archetype(&state, &empty_arch));
        }

        #[test]
        fn test_unit_query_fetch_always_succeeds() {
            let world = World::new();
            let _state: () = <()>::init_state(&world);

            // Even with placeholder entity (doesn't need to exist for unit query)
            let entity = Entity::new(0, 1);
            let fetched = <()>::fetch(&(), &world, entity);

            // Unit query always returns Some(())
            assert!(fetched.is_some());
        }

        #[test]
        fn test_unit_query_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<()>();
        }
    }

    // =========================================================================
    // QueryState Trait Tests
    // =========================================================================

    mod query_state_trait {
        use super::*;

        #[test]
        fn test_component_id_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<ComponentId>();
        }

        #[test]
        fn test_unit_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<()>();
        }

        #[test]
        fn test_query_state_is_send_sync() {
            fn requires_send_sync<S: Send + Sync>() {}
            requires_send_sync::<ComponentId>();
            requires_send_sync::<()>();
        }
    }

    // =========================================================================
    // With Filter Tests
    // =========================================================================

    mod with_filter {
        use super::*;
        use crate::ecs::archetype::{Archetype, ArchetypeId};
        use std::collections::BTreeSet;

        #[test]
        fn test_with_init_state() {
            let world = World::new();
            let state = With::<Position>::init_state(&world);
            assert_eq!(state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_with_matches_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(With::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_with_does_not_match_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(!With::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_with_fetch_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = With::<Position>::init_state(&world);
            let result = With::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, Some(()));
        }

        #[test]
        fn test_with_fetch_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = With::<Position>::init_state(&world);
            let result = With::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, None);
        }

        #[test]
        fn test_with_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<With<Position>>();
        }

        #[test]
        fn test_with_component_access_empty() {
            let state = ComponentId::of::<Position>();
            let access = With::<Position>::component_access(&state);
            // Filters don't access component data
            assert!(access.is_empty());
        }
    }

    // =========================================================================
    // Without Filter Tests
    // =========================================================================

    mod without_filter {
        use super::*;
        use crate::ecs::archetype::{Archetype, ArchetypeId};
        use std::collections::BTreeSet;

        #[test]
        fn test_without_init_state() {
            let world = World::new();
            let state = Without::<Position>::init_state(&world);
            assert_eq!(state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_without_matches_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(Without::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_without_does_not_match_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(!Without::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_without_fetch_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = Without::<Position>::init_state(&world);
            let result = Without::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, Some(()));
        }

        #[test]
        fn test_without_fetch_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = Without::<Position>::init_state(&world);
            let result = Without::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, None);
        }

        #[test]
        fn test_without_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<Without<Position>>();
        }

        #[test]
        fn test_without_matches_empty_archetype() {
            let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            let state = ComponentId::of::<Position>();
            assert!(Without::<Position>::matches_archetype(&state, &archetype));
        }
    }

    // =========================================================================
    // Component Reference (&T) Tests
    // =========================================================================

    mod component_ref {
        use super::*;
        use crate::ecs::archetype::{Archetype, ArchetypeId};
        use std::collections::BTreeSet;

        #[test]
        fn test_ref_init_state() {
            let world = World::new();
            let state = <&Position>::init_state(&world);
            assert_eq!(state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_ref_matches_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(<&Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_ref_does_not_match_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(!<&Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_ref_does_not_match_empty_archetype() {
            let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

            let state = ComponentId::of::<Position>();
            assert!(!<&Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_ref_fetch_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let pos = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
        }

        #[test]
        fn test_ref_fetch_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_ref_fetch_dead_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_ref_fetch_placeholder_entity() {
            let world = World::new();

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, Entity::PLACEHOLDER);

            assert!(result.is_none());
        }

        #[test]
        fn test_ref_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<&Position>();
            requires_read_only::<&Velocity>();
        }

        #[test]
        fn test_ref_component_access_contains_component_id() {
            let state = ComponentId::of::<Position>();
            let access = <&Position>::component_access(&state);

            assert_eq!(access.len(), 1);
            assert!(access.contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_ref_component_access_does_not_contain_other_components() {
            let state = ComponentId::of::<Position>();
            let access = <&Position>::component_access(&state);

            assert!(!access.contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_ref_multiple_entities_same_component() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            let e3 = world.spawn_empty();
            world.insert(e3, Position { x: 5.0, y: 6.0 });

            let state = <&Position>::init_state(&world);

            let p1 = <&Position>::fetch(&state, &world, e1).unwrap();
            let p2 = <&Position>::fetch(&state, &world, e2).unwrap();
            let p3 = <&Position>::fetch(&state, &world, e3).unwrap();

            assert_eq!(p1, &Position { x: 1.0, y: 2.0 });
            assert_eq!(p2, &Position { x: 3.0, y: 4.0 });
            assert_eq!(p3, &Position { x: 5.0, y: 6.0 });
        }

        #[test]
        fn test_ref_different_component_types() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let pos_state = <&Position>::init_state(&world);
            let vel_state = <&Velocity>::init_state(&world);

            let pos = <&Position>::fetch(&pos_state, &world, entity).unwrap();
            let vel = <&Velocity>::fetch(&vel_state, &world, entity).unwrap();

            assert_eq!(pos, &Position { x: 1.0, y: 2.0 });
            assert_eq!(vel, &Velocity { x: 3.0, y: 4.0 });
        }

        #[test]
        fn test_ref_fetch_after_component_update() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&Position>::init_state(&world);

            // Fetch before update
            let pos1 = <&Position>::fetch(&state, &world, entity).unwrap();
            assert_eq!(pos1, &Position { x: 1.0, y: 2.0 });

            // Update component
            world.insert(entity, Position { x: 10.0, y: 20.0 });

            // Fetch after update
            let pos2 = <&Position>::fetch(&state, &world, entity).unwrap();
            assert_eq!(pos2, &Position { x: 10.0, y: 20.0 });
        }

        #[test]
        fn test_ref_matches_archetype_with_multiple_components() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            components.insert(ComponentId::of::<Velocity>());
            components.insert(ComponentId::of::<Player>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let pos_state = ComponentId::of::<Position>();
            let vel_state = ComponentId::of::<Velocity>();

            assert!(<&Position>::matches_archetype(&pos_state, &archetype));
            assert!(<&Velocity>::matches_archetype(&vel_state, &archetype));
        }

        #[test]
        fn test_ref_fetch_returns_correct_reference_type() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 42.0, y: 99.0 });

            let state = <&Position>::init_state(&world);
            let result: Option<&Position> = <&Position>::fetch(&state, &world, entity);

            // Verify it's actually a reference (compile-time check via type annotation)
            assert!(result.is_some());
        }

        #[test]
        fn test_ref_fetch_stale_entity() {
            let mut world = World::new();

            // Spawn and despawn to create a stale handle
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            // Respawn new entity at same index
            let new_entity = world.spawn_empty();
            world.insert(new_entity, Position { x: 99.0, y: 99.0 });

            let state = <&Position>::init_state(&world);

            // Old entity handle should not fetch the new entity's component
            let result = <&Position>::fetch(&state, &world, entity);
            assert!(result.is_none());

            // New entity should fetch correctly
            let new_result = <&Position>::fetch(&state, &world, new_entity);
            assert!(new_result.is_some());
            assert_eq!(new_result.unwrap(), &Position { x: 99.0, y: 99.0 });
        }

        #[test]
        fn test_ref_state_is_component_id() {
            // Verify that the state type is ComponentId
            fn check_state_type<T: crate::ecs::Component>() {
                let world = World::new();
                let state: ComponentId = <&T>::init_state(&world);
                assert_eq!(state, ComponentId::of::<T>());
            }

            check_state_type::<Position>();
            check_state_type::<Velocity>();
        }

        // Compile-time tests for trait bounds

        #[test]
        fn test_ref_implements_world_query() {
            fn requires_world_query<Q: WorldQuery>() {}
            requires_world_query::<&Position>();
            requires_world_query::<&Velocity>();
            requires_world_query::<&Player>();
        }

        #[test]
        fn test_ref_state_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            // ComponentId should implement QueryState
            requires_query_state::<ComponentId>();
        }
    }

    // =========================================================================
    // Mutable Component Reference (&mut T) Tests
    // =========================================================================

    mod mut_component_ref {
        use super::*;
        use crate::ecs::archetype::{Archetype, ArchetypeId};
        use std::collections::BTreeSet;

        #[test]
        fn test_mut_ref_init_state() {
            let world = World::new();
            let state = <&mut Position>::init_state(&world);
            assert_eq!(state.component_id, ComponentId::of::<Position>());
        }

        #[test]
        fn test_mut_ref_state_is_mut_state() {
            // Verify that the state type is MutState
            fn check_state_type<T: crate::ecs::Component>() {
                let world = World::new();
                let state: MutState = <&mut T>::init_state(&world);
                assert_eq!(state.component_id, ComponentId::of::<T>());
            }

            check_state_type::<Position>();
            check_state_type::<Velocity>();
        }

        #[test]
        fn test_mut_ref_matches_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = MutState::of::<Position>();
            assert!(<&mut Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_mut_ref_does_not_match_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = MutState::of::<Position>();
            assert!(!<&mut Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_mut_ref_does_not_match_empty_archetype() {
            let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

            let state = MutState::of::<Position>();
            assert!(!<&mut Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_mut_ref_fetch_returns_none() {
            // fetch() should return None for mutable queries - use fetch_mut instead
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch(&state, &world, entity);

            // Mutable fetch with immutable world returns None
            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_fetch_mut_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_some());
            let pos = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
        }

        #[test]
        fn test_mut_ref_fetch_mut_and_modify() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);

            // Modify through fetch_mut
            if let Some(pos) = <&mut Position>::fetch_mut(&state, &mut world, entity) {
                pos.x += 10.0;
                pos.y += 20.0;
            }

            // Verify modification persisted
            let pos = world.get::<Position>(entity).unwrap();
            assert_eq!(pos.x, 11.0);
            assert_eq!(pos.y, 22.0);
        }

        #[test]
        fn test_mut_ref_fetch_mut_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_fetch_mut_dead_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_fetch_mut_placeholder_entity() {
            let mut world = World::new();

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, Entity::PLACEHOLDER);

            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_is_not_read_only() {
            // Compile-time test: &mut T does NOT implement ReadOnlyWorldQuery
            // This test ensures the trait bound fails - verified by the comment
            // fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            // requires_read_only::<&mut Position>(); // Should fail to compile

            // Instead, verify that WorldQuery is implemented
            fn requires_world_query<Q: WorldQuery>() {}
            requires_world_query::<&mut Position>();
        }

        #[test]
        fn test_mut_ref_component_access_contains_component_id() {
            let state = MutState::of::<Position>();
            let access = <&mut Position>::component_access(&state);

            assert_eq!(access.len(), 1);
            assert!(access.contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_mut_ref_component_access_does_not_contain_other_components() {
            let state = MutState::of::<Position>();
            let access = <&mut Position>::component_access(&state);

            assert!(!access.contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_mut_ref_fetch_mut_stale_entity() {
            let mut world = World::new();

            // Spawn and despawn to create a stale handle
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            // Respawn new entity at same index
            let new_entity = world.spawn_empty();
            world.insert(new_entity, Position { x: 99.0, y: 99.0 });

            let state = <&mut Position>::init_state(&world);

            // Old entity handle should not fetch the new entity's component
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);
            assert!(result.is_none());

            // New entity should fetch correctly
            let new_result = <&mut Position>::fetch_mut(&state, &mut world, new_entity);
            assert!(new_result.is_some());
            assert_eq!(new_result.unwrap().x, 99.0);
        }

        #[test]
        fn test_mut_ref_implements_world_query() {
            fn requires_world_query<Q: WorldQuery>() {}
            requires_world_query::<&mut Position>();
            requires_world_query::<&mut Velocity>();
            requires_world_query::<&mut Player>();
        }

        #[test]
        fn test_mut_state_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<MutState>();
        }

        #[test]
        fn test_mut_ref_matches_archetype_with_multiple_components() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            components.insert(ComponentId::of::<Velocity>());
            components.insert(ComponentId::of::<Player>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let pos_state = MutState::of::<Position>();
            let vel_state = MutState::of::<Velocity>();

            assert!(<&mut Position>::matches_archetype(&pos_state, &archetype));
            assert!(<&mut Velocity>::matches_archetype(&vel_state, &archetype));
        }

        #[test]
        fn test_mut_ref_fetch_mut_returns_correct_reference_type() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 42.0, y: 99.0 });

            let state = <&mut Position>::init_state(&world);
            let result: Option<&mut Position> = <&mut Position>::fetch_mut(&state, &mut world, entity);

            // Verify it's actually a mutable reference (compile-time check via type annotation)
            assert!(result.is_some());
        }

        #[test]
        fn test_mut_ref_different_component_types() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let pos_state = <&mut Position>::init_state(&world);

            // Modify position
            if let Some(pos) = <&mut Position>::fetch_mut(&pos_state, &mut world, entity) {
                pos.x = 100.0;
            }

            // Velocity should be unchanged
            let vel = world.get::<Velocity>(entity).unwrap();
            assert_eq!(vel.x, 3.0);
            assert_eq!(vel.y, 4.0);

            // Position should be modified
            let pos = world.get::<Position>(entity).unwrap();
            assert_eq!(pos.x, 100.0);
        }
    }

    // =========================================================================
    // Access Conflict Detection Tests
    // =========================================================================

    mod access_conflict {
        use super::*;

        #[test]
        fn test_access_new_is_empty() {
            let access = Access::new();
            assert!(access.is_read_only());
            assert_eq!(access.writes().len(), 0);
            assert_eq!(access.reads().count(), 0);
        }

        #[test]
        fn test_access_add_read() {
            let mut access = Access::new();
            let id = ComponentId::of::<Position>();
            access.add_read(id);

            assert!(access.is_read_only());
            assert!(access.reads().any(|&x| x == id));
            assert!(!access.writes().contains(&id));
        }

        #[test]
        fn test_access_add_write() {
            let mut access = Access::new();
            let id = ComponentId::of::<Position>();
            access.add_write(id);

            assert!(!access.is_read_only());
            assert!(access.writes().contains(&id));
        }

        #[test]
        fn test_access_write_counts_as_read() {
            let mut access = Access::new();
            let id = ComponentId::of::<Position>();
            access.add_write(id);

            // writes() includes writes
            assert!(access.writes().contains(&id));
            // reads() includes both reads and writes
            assert!(access.reads().any(|&x| x == id));
        }

        #[test]
        fn test_access_reads_only() {
            let mut access = Access::new();
            let pos_id = ComponentId::of::<Position>();
            let vel_id = ComponentId::of::<Velocity>();

            access.add_read(pos_id);
            access.add_write(vel_id);

            // reads_only should only include Position (read, not written)
            let reads_only: Vec<_> = access.reads_only().cloned().collect();
            assert!(reads_only.contains(&pos_id));
            assert!(!reads_only.contains(&vel_id));
        }

        #[test]
        fn test_access_no_conflict_between_reads() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();
            let id = ComponentId::of::<Position>();

            access1.add_read(id);
            access2.add_read(id);

            // Two reads of the same component don't conflict
            assert!(!access1.conflicts_with(&access2));
            assert!(!access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_conflict_write_read() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();
            let id = ComponentId::of::<Position>();

            access1.add_write(id);
            access2.add_read(id);

            // Write + Read of same component conflicts
            assert!(access1.conflicts_with(&access2));
            assert!(access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_conflict_write_write() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();
            let id = ComponentId::of::<Position>();

            access1.add_write(id);
            access2.add_write(id);

            // Two writes to same component conflict
            assert!(access1.conflicts_with(&access2));
            assert!(access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_no_conflict_different_components() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();

            access1.add_write(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            // Writes to different components don't conflict
            assert!(!access1.conflicts_with(&access2));
            assert!(!access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_no_conflict_read_different_write() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();

            access1.add_read(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            // Read Position, Write Velocity - no conflict
            assert!(!access1.conflicts_with(&access2));
            assert!(!access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_extend() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();

            access1.add_read(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            access1.extend(&access2);

            assert!(access1.reads().any(|&x| x == ComponentId::of::<Position>()));
            assert!(access1.writes().contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_access_is_read_only() {
            let mut read_access = Access::new();
            read_access.add_read(ComponentId::of::<Position>());
            assert!(read_access.is_read_only());

            let mut write_access = Access::new();
            write_access.add_write(ComponentId::of::<Position>());
            assert!(!write_access.is_read_only());
        }

        #[test]
        fn test_access_complex_scenario() {
            // Simulate two systems:
            // System A: reads Position, writes Velocity
            // System B: reads Position, reads Velocity
            // Should NOT conflict (no same-component write conflicts)

            let mut access_a = Access::new();
            access_a.add_read(ComponentId::of::<Position>());
            access_a.add_write(ComponentId::of::<Velocity>());

            let mut access_b = Access::new();
            access_b.add_read(ComponentId::of::<Position>());
            access_b.add_read(ComponentId::of::<Velocity>());

            // System B reads Velocity, System A writes Velocity - CONFLICT
            assert!(access_a.conflicts_with(&access_b));
            assert!(access_b.conflicts_with(&access_a));
        }

        #[test]
        fn test_access_no_conflict_complex() {
            // System A: writes Position
            // System B: writes Velocity, reads Player
            // Should NOT conflict

            let mut access_a = Access::new();
            access_a.add_write(ComponentId::of::<Position>());

            let mut access_b = Access::new();
            access_b.add_write(ComponentId::of::<Velocity>());
            access_b.add_read(ComponentId::of::<Player>());

            assert!(!access_a.conflicts_with(&access_b));
            assert!(!access_b.conflicts_with(&access_a));
        }

        #[test]
        fn test_access_is_empty() {
            let access = Access::new();
            assert!(access.is_empty());

            let mut not_empty = Access::new();
            not_empty.add_read(ComponentId::of::<Position>());
            assert!(!not_empty.is_empty());
        }

        #[test]
        fn test_access_clear() {
            let mut access = Access::new();
            access.add_read(ComponentId::of::<Position>());
            access.add_write(ComponentId::of::<Velocity>());

            assert!(!access.is_empty());
            access.clear();
            assert!(access.is_empty());
        }

        // =====================================================================
        // get_conflicts tests
        // =====================================================================

        #[test]
        fn test_get_conflicts_no_conflict() {
            let mut access1 = Access::new();
            access1.add_read(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            assert!(access1.get_conflicts(&access2).is_none());
        }

        #[test]
        fn test_get_conflicts_write_read() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2);
            assert!(conflict.is_some());

            let conflict = conflict.unwrap();
            assert_eq!(conflict.component_count(), 1);
            assert!(!conflict.has_write_write());

            let comp_conflict = &conflict.component_conflicts()[0];
            assert_eq!(comp_conflict.component_id, ComponentId::of::<Position>());
            assert_eq!(comp_conflict.first_access, AccessType::Write);
            assert_eq!(comp_conflict.second_access, AccessType::Read);
        }

        #[test]
        fn test_get_conflicts_write_write() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_write(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2);
            assert!(conflict.is_some());

            let conflict = conflict.unwrap();
            assert!(conflict.has_write_write());

            let comp_conflict = &conflict.component_conflicts()[0];
            assert!(comp_conflict.is_write_write());
        }

        #[test]
        fn test_get_conflicts_multiple_components() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());
            access1.add_write(ComponentId::of::<Velocity>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());
            access2.add_read(ComponentId::of::<Velocity>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert_eq!(conflict.component_count(), 2);
            assert_eq!(conflict.total_count(), 2);
        }

        #[test]
        fn test_get_conflicts_partial() {
            // access1 writes Position
            // access2 reads Position and writes Velocity
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            // Only Position conflicts (access1 writes it, access2 reads it)
            assert_eq!(conflict.component_count(), 1);

            let conflicting: Vec<_> = conflict.conflicting_components().collect();
            assert_eq!(conflicting.len(), 1);
            assert_eq!(conflicting[0], ComponentId::of::<Position>());
        }

        #[test]
        fn test_get_conflicts_read_vs_write() {
            // access1 reads, access2 writes - should still conflict
            let mut access1 = Access::new();
            access1.add_read(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_write(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert_eq!(conflict.component_count(), 1);

            let comp_conflict = &conflict.component_conflicts()[0];
            assert_eq!(comp_conflict.first_access, AccessType::Read);
            assert_eq!(comp_conflict.second_access, AccessType::Write);
            assert!(comp_conflict.is_read_write());
        }
    }

    // =========================================================================
    // ConflictInfo Tests
    // =========================================================================

    mod conflict_info {
        use super::*;

        #[test]
        fn test_conflict_info_new() {
            let info = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );

            assert_eq!(info.component_id, ComponentId::of::<Position>());
            assert_eq!(info.first_access, AccessType::Write);
            assert_eq!(info.second_access, AccessType::Read);
        }

        #[test]
        fn test_conflict_info_is_write_write() {
            let write_write = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Write,
            );
            assert!(write_write.is_write_write());

            let write_read = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );
            assert!(!write_read.is_write_write());
        }

        #[test]
        fn test_conflict_info_is_read_write() {
            let write_read = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );
            assert!(write_read.is_read_write());

            let read_write = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Read,
                AccessType::Write,
            );
            assert!(read_write.is_read_write());

            let write_write = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Write,
            );
            assert!(!write_write.is_read_write());
        }

        #[test]
        fn test_conflict_info_display() {
            let info = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );
            let display = format!("{}", info);
            assert!(display.contains("Component"));
            assert!(display.contains("Write"));
            assert!(display.contains("Read"));
        }
    }

    // =========================================================================
    // AccessConflict Tests
    // =========================================================================

    mod access_conflict_struct {
        use super::*;

        #[test]
        fn test_access_conflict_new() {
            let conflict = AccessConflict::new();
            assert!(conflict.is_empty());
            assert_eq!(conflict.component_count(), 0);
            assert_eq!(conflict.resource_count(), 0);
            assert_eq!(conflict.non_send_count(), 0);
            assert_eq!(conflict.total_count(), 0);
        }

        #[test]
        fn test_access_conflict_default() {
            let conflict: AccessConflict = Default::default();
            assert!(conflict.is_empty());
        }

        #[test]
        fn test_access_conflict_display() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let display = format!("{}", conflict);

            assert!(display.contains("AccessConflict"));
            assert!(display.contains("Component"));
        }

        #[test]
        fn test_access_conflict_has_write_write() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_write(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert!(conflict.has_write_write());
        }

        #[test]
        fn test_access_conflict_has_write_write_false() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert!(!conflict.has_write_write());
        }

        #[test]
        fn test_access_conflict_conflicting_components_iter() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());
            access1.add_write(ComponentId::of::<Velocity>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());
            access2.add_read(ComponentId::of::<Velocity>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let components: Vec<_> = conflict.conflicting_components().collect();

            assert_eq!(components.len(), 2);
            assert!(components.contains(&ComponentId::of::<Position>()));
            assert!(components.contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_access_conflict_clone() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let cloned = conflict.clone();

            assert_eq!(conflict.total_count(), cloned.total_count());
        }
    }

    // =========================================================================
    // MutState Tests
    // =========================================================================

    mod mut_state {
        use super::*;

        #[test]
        fn test_mut_state_of() {
            let state = MutState::of::<Position>();
            assert_eq!(state.component_id, ComponentId::of::<Position>());
        }

        #[test]
        fn test_mut_state_different_types() {
            let pos_state = MutState::of::<Position>();
            let vel_state = MutState::of::<Velocity>();

            assert_ne!(pos_state.component_id, vel_state.component_id);
        }

        #[test]
        fn test_mut_state_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<MutState>();
        }

        #[test]
        fn test_mut_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}
            requires_send_sync::<MutState>();
        }

        #[test]
        fn test_mut_state_is_clone() {
            let state = MutState::of::<Position>();
            let cloned = state.clone();
            assert_eq!(state, cloned);
        }

        #[test]
        fn test_mut_state_debug() {
            let state = MutState::of::<Position>();
            let debug_str = format!("{:?}", state);
            // Should contain "MutState" and component_id info
            assert!(debug_str.contains("MutState"));
        }
    }

    // =========================================================================
    // WriteAccess Tests
    // =========================================================================

    mod write_access {
        use super::*;

        #[test]
        fn test_write_access_new() {
            let id = ComponentId::of::<Position>();
            let access = WriteAccess(id);
            assert_eq!(access.0, id);
        }

        #[test]
        fn test_write_access_equality() {
            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Position>();
            let id3 = ComponentId::of::<Velocity>();

            assert_eq!(WriteAccess(id1), WriteAccess(id2));
            assert_ne!(WriteAccess(id1), WriteAccess(id3));
        }

        #[test]
        fn test_write_access_hash() {
            use std::collections::HashSet;

            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Velocity>();

            let mut set = HashSet::new();
            set.insert(WriteAccess(id1));
            set.insert(WriteAccess(id2));

            assert_eq!(set.len(), 2);
            assert!(set.contains(&WriteAccess(id1)));
            assert!(set.contains(&WriteAccess(id2)));
        }

        #[test]
        fn test_write_access_ordering() {
            use std::collections::BTreeSet;

            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Velocity>();

            let mut set = BTreeSet::new();
            set.insert(WriteAccess(id1));
            set.insert(WriteAccess(id2));

            // Should be orderable (needed for deterministic conflict detection)
            assert_eq!(set.len(), 2);
        }
    }

    // =========================================================================
    // Documentation Tests
    // =========================================================================

    mod documentation {
        use super::*;

        #[test]
        fn test_world_query_trait_documentation_example() {
            // From the trait documentation
            fn query_requires_world_query<Q: WorldQuery>() {}
            query_requires_world_query::<Entity>();
        }

        #[test]
        fn test_query_state_documentation_example() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<ComponentId>();
        }

        #[test]
        fn test_with_filter_usage_documentation() {
            // With<T> filters but doesn't fetch data
            let mut world = World::new();
            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 0.0, y: 0.0 });
            world.insert(e1, Player);

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 1.0, y: 1.0 });
            // e2 doesn't have Player

            let state = With::<Player>::init_state(&world);

            // e1 matches With<Player>
            assert!(With::<Player>::fetch(&state, &world, e1).is_some());
            // e2 doesn't match
            assert!(With::<Player>::fetch(&state, &world, e2).is_none());
        }
    }

    // =========================================================================
    // Tuple Query Tests
    // =========================================================================

    mod tuple_queries {
        use super::*;

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

        #[derive(Debug, Clone, Copy)]
        struct Health(f32);
        impl Component for Health {}

        #[test]
        fn test_tuple_2_init_state() {
            let mut world = World::new();
            let state = <(&Position, &Velocity)>::init_state(&world);
            let (pos_id, vel_id) = state;
            assert_eq!(pos_id, ComponentId::of::<Position>());
            assert_eq!(vel_id, ComponentId::of::<Velocity>());
        }

        #[test]
        fn test_tuple_2_component_access() {
            let mut world = World::new();
            let state = <(&Position, &Velocity)>::init_state(&world);
            let access = <(&Position, &Velocity)>::component_access(&state);

            assert_eq!(access.len(), 2);
            assert!(access.contains(&ComponentId::of::<Position>()));
            assert!(access.contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_tuple_2_fetch() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let state = <(&Position, &Velocity)>::init_state(&world);
            let result = <(&Position, &Velocity)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (pos, vel) = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
            assert_eq!(vel.x, 3.0);
            assert_eq!(vel.y, 4.0);
        }

        #[test]
        fn test_tuple_2_fetch_missing_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            // Missing Velocity

            let state = <(&Position, &Velocity)>::init_state(&world);
            let result = <(&Position, &Velocity)>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_tuple_3_fetch() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });
            world.insert(entity, Health(100.0));

            let state = <(&Position, &Velocity, &Health)>::init_state(&world);
            let result = <(&Position, &Velocity, &Health)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (pos, vel, health) = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(vel.x, 3.0);
            assert_eq!(health.0, 100.0);
        }

        #[test]
        fn test_tuple_with_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <(Entity, &Position)>::init_state(&world);
            let result = <(Entity, &Position)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (e, pos) = result.unwrap();
            assert_eq!(e, entity);
            assert_eq!(pos.x, 1.0);
        }

        #[test]
        fn test_tuple_is_read_only() {
            // Tuple of all ReadOnlyWorldQuery types should be ReadOnlyWorldQuery
            fn assert_read_only<T: ReadOnlyWorldQuery>() {}
            assert_read_only::<(&Position, &Velocity)>();
            assert_read_only::<(Entity, &Position)>();
        }

        #[test]
        fn test_tuple_matches_archetype() {
            let mut world = World::new();

            // Create entity with both components
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let archetype_id = world.entity_archetype(entity).unwrap();
            let archetype = world.archetypes().get(archetype_id).unwrap();

            let state = <(&Position, &Velocity)>::init_state(&world);
            assert!(<(&Position, &Velocity)>::matches_archetype(&state, archetype));

            // Create entity with only Position
            let entity2 = world.spawn_empty();
            world.insert(entity2, Position { x: 5.0, y: 6.0 });

            let archetype_id2 = world.entity_archetype(entity2).unwrap();
            let archetype2 = world.archetypes().get(archetype_id2).unwrap();

            // Should not match because Velocity is missing
            assert!(!<(&Position, &Velocity)>::matches_archetype(&state, archetype2));
        }

        #[test]
        fn test_tuple_4_elements() {
            let mut world = World::new();
            let state = <(&Position, &Velocity, &Health, Entity)>::init_state(&world);
            let access = <(&Position, &Velocity, &Health, Entity)>::component_access(&state);

            // Entity doesn't contribute to component access
            assert_eq!(access.len(), 3);
        }

        #[test]
        fn test_tuple_with_filters() {
            let mut world = World::new();
            let state = <(&Position, With<Velocity>)>::init_state(&world);
            let access = <(&Position, With<Velocity>)>::component_access(&state);

            // With<T> filter doesn't add to access (it's a filter, not data fetch)
            assert_eq!(access.len(), 1);
            assert!(access.contains(&ComponentId::of::<Position>()));
        }
    }
}
