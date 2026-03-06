//! Tests for EntityAllocator bulk operations: allocate_batch, deallocate_batch, reserve.

use crate::ecs::entity::{Entity, EntityAllocator};
use std::collections::HashSet;

// =========================================================================
// allocate_batch Tests
// =========================================================================

#[test]
fn test_allocate_batch_empty() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(0);
    assert!(entities.is_empty());
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0);
}

#[test]
fn test_allocate_batch_basic() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(100);
    assert_eq!(entities.len(), 100);
    assert_eq!(allocator.len(), 100);
    assert_eq!(allocator.capacity(), 100);

    // All entities should be unique
    let unique: HashSet<_> = entities.iter().collect();
    assert_eq!(unique.len(), 100);

    // All entities should be alive
    for entity in &entities {
        assert!(allocator.is_alive(*entity));
        assert!(!entity.is_placeholder());
    }

    // All should have generation 1 (first allocation)
    for entity in &entities {
        assert_eq!(entity.generation(), 1);
    }
}

#[test]
fn test_allocate_batch_reuses_free_slots() {
    let mut allocator = EntityAllocator::new();

    // Allocate 50 entities, then deallocate them all
    let first_batch = allocator.allocate_batch(50);
    assert_eq!(allocator.len(), 50);

    for entity in &first_batch {
        allocator.deallocate(*entity);
    }
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 50);

    // Allocate 50 more - should reuse all freed slots
    let second_batch = allocator.allocate_batch(50);
    assert_eq!(allocator.len(), 50);
    assert_eq!(allocator.capacity(), 50); // No new slots created

    // All should have generation 2 (recycled)
    for entity in &second_batch {
        assert_eq!(entity.generation(), 2);
    }

    // All original entities should be dead
    for entity in &first_batch {
        assert!(!allocator.is_alive(*entity));
    }

    // All new entities should be alive
    for entity in &second_batch {
        assert!(allocator.is_alive(*entity));
    }
}

#[test]
fn test_allocate_batch_mixed_reuse_and_new() {
    let mut allocator = EntityAllocator::new();

    // Allocate 30, deallocate all
    let first = allocator.allocate_batch(30);
    for e in &first {
        allocator.deallocate(*e);
    }
    assert_eq!(allocator.capacity(), 30);

    // Now allocate 50 - should reuse 30, create 20 new
    let second = allocator.allocate_batch(50);
    assert_eq!(second.len(), 50);
    assert_eq!(allocator.len(), 50);
    assert_eq!(allocator.capacity(), 50); // Grew by 20

    // Count generations
    let gen1_count = second.iter().filter(|e| e.generation() == 1).count();
    let gen2_count = second.iter().filter(|e| e.generation() == 2).count();

    assert_eq!(gen1_count, 20); // New slots
    assert_eq!(gen2_count, 30); // Recycled slots
}

#[test]
fn test_allocate_batch_large() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(10_000);
    assert_eq!(entities.len(), 10_000);
    assert_eq!(allocator.len(), 10_000);

    // All should be unique
    let unique: HashSet<_> = entities.iter().collect();
    assert_eq!(unique.len(), 10_000);

    // All should be alive
    for entity in &entities {
        assert!(allocator.is_alive(*entity));
    }
}

// =========================================================================
// deallocate_batch Tests
// =========================================================================

#[test]
fn test_deallocate_batch_empty() {
    let mut allocator = EntityAllocator::new();

    let deallocated = allocator.deallocate_batch(&[]);
    assert_eq!(deallocated, 0);
}

#[test]
fn test_deallocate_batch_basic() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(100);
    assert_eq!(allocator.len(), 100);

    let deallocated = allocator.deallocate_batch(&entities);
    assert_eq!(deallocated, 100);
    assert_eq!(allocator.len(), 0);

    // All should be dead
    for entity in &entities {
        assert!(!allocator.is_alive(*entity));
    }
}

#[test]
fn test_deallocate_batch_partial_invalid() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(10);

    // Deallocate some individually first
    allocator.deallocate(entities[0]);
    allocator.deallocate(entities[2]);
    allocator.deallocate(entities[4]);

    // Now batch deallocate all - should only succeed for 7
    let deallocated = allocator.deallocate_batch(&entities);
    assert_eq!(deallocated, 7); // 10 - 3 already deallocated

    // All should be dead now
    for entity in &entities {
        assert!(!allocator.is_alive(*entity));
    }
}

#[test]
fn test_deallocate_batch_all_invalid() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(10);

    // Deallocate all individually
    for entity in &entities {
        allocator.deallocate(*entity);
    }

    // Batch deallocate should return 0
    let deallocated = allocator.deallocate_batch(&entities);
    assert_eq!(deallocated, 0);
}

