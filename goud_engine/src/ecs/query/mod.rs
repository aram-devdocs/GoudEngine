//! Query system for the ECS.
//!
//! The query module provides type-safe access to component data stored in the
//! [`World`]. Queries are the primary way systems interact
//! with entities and their components.
//!
//! # Architecture
//!
//! The query system is built around several key concepts:
//!
//! - **WorldQuery**: Trait that defines what data a query fetches
//! - **Query**: The main query type used in systems
//! - **QueryState**: Cached state for efficient query execution
//! - **Filters**: Types like `With<T>` and `Without<T>` that filter entities
//!
//! # Basic Usage
//!
//! ```
//! use goud_engine::ecs::{World, Component, Entity};
//! use goud_engine::ecs::query::{Query, WorldQuery, With, Without};
//!
//! // Define components
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Velocity { x: f32, y: f32 }
//! impl Component for Velocity {}
//!
//! // Create and use a query
//! let mut world = World::new();
//! let entity = world.spawn_empty();
//! world.insert(entity, Position { x: 1.0, y: 2.0 });
//!
//! // In a system, queries are obtained via SystemParam
//! // For direct use:
//! let mut query: Query<&Position> = Query::new(&world);
//! for pos in query.iter(&world) {
//!     println!("Position: {:?}", pos);
//! }
//! ```
//!
//! # Query Types
//!
//! ## Data Queries
//!
//! - `Entity` - Returns the entity ID itself
//! - `&T` - Immutable reference to component T
//! - `&mut T` - Mutable reference to component T
//! - `Option<Q>` - Optional query, matches even if inner doesn't (future)
//!
//! ## Filters
//!
//! - `With<T>` - Match entities that have component T
//! - `Without<T>` - Match entities that don't have component T
//!
//! # Access Conflict Detection
//!
//! The query system tracks read and write access to prevent data races:
//!
//! - `&T` marks a read access to component T
//! - `&mut T` marks a write access to component T
//! - Two queries conflict if one writes a component the other reads/writes
//!
//! Use the [`Access`] type to build and check access patterns.
//!
//! # Using Query as a System Parameter
//!
//! `Query<Q, F>` implements [`SystemParam`], allowing it to be used as a
//! function system parameter:
//!
//! ```ignore
//! fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
//!     for (mut pos, vel) in query.iter_mut() {
//!         pos.x += vel.x;
//!         pos.y += vel.y;
//!     }
//! }
//! ```
//!
//! # Design Principles
//!
//! 1. **Type Safety**: Query types checked at compile time
//! 2. **Performance**: State caching avoids runtime lookups
//! 3. **Flexibility**: Compose queries with tuples and filters
//! 4. **Parallel Safety**: Read/write access tracked for safe parallelism

pub mod fetch;

use crate::ecs::archetype::Archetype;
use crate::ecs::entity::Entity;
use crate::ecs::system::{ReadOnlySystemParam, SystemParam, SystemParamState};
use crate::ecs::World;

// Re-export query types
pub use fetch::{
    Access, AccessConflict, AccessType, ConflictInfo, MutState, NonSendConflictInfo, QueryState,
    ReadOnlyWorldQuery, ResourceConflictInfo, With, Without, WorldQuery, WriteAccess,
};

// =============================================================================
// Query Struct
// =============================================================================

/// A cached query that can efficiently iterate over entities and components.
///
/// `Query<Q, F>` is the main query type used in systems. It caches the query
/// state (component IDs, matched archetypes) for efficient repeated iteration.
///
/// # Type Parameters
///
/// - `Q`: The [`WorldQuery`] type defining what data to fetch
/// - `F`: An optional filter type (defaults to `()` for no filter)
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World, Component};
/// use goud_engine::ecs::query::{Query, With};
///
/// #[derive(Debug, Clone, Copy)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// #[derive(Debug, Clone, Copy)]
/// struct Player;
/// impl Component for Player {}
///
/// let mut world = World::new();
/// let e = world.spawn_empty();
/// world.insert(e, Position { x: 0.0, y: 0.0 });
/// world.insert(e, Player);
///
/// // Query for Position with Player filter
/// let query: Query<&Position, With<Player>> = Query::new(&world);
/// ```
///
/// # Performance
///
/// The query caches:
/// - Component IDs for O(1) archetype matching
/// - Matched archetype indices for efficient iteration
///
/// Iteration is cache-friendly as it processes entities in archetype order.
pub struct Query<Q: WorldQuery, F: WorldQuery = ()> {
    /// Cached state for the data query.
    query_state: Q::State,
    /// Cached state for the filter.
    filter_state: F::State,
}

