//! Tests for out-of-range entity IDs, fabricated handles, zero-count
//! operations, and rapid spawn/despawn cycling.

use super::helpers::*;

use goud_engine::ffi::component::{goud_component_get, goud_component_has};
use goud_engine::ffi::entity::{
    goud_entity_clone, goud_entity_count, goud_entity_despawn, goud_entity_despawn_batch,
    goud_entity_is_alive, goud_entity_is_alive_batch, goud_entity_spawn_batch,
    goud_entity_spawn_empty,
};

// ===========================================================================
// Fabricated / out-of-range entity IDs
// ===========================================================================

#[test]
fn test_is_alive_fabricated_entity_returns_false() {
    let ctx = create_test_context();

    // Use a fabricated ID that was never spawned.
    let fabricated: u64 = u64::MAX - 1;
    let alive = goud_entity_is_alive(ctx, fabricated);
    assert!(!alive, "fabricated entity ID should not be alive");

    cleanup_context(ctx);
}

#[test]
fn test_despawn_fabricated_entity_returns_error() {
    let ctx = create_test_context();

    let fabricated: u64 = 0xDEAD_BEEF_CAFE_0000;
    let result = goud_entity_despawn(ctx, fabricated);
    assert!(
        result.is_err(),
        "despawning a fabricated entity should return an error"
    );

    cleanup_context(ctx);
}

#[test]
fn test_component_has_fabricated_entity_returns_false() {
    let ctx = create_test_context();

    let fabricated = GoudEntityId::new(0xBAAD_F00D);
    let has = goud_component_has(ctx, fabricated, 12345);
    assert!(
        !has,
        "component_has on fabricated entity should return false"
    );

    cleanup_context(ctx);
}

#[test]
fn test_component_get_fabricated_entity_returns_null() {
    let ctx = create_test_context();

    let fabricated = GoudEntityId::new(0xBAAD_F00D);
    let ptr = goud_component_get(ctx, fabricated, 12345);
    assert!(
        ptr.is_null(),
        "component_get on fabricated entity should return null"
    );

    cleanup_context(ctx);
}

