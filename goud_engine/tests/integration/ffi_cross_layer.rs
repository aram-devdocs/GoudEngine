//! FFI cross-layer integration tests.
//!
//! These tests exercise the full FFI -> engine stack by calling the public
//! `extern "C"` functions through the `goud_engine::ffi` module. Each test
//! creates its own context to avoid shared state.

use goud_engine::context_registry::GOUD_INVALID_CONTEXT_ID;
use goud_engine::ffi::context::{goud_context_create, goud_context_destroy, goud_context_is_valid};
use goud_engine::ffi::entity::{
    goud_entity_count, goud_entity_despawn, goud_entity_despawn_batch, goud_entity_is_alive,
    goud_entity_spawn_batch, goud_entity_spawn_empty, GOUD_INVALID_ENTITY_ID,
};

use super::helpers::{cleanup_context, create_test_context};

// ===========================================================================
// Full Entity Lifecycle via FFI
// ===========================================================================

#[test]
fn test_ffi_full_entity_lifecycle() {
    let ctx = create_test_context();

    // Spawn
    let entity = goud_entity_spawn_empty(ctx);
    assert_ne!(
        entity, GOUD_INVALID_ENTITY_ID,
        "goud_entity_spawn_empty should return a valid entity ID"
    );

    // Verify alive
    assert!(
        goud_entity_is_alive(ctx, entity),
        "Freshly spawned entity should be alive via FFI"
    );

    // Despawn
    let result = goud_entity_despawn(ctx, entity);
    assert!(
        result.is_ok(),
        "goud_entity_despawn should succeed for a live entity"
    );

    // Verify dead
    assert!(
        !goud_entity_is_alive(ctx, entity),
        "Despawned entity should no longer be alive via FFI"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Entity Count Consistency
// ===========================================================================

#[test]
fn test_ffi_entity_count_after_spawn_and_despawn() {
    let ctx = create_test_context();

    assert_eq!(
        goud_entity_count(ctx),
        0,
        "Fresh context should have 0 entities"
    );

    // Spawn 5 entities
    let mut entities = Vec::new();
    for i in 0..5 {
        let e = goud_entity_spawn_empty(ctx);
        assert_ne!(e, GOUD_INVALID_ENTITY_ID, "Spawn #{} should succeed", i);
        entities.push(e);
    }
    assert_eq!(
        goud_entity_count(ctx),
        5,
        "After spawning 5 entities, count should be 5"
    );

    // Despawn 2
    for e in &entities[..2] {
        let result = goud_entity_despawn(ctx, *e);
        assert!(result.is_ok(), "Despawn should succeed for live entity");
    }
    assert_eq!(
        goud_entity_count(ctx),
        3,
        "After spawning 5 and despawning 2, count should be 3"
    );

    cleanup_context(ctx);
}

#[test]
fn test_ffi_entity_count_spawn_n_despawn_m() {
    let ctx = create_test_context();
    let n: u32 = 20;
    let m: u32 = 7;

    let mut ids = vec![0u64; n as usize];
    // SAFETY: ids has capacity for n u64 values, matching count parameter.
    let spawned = unsafe { goud_entity_spawn_batch(ctx, n, ids.as_mut_ptr()) };
    assert_eq!(spawned, n, "Batch spawn should create exactly N entities");
    assert_eq!(
        goud_entity_count(ctx),
        n,
        "Entity count should equal N after batch spawn"
    );

    // Despawn first M
    // SAFETY: ids[..m] has exactly m elements matching count parameter.
    let despawned = unsafe { goud_entity_despawn_batch(ctx, ids.as_ptr(), m) };
    assert_eq!(
        despawned, m,
        "Batch despawn should remove exactly M entities"
    );
    assert_eq!(
        goud_entity_count(ctx),
        n - m,
        "Entity count should equal N-M after batch despawn"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Multiple Entities
// ===========================================================================

#[test]
fn test_ffi_multiple_entities_all_alive() {
    let ctx = create_test_context();

    let mut entities = Vec::new();
    for _ in 0..10 {
        entities.push(goud_entity_spawn_empty(ctx));
    }

    for (i, e) in entities.iter().enumerate() {
        assert_ne!(
            *e, GOUD_INVALID_ENTITY_ID,
            "Entity at index {} should have a valid ID",
            i
        );
        assert!(
            goud_entity_is_alive(ctx, *e),
            "Entity at index {} should be alive",
            i
        );
    }

    // Despawn odd-indexed entities
    for i in (1..10).step_by(2) {
        let result = goud_entity_despawn(ctx, entities[i]);
        assert!(
            result.is_ok(),
            "Despawning entity at index {} should succeed",
            i
        );
    }

    // Verify: even alive, odd dead
    for (i, e) in entities.iter().enumerate() {
        if i % 2 == 0 {
            assert!(
                goud_entity_is_alive(ctx, *e),
                "Even-indexed entity {} should still be alive",
                i
            );
        } else {
            assert!(
                !goud_entity_is_alive(ctx, *e),
                "Odd-indexed entity {} should be dead after despawn",
                i
            );
        }
    }

    assert_eq!(
        goud_entity_count(ctx),
        5,
        "After despawning 5 of 10 entities, count should be 5"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Context Isolation
// ===========================================================================

#[test]
fn test_ffi_two_contexts_do_not_share_entities() {
    let ctx_a = create_test_context();
    let ctx_b = create_test_context();

    // Spawn entities in context A
    let e_a1 = goud_entity_spawn_empty(ctx_a);
    let e_a2 = goud_entity_spawn_empty(ctx_a);
    assert_ne!(
        e_a1, GOUD_INVALID_ENTITY_ID,
        "Spawn in ctx_a should succeed"
    );
    assert_ne!(
        e_a2, GOUD_INVALID_ENTITY_ID,
        "Spawn in ctx_a should succeed"
    );

    // Spawn entity in context B
    let e_b1 = goud_entity_spawn_empty(ctx_b);
    assert_ne!(
        e_b1, GOUD_INVALID_ENTITY_ID,
        "Spawn in ctx_b should succeed"
    );

    // Counts should be independent
    assert_eq!(
        goud_entity_count(ctx_a),
        2,
        "Context A should have 2 entities"
    );
    assert_eq!(
        goud_entity_count(ctx_b),
        1,
        "Context B should have 1 entity"
    );

    // Despawning in A should not affect B's count
    goud_entity_despawn(ctx_a, e_a1);
    assert_eq!(
        goud_entity_count(ctx_a),
        1,
        "Context A should have 1 entity after despawn"
    );
    assert_eq!(
        goud_entity_count(ctx_b),
        1,
        "Context B count should be unaffected by A's despawn"
    );

    cleanup_context(ctx_a);
    cleanup_context(ctx_b);
}

// ===========================================================================
// Error State: Invalid Context
// ===========================================================================

#[test]
fn test_ffi_spawn_with_invalid_context() {
    let entity = goud_entity_spawn_empty(GOUD_INVALID_CONTEXT_ID);
    assert_eq!(
        entity, GOUD_INVALID_ENTITY_ID,
        "Spawning on INVALID context should return INVALID entity ID"
    );
}

#[test]
fn test_ffi_despawn_with_invalid_context() {
    let result = goud_entity_despawn(GOUD_INVALID_CONTEXT_ID, 42);
    assert!(
        result.is_err(),
        "Despawning on INVALID context should return an error"
    );
}

#[test]
fn test_ffi_entity_count_with_invalid_context() {
    let count = goud_entity_count(GOUD_INVALID_CONTEXT_ID);
    assert_eq!(count, 0, "Entity count on INVALID context should return 0");
}

#[test]
fn test_ffi_is_alive_with_invalid_context() {
    let alive = goud_entity_is_alive(GOUD_INVALID_CONTEXT_ID, 42);
    assert!(!alive, "is_alive on INVALID context should return false");
}

#[test]
fn test_ffi_despawn_invalid_entity_id() {
    let ctx = create_test_context();

    let result = goud_entity_despawn(ctx, GOUD_INVALID_ENTITY_ID);
    assert!(
        result.is_err(),
        "Despawning INVALID entity ID should return an error"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Error State: Destroyed Context
// ===========================================================================

#[test]
fn test_ffi_operations_on_destroyed_context() {
    let ctx = create_test_context();
    let entity = goud_entity_spawn_empty(ctx);
    assert_ne!(entity, GOUD_INVALID_ENTITY_ID, "Spawn should succeed");

    cleanup_context(ctx);

    // All operations on destroyed context should fail gracefully
    assert!(
        !goud_context_is_valid(ctx),
        "Destroyed context should not be valid"
    );

    let spawn_result = goud_entity_spawn_empty(ctx);
    assert_eq!(
        spawn_result, GOUD_INVALID_ENTITY_ID,
        "Spawning on destroyed context should return INVALID entity"
    );

    assert_eq!(
        goud_entity_count(ctx),
        0,
        "Entity count on destroyed context should return 0"
    );
}

// ===========================================================================
// Batch Operations via FFI
// ===========================================================================

#[test]
fn test_ffi_spawn_batch_and_verify() {
    let ctx = create_test_context();
    let count: u32 = 25;
    let mut ids = vec![0u64; count as usize];

    // SAFETY: ids has capacity for count u64 values matching the count parameter.
    let spawned = unsafe { goud_entity_spawn_batch(ctx, count, ids.as_mut_ptr()) };
    assert_eq!(
        spawned, count,
        "Batch spawn should create exactly {} entities",
        count
    );

    for (i, &id) in ids.iter().enumerate() {
        assert_ne!(
            id, GOUD_INVALID_ENTITY_ID,
            "Batch-spawned entity at index {} should have a valid ID",
            i
        );
        assert!(
            goud_entity_is_alive(ctx, id),
            "Batch-spawned entity at index {} should be alive",
            i
        );
    }

    assert_eq!(
        goud_entity_count(ctx),
        count,
        "Entity count should match batch spawn count"
    );

    cleanup_context(ctx);
}

#[test]
fn test_ffi_despawn_batch_and_verify() {
    let ctx = create_test_context();
    let total: u32 = 15;
    let mut ids = vec![0u64; total as usize];

    // SAFETY: ids has capacity for total u64 values matching the count parameter.
    let spawned = unsafe { goud_entity_spawn_batch(ctx, total, ids.as_mut_ptr()) };
    assert_eq!(spawned, total, "Should spawn all entities");

    let remove_count: u32 = 10;
    // SAFETY: ids[..remove_count] has exactly remove_count elements.
    let despawned = unsafe { goud_entity_despawn_batch(ctx, ids.as_ptr(), remove_count) };
    assert_eq!(
        despawned, remove_count,
        "Batch despawn should remove exactly {} entities",
        remove_count
    );

    // Verify first 10 dead, last 5 alive
    for (i, &id) in ids.iter().enumerate() {
        if (i as u32) < remove_count {
            assert!(
                !goud_entity_is_alive(ctx, id),
                "Entity at index {} should be dead after batch despawn",
                i
            );
        } else {
            assert!(
                goud_entity_is_alive(ctx, id),
                "Entity at index {} should still be alive",
                i
            );
        }
    }

    assert_eq!(
        goud_entity_count(ctx),
        total - remove_count,
        "Remaining entity count should be total - removed"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Context Validity
// ===========================================================================

#[test]
fn test_ffi_context_validity_lifecycle() {
    let ctx = goud_context_create();
    assert_ne!(ctx, GOUD_INVALID_CONTEXT_ID, "Create should succeed");

    assert!(
        goud_context_is_valid(ctx),
        "Newly created context should be valid"
    );

    let destroyed = goud_context_destroy(ctx);
    assert!(destroyed, "Destroy should succeed");

    assert!(
        !goud_context_is_valid(ctx),
        "Destroyed context should not be valid"
    );
}

#[test]
fn test_ffi_invalid_context_id_is_not_valid() {
    assert!(
        !goud_context_is_valid(GOUD_INVALID_CONTEXT_ID),
        "GOUD_INVALID_CONTEXT_ID should never be valid"
    );
}