impl<Q: WorldQuery, F: WorldQuery> Query<Q, F> {
    /// Creates a new query from a world reference.
    ///
    /// This initializes the query state by computing component IDs and
    /// caching archetype matches.
    ///
    /// # Arguments
    ///
    /// * `world` - Reference to the world to query
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    /// use goud_engine::ecs::query::Query;
    ///
    /// #[derive(Debug)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let world = World::new();
    /// let query: Query<&Position> = Query::new(&world);
    /// ```
    #[inline]
    pub fn new(world: &World) -> Self {
        Self {
            query_state: Q::init_state(world),
            filter_state: F::init_state(world),
        }
    }

    /// Creates a query from pre-initialized state.
    ///
    /// This is used by the system parameter infrastructure to reuse cached
    /// state across multiple system runs.
    ///
    /// # Arguments
    ///
    /// * `query_state` - Pre-initialized query state
    /// * `filter_state` - Pre-initialized filter state
    #[inline]
    pub fn from_state(query_state: Q::State, filter_state: F::State) -> Self {
        Self {
            query_state,
            filter_state,
        }
    }

    /// Returns the component access pattern for this query.
    ///
    /// Used for conflict detection between queries. Two queries conflict if
    /// one writes a component the other reads or writes.
    ///
    /// # Returns
    ///
    /// An [`Access`] describing which components are read or written.
    #[inline]
    pub fn component_access(&self) -> Access {
        let mut access = Access::new();
        for id in Q::component_access(&self.query_state) {
            access.add_read(id);
        }
        // Filters don't count as reads (they only check existence)
        access
    }

    /// Returns `true` if this query is read-only.
    ///
    /// Read-only queries can run in parallel with other read-only queries
    /// on the same components.
    #[inline]
    pub fn is_read_only(&self) -> bool
    where
        Q: ReadOnlyWorldQuery,
        F: ReadOnlyWorldQuery,
    {
        true
    }

    /// Checks whether an archetype matches this query.
    ///
    /// An archetype matches if it satisfies both the data query requirements
    /// and all filter conditions.
    #[inline]
    pub fn matches_archetype(&self, archetype: &Archetype) -> bool {
        Q::matches_archetype(&self.query_state, archetype)
            && F::matches_archetype(&self.filter_state, archetype)
    }

