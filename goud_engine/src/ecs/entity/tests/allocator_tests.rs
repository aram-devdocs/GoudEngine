//! Tests for EntityAllocator deallocation, recycling, sizing, and stress behaviour.

use crate::ecs::entity::{Entity, EntityAllocator};
use std::collections::HashSet;

// -------------------------------------------------------------------------
// Deallocation Tests
// -------------------------------------------------------------------------

#[test]
fn test_allocator_deallocate_basic() {
    let mut allocator = EntityAllocator::new();
    let entity = allocator.allocate();

    assert!(allocator.is_alive(entity));
    assert!(allocator.deallocate(entity));
    assert!(!allocator.is_alive(entity));
}

#[test]
fn test_allocator_deallocate_returns_false_for_dead_entity() {
    let mut allocator = EntityAllocator::new();
    let entity = allocator.allocate();

    // First deallocation succeeds
    assert!(allocator.deallocate(entity));

    // Second deallocation fails
    assert!(!allocator.deallocate(entity));
}

#[test]
fn test_allocator_deallocate_returns_false_for_placeholder() {
    let mut allocator = EntityAllocator::new();

    // PLACEHOLDER cannot be deallocated
    assert!(!allocator.deallocate(Entity::PLACEHOLDER));
}

#[test]
fn test_allocator_deallocate_returns_false_for_out_of_bounds() {
    let mut allocator = EntityAllocator::new();
    allocator.allocate(); // Allocate slot 0

    // Entity with index 999 is out of bounds
    let fake_entity = Entity::new(999, 1);
    assert!(!allocator.deallocate(fake_entity));
}

#[test]
fn test_allocator_deallocate_returns_false_for_wrong_generation() {
    let mut allocator = EntityAllocator::new();
    let entity = allocator.allocate();

    // Create a fake entity with same index but different generation
    let fake_entity = Entity::new(entity.index(), entity.generation() + 1);
    assert!(!allocator.deallocate(fake_entity));

    // Original entity is still alive
    assert!(allocator.is_alive(entity));
}

// -------------------------------------------------------------------------
// is_alive Tests
// -------------------------------------------------------------------------

#[test]
fn test_allocator_is_alive() {
    let mut allocator = EntityAllocator::new();
    let entity = allocator.allocate();

    assert!(allocator.is_alive(entity));

    allocator.deallocate(entity);
    assert!(!allocator.is_alive(entity));
}

#[test]
fn test_allocator_is_alive_placeholder() {
    let allocator = EntityAllocator::new();
    assert!(!allocator.is_alive(Entity::PLACEHOLDER));
}

#[test]
fn test_allocator_is_alive_out_of_bounds() {
    let allocator = EntityAllocator::new();
    let fake_entity = Entity::new(999, 1);
    assert!(!allocator.is_alive(fake_entity));
}

#[test]
fn test_allocator_is_alive_wrong_generation() {
    let mut allocator = EntityAllocator::new();
    let entity = allocator.allocate();

    // Wrong generation
    let stale = Entity::new(entity.index(), entity.generation() + 1);
    assert!(!allocator.is_alive(stale));
}

// -------------------------------------------------------------------------
// Slot Recycling Tests
// -------------------------------------------------------------------------

#[test]
fn test_allocator_recycling_basic() {
    let mut allocator = EntityAllocator::new();

    // Allocate and deallocate
    let e1 = allocator.allocate();
    assert!(allocator.deallocate(e1));

    // Allocate again - should reuse the slot
    let e2 = allocator.allocate();

    // Same index, different generation
    assert_eq!(e1.index(), e2.index());
    assert_ne!(e1.generation(), e2.generation());
    assert_eq!(e2.generation(), 2); // Generation incremented

    // e1 is dead, e2 is alive
    assert!(!allocator.is_alive(e1));
    assert!(allocator.is_alive(e2));
}

#[test]
fn test_allocator_recycling_multiple() {
    let mut allocator = EntityAllocator::new();

    // Allocate 5 entities
    let entities: Vec<_> = (0..5).map(|_| allocator.allocate()).collect();
    assert_eq!(allocator.len(), 5);
    assert_eq!(allocator.capacity(), 5);

    // Deallocate all
    for entity in &entities {
        assert!(allocator.deallocate(*entity));
    }
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 5); // Capacity unchanged

    // Allocate 5 more - should reuse slots
    let new_entities: Vec<_> = (0..5).map(|_| allocator.allocate()).collect();
    assert_eq!(allocator.len(), 5);
    assert_eq!(allocator.capacity(), 5); // Still 5, no new slots created

    // All new entities should have generation 2
    for entity in &new_entities {
        assert_eq!(entity.generation(), 2);
    }

    // Old entities are dead, new ones are alive
    for entity in &entities {
        assert!(!allocator.is_alive(*entity));
    }
    for entity in &new_entities {
        assert!(allocator.is_alive(*entity));
    }
}

