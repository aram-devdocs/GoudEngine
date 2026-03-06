//! Tests for entity spawn, despawn, query, and integration scenarios.

use crate::ffi::context::{goud_context_create, goud_context_destroy};
use crate::ffi::entity::{
    lifecycle::{
        goud_entity_despawn, goud_entity_despawn_batch, goud_entity_spawn_batch,
        goud_entity_spawn_empty,
    },
    queries::{goud_entity_count, goud_entity_is_alive},
    GOUD_INVALID_ENTITY_ID,
};
use crate::ffi::GOUD_INVALID_CONTEXT_ID;

// ============================================================================
// Entity Spawn Tests
// ============================================================================

#[test]
fn test_spawn_empty_basic() {
    let ctx = goud_context_create();
    assert_ne!(ctx, GOUD_INVALID_CONTEXT_ID);

    let entity = goud_entity_spawn_empty(ctx);
    assert_ne!(entity, GOUD_INVALID_ENTITY_ID);
    assert!(goud_entity_is_alive(ctx, entity));

    goud_context_destroy(ctx);
}

#[test]
fn test_spawn_empty_invalid_context() {
    let entity = goud_entity_spawn_empty(GOUD_INVALID_CONTEXT_ID);
    assert_eq!(entity, GOUD_INVALID_ENTITY_ID);
}

#[test]
fn test_spawn_empty_multiple() {
    let ctx = goud_context_create();

    let e1 = goud_entity_spawn_empty(ctx);
    let e2 = goud_entity_spawn_empty(ctx);
    let e3 = goud_entity_spawn_empty(ctx);

    assert_ne!(e1, e2);
    assert_ne!(e2, e3);
    assert_ne!(e1, e3);

    assert!(goud_entity_is_alive(ctx, e1));
    assert!(goud_entity_is_alive(ctx, e2));
    assert!(goud_entity_is_alive(ctx, e3));

    goud_context_destroy(ctx);
}

#[test]
fn test_spawn_batch_basic() {
    let ctx = goud_context_create();
    let mut entities = vec![0u64; 10];

    // SAFETY: entities has capacity for 10 u64 values.
    let count = unsafe { goud_entity_spawn_batch(ctx, 10, entities.as_mut_ptr()) };

    assert_eq!(count, 10);
    for entity in &entities {
        assert_ne!(*entity, GOUD_INVALID_ENTITY_ID);
        assert!(goud_entity_is_alive(ctx, *entity));
    }

    goud_context_destroy(ctx);
}

#[test]
fn test_spawn_batch_zero_count() {
    let ctx = goud_context_create();
    let mut entities = vec![0u64; 1];

    // SAFETY: entities has capacity for 1 u64 value; count 0 means no writes occur.
    let count = unsafe { goud_entity_spawn_batch(ctx, 0, entities.as_mut_ptr()) };

    assert_eq!(count, 0);
    goud_context_destroy(ctx);
}

#[test]
fn test_spawn_batch_invalid_context() {
    let mut entities = vec![0u64; 10];

    // SAFETY: entities has capacity for 10 u64 values; the function handles invalid context gracefully.
    let count =
        unsafe { goud_entity_spawn_batch(GOUD_INVALID_CONTEXT_ID, 10, entities.as_mut_ptr()) };

    assert_eq!(count, 0);
}

#[test]
fn test_spawn_batch_null_pointer() {
    let ctx = goud_context_create();

    // SAFETY: Passing null pointer tests that the function handles null gracefully.
    let count = unsafe { goud_entity_spawn_batch(ctx, 10, std::ptr::null_mut()) };

    assert_eq!(count, 0);
    goud_context_destroy(ctx);
}

#[test]
fn test_spawn_batch_large() {
    let ctx = goud_context_create();
    let mut entities = vec![0u64; 1000];

    // SAFETY: entities has capacity for 1000 u64 values.
    let count = unsafe { goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr()) };

    assert_eq!(count, 1000);

    // Verify all are unique
    let unique: std::collections::HashSet<_> = entities.iter().copied().collect();
    assert_eq!(unique.len(), 1000);

    goud_context_destroy(ctx);
}

// ============================================================================
// Entity Despawn Tests
// ============================================================================

#[test]
fn test_despawn_basic() {
    let ctx = goud_context_create();
    let entity = goud_entity_spawn_empty(ctx);

    assert!(goud_entity_is_alive(ctx, entity));

    let result = goud_entity_despawn(ctx, entity);
    assert!(result.success);
    assert!(!goud_entity_is_alive(ctx, entity));

    goud_context_destroy(ctx);
}

#[test]
fn test_despawn_invalid_context() {
    let result = goud_entity_despawn(GOUD_INVALID_CONTEXT_ID, 123);
    assert!(!result.success);
}

#[test]
fn test_despawn_invalid_entity() {
    let ctx = goud_context_create();

    let result = goud_entity_despawn(ctx, GOUD_INVALID_ENTITY_ID);
    assert!(!result.success);

    goud_context_destroy(ctx);
}

#[test]
fn test_despawn_already_despawned() {
    let ctx = goud_context_create();
    let entity = goud_entity_spawn_empty(ctx);

    let result1 = goud_entity_despawn(ctx, entity);
    assert!(result1.success);

    let result2 = goud_entity_despawn(ctx, entity);
    assert!(!result2.success);

    goud_context_destroy(ctx);
}

