//! Query filter types: [`With`] and [`Without`].
//!
//! Filters narrow the set of entities a query matches without fetching
//! component data. They implement [`WorldQuery`] and [`ReadOnlyWorldQuery`].

use std::collections::BTreeSet;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::traits::{ReadOnlyWorldQuery, WorldQuery};

// =============================================================================
// With Filter
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

// =============================================================================
// Without Filter
// =============================================================================

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
