//! Tests for Changed<T> and Added<T> query filters.

use crate::ecs::query::fetch::change_filters::{Added, Changed};
use crate::ecs::query::fetch::traits::WorldQuery;
use crate::ecs::Component;
use crate::ecs::World;

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

// =========================================================================
// SparseSet tick storage tests
// =========================================================================

#[test]
fn sparse_set_insert_with_tick_stores_correct_ticks() {
    use crate::ecs::{Entity, SparseSet};

    let mut set = SparseSet::new();
    let e = Entity::new(0, 1);

    set.insert_with_tick(e, 42i32, 5);

    assert_eq!(set.get_added_tick(e), Some(5));
    assert_eq!(set.get_changed_tick(e), Some(5));
}

#[test]
fn sparse_set_replace_updates_changed_tick_only() {
    use crate::ecs::{Entity, SparseSet};

    let mut set = SparseSet::new();
    let e = Entity::new(0, 1);

    set.insert_with_tick(e, 10i32, 1);
    set.insert_with_tick(e, 20i32, 5);

    assert_eq!(set.get_added_tick(e), Some(1));
    assert_eq!(set.get_changed_tick(e), Some(5));
    assert_eq!(set.get(e), Some(&20));
}

#[test]
fn sparse_set_set_changed_tick_works() {
    use crate::ecs::{Entity, SparseSet};

    let mut set = SparseSet::new();
    let e = Entity::new(0, 1);

    set.insert_with_tick(e, "hello", 1);
    set.set_changed_tick(e, 42);

    assert_eq!(set.get_changed_tick(e), Some(42));
    assert_eq!(set.get_added_tick(e), Some(1));
}

#[test]
fn sparse_set_remove_cleans_up_ticks() {
    use crate::ecs::{Entity, SparseSet};

    let mut set = SparseSet::new();
    let e1 = Entity::new(0, 1);
    let e2 = Entity::new(1, 1);

    set.insert_with_tick(e1, 100i32, 1);
    set.insert_with_tick(e2, 200i32, 2);
    set.remove(e1);

    assert_eq!(set.get_added_tick(e1), None);
    assert_eq!(set.get_changed_tick(e1), None);
    // e2 should still be accessible (may have been swap-removed to index 0)
    assert_eq!(set.get(e2), Some(&200));
    assert_eq!(set.get_added_tick(e2), Some(2));
}

// =========================================================================
// Changed<T> filter tests
// =========================================================================

#[test]
fn changed_detects_component_mutation_via_get_mut() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    // Insert at tick 0
    world.insert(entity, Position { x: 1.0, y: 2.0 });

    // Advance: simulate system boundary
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    // Mutate the component at the new tick
    world.increment_change_tick();
    if let Some(pos) = world.get_mut::<Position>(entity) {
        pos.x = 99.0;
    }

    // Changed should detect the mutation
    let state = <Changed<Position> as WorldQuery>::init_state(&world);
    let result = <Changed<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_some(), "Changed should detect mutation");
}

#[test]
fn changed_does_not_match_unmodified_components() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    // Insert at tick 0
    world.insert(entity, Position { x: 1.0, y: 2.0 });

    // Advance tick past the insertion
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    // Don't mutate -- just read
    let _ = world.get::<Position>(entity);

    // Changed should NOT match
    let state = <Changed<Position> as WorldQuery>::init_state(&world);
    let result = <Changed<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_none(), "Changed should not match unmodified");
}

#[test]
fn changed_detects_component_replacement() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 0.0, y: 0.0 });

    // Advance past insertion
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    // Replace the component at a new tick
    world.increment_change_tick();
    world.insert(entity, Position { x: 5.0, y: 5.0 });

    let state = <Changed<Position> as WorldQuery>::init_state(&world);
    let result = <Changed<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_some(), "Changed should detect replacement");
}

// =========================================================================
// Added<T> filter tests
// =========================================================================

#[test]
fn added_detects_newly_inserted_components() {
    let mut world = World::new();

    // Advance tick so insertion happens after last_change_tick = 0
    world.increment_change_tick();
    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 1.0, y: 2.0 });

    let state = <Added<Position> as WorldQuery>::init_state(&world);
    let result = <Added<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_some(), "Added should detect new insertion");
}

#[test]
fn added_does_not_match_after_tick_advance() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 1.0, y: 2.0 });

    // Advance past the insertion tick
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    let state = <Added<Position> as WorldQuery>::init_state(&world);
    let result = <Added<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_none(), "Added should not match old insertions");
}

#[test]
fn added_not_triggered_by_mutation() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 0.0, y: 0.0 });

    // Advance past insertion
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    // Mutate at a new tick -- should NOT re-trigger Added
    world.increment_change_tick();
    if let Some(pos) = world.get_mut::<Position>(entity) {
        pos.x = 50.0;
    }

    let state = <Added<Position> as WorldQuery>::init_state(&world);
    let result = <Added<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_none(), "Added should not trigger on mutation");
}

// =========================================================================
// Composition tests
// =========================================================================

#[test]
fn changed_filter_composes_with_data_query() {
    let mut world = World::new();
    let e1 = world.spawn_empty();
    let e2 = world.spawn_empty();

    world.insert(e1, Position { x: 0.0, y: 0.0 });
    world.insert(e1, Velocity { x: 1.0, y: 1.0 });
    world.insert(e2, Position { x: 5.0, y: 5.0 });
    world.insert(e2, Velocity { x: 2.0, y: 2.0 });

    // Advance past insertions
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    // Only mutate e1's velocity
    world.increment_change_tick();
    if let Some(vel) = world.get_mut::<Velocity>(e1) {
        vel.x = 99.0;
    }

    // Check Changed<Velocity> on both entities
    let state = <Changed<Velocity> as WorldQuery>::init_state(&world);
    assert!(
        <Changed<Velocity> as WorldQuery>::fetch(&state, &world, e1).is_some(),
        "e1 velocity was mutated"
    );
    assert!(
        <Changed<Velocity> as WorldQuery>::fetch(&state, &world, e2).is_none(),
        "e2 velocity was not mutated"
    );
}

#[test]
fn read_only_access_does_not_trigger_changed() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 1.0, y: 2.0 });

    // Advance past insertion
    let tick = world.increment_change_tick();
    world.set_last_change_tick(tick);

    // Increment tick, then only read (not write)
    world.increment_change_tick();
    let _ = world.get::<Position>(entity);

    let state = <Changed<Position> as WorldQuery>::init_state(&world);
    let result = <Changed<Position> as WorldQuery>::fetch(&state, &world, entity);
    assert!(result.is_none(), "Read-only get should not trigger Changed");
}

#[test]
fn change_detection_across_multiple_tick_increments() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    // Tick 1: insert
    world.increment_change_tick();
    world.insert(entity, Position { x: 0.0, y: 0.0 });

    // System boundary at tick 1
    world.set_last_change_tick(world.change_tick());

    // Tick 2: mutate
    world.increment_change_tick();
    if let Some(pos) = world.get_mut::<Position>(entity) {
        pos.x = 10.0;
    }

    let state_c = <Changed<Position> as WorldQuery>::init_state(&world);
    assert!(<Changed<Position> as WorldQuery>::fetch(&state_c, &world, entity).is_some());

    // System boundary at tick 2
    world.set_last_change_tick(world.change_tick());

    // Tick 3: no mutation
    world.increment_change_tick();

    assert!(
        <Changed<Position> as WorldQuery>::fetch(&state_c, &world, entity).is_none(),
        "No mutation in tick 3 should mean Changed is false"
    );
}
