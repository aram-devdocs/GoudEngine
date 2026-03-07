//! Tests for context and entity lifecycle safety: double-destroy,
//! stale handles, use-after-despawn.

use super::helpers::*;

use goud_engine::ffi::component::{
    goud_component_get, goud_component_get_mut, goud_component_has, goud_component_remove,
};
use goud_engine::ffi::component_sprite::builder::{
    goud_sprite_builder_default, goud_sprite_builder_free,
};
use goud_engine::ffi::component_transform2d::builder::{
    goud_transform2d_builder_free, goud_transform2d_builder_new,
};
use goud_engine::ffi::context::{goud_context_destroy, goud_context_is_valid};
use goud_engine::ffi::entity::{
    goud_entity_clone, goud_entity_clone_recursive, goud_entity_count, goud_entity_despawn,
    goud_entity_is_alive, goud_entity_spawn_empty,
};

// ===========================================================================
// Context double-destroy
// ===========================================================================

#[test]
fn test_context_double_destroy_returns_false() {
    let ctx = create_test_context();
    cleanup_context(ctx);

    // Second destroy on already-destroyed context should return false.
    let second = goud_context_destroy(ctx);
    assert!(
        !second,
        "destroying an already-destroyed context should return false"
    );
}

#[test]
fn test_context_is_valid_after_destroy_returns_false() {
    let ctx = create_test_context();
    cleanup_context(ctx);

    let valid = goud_context_is_valid(ctx);
    assert!(
        !valid,
        "is_valid should return false for a destroyed context"
    );
}

// ===========================================================================
// Stale context -- operations after destroy
// ===========================================================================

#[test]
fn test_spawn_with_stale_context_returns_invalid() {
    let ctx = create_test_context();
    cleanup_context(ctx);

    let entity = goud_entity_spawn_empty(ctx);
    assert_eq!(
        entity, INVALID_ENTITY,
        "spawn_empty on a destroyed context should return INVALID_ENTITY"
    );
}

#[test]
fn test_entity_count_with_stale_context_returns_zero() {
    let ctx = create_test_context();
    cleanup_context(ctx);

    let count = goud_entity_count(ctx);
    assert_eq!(
        count, 0,
        "entity_count on a destroyed context should return 0"
    );
}

#[test]
fn test_despawn_with_stale_context_returns_error() {
    let ctx = create_test_context();
    cleanup_context(ctx);

    let result = goud_entity_despawn(ctx, 42);
    assert!(
        result.is_err(),
        "despawn on a destroyed context should return an error"
    );
}

// ===========================================================================
// Entity use-after-despawn
// ===========================================================================

#[test]
fn test_is_alive_after_despawn_returns_false() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    assert_ne!(entity, INVALID_ENTITY, "spawn should succeed");

    let result = goud_entity_despawn(ctx, entity);
    assert!(result.is_ok(), "first despawn should succeed");

    let alive = goud_entity_is_alive(ctx, entity);
    assert!(!alive, "entity should not be alive after despawn");

    cleanup_context(ctx);
}

#[test]
fn test_double_despawn_returns_error() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    assert_ne!(entity, INVALID_ENTITY, "spawn should succeed");

    let first = goud_entity_despawn(ctx, entity);
    assert!(first.is_ok(), "first despawn should succeed");

    let second = goud_entity_despawn(ctx, entity);
    assert!(
        second.is_err(),
        "second despawn of the same entity should fail"
    );

    cleanup_context(ctx);
}

#[test]
fn test_component_has_after_despawn_returns_false() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    let _ = goud_entity_despawn(ctx, entity);

    let entity_id = GoudEntityId::new(entity);
    let has = goud_component_has(ctx, entity_id, 99999);
    assert!(
        !has,
        "component_has on despawned entity should return false"
    );

    cleanup_context(ctx);
}

#[test]
fn test_component_get_after_despawn_returns_null() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    let _ = goud_entity_despawn(ctx, entity);

    let entity_id = GoudEntityId::new(entity);
    let ptr = goud_component_get(ctx, entity_id, 99999);
    assert!(
        ptr.is_null(),
        "component_get on despawned entity should return null"
    );

    cleanup_context(ctx);
}

#[test]
fn test_component_get_mut_after_despawn_returns_null() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    let _ = goud_entity_despawn(ctx, entity);

    let entity_id = GoudEntityId::new(entity);
    let ptr = goud_component_get_mut(ctx, entity_id, 99999);
    assert!(
        ptr.is_null(),
        "component_get_mut on despawned entity should return null"
    );

    cleanup_context(ctx);
}

#[test]
fn test_component_remove_after_despawn_returns_error() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    let _ = goud_entity_despawn(ctx, entity);

    let entity_id = GoudEntityId::new(entity);
    let result = goud_component_remove(ctx, entity_id, 99999);
    assert!(
        result.is_err(),
        "component_remove on despawned entity should fail"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Clone with invalid / despawned entities
// ===========================================================================

#[test]
fn test_clone_invalid_entity_returns_invalid() {
    let ctx = create_test_context();

    let cloned = goud_entity_clone(ctx, INVALID_ENTITY);
    assert_eq!(
        cloned, INVALID_ENTITY,
        "clone of INVALID_ENTITY should return INVALID_ENTITY"
    );

    cleanup_context(ctx);
}

#[test]
fn test_clone_despawned_entity_returns_invalid() {
    let ctx = create_test_context();

    let entity = goud_entity_spawn_empty(ctx);
    let _ = goud_entity_despawn(ctx, entity);

    let cloned = goud_entity_clone(ctx, entity);
    assert_eq!(
        cloned, INVALID_ENTITY,
        "clone of despawned entity should return INVALID_ENTITY"
    );

    cleanup_context(ctx);
}

#[test]
fn test_clone_recursive_invalid_entity_returns_invalid() {
    let ctx = create_test_context();

    let cloned = goud_entity_clone_recursive(ctx, INVALID_ENTITY);
    assert_eq!(
        cloned, INVALID_ENTITY,
        "clone_recursive of INVALID_ENTITY should return INVALID_ENTITY"
    );

    cleanup_context(ctx);
}

// ===========================================================================
// Builder normal lifecycle (allocate then free)
// ===========================================================================

#[test]
fn test_transform2d_builder_normal_lifecycle() {
    let builder = goud_transform2d_builder_new();
    assert!(!builder.is_null(), "builder allocation should succeed");
    // SAFETY: builder was allocated by goud_transform2d_builder_new.
    unsafe { goud_transform2d_builder_free(builder) };
}

#[test]
fn test_sprite_builder_normal_lifecycle() {
    let builder = goud_sprite_builder_default();
    assert!(
        !builder.is_null(),
        "sprite builder allocation should succeed"
    );
    // SAFETY: builder was allocated by goud_sprite_builder_default.
    unsafe { goud_sprite_builder_free(builder) };
}