#[test]
fn test_despawn_batch_basic() {
    let ctx = goud_context_create();
    let mut entities = vec![0u64; 5];

    // SAFETY: entities has capacity for 5 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 5, entities.as_mut_ptr());
    }

    // SAFETY: entities is a valid slice of 5 u64 values.
    let count = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), 5) };

    assert_eq!(count, 5);
    for entity in &entities {
        assert!(!goud_entity_is_alive(ctx, *entity));
    }

    goud_context_destroy(ctx);
}

#[test]
fn test_despawn_batch_partial_invalid() {
    let ctx = goud_context_create();
    let entities = vec![
        goud_entity_spawn_empty(ctx),
        GOUD_INVALID_ENTITY_ID,
        goud_entity_spawn_empty(ctx),
    ];

    // SAFETY: entities is a valid slice of 3 u64 values.
    let count = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), 3) };

    // Should despawn 2 valid entities, skip 1 invalid
    assert_eq!(count, 2);

    goud_context_destroy(ctx);
}

#[test]
fn test_despawn_batch_zero_count() {
    let ctx = goud_context_create();
    let entities = vec![0u64; 1];

    // SAFETY: entities is a valid slice; count 0 means no elements are accessed.
    let count = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), 0) };

    assert_eq!(count, 0);
    goud_context_destroy(ctx);
}

// ============================================================================
// Entity Query Tests
// ============================================================================

#[test]
fn test_is_alive_basic() {
    let ctx = goud_context_create();
    let entity = goud_entity_spawn_empty(ctx);

    assert!(goud_entity_is_alive(ctx, entity));

    goud_entity_despawn(ctx, entity);
    assert!(!goud_entity_is_alive(ctx, entity));

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_invalid_context() {
    let alive = goud_entity_is_alive(GOUD_INVALID_CONTEXT_ID, 123);
    assert!(!alive);
}

#[test]
fn test_is_alive_invalid_entity() {
    let ctx = goud_context_create();
    let alive = goud_entity_is_alive(ctx, GOUD_INVALID_ENTITY_ID);
    assert!(!alive);
    goud_context_destroy(ctx);
}

#[test]
fn test_entity_count_basic() {
    let ctx = goud_context_create();

    assert_eq!(goud_entity_count(ctx), 0);

    goud_entity_spawn_empty(ctx);
    assert_eq!(goud_entity_count(ctx), 1);

    goud_entity_spawn_empty(ctx);
    assert_eq!(goud_entity_count(ctx), 2);

    goud_context_destroy(ctx);
}

#[test]
fn test_entity_count_after_despawn() {
    let ctx = goud_context_create();

    let e1 = goud_entity_spawn_empty(ctx);
    let e2 = goud_entity_spawn_empty(ctx);
    assert_eq!(goud_entity_count(ctx), 2);

    goud_entity_despawn(ctx, e1);
    assert_eq!(goud_entity_count(ctx), 1);

    goud_entity_despawn(ctx, e2);
    assert_eq!(goud_entity_count(ctx), 0);

    goud_context_destroy(ctx);
}

#[test]
fn test_entity_count_invalid_context() {
    let count = goud_entity_count(GOUD_INVALID_CONTEXT_ID);
    assert_eq!(count, 0);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_spawn_despawn_respawn() {
    let ctx = goud_context_create();

    // Spawn, despawn, then spawn again (should reuse slot)
    let e1 = goud_entity_spawn_empty(ctx);
    goud_entity_despawn(ctx, e1);
    let e2 = goud_entity_spawn_empty(ctx);

    // e1 should be dead, e2 should be alive
    assert!(!goud_entity_is_alive(ctx, e1));
    assert!(goud_entity_is_alive(ctx, e2));

    goud_context_destroy(ctx);
}

#[test]
fn test_mixed_operations() {
    let ctx = goud_context_create();

    // Spawn some entities
    let mut entities = vec![0u64; 10];
    // SAFETY: entities has capacity for 10 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 10, entities.as_mut_ptr());
    }

    assert_eq!(goud_entity_count(ctx), 10);

    // Despawn half of them
    // SAFETY: entities is a valid slice with at least 5 elements.
    unsafe {
        goud_entity_despawn_batch(ctx, entities.as_ptr(), 5);
    }

    assert_eq!(goud_entity_count(ctx), 5);

    // Spawn more
    let e = goud_entity_spawn_empty(ctx);
    assert_eq!(goud_entity_count(ctx), 6);
    assert!(goud_entity_is_alive(ctx, e));

    goud_context_destroy(ctx);
}

#[test]
fn test_stress_spawn_despawn() {
    let ctx = goud_context_create();

    // Spawn 1000 entities
    let mut entities = vec![0u64; 1000];
    // SAFETY: entities has capacity for 1000 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr());
    }

    assert_eq!(goud_entity_count(ctx), 1000);

    // Despawn all
    // SAFETY: entities is a valid slice with 1000 elements.
    unsafe {
        goud_entity_despawn_batch(ctx, entities.as_ptr(), 1000);
    }

    assert_eq!(goud_entity_count(ctx), 0);

    // Spawn again
    // SAFETY: entities has capacity for 1000 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr());
    }

    assert_eq!(goud_entity_count(ctx), 1000);

    goud_context_destroy(ctx);
}