    /// Fetches data for a specific entity.
    ///
    /// Returns `Some(item)` if the entity matches the query and filter,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `world` - Reference to the world
    /// * `entity` - Entity to fetch data for
    ///
    /// # Returns
    ///
    /// `Some(Q::Item)` if the entity matches, `None` otherwise.
    #[inline]
    pub fn get<'w>(&self, world: &'w World, entity: Entity) -> Option<Q::Item<'w>> {
        // Check filter first
        F::fetch(&self.filter_state, world, entity)?;
        // Then fetch data
        Q::fetch(&self.query_state, world, entity)
    }

    /// Fetches mutable data for a specific entity.
    ///
    /// Returns `Some(item)` if the entity matches the query and filter,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the world
    /// * `entity` - Entity to fetch data for
    ///
    /// # Returns
    ///
    /// `Some(Q::Item)` if the entity matches, `None` otherwise.
    #[inline]
    pub fn get_mut<'w>(&self, world: &'w mut World, entity: Entity) -> Option<Q::Item<'w>> {
        // Check filter first (uses immutable access)
        F::fetch(&self.filter_state, world, entity)?;
        // Then fetch mutable data
        Q::fetch_mut(&self.query_state, world, entity)
    }

    /// Iterates over all matching entities, yielding immutable query results.
    ///
    /// This iterates through all archetypes that match the query and yields
    /// the query result for each matching entity.
    ///
    /// # Arguments
    ///
    /// * `world` - Reference to the world
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    /// use goud_engine::ecs::query::Query;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let e = world.spawn_empty();
    /// world.insert(e, Position { x: 1.0, y: 2.0 });
    ///
    /// let query: Query<&Position> = Query::new(&world);
    /// for pos in query.iter(&world) {
    ///     println!("Position: {:?}", pos);
    /// }
    /// ```
    #[inline]
    pub fn iter<'w, 'q>(&'q self, world: &'w World) -> QueryIter<'w, 'q, Q, F> {
        QueryIter::new(self, world)
    }

    /// Iterates over all matching entities, yielding mutable query results.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the world
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    /// use goud_engine::ecs::query::Query;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let e = world.spawn_empty();
    /// world.insert(e, Position { x: 1.0, y: 2.0 });
    ///
    /// let query: Query<&mut Position> = Query::new(&world);
    /// for pos in query.iter_mut(&mut world) {
    ///     pos.x += 10.0;
    /// }
    /// ```
    #[inline]
    pub fn iter_mut<'w, 'q>(&'q self, world: &'w mut World) -> QueryIterMut<'w, 'q, Q, F> {
        QueryIterMut::new(self, world)
    }

    /// Returns the number of entities that match this query.
    ///
    /// This iterates through all archetypes to count matching entities.
    /// For large worlds, consider caching this count if needed frequently.
    #[inline]
    pub fn count(&self, world: &World) -> usize {
        let mut count = 0;
        for archetype in world.archetypes().iter() {
            if self.matches_archetype(archetype) {
                count += archetype.len();
            }
        }
        count
    }

    /// Returns `true` if any entity matches this query.
    #[inline]
    pub fn is_empty(&self, world: &World) -> bool {
        for archetype in world.archetypes().iter() {
            if self.matches_archetype(archetype) && !archetype.is_empty() {
                return false;
            }
        }
        true
    }

    /// Gets a single result from this query.
    ///
    /// Returns `Some(item)` if exactly one entity matches, `None` otherwise.
    /// Useful for singleton entities.
    #[inline]
    pub fn single<'w>(&self, world: &'w World) -> Option<Q::Item<'w>> {
        let mut iter = self.iter(world);
        let first = iter.next()?;
        // Ensure there's only one
        if iter.next().is_some() {
            None
        } else {
            Some(first)
        }
    }
}

impl<Q: WorldQuery, F: WorldQuery> std::fmt::Debug for Query<Q, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query")
            .field("query_type", &std::any::type_name::<Q>())
            .field("filter_type", &std::any::type_name::<F>())
            .finish()
    }
}

// =============================================================================
// Query Iterator
// =============================================================================

/// Iterator over query results (immutable access).
pub struct QueryIter<'w, 'q, Q: WorldQuery, F: WorldQuery> {
    /// Reference to the query.
    query: &'q Query<Q, F>,
    /// Reference to the world.
    world: &'w World,
    /// Current archetype index in the archetype graph.
    archetype_index: usize,
    /// Current entity index within the current archetype.
    entity_index: usize,
}

impl<'w, 'q, Q: WorldQuery, F: WorldQuery> QueryIter<'w, 'q, Q, F> {
    /// Creates a new query iterator.
    #[inline]
    fn new(query: &'q Query<Q, F>, world: &'w World) -> Self {
        Self {
            query,
            world,
            archetype_index: 0,
            entity_index: 0,
        }
    }
}

