//! ECS integration tests using the Rust ECS API directly.
//!
//! These tests exercise entity lifecycle, component operations, and bulk
//! operations through the public `goud_engine::ecs` API without going
//! through the FFI layer.

use goud_engine::ecs::{Component, Entity, World};

// ---------------------------------------------------------------------------
// Test-local component types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}
impl Component for Velocity {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Health(i32);
impl Component for Health {}

// ===========================================================================
// Entity Lifecycle Tests
// ===========================================================================

#[test]
fn test_spawn_entity_and_verify_alive() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    assert!(
        world.is_alive(entity),
        "Freshly spawned entity should be alive"
    );
    assert_eq!(
        world.entity_count(),
        1,
        "World should contain exactly 1 entity after spawning"
    );
}

#[test]
fn test_despawn_entity_and_verify_dead() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let despawned = world.despawn(entity);
    assert!(despawned, "Despawn of a live entity should return true");
    assert!(
        !world.is_alive(entity),
        "Entity should be dead after despawn"
    );
    assert_eq!(
        world.entity_count(),
        0,
        "World should be empty after despawning only entity"
    );
}

#[test]
fn test_despawn_already_dead_entity_returns_false() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.despawn(entity);

    let second_despawn = world.despawn(entity);
    assert!(
        !second_despawn,
        "Despawning an already-dead entity should return false"
    );
}

#[test]
fn test_is_alive_on_never_spawned_entity() {
    let world = World::new();
    let fabricated = Entity::new(999, 1);

    assert!(
        !world.is_alive(fabricated),
        "A fabricated entity that was never spawned should not be alive"
    );
}

#[test]
fn test_placeholder_entity_is_not_alive() {
    let world = World::new();
    assert!(
        !world.is_alive(Entity::PLACEHOLDER),
        "PLACEHOLDER entity should never be alive"
    );
}

#[test]
fn test_spawn_multiple_entities_unique_ids() {
    let mut world = World::new();
    let e1 = world.spawn_empty();
    let e2 = world.spawn_empty();
    let e3 = world.spawn_empty();

    assert_ne!(e1, e2, "Spawned entities should have unique IDs");
    assert_ne!(e2, e3, "Spawned entities should have unique IDs");
    assert_ne!(e1, e3, "Spawned entities should have unique IDs");
    assert_eq!(
        world.entity_count(),
        3,
        "World should contain 3 entities after 3 spawns"
    );
}

// ===========================================================================
// Component Add / Remove / Query Tests
// ===========================================================================

#[test]
fn test_insert_and_get_component() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let old = world.insert(entity, Position { x: 10.0, y: 20.0 });
    assert!(
        old.is_none(),
        "First insert should return None (no previous value)"
    );

    let pos = world.get::<Position>(entity);
    assert_eq!(
        pos,
        Some(&Position { x: 10.0, y: 20.0 }),
        "get() should return the inserted position"
    );
}

#[test]
fn test_insert_replaces_existing_component() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    world.insert(entity, Health(100));
    let old = world.insert(entity, Health(50));

    assert_eq!(
        old,
        Some(Health(100)),
        "Re-inserting should return the previous value"
    );
    assert_eq!(
        world.get::<Health>(entity),
        Some(&Health(50)),
        "Component should hold the new value"
    );
}

#[test]
fn test_has_component() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    assert!(
        !world.has::<Position>(entity),
        "Entity without Position should report has::<Position>() == false"
    );

    world.insert(entity, Position { x: 0.0, y: 0.0 });
    assert!(
        world.has::<Position>(entity),
        "Entity with Position should report has::<Position>() == true"
    );
}

#[test]
fn test_remove_component() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Velocity { dx: 1.0, dy: -1.0 });

    let removed = world.remove::<Velocity>(entity);
    assert_eq!(
        removed,
        Some(Velocity { dx: 1.0, dy: -1.0 }),
        "remove() should return the removed component"
    );
    assert!(
        !world.has::<Velocity>(entity),
        "Entity should no longer have Velocity after removal"
    );
}

#[test]
fn test_remove_nonexistent_component_returns_none() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let removed = world.remove::<Health>(entity);
    assert!(
        removed.is_none(),
        "Removing a component the entity never had should return None"
    );
}

#[test]
fn test_get_component_on_dead_entity_returns_none() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 5.0, y: 5.0 });
    world.despawn(entity);

    let pos = world.get::<Position>(entity);
    assert!(
        pos.is_none(),
        "get() on a despawned entity should return None"
    );
}

