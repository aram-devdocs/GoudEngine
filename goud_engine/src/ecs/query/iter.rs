//! Query iterators: [`QueryIter`] (immutable) and [`QueryIterMut`] (mutable).

use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::fetch::WorldQuery;
use super::query_type::Query;

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
    pub(crate) fn new(query: &'q Query<Q, F>, world: &'w World) -> Self {
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
    pub(crate) fn new(query: &'q Query<Q, F>, world: &'w mut World) -> Self {
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
