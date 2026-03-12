//! Optional query support: [`WorldQuery`] implementation for `Option<Q>`.
//!
//! Allows querying for components that may or may not be present on an entity.
//! `Option<&T>` always matches every archetype and returns `Some(Some(&T))`
//! when the component is present, or `Some(None)` when it is absent.

use std::collections::BTreeSet;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::traits::{ReadOnlyWorldQuery, WorldQuery};

// =============================================================================
// Option<Q> WorldQuery Implementation
// =============================================================================

/// Optional query wrapper.
///
/// `Option<Q>` always matches every archetype. When the inner query `Q` can
/// fetch data for an entity, the result is `Some(Q::Item)`. When it cannot,
/// the result is `None`. In both cases, the outer `fetch` returns `Some(...)`,
/// so the entity is never skipped by the query system.
///
/// # Use Cases
///
/// - Querying for a component that only some entities have
/// - Avoiding the need for separate queries when a component is optional
/// - Combining required and optional components in tuple queries
///
/// # Example
///
/// ```text
/// // Query all entities with Position, optionally fetching Velocity
/// // (&Position, Option<&Velocity>)
/// // - Entities with both: (pos, Some(vel))
/// // - Entities with only Position: (pos, None)
/// ```
impl<Q: WorldQuery> WorldQuery for Option<Q> {
    type Item<'w> = Option<Q::Item<'w>>;
    type State = Q::State;

    #[inline]
    fn init_state(world: &World) -> Self::State {
        Q::init_state(world)
    }

    #[inline]
    fn component_access(state: &Self::State) -> BTreeSet<ComponentId> {
        // Forward to inner query so mutable optional queries (Option<&mut T>)
        // correctly declare write access for conflict detection.
        Q::component_access(state)
    }

    #[inline]
    fn matches_archetype(_state: &Self::State, _archetype: &Archetype) -> bool {
        // Optional queries match every archetype — the component is optional.
        true
    }

    #[inline]
    fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        // Always return Some. The inner Option indicates presence/absence.
        Some(Q::fetch(state, world, entity))
    }

    #[inline]
    fn fetch_mut<'w>(
        state: &Self::State,
        world: &'w mut World,
        entity: Entity,
    ) -> Option<Self::Item<'w>> {
        // Always return Some. The inner Option indicates presence/absence.
        Some(Q::fetch_mut(state, world, entity))
    }
}

// Option<Q> is read-only when the inner query is read-only.
impl<Q: ReadOnlyWorldQuery> ReadOnlyWorldQuery for Option<Q> {}
