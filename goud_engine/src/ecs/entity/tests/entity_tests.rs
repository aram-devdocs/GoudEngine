//! Tests for the Entity type.

use crate::ecs::entity::{Entity, EntityAllocator};
use std::collections::HashSet;

// -------------------------------------------------------------------------
// Structure Tests
// -------------------------------------------------------------------------

#[test]
fn test_entity_new() {
    let entity = Entity::new(42, 7);
    assert_eq!(entity.index(), 42);
    assert_eq!(entity.generation(), 7);
}

#[test]
fn test_entity_placeholder() {
    let placeholder = Entity::PLACEHOLDER;
    assert_eq!(placeholder.index(), u32::MAX);
    assert_eq!(placeholder.generation(), 0);
    assert!(placeholder.is_placeholder());

    // Non-placeholder entities
    let entity = Entity::new(0, 0);
    assert!(!entity.is_placeholder());

    let entity = Entity::new(u32::MAX, 1);
    assert!(!entity.is_placeholder());

    let entity = Entity::new(0, 1);
    assert!(!entity.is_placeholder());
}

#[test]
fn test_entity_size() {
    // Entity should be exactly 8 bytes (2 x u32)
    assert_eq!(std::mem::size_of::<Entity>(), 8);
    assert_eq!(std::mem::align_of::<Entity>(), 4);
}

#[test]
fn test_entity_copy_clone() {
    let entity1 = Entity::new(10, 5);
    let entity2 = entity1; // Copy
    let entity3 = entity1.clone(); // Clone

    assert_eq!(entity1, entity2);
    assert_eq!(entity1, entity3);
}

#[test]
fn test_entity_equality() {
    let e1 = Entity::new(0, 1);
    let e2 = Entity::new(0, 1);
    let e3 = Entity::new(0, 2); // Different generation
    let e4 = Entity::new(1, 1); // Different index

    assert_eq!(e1, e2);
    assert_ne!(e1, e3);
    assert_ne!(e1, e4);
}

// -------------------------------------------------------------------------
// Bit Packing Tests
// -------------------------------------------------------------------------

#[test]
fn test_entity_to_bits() {
    let entity = Entity::new(42, 7);
    let bits = entity.to_bits();

    // Upper 32 bits: generation (7), Lower 32 bits: index (42)
    assert_eq!(bits, (7_u64 << 32) | 42);
}

#[test]
fn test_entity_from_bits() {
    let bits = (3_u64 << 32) | 100;
    let entity = Entity::from_bits(bits);

    assert_eq!(entity.index(), 100);
    assert_eq!(entity.generation(), 3);
}

#[test]
fn test_entity_bits_roundtrip() {
    let original = Entity::new(999, 42);
    let bits = original.to_bits();
    let restored = Entity::from_bits(bits);

    assert_eq!(original, restored);
}

#[test]
fn test_entity_bits_edge_cases() {
    // Maximum values
    let max = Entity::new(u32::MAX, u32::MAX);
    assert_eq!(max, Entity::from_bits(max.to_bits()));

    // Zero values
    let zero = Entity::new(0, 0);
    assert_eq!(zero, Entity::from_bits(zero.to_bits()));

    // Mixed
    let mixed = Entity::new(u32::MAX, 0);
    assert_eq!(mixed, Entity::from_bits(mixed.to_bits()));
}

// -------------------------------------------------------------------------
// Trait Implementation Tests
// -------------------------------------------------------------------------

#[test]
fn test_entity_hash() {
    let mut set = HashSet::new();

    let e1 = Entity::new(0, 1);
    let e2 = Entity::new(0, 1);
    let e3 = Entity::new(0, 2);
    let e4 = Entity::new(1, 1);

    set.insert(e1);

    // Same entity should be found
    assert!(set.contains(&e2));

    // Different entities should not be found
    assert!(!set.contains(&e3));
    assert!(!set.contains(&e4));
}

#[test]
fn test_entity_debug_format() {
    let entity = Entity::new(42, 3);
    assert_eq!(format!("{:?}", entity), "Entity(42:3)");

    let placeholder = Entity::PLACEHOLDER;
    assert_eq!(format!("{:?}", placeholder), "Entity(4294967295:0)");
}

#[test]
fn test_entity_display_format() {
    let entity = Entity::new(100, 7);
    assert_eq!(format!("{}", entity), "Entity(100:7)");
}

#[test]
fn test_entity_default() {
    let default_entity: Entity = Default::default();
    assert_eq!(default_entity, Entity::PLACEHOLDER);
    assert!(default_entity.is_placeholder());
}

// -------------------------------------------------------------------------
// Conversion Tests
// -------------------------------------------------------------------------

#[test]
fn test_entity_from_into_u64() {
    let entity = Entity::new(42, 7);

    // Into<u64>
    let bits: u64 = entity.into();
    assert_eq!(bits, entity.to_bits());

    // From<u64>
    let restored: Entity = bits.into();
    assert_eq!(restored, entity);
}

// -------------------------------------------------------------------------
// Thread Safety Tests
// -------------------------------------------------------------------------

#[test]
fn test_entity_send_sync() {
    // Compile-time check that Entity is Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Entity>();
}

// -------------------------------------------------------------------------
// EntityAllocator basic tests
// -------------------------------------------------------------------------

#[test]
fn test_allocator_new() {
    let allocator = EntityAllocator::new();
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0);
    assert!(allocator.is_empty());
}

#[test]
fn test_allocator_with_capacity() {
    let allocator = EntityAllocator::with_capacity(100);
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0); // capacity() returns slots used, not reserved
    assert!(allocator.is_empty());
}

#[test]
fn test_allocator_default() {
    let allocator: EntityAllocator = Default::default();
    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
}

#[test]
fn test_allocator_debug() {
    let mut allocator = EntityAllocator::new();
    let _e1 = allocator.allocate();
    let e2 = allocator.allocate();
    allocator.deallocate(e2);

    let debug_str = format!("{:?}", allocator);
    assert!(debug_str.contains("EntityAllocator"));
    assert!(debug_str.contains("len"));
    assert!(debug_str.contains("capacity"));
    assert!(debug_str.contains("free_slots"));
}

#[test]
fn test_allocator_allocate_basic() {
    let mut allocator = EntityAllocator::new();

    let e1 = allocator.allocate();
    assert_eq!(e1.index(), 0);
    assert_eq!(e1.generation(), 1);
    assert!(!e1.is_placeholder());

    let e2 = allocator.allocate();
    assert_eq!(e2.index(), 1);
    assert_eq!(e2.generation(), 1);

    assert_ne!(e1, e2);
    assert_eq!(allocator.len(), 2);
}

#[test]
fn test_allocator_allocate_multiple() {
    let mut allocator = EntityAllocator::new();
    let mut entities = Vec::new();

    for _ in 0..100 {
        entities.push(allocator.allocate());
    }

    assert_eq!(allocator.len(), 100);
    assert_eq!(allocator.capacity(), 100);

    // All entities should be unique
    let unique: HashSet<_> = entities.iter().collect();
    assert_eq!(unique.len(), 100);

    // All entities should be alive
    for entity in &entities {
        assert!(allocator.is_alive(*entity));
    }
}

#[test]
fn test_allocator_first_generation_is_one() {
    let mut allocator = EntityAllocator::new();

    // All newly allocated entities should have generation 1
    for _ in 0..10 {
        let entity = allocator.allocate();
        assert_eq!(entity.generation(), 1);
    }
}