impl<'w, 'q, Q: WorldQuery, F: WorldQuery> Iterator for QueryIter<'w, 'q, Q, F> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Get current archetype
            let archetypes: Vec<_> = self.world.archetypes().iter().collect();
            if self.archetype_index >= archetypes.len() {
                return None;
            }

            let archetype = archetypes[self.archetype_index];

            // Skip archetypes that don't match
            if !self.query.matches_archetype(archetype) {
                self.archetype_index += 1;
                self.entity_index = 0;
                continue;
            }

            // Get entities in this archetype
            let entities = archetype.entities();
            if self.entity_index >= entities.len() {
                // Move to next archetype
                self.archetype_index += 1;
                self.entity_index = 0;
                continue;
            }

            let entity = entities[self.entity_index];
            self.entity_index += 1;

            // Try to fetch data for this entity
            if let Some(item) = self.query.get(self.world, entity) {
                return Some(item);
            }
            // If fetch failed (entity despawned, etc.), continue to next
        }
    }
}

// =============================================================================
// Query Iterator Mut
// =============================================================================

/// Iterator over query results (mutable access).
///
/// Due to Rust's borrowing rules, this iterator collects matching entities
/// first, then yields them one at a time with mutable access.
pub struct QueryIterMut<'w, 'q, Q: WorldQuery, F: WorldQuery> {
    /// Reference to the query.
    query: &'q Query<Q, F>,
    /// Mutable reference to the world.
    world: &'w mut World,
    /// Collected entities that match the query.
    entities: Vec<Entity>,
    /// Current index in the entities vec.
    current_index: usize,
}

impl<'w, 'q, Q: WorldQuery, F: WorldQuery> QueryIterMut<'w, 'q, Q, F> {
    /// Creates a new mutable query iterator.
    #[inline]
    fn new(query: &'q Query<Q, F>, world: &'w mut World) -> Self {
        // Collect all matching entities first
        let mut entities = Vec::new();
        for archetype in world.archetypes().iter() {
            if query.matches_archetype(archetype) {
                entities.extend_from_slice(archetype.entities());
            }
        }

        Self {
            query,
            world,
            entities,
            current_index: 0,
        }
    }
}

impl<'w, 'q, Q: WorldQuery, F: WorldQuery> Iterator for QueryIterMut<'w, 'q, Q, F> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_index >= self.entities.len() {
                return None;
            }

            let entity = self.entities[self.current_index];
            self.current_index += 1;

            // SAFETY: We ensure exclusive access to the world, and the query
            // type Q determines what components are accessed. The borrow
            // checker ensures we don't have overlapping mutable borrows.
            // We need to reborrow the world for each iteration.
            let world_ptr = self.world as *mut World;
            let world_ref: &'w mut World = unsafe { &mut *world_ptr };

            // Check filter first
            if F::fetch(&self.query.filter_state, world_ref, entity).is_none() {
                continue;
            }

            // Then fetch mutable data
            if let Some(item) = Q::fetch_mut(&self.query.query_state, world_ref, entity) {
                return Some(item);
            }
            // If fetch failed (entity despawned, etc.), continue to next
        }
    }
}

// =============================================================================
// Query SystemParam Implementation
// =============================================================================

/// Cached state for Query as a system parameter.
///
/// This stores the initialized query and filter states, allowing efficient
/// reuse across multiple system runs.
#[derive(Clone)]
pub struct QuerySystemParamState<Q: WorldQuery, F: WorldQuery> {
    /// Cached query state.
    query_state: Q::State,
    /// Cached filter state.
    filter_state: F::State,
}

impl<Q: WorldQuery, F: WorldQuery> std::fmt::Debug for QuerySystemParamState<Q, F>
where
    Q::State: std::fmt::Debug,
    F::State: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuerySystemParamState")
            .field("query_state", &self.query_state)
            .field("filter_state", &self.filter_state)
            .finish()
    }
}

// SAFETY: QuerySystemParamState is Send + Sync if the underlying states are
unsafe impl<Q: WorldQuery, F: WorldQuery> Send for QuerySystemParamState<Q, F>
where
    Q::State: Send,
    F::State: Send,
{
}

unsafe impl<Q: WorldQuery, F: WorldQuery> Sync for QuerySystemParamState<Q, F>
where
    Q::State: Sync,
    F::State: Sync,
{
}

