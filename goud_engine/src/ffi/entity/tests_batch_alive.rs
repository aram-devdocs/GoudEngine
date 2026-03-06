//! Tests for the `goud_entity_is_alive_batch` FFI function.

use crate::ffi::context::{goud_context_create, goud_context_destroy};
use crate::ffi::entity::{
    lifecycle::{goud_entity_despawn, goud_entity_spawn_batch},
    queries::goud_entity_is_alive_batch,
    GOUD_INVALID_ENTITY_ID,
};
use crate::ffi::GOUD_INVALID_CONTEXT_ID;

#[test]
fn test_is_alive_batch_basic() {
    let ctx = goud_context_create();

    // Spawn 5 entities
    let mut entities = [0u64; 5];
    // SAFETY: entities has capacity for 5 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 5, entities.as_mut_ptr());
    }

    // Check all are alive
    let mut results = [0u8; 5];
    // SAFETY: entities and results are valid slices of length 5.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 5, results.as_mut_ptr()) };

    assert_eq!(count, 5);
    for result in &results {
        assert_eq!(*result, 1); // All alive
    }

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_batch_mixed() {
    let ctx = goud_context_create();

    // Spawn 5 entities
    let mut entities = [0u64; 5];
    // SAFETY: entities has capacity for 5 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 5, entities.as_mut_ptr());
    }

    // Despawn entities at indices 1 and 3
    let _ = goud_entity_despawn(ctx, entities[1]);
    let _ = goud_entity_despawn(ctx, entities[3]);

    // Check alive status
    let mut results = [0u8; 5];
    // SAFETY: entities and results are valid slices of length 5.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 5, results.as_mut_ptr()) };

    assert_eq!(count, 5);
    assert_eq!(results[0], 1); // Alive
    assert_eq!(results[1], 0); // Despawned
    assert_eq!(results[2], 1); // Alive
    assert_eq!(results[3], 0); // Despawned
    assert_eq!(results[4], 1); // Alive

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_batch_invalid_context() {
    let entities = [1u64, 2, 3];
    let mut results = [0u8; 3];

    // SAFETY: entities and results are valid slices; the function handles invalid context gracefully.
    let count = unsafe {
        goud_entity_is_alive_batch(
            GOUD_INVALID_CONTEXT_ID,
            entities.as_ptr(),
            3,
            results.as_mut_ptr(),
        )
    };

    assert_eq!(count, 0);
}

#[test]
fn test_is_alive_batch_null_entities() {
    let ctx = goud_context_create();
    let mut results = [0u8; 3];

    // SAFETY: Passing null for entities tests that the function handles null gracefully.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, std::ptr::null(), 3, results.as_mut_ptr()) };

    assert_eq!(count, 0);

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_batch_null_results() {
    let ctx = goud_context_create();
    let entities = [1u64, 2, 3];

    // SAFETY: Passing null for results tests that the function handles null gracefully.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 3, std::ptr::null_mut()) };

    assert_eq!(count, 0);

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_batch_zero_count() {
    let ctx = goud_context_create();
    let entities = [1u64];
    let mut results = [0u8; 1];

    // SAFETY: entities and results are valid slices; count 0 means no elements are accessed.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 0, results.as_mut_ptr()) };

    assert_eq!(count, 0);

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_batch_invalid_entities() {
    let ctx = goud_context_create();

    // Use entity IDs that were never spawned
    let entities = [GOUD_INVALID_ENTITY_ID, u64::MAX - 1, 123456];
    let mut results = [1u8; 3]; // Initialize to 1 to see they're cleared

    // SAFETY: entities and results are valid slices of length 3.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 3, results.as_mut_ptr()) };

    assert_eq!(count, 3);
    for result in &results {
        assert_eq!(*result, 0); // All should be not alive
    }

    goud_context_destroy(ctx);
}

#[test]
fn test_is_alive_batch_large() {
    let ctx = goud_context_create();

    // Spawn 1000 entities
    let mut entities = vec![0u64; 1000];
    // SAFETY: entities has capacity for 1000 u64 values.
    unsafe {
        goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr());
    }

    // Check all are alive
    let mut results = vec![0u8; 1000];
    // SAFETY: entities and results are valid slices of length 1000.
    let count =
        unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 1000, results.as_mut_ptr()) };

    assert_eq!(count, 1000);
    for result in &results {
        assert_eq!(*result, 1);
    }

    goud_context_destroy(ctx);
}