#[test]
fn test_allocator_recycling_lifo_order() {
    let mut allocator = EntityAllocator::new();

    // Allocate 3 entities
    let e0 = allocator.allocate();
    let e1 = allocator.allocate();
    let e2 = allocator.allocate();

    // Deallocate in order: e0, e1, e2
    allocator.deallocate(e0);
    allocator.deallocate(e1);
    allocator.deallocate(e2);

    // Reallocate - should come back in reverse order (LIFO)
    let new_e2 = allocator.allocate(); // Should reuse e2's slot
    let new_e1 = allocator.allocate(); // Should reuse e1's slot
    let new_e0 = allocator.allocate(); // Should reuse e0's slot

    assert_eq!(new_e2.index(), e2.index());
    assert_eq!(new_e1.index(), e1.index());
    assert_eq!(new_e0.index(), e0.index());
}

#[test]
fn test_allocator_generation_increment() {
    let mut allocator = EntityAllocator::new();

    // Allocate and deallocate the same slot multiple times
    let mut last_gen = 0;
    for expected_gen in 1..=10 {
        let entity = allocator.allocate();
        assert_eq!(entity.index(), 0); // Always slot 0
        assert_eq!(entity.generation(), expected_gen);
        assert!(entity.generation() > last_gen);
        last_gen = entity.generation();

        allocator.deallocate(entity);
    }
}

// -------------------------------------------------------------------------
// len(), capacity(), is_empty() Tests
// -------------------------------------------------------------------------

#[test]
fn test_allocator_len() {
    let mut allocator = EntityAllocator::new();
    assert_eq!(allocator.len(), 0);

    let e1 = allocator.allocate();
    assert_eq!(allocator.len(), 1);

    let e2 = allocator.allocate();
    assert_eq!(allocator.len(), 2);

    allocator.deallocate(e1);
    assert_eq!(allocator.len(), 1);

    allocator.deallocate(e2);
    assert_eq!(allocator.len(), 0);
}

#[test]
fn test_allocator_capacity() {
    let mut allocator = EntityAllocator::new();
    assert_eq!(allocator.capacity(), 0);

    let e1 = allocator.allocate();
    assert_eq!(allocator.capacity(), 1);

    let e2 = allocator.allocate();
    assert_eq!(allocator.capacity(), 2);

    // Capacity doesn't decrease on deallocation
    allocator.deallocate(e1);
    assert_eq!(allocator.capacity(), 2);

    allocator.deallocate(e2);
    assert_eq!(allocator.capacity(), 2);

    // Reusing slots doesn't increase capacity
    allocator.allocate();
    allocator.allocate();
    assert_eq!(allocator.capacity(), 2);

    // New allocation beyond capacity increases it
    allocator.allocate();
    assert_eq!(allocator.capacity(), 3);
}

#[test]
fn test_allocator_is_empty() {
    let mut allocator = EntityAllocator::new();
    assert!(allocator.is_empty());

    let entity = allocator.allocate();
    assert!(!allocator.is_empty());

    allocator.deallocate(entity);
    assert!(allocator.is_empty());
}

// -------------------------------------------------------------------------
// Stress Tests
// -------------------------------------------------------------------------

#[test]
fn test_allocator_many_allocations() {
    let mut allocator = EntityAllocator::new();
    const COUNT: usize = 10_000;

    // Allocate many entities
    let entities: Vec<_> = (0..COUNT).map(|_| allocator.allocate()).collect();
    assert_eq!(allocator.len(), COUNT);

    // All should be alive
    for entity in &entities {
        assert!(allocator.is_alive(*entity));
    }

    // Deallocate half
    for entity in entities.iter().take(COUNT / 2) {
        allocator.deallocate(*entity);
    }
    assert_eq!(allocator.len(), COUNT / 2);

    // Deallocated ones are dead
    for entity in entities.iter().take(COUNT / 2) {
        assert!(!allocator.is_alive(*entity));
    }

    // Remaining are alive
    for entity in entities.iter().skip(COUNT / 2) {
        assert!(allocator.is_alive(*entity));
    }
}

#[test]
fn test_allocator_stress_allocate_deallocate_cycle() {
    let mut allocator = EntityAllocator::new();
    const CYCLES: usize = 100;
    const ENTITIES_PER_CYCLE: usize = 100;

    for _ in 0..CYCLES {
        // Allocate
        let entities: Vec<_> = (0..ENTITIES_PER_CYCLE)
            .map(|_| allocator.allocate())
            .collect();

        // Verify all alive
        for entity in &entities {
            assert!(allocator.is_alive(*entity));
        }

        // Deallocate all
        for entity in &entities {
            assert!(allocator.deallocate(*entity));
        }

        // Verify all dead
        for entity in &entities {
            assert!(!allocator.is_alive(*entity));
        }
    }

    // After all cycles, should be empty but have capacity
    assert!(allocator.is_empty());
    assert_eq!(allocator.capacity(), ENTITIES_PER_CYCLE);
}

#[test]
fn test_allocator_unique_entities() {
    let mut allocator = EntityAllocator::new();
    let mut seen = HashSet::new();

    // Allocate, deallocate, and reallocate many times
    for _ in 0..1000 {
        let entity = allocator.allocate();

        // Each entity should be unique (index + generation combination)
        let key = entity.to_bits();
        assert!(
            seen.insert(key),
            "Duplicate entity: {:?} (bits: {})",
            entity,
            key
        );

        // 50% chance of deallocating
        if seen.len() % 2 == 0 {
            allocator.deallocate(entity);
        }
    }
}
