//! `WorldQuery` implementations for primitive query types.
//!
//! Provides implementations for [`Entity`], unit type `()`, and tuples of
//! up to 8 query elements.

use std::collections::BTreeSet;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::traits::{ReadOnlyWorldQuery, WorldQuery};

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
    fn fetch<'w>(
        _state: &Self::State,
        _world: &'w World,
        _entity: Entity,
    ) -> Option<Self::Item<'w>> {
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
