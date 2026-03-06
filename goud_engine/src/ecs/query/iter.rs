//! Query iterators: [`QueryIter`] (immutable) and [`QueryIterMut`] (mutable).

use crate::ecs::archetype::ArchetypeId;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::fetch::WorldQuery;
use super::query_type::Query;

// =============================================================================
// Query Iterator
// =============================================================================

/// Iterator over query results (immutable access).
///
/// When the query has an archetype cache, the iterator only visits cached
/// archetype indices instead of scanning every archetype in the world.
pub struct QueryIter<'w, 'q, Q: WorldQuery, F: WorldQuery> {
    /// Reference to the query.
    query: &'q Query<Q, F>,
    /// Reference to the world.
    world: &'w World,
    /// Current position in the iteration (either archetype graph index or
    /// index into the cached archetype list, depending on `cached`).
    cursor: usize,
    /// Current entity index within the current archetype.
    entity_index: usize,
    /// Whether we are iterating over cached archetype indices.
    cached: bool,
}

impl<'w, 'q, Q: WorldQuery, F: WorldQuery> QueryIter<'w, 'q, Q, F> {
    /// Creates a new query iterator.
    #[inline]
    pub(crate) fn new(query: &'q Query<Q, F>, world: &'w World) -> Self {
        let cached = query.archetype_cache.is_some();
        Self {
            query,
            world,
            cursor: 0,
            entity_index: 0,
            cached,
        }
    }

    /// Resolves the current cursor to an actual archetype graph index.
    #[inline]
    fn current_archetype_index(&self) -> Option<usize> {
        if self.cached {
            // SAFETY: `cached` is only true when archetype_cache is Some
            let cache = self.query.archetype_cache.as_ref()?;
            let indices = cache.archetype_indices();
            if self.cursor < indices.len() {
                Some(indices[self.cursor])
            } else {
                None
            }
        } else {
            let total = self.world.archetypes().len();
            if self.cursor < total {
                Some(self.cursor)
            } else {
                None
            }
        }
    }
}

impl<'w, 'q, Q: WorldQuery, F: WorldQuery> Iterator for QueryIter<'w, 'q, Q, F> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let graph_index = self.current_archetype_index()?;
            let id = ArchetypeId::new(graph_index as u32);
            let archetype = self.world.archetypes().get(id)?;

            // When not using cache, skip non-matching archetypes
            if !self.cached && !self.query.matches_archetype(archetype) {
                self.cursor += 1;
                self.entity_index = 0;
                continue;
            }

            let entities = archetype.entities();

            if self.entity_index >= entities.len() {
                self.cursor += 1;
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
    pub(crate) fn new(query: &'q Query<Q, F>, world: &'w mut World) -> Self {
        // Collect all matching entities first
        let mut entities = Vec::new();

        if let Some(cache) = &query.archetype_cache {
            // Use cached archetype indices -- skip the full scan
            let archetypes = world.archetypes();
            for &idx in cache.archetype_indices() {
                let id = ArchetypeId::new(idx as u32);
                if let Some(archetype) = archetypes.get(id) {
                    entities.extend_from_slice(archetype.entities());
                }
            }
        } else {
            // Fallback: scan all archetypes
            for archetype in world.archetypes().iter() {
                if query.matches_archetype(archetype) {
                    entities.extend_from_slice(archetype.entities());
                }
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