#[test]
fn test_deallocate_batch_with_placeholder() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(5);
    let mut with_placeholder: Vec<Entity> = entities.clone();
    with_placeholder.push(Entity::PLACEHOLDER);
    with_placeholder.push(Entity::PLACEHOLDER);

    let deallocated = allocator.deallocate_batch(&with_placeholder);
    assert_eq!(deallocated, 5); // Only the 5 valid entities

    assert!(allocator.is_empty());
}

#[test]
fn test_deallocate_batch_with_out_of_bounds() {
    let mut allocator = EntityAllocator::new();

    let entities = allocator.allocate_batch(5);
    let mut with_invalid: Vec<Entity> = entities.clone();
    with_invalid.push(Entity::new(9999, 1)); // Out of bounds
    with_invalid.push(Entity::new(10000, 1)); // Out of bounds

    let deallocated = allocator.deallocate_batch(&with_invalid);
    assert_eq!(deallocated, 5); // Only the 5 valid entities
}

// =========================================================================
// reserve Tests
// =========================================================================

#[test]
fn test_reserve_basic() {
    let mut allocator = EntityAllocator::new();

    allocator.reserve(1000);

    // No entities allocated, but memory reserved
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0);

    // Now allocate - no reallocation should occur
    let entities = allocator.allocate_batch(500);
    assert_eq!(entities.len(), 500);
    assert_eq!(allocator.len(), 500);
}

#[test]
fn test_reserve_after_allocations() {
    let mut allocator = EntityAllocator::new();

    // Allocate some entities first
    let _first = allocator.allocate_batch(100);
    assert_eq!(allocator.capacity(), 100);

    // Reserve more
    allocator.reserve(1000);

    // Allocate many more
    let second = allocator.allocate_batch(1000);
    assert_eq!(second.len(), 1000);
    assert_eq!(allocator.len(), 1100);
}

#[test]
fn test_reserve_zero() {
    let mut allocator = EntityAllocator::new();

    allocator.reserve(0);
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0);
}

// =========================================================================
// Batch Stress and Equivalence Tests
// =========================================================================

#[test]
fn test_batch_stress_test() {
    let mut allocator = EntityAllocator::new();
    const BATCH_SIZE: usize = 1000;
    const ITERATIONS: usize = 100;

    for _ in 0..ITERATIONS {
        // Allocate batch
        let entities = allocator.allocate_batch(BATCH_SIZE);
        assert_eq!(entities.len(), BATCH_SIZE);

        // Verify all alive
        for entity in &entities {
            assert!(allocator.is_alive(*entity));
        }

        // Deallocate batch
        let deallocated = allocator.deallocate_batch(&entities);
        assert_eq!(deallocated, BATCH_SIZE);

        // Verify all dead
        for entity in &entities {
            assert!(!allocator.is_alive(*entity));
        }
    }

    // After all iterations, should be empty
    assert!(allocator.is_empty());

    // Capacity should be exactly BATCH_SIZE (slots reused each iteration)
    assert_eq!(allocator.capacity(), BATCH_SIZE);
}

#[test]
fn test_batch_vs_individual_equivalence() {
    // Verify that batch operations produce equivalent results to individual ops

    let mut batch_allocator = EntityAllocator::new();
    let mut individual_allocator = EntityAllocator::new();

    // Allocate same count via batch vs individual
    let batch_entities = batch_allocator.allocate_batch(100);
    let individual_entities: Vec<_> = (0..100).map(|_| individual_allocator.allocate()).collect();

    assert_eq!(batch_allocator.len(), individual_allocator.len());
    assert_eq!(batch_allocator.capacity(), individual_allocator.capacity());

    // Same number of unique entities
    assert_eq!(batch_entities.len(), individual_entities.len());

    // All entities should match in structure (index 0-99, generation 1)
    for i in 0..100 {
        // Since both allocate sequentially with no free slots, indices match
        assert_eq!(batch_entities[i].index() as usize, i);
        assert_eq!(individual_entities[i].index() as usize, i);
        assert_eq!(batch_entities[i].generation(), 1);
        assert_eq!(individual_entities[i].generation(), 1);
    }

    // Deallocate via batch vs individual
    let batch_count = batch_allocator.deallocate_batch(&batch_entities);
    let individual_count = individual_entities
        .iter()
        .filter(|e| individual_allocator.deallocate(**e))
        .count();

    assert_eq!(batch_count, individual_count);
    assert_eq!(batch_allocator.len(), individual_allocator.len());
}
