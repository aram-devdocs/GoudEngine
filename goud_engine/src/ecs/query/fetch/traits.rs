//! Core traits for the query fetch system.
//!
//! Defines [`WorldQuery`], [`ReadOnlyWorldQuery`], and [`QueryState`], which
//! are the foundational building blocks of the ECS query system.

use std::collections::BTreeSet;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
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