impl<Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamState
    for QuerySystemParamState<Q, F>
where
    Q::State: Clone + Send + Sync,
    F::State: Clone + Send + Sync,
{
    fn init(world: &mut World) -> Self {
        Self {
            query_state: Q::init_state(world),
            filter_state: F::init_state(world),
        }
    }
}

/// `Query<Q, F>` as a system parameter.
///
/// This allows functions to declare queries as parameters:
///
/// ```ignore
/// fn my_system(query: Query<&Position, With<Player>>) {
///     for pos in query.iter() {
///         println!("Player position: {:?}", pos);
///     }
/// }
/// ```
impl<Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParam for Query<Q, F>
where
    Q::State: Clone + Send + Sync + 'static,
    F::State: Clone + Send + Sync + 'static,
{
    type State = QuerySystemParamState<Q, F>;
    type Item<'w, 's> = Query<Q, F>;

    fn update_access(state: &Self::State, access: &mut Access) {
        // Add component access from the query
        for id in Q::component_access(&state.query_state) {
            // For now, we add as read. Mutable queries need special handling.
            // The WorldQuery implementation determines read vs write.
            access.add_read(id);
        }
        // Filters don't count as access (they only check archetype)
    }

    fn get_param<'w, 's>(state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
        Query::from_state(state.query_state.clone(), state.filter_state.clone())
    }

    fn get_param_mut<'w, 's>(
        state: &'s mut Self::State,
        _world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        // For mutable access, we still return a Query that can iterate mutably
        Query::from_state(state.query_state.clone(), state.filter_state.clone())
    }
}

/// Query with read-only data query is a read-only system parameter.
impl<Q: ReadOnlyWorldQuery + 'static, F: ReadOnlyWorldQuery + 'static> ReadOnlySystemParam
    for Query<Q, F>