#[test]
fn test_clone_fabricated_entity_returns_invalid() {
    let ctx = create_test_context();

    let fabricated: u64 = 0xBAAD_F00D;
    let cloned = goud_entity_clone(ctx, fabricated);
    assert_eq!(
        cloned, INVALID_ENTITY,
        "clone of fabricated entity should return INVALID_ENTITY"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Zero-count batch operations
// ===========================================================================

#[test]
fn test_despawn_batch_zero_count_returns_zero() {
    let ctx = create_test_context();
    let ids = [0u64; 1];
    // SAFETY: count=0 means the array is not read.
    let despawned = unsafe { goud_entity_despawn_batch(ctx, ids.as_ptr(), 0) };
    assert_eq!(despawned, 0, "despawn_batch with count=0 should return 0");
    cleanup_context(ctx);
}

#[test]
fn test_is_alive_batch_zero_count_returns_zero() {
    let ctx = create_test_context();
    let ids = [0u64; 1];
    let mut results = [0u8; 1];
    // SAFETY: count=0 means the arrays are not accessed.
    let written = unsafe { goud_entity_is_alive_batch(ctx, ids.as_ptr(), 0, results.as_mut_ptr()) };
    assert_eq!(written, 0, "is_alive_batch with count=0 should return 0");
    cleanup_context(ctx);
}

// ===========================================================================
// Entity count accuracy after spawn/despawn
// ===========================================================================

#[test]
fn test_entity_count_after_spawn_and_despawn() {
    let ctx = create_test_context();

    let initial = goud_entity_count(ctx);

    let e1 = goud_entity_spawn_empty(ctx);
    let e2 = goud_entity_spawn_empty(ctx);
    let e3 = goud_entity_spawn_empty(ctx);
    assert_ne!(e1, INVALID_ENTITY, "spawn 1 failed");
    assert_ne!(e2, INVALID_ENTITY, "spawn 2 failed");
    assert_ne!(e3, INVALID_ENTITY, "spawn 3 failed");

    let after_spawn = goud_entity_count(ctx);
    assert_eq!(
        after_spawn,
        initial + 3,
        "count should increase by 3 after spawning 3 entities"
    );

    let r = goud_entity_despawn(ctx, e2);
    assert!(r.is_ok(), "despawn e2 should succeed");

    let after_despawn = goud_entity_count(ctx);
    assert_eq!(
        after_despawn,
        initial + 2,
        "count should decrease by 1 after despawning 1 entity"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Rapid spawn/despawn cycling
// ===========================================================================

#[test]
fn test_rapid_spawn_despawn_cycle() {
    let ctx = create_test_context();
    let iterations = 100;

    for i in 0..iterations {
        let entity = goud_entity_spawn_empty(ctx);
        assert_ne!(entity, INVALID_ENTITY, "spawn failed on iteration {i}");

        let alive = goud_entity_is_alive(ctx, entity);
        assert!(
            alive,
            "entity should be alive immediately after spawn (iter {i})"
        );

        let result = goud_entity_despawn(ctx, entity);
        assert!(result.is_ok(), "despawn should succeed on iteration {i}");

        let dead = goud_entity_is_alive(ctx, entity);
        assert!(!dead, "entity should be dead after despawn (iter {i})");
    }

    // After all cycles, count should be back to 0.
    let final_count = goud_entity_count(ctx);
    assert_eq!(
        final_count, 0,
        "after {iterations} spawn/despawn cycles, count should be 0"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Batch spawn then batch despawn
// ===========================================================================

#[test]
fn test_batch_spawn_then_batch_despawn() {
    let ctx = create_test_context();
    let count: u32 = 10;
    let mut entities = vec![0u64; count as usize];

    // SAFETY: entities buffer is properly sized for count elements.
    let spawned = unsafe { goud_entity_spawn_batch(ctx, count, entities.as_mut_ptr()) };
    assert_eq!(
        spawned, count,
        "batch spawn should return the requested count"
    );

    // Verify all are alive.
    for (i, &eid) in entities.iter().enumerate() {
        let alive = goud_entity_is_alive(ctx, eid);
        assert!(alive, "entity {i} should be alive after batch spawn");
    }

    // Batch despawn.
    // SAFETY: entities slice is valid and contains count elements.
    let despawned = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), count) };
    assert_eq!(
        despawned, count,
        "batch despawn should succeed for all entities"
    );

    // Verify all are dead.
    for (i, &eid) in entities.iter().enumerate() {
        let alive = goud_entity_is_alive(ctx, eid);
        assert!(!alive, "entity {i} should be dead after batch despawn");
    }

    cleanup_context(ctx);
}

// ===========================================================================
// is_alive_batch correctness
// ===========================================================================

#[test]
fn test_is_alive_batch_mixed_results() {
    let ctx = create_test_context();

    let e1 = goud_entity_spawn_empty(ctx);
    let e2 = goud_entity_spawn_empty(ctx);
    let e3 = goud_entity_spawn_empty(ctx);

    // Despawn e2 only.
    let _ = goud_entity_despawn(ctx, e2);

    let ids = [e1, e2, e3];
    let mut results = [0u8; 3];
    // SAFETY: Both slices are valid for 3 elements.
    let written = unsafe { goud_entity_is_alive_batch(ctx, ids.as_ptr(), 3, results.as_mut_ptr()) };

    assert_eq!(written, 3, "should write 3 results");
    assert_eq!(results[0], 1, "e1 should be alive");
    assert_eq!(results[1], 0, "e2 should be dead");
    assert_eq!(results[2], 1, "e3 should be alive");

    cleanup_context(ctx);
}

// ===========================================================================
// Stale context ID (valid-looking but destroyed)
// ===========================================================================

#[test]
fn test_operations_with_stale_context_id() {
    // Create a real context, destroy it, then try operations.
    let ctx = create_test_context();
    cleanup_context(ctx);

    let entity = goud_entity_spawn_empty(ctx);
    assert_eq!(
        entity, INVALID_ENTITY,
        "spawn with stale context should return INVALID_ENTITY"
    );

    let count = goud_entity_count(ctx);
    assert_eq!(count, 0, "count with stale context should be 0");

    let alive = goud_entity_is_alive(ctx, 0);
    assert!(!alive, "is_alive with stale context should be false");

    let result = goud_entity_despawn(ctx, 42);
    assert!(result.is_err(), "despawn with stale context should fail");
}
