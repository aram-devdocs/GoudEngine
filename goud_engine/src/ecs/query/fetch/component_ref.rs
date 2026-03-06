//! Component reference query implementations: `&T` and `&mut T`.
//!
//! Provides [`WorldQuery`] implementations for immutable and mutable component
//! references, along with [`MutState`] and [`WriteAccess`] helper types.

use std::collections::BTreeSet;

use crate::ecs::archetype::Archetype;
use crate::ecs::component::ComponentId;
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::traits::{ReadOnlyWorldQuery, WorldQuery};

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
    fn fetch<'w>(
        _state: &Self::State,
        _world: &'w World,
        _entity: Entity,
    ) -> Option<Self::Item<'w>> {
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