where
    Q::State: Clone + Send + Sync + 'static,
    F::State: Clone + Send + Sync + 'static,
{
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::ComponentId;
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

    #[derive(Debug, Clone, Copy)]
    struct Enemy;
    impl Component for Enemy {}

    // =========================================================================
    // Query Structure Tests
    // =========================================================================

    mod query_struct {
        use super::*;

        #[test]
        fn test_query_new() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            // Should compile and run
            assert!(query.is_empty(&world));
        }

        #[test]
        fn test_query_with_filter() {
            let world = World::new();
            let query: Query<&Position, With<Player>> = Query::new(&world);
            assert!(query.is_empty(&world));
        }

        #[test]
        fn test_query_from_state() {
            let world = World::new();
            let query_state = <&Position>::init_state(&world);
            let filter_state = ();
            let query: Query<&Position> = Query::from_state(query_state, filter_state);
            assert!(query.is_empty(&world));
        }

        #[test]
        fn test_query_debug() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            let debug_str = format!("{:?}", query);
            assert!(debug_str.contains("Query"));
        }

        #[test]
        fn test_query_component_access() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            let access = query.component_access();
            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Position>()));
        }
    }

    // =========================================================================
    // Query Get Tests
    // =========================================================================

    mod query_get {
        use super::*;

        #[test]
        fn test_query_get_existing() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let query: Query<&Position> = Query::new(&world);
            let result = query.get(&world, entity);

            assert!(result.is_some());
            assert_eq!(result.unwrap(), &Position { x: 1.0, y: 2.0 });
        }

        #[test]
        fn test_query_get_missing_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let query: Query<&Position> = Query::new(&world);
            let result = query.get(&world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_query_get_dead_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let query: Query<&Position> = Query::new(&world);
            let result = query.get(&world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_query_get_with_filter_passing() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Player);

            let query: Query<&Position, With<Player>> = Query::new(&world);
            let result = query.get(&world, entity);

            assert!(result.is_some());
        }

        #[test]
        fn test_query_get_with_filter_failing() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            // No Player component

            let query: Query<&Position, With<Player>> = Query::new(&world);
            let result = query.get(&world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_query_get_with_without_filter() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });
            world.insert(e1, Enemy); // Has Enemy

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });
            // No Enemy

            let query: Query<&Position, Without<Enemy>> = Query::new(&world);

            // e1 has Enemy, should fail
            assert!(query.get(&world, e1).is_none());
            // e2 doesn't have Enemy, should pass
            assert!(query.get(&world, e2).is_some());
        }
    }

    // =========================================================================
    // Query Iteration Tests
    // =========================================================================

    mod query_iter {
        use super::*;

        #[test]
        fn test_query_iter_empty_world() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            let count = query.iter(&world).count();
            assert_eq!(count, 0);
        }

        #[test]
        fn test_query_iter_single_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let query: Query<&Position> = Query::new(&world);
            let results: Vec<_> = query.iter(&world).collect();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0], &Position { x: 1.0, y: 2.0 });
        }

        #[test]
        fn test_query_iter_multiple_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            let e3 = world.spawn_empty();
            world.insert(e3, Position { x: 5.0, y: 6.0 });

            let query: Query<&Position> = Query::new(&world);
            let results: Vec<_> = query.iter(&world).collect();

            assert_eq!(results.len(), 3);
        }

        #[test]
        fn test_query_iter_with_filter() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });
            world.insert(e1, Player);

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });
            // No Player

            let e3 = world.spawn_empty();
            world.insert(e3, Position { x: 5.0, y: 6.0 });
            world.insert(e3, Player);

            let query: Query<&Position, With<Player>> = Query::new(&world);
            let results: Vec<_> = query.iter(&world).collect();

            // Only e1 and e3 have Player
            assert_eq!(results.len(), 2);
        }

        #[test]
        fn test_query_iter_skips_non_matching() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Velocity { x: 3.0, y: 4.0 });
            // No Position

            let query: Query<&Position> = Query::new(&world);
            let results: Vec<_> = query.iter(&world).collect();

            assert_eq!(results.len(), 1);
        }

        #[test]
        fn test_query_iter_entity() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            let query: Query<Entity> = Query::new(&world);
            let entities: Vec<_> = query.iter(&world).collect();

            assert_eq!(entities.len(), 2);
            assert!(entities.contains(&e1));
            assert!(entities.contains(&e2));
        }
    }

    // =========================================================================
    // Query Iter Mut Tests
    // =========================================================================

    mod query_iter_mut {
        use super::*;

        #[test]
        fn test_query_iter_mut_modify() {
            let mut world = World::new();
            let e = world.spawn_empty();
            world.insert(e, Position { x: 1.0, y: 2.0 });

            {
                let query: Query<&mut Position> = Query::new(&world);
                for pos in query.iter_mut(&mut world) {
                    pos.x += 10.0;
                    pos.y += 20.0;
                }
            }

            // Verify modification
            let pos = world.get::<Position>(e).unwrap();
            assert_eq!(pos.x, 11.0);
            assert_eq!(pos.y, 22.0);
        }

        #[test]
        fn test_query_iter_mut_multiple() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            {
                let query: Query<&mut Position> = Query::new(&world);
                for pos in query.iter_mut(&mut world) {
                    pos.x *= 2.0;
                }
            }

            // Verify modifications
            assert_eq!(world.get::<Position>(e1).unwrap().x, 2.0);
            assert_eq!(world.get::<Position>(e2).unwrap().x, 6.0);
        }

        #[test]
        fn test_query_iter_mut_with_filter() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });
            world.insert(e1, Player);

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });
            // No Player

            {
                let query: Query<&mut Position, With<Player>> = Query::new(&world);
                for pos in query.iter_mut(&mut world) {
                    pos.x = 100.0;
                }
            }

            // Only e1 should be modified
            assert_eq!(world.get::<Position>(e1).unwrap().x, 100.0);
            assert_eq!(world.get::<Position>(e2).unwrap().x, 3.0); // Unchanged
        }
    }

    // =========================================================================
    // Query Count and Single Tests
    // =========================================================================

    mod query_count {
        use super::*;

        #[test]
        fn test_query_count_empty() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            assert_eq!(query.count(&world), 0);
        }

        #[test]
        fn test_query_count_multiple() {
            let mut world = World::new();

            for _ in 0..5 {
                let e = world.spawn_empty();
                world.insert(e, Position { x: 0.0, y: 0.0 });
            }

            let query: Query<&Position> = Query::new(&world);
            assert_eq!(query.count(&world), 5);
        }

        #[test]
        fn test_query_is_empty() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            assert!(query.is_empty(&world));
        }

        #[test]
        fn test_query_is_not_empty() {
            let mut world = World::new();
            let e = world.spawn_empty();
            world.insert(e, Position { x: 0.0, y: 0.0 });

            let query: Query<&Position> = Query::new(&world);
            assert!(!query.is_empty(&world));
        }

        #[test]
        fn test_query_single_one_entity() {
            let mut world = World::new();
            let e = world.spawn_empty();
            world.insert(e, Position { x: 1.0, y: 2.0 });

            let query: Query<&Position> = Query::new(&world);
            let result = query.single(&world);

            assert!(result.is_some());
            assert_eq!(result.unwrap(), &Position { x: 1.0, y: 2.0 });
        }

        #[test]
        fn test_query_single_no_entities() {
            let world = World::new();
            let query: Query<&Position> = Query::new(&world);
            assert!(query.single(&world).is_none());
        }

        #[test]
        fn test_query_single_multiple_entities() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            let query: Query<&Position> = Query::new(&world);
            // Should return None because there's more than one
            assert!(query.single(&world).is_none());
        }
    }

    // =========================================================================
    // Query SystemParam Tests
    // =========================================================================

    mod query_system_param {
        use super::*;

        #[test]
        fn test_query_state_init() {
            let mut world = World::new();
            let state: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);
            assert_eq!(state.query_state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_query_state_with_filter() {
            let mut world = World::new();
            let state: QuerySystemParamState<&Position, With<Player>> =
                QuerySystemParamState::init(&mut world);
            assert_eq!(state.query_state, ComponentId::of::<Position>());
            assert_eq!(state.filter_state, ComponentId::of::<Player>());
        }

        #[test]
        fn test_query_update_access() {
            let mut world = World::new();
            let state: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);

            let mut access = Access::new();
            Query::<&Position>::update_access(&state, &mut access);

            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Position>()));
            assert!(access.is_read_only());
        }

        #[test]
        fn test_query_get_param() {
            let mut world = World::new();
            let e = world.spawn_empty();
            world.insert(e, Position { x: 1.0, y: 2.0 });

            let mut state: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);

            let query: Query<&Position> = Query::get_param(&mut state, &world);
            let results: Vec<_> = query.iter(&world).collect();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0], &Position { x: 1.0, y: 2.0 });
        }

        #[test]
        fn test_query_get_param_mut() {
            let mut world = World::new();
            let e = world.spawn_empty();
            world.insert(e, Position { x: 1.0, y: 2.0 });

            let mut state: QuerySystemParamState<&mut Position, ()> =
                QuerySystemParamState::init(&mut world);

            let query: Query<&mut Position> = Query::get_param_mut(&mut state, &mut world);

            for pos in query.iter_mut(&mut world) {
                pos.x += 10.0;
            }

            assert_eq!(world.get::<Position>(e).unwrap().x, 11.0);
        }

        #[test]
        fn test_query_implements_system_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<Query<&Position>>();
            requires_system_param::<Query<&Position, With<Player>>>();
            requires_system_param::<Query<&mut Position>>();
        }

        #[test]
        fn test_read_only_query_implements_read_only_param() {
            fn requires_read_only<T: ReadOnlySystemParam>() {}
            requires_read_only::<Query<&Position>>();
            requires_read_only::<Query<&Position, With<Player>>>();
            requires_read_only::<Query<Entity>>();
        }

        #[test]
        fn test_query_state_is_clone() {
            let mut world = World::new();
            let state: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);
            let _cloned = state.clone();
        }

        #[test]
        fn test_query_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}
            requires_send_sync::<QuerySystemParamState<&Position, ()>>();
            requires_send_sync::<QuerySystemParamState<&Position, With<Player>>>();
        }
    }

    // =========================================================================
    // Query Access Conflict Tests
    // =========================================================================

    mod query_access {
        use super::*;

        #[test]
        fn test_read_queries_no_conflict() {
            let mut world = World::new();

            let state1: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);
            let state2: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);

            let mut access1 = Access::new();
            Query::<&Position>::update_access(&state1, &mut access1);

            let mut access2 = Access::new();
            Query::<&Position>::update_access(&state2, &mut access2);

            // Two read-only queries on the same component don't conflict
            assert!(!access1.conflicts_with(&access2));
        }

        #[test]
        fn test_different_component_queries_no_conflict() {
            let mut world = World::new();

            let state1: QuerySystemParamState<&Position, ()> =
                QuerySystemParamState::init(&mut world);
            let state2: QuerySystemParamState<&Velocity, ()> =
                QuerySystemParamState::init(&mut world);

            let mut access1 = Access::new();
            Query::<&Position>::update_access(&state1, &mut access1);

            let mut access2 = Access::new();
            Query::<&Velocity>::update_access(&state2, &mut access2);

            // Queries on different components don't conflict
            assert!(!access1.conflicts_with(&access2));
        }

        #[test]
        fn test_query_with_filter_access() {
            let mut world = World::new();

            let state: QuerySystemParamState<&Position, With<Player>> =
                QuerySystemParamState::init(&mut world);

            let mut access = Access::new();
            Query::<&Position, With<Player>>::update_access(&state, &mut access);

            // Only Position should be in the access, not Player (filters don't access data)
            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Position>()));
            // Player filter doesn't add to reads because filters only check archetype membership
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_query_with_entity_and_component() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            // Query for both Entity and Position would require tuple queries
            // For now, test separate queries
            let entity_query: Query<Entity> = Query::new(&world);
            let pos_query: Query<&Position> = Query::new(&world);

            let entities: Vec<_> = entity_query.iter(&world).collect();
            let positions: Vec<_> = pos_query.iter(&world).collect();

            assert_eq!(entities.len(), 2);
            assert_eq!(positions.len(), 2);
        }

        #[test]
        fn test_complex_filter_chain() {
            let mut world = World::new();

            // Entity with Position + Player
            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });
            world.insert(e1, Player);

            // Entity with Position + Enemy
            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });
            world.insert(e2, Enemy);

            // Entity with Position only
            let e3 = world.spawn_empty();
            world.insert(e3, Position { x: 5.0, y: 6.0 });

            // Query Position with Player filter
            let player_query: Query<&Position, With<Player>> = Query::new(&world);
            assert_eq!(player_query.count(&world), 1);

            // Query Position without Enemy
            let non_enemy_query: Query<&Position, Without<Enemy>> = Query::new(&world);
            assert_eq!(non_enemy_query.count(&world), 2); // e1 and e3
        }

        #[test]
        fn test_query_after_despawn() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            // Query before despawn
            let query: Query<&Position> = Query::new(&world);
            assert_eq!(query.count(&world), 2);

            // Despawn e1
            world.despawn(e1);

            // Re-create query (state needs refresh)
            let query: Query<&Position> = Query::new(&world);
            assert_eq!(query.count(&world), 1);
        }

        #[test]
        fn test_query_stress_test() {
            let mut world = World::new();

            // Create 1000 entities
            for i in 0..1000 {
                let e = world.spawn_empty();
                world.insert(
                    e,
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                );
                if i % 2 == 0 {
                    world.insert(e, Player);
                }
            }

            // Query all positions
            let query: Query<&Position> = Query::new(&world);
            assert_eq!(query.count(&world), 1000);

            // Query only players
            let player_query: Query<&Position, With<Player>> = Query::new(&world);
            assert_eq!(player_query.count(&world), 500);
        }
    }
}
