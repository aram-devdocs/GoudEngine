//! The [`Query`] struct and its core methods.

use crate::ecs::archetype::Archetype;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::cache::QueryArchetypeCache;
use super::fetch::{Access, ReadOnlyWorldQuery, WorldQuery};
use super::iter::{QueryIter, QueryIterMut};

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
    pub(crate) query_state: Q::State,
    /// Cached state for the filter.
    pub(crate) filter_state: F::State,
    /// Optional archetype cache for faster iteration between frames.
    pub(crate) archetype_cache: Option<QueryArchetypeCache>,
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
            archetype_cache: None,
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
            archetype_cache: None,
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

    /// Returns a reference to the archetype cache, if present.
    #[inline]
    pub fn archetype_cache(&self) -> Option<&QueryArchetypeCache> {
        self.archetype_cache.as_ref()
    }

    /// Updates the archetype cache from the current world state.
    ///
    /// If no cache exists, one is created and all archetypes are evaluated.
    /// If a cache exists but is stale, only newly added archetypes are checked.
    /// If the cache is already current, this is a no-op.
    #[inline]
    pub fn update_cache(&mut self, world: &World) {
        let archetypes = world.archetypes();
        let count = archetypes.len();

        // Borrow query_state and filter_state directly to avoid borrowing all
        // of `self` while also mutating the cache.
        let query_state = &self.query_state;
        let filter_state = &self.filter_state;

        let cache = self.archetype_cache.get_or_insert_with(QueryArchetypeCache::new);
        cache.update(count, |id| {
            if let Some(archetype) = archetypes.get(id) {
                Q::matches_archetype(query_state, archetype)
                    && F::matches_archetype(filter_state, archetype)
            } else {
                false
            }
        });
    }

    /// Clears the archetype cache, forcing a full re-evaluation next time.
    #[inline]
    pub fn invalidate_cache(&mut self) {
        self.archetype_cache = None;
    }

    /// Builder method: creates the query with an initialized archetype cache.
    #[inline]
    pub fn with_cache(mut self, world: &World) -> Self {
        self.update_cache(world);
        self
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