#[test]
fn test_multiple_components_on_same_entity() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    world.insert(entity, Position { x: 1.0, y: 2.0 });
    world.insert(entity, Velocity { dx: 3.0, dy: 4.0 });
    world.insert(entity, Health(100));

    assert_eq!(
        world.get::<Position>(entity),
        Some(&Position { x: 1.0, y: 2.0 }),
        "Position component should be retrievable alongside other components"
    );
    assert_eq!(
        world.get::<Velocity>(entity),
        Some(&Velocity { dx: 3.0, dy: 4.0 }),
        "Velocity component should be retrievable alongside other components"
    );
    assert_eq!(
        world.get::<Health>(entity),
        Some(&Health(100)),
        "Health component should be retrievable alongside other components"
    );
}

#[test]
fn test_get_mut_updates_component() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Health(100));

    if let Some(health) = world.get_mut::<Health>(entity) {
        health.0 -= 25;
    }

    assert_eq!(
        world.get::<Health>(entity),
        Some(&Health(75)),
        "get_mut() modifications should be visible through get()"
    );
}

#[test]
fn test_spawn_builder_with_insert() {
    let mut world = World::new();
    let entity = world.spawn().insert(Position { x: 42.0, y: 99.0 }).id();

    assert!(
        world.is_alive(entity),
        "Builder-spawned entity should be alive"
    );
    assert_eq!(
        world.get::<Position>(entity),
        Some(&Position { x: 42.0, y: 99.0 }),
        "Builder-inserted component should be retrievable"
    );
}

// ===========================================================================
// Bulk Operation Tests
// ===========================================================================

#[test]
fn test_spawn_batch_creates_correct_count() {
    let mut world = World::new();
    let entities = world.spawn_batch(50);

    assert_eq!(
        entities.len(),
        50,
        "spawn_batch(50) should return 50 entities"
    );
    assert_eq!(
        world.entity_count(),
        50,
        "World should contain 50 entities after batch spawn"
    );

    for (i, entity) in entities.iter().enumerate() {
        assert!(
            world.is_alive(*entity),
            "Batch-spawned entity at index {} should be alive",
            i
        );
    }
}

#[test]
fn test_spawn_batch_zero_returns_empty() {
    let mut world = World::new();
    let entities = world.spawn_batch(0);

    assert!(
        entities.is_empty(),
        "spawn_batch(0) should return an empty vec"
    );
    assert_eq!(
        world.entity_count(),
        0,
        "World should remain empty after spawn_batch(0)"
    );
}

#[test]
fn test_despawn_batch() {
    let mut world = World::new();
    let entities = world.spawn_batch(10);

    let despawned = world.despawn_batch(&entities[..5]);
    assert_eq!(despawned, 5, "despawn_batch of 5 entities should return 5");
    assert_eq!(
        world.entity_count(),
        5,
        "World should have 5 entities remaining after despawning 5 of 10"
    );

    // First 5 should be dead, last 5 alive
    for (i, entity) in entities.iter().enumerate() {
        if i < 5 {
            assert!(
                !world.is_alive(*entity),
                "Entity at index {} should be dead after batch despawn",
                i
            );
        } else {
            assert!(
                world.is_alive(*entity),
                "Entity at index {} should still be alive",
                i
            );
        }
    }
}

#[test]
fn test_insert_batch_components() {
    let mut world = World::new();
    let entities = world.spawn_batch(3);

    let batch = vec![
        (entities[0], Health(100)),
        (entities[1], Health(80)),
        (entities[2], Health(60)),
    ];

    let count = world.insert_batch(batch);
    assert_eq!(
        count, 3,
        "insert_batch should successfully insert 3 components"
    );

    assert_eq!(
        world.get::<Health>(entities[0]),
        Some(&Health(100)),
        "First entity should have Health(100)"
    );
    assert_eq!(
        world.get::<Health>(entities[1]),
        Some(&Health(80)),
        "Second entity should have Health(80)"
    );
    assert_eq!(
        world.get::<Health>(entities[2]),
        Some(&Health(60)),
        "Third entity should have Health(60)"
    );
}

#[test]
fn test_component_type_count_tracks_registrations() {
    let mut world = World::new();
    assert_eq!(
        world.component_type_count(),
        0,
        "New world should have 0 component types"
    );

    let entity = world.spawn_empty();
    world.insert(entity, Position { x: 0.0, y: 0.0 });
    assert_eq!(
        world.component_type_count(),
        1,
        "After inserting Position, component type count should be 1"
    );

    world.insert(entity, Velocity { dx: 0.0, dy: 0.0 });
    assert_eq!(
        world.component_type_count(),
        2,
        "After inserting Velocity, component type count should be 2"
    );

    // Inserting same type on another entity should not increase count
    let entity2 = world.spawn_empty();
    world.insert(entity2, Position { x: 1.0, y: 1.0 });
    assert_eq!(
        world.component_type_count(),
        2,
        "Re-using Position on another entity should not increase type count"
    );
}
