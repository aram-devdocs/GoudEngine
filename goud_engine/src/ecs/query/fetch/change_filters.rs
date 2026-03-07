//! Change detection query filters: [`Changed`] and [`Added`].
//!
//! These filters narrow a query to entities whose components have been
//! modified or newly inserted since the last system ran.

use std::collections::BTreeSet;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::traits::{ReadOnlyWorldQuery, WorldQuery};

// =============================================================================
// Changed Filter
// =============================================================================

/// Query filter matching entities whose component `T` was mutated since
/// the last system boundary.
///
/// `Changed<T>` checks whether the component's `changed_tick` is newer
/// than the world's `last_change_tick`. Mutations are detected when
/// `World::get_mut::<T>()` is called, or when the component is replaced
/// via `World::insert()`.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, World};
/// use goud_engine::ecs::query::Changed;
///
/// #[derive(Debug, Clone, Copy)]
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// // Query: (&Position, Changed<Velocity>)
/// // Only returns entities where Velocity was modified this tick.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Changed<T>(std::marker::PhantomData<T>);

impl<T> Changed<T> {
    /// Creates a new `Changed` filter.
    #[inline]
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: crate::ecs::Component> WorldQuery for Changed<T> {
    type Item<'w> = ();
    type State = ComponentId;

    #[inline]
    fn init_state(_world: &World) -> Self::State {
        ComponentId::of::<T>()
    }

    #[inline]
    fn component_access(_state: &Self::State) -> BTreeSet<ComponentId> {
        // Filters don't access component data, just check ticks
        BTreeSet::new()
    }

    #[inline]
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(*state)
    }

    #[inline]
    fn fetch<'w>(_state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        let changed_tick = world.get_component_changed_tick::<T>(entity)?;
        if changed_tick > world.last_change_tick() {
            Some(())
        } else {
            None
        }
    }
}

impl<T: crate::ecs::Component> ReadOnlyWorldQuery for Changed<T> {}

// =============================================================================
// Added Filter
// =============================================================================

/// Query filter matching entities whose component `T` was added since
/// the last system boundary.
///
/// `Added<T>` checks whether the component's `added_tick` is newer than
/// the world's `last_change_tick`. A component is "added" when it is
/// first inserted on an entity.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, World};
/// use goud_engine::ecs::query::Added;
///
/// struct Spawned;
/// impl Component for Spawned {}
///
/// // Query: (Entity, Added<Spawned>)
/// // Only returns entities that received the Spawned marker this tick.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Added<T>(std::marker::PhantomData<T>);

impl<T> Added<T> {
    /// Creates a new `Added` filter.
    #[inline]
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: crate::ecs::Component> WorldQuery for Added<T> {
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
        archetype.has_component(*state)
    }

    #[inline]
    fn fetch<'w>(_state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        let added_tick = world.get_component_added_tick::<T>(entity)?;
        if added_tick > world.last_change_tick() {
            Some(())
        } else {
            None
        }
    }
}

impl<T: crate::ecs::Component> ReadOnlyWorldQuery for Added<T> {}
