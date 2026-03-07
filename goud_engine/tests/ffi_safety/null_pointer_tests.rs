//! Tests that null pointers and sentinel inputs are handled gracefully
//! (return an error / sentinel rather than crashing).

use super::helpers::*;

use goud_engine::ffi::component::{
    goud_component_get, goud_component_get_mut, goud_component_has, goud_component_remove,
};
use goud_engine::ffi::component_sprite::builder::{
    goud_sprite_builder_free, goud_sprite_builder_with_alpha,
};
use goud_engine::ffi::component_sprite_animator::factory::goud_animation_clip_builder_free;
use goud_engine::ffi::component_transform2d::builder::{
    goud_transform2d_builder_free, goud_transform2d_builder_with_position,
    goud_transform2d_builder_with_rotation, goud_transform2d_builder_with_scale,
};
use goud_engine::ffi::entity::{
    goud_entity_count, goud_entity_despawn, goud_entity_despawn_batch, goud_entity_is_alive,
    goud_entity_is_alive_batch, goud_entity_spawn_batch, goud_entity_spawn_empty,
};
use goud_engine::ffi::error::{
    goud_clear_last_error, goud_last_error_code, goud_last_error_message,
};

// ===========================================================================
// Builder null-pointer tests
// ===========================================================================

#[test]
fn test_transform2d_builder_free_null_is_noop() {
    // SAFETY: Passing null to a function documented to accept null.
    unsafe { goud_transform2d_builder_free(std::ptr::null_mut()) };
    assert!(true, "goud_transform2d_builder_free(null) should not panic or crash");
}

#[test]
fn test_sprite_builder_free_null_is_noop() {
    // SAFETY: Passing null to a function documented to accept null.
    unsafe { goud_sprite_builder_free(std::ptr::null_mut()) };
    assert!(true, "goud_sprite_builder_free(null) should not panic or crash");
}

#[test]
fn test_animation_clip_builder_free_null_is_noop() {
    // SAFETY: Passing null to a function documented to accept null.
    unsafe { goud_animation_clip_builder_free(std::ptr::null_mut()) };
    assert!(true, "goud_animation_clip_builder_free(null) should not panic or crash");
}

#[test]
fn test_transform2d_builder_with_position_null_returns_null() {
    // SAFETY: Passing null to a function documented to accept null.
    let result = unsafe { goud_transform2d_builder_with_position(std::ptr::null_mut(), 1.0, 2.0) };
    assert!(
        result.is_null(),
        "with_position(null) should return null, got {:?}",
        result
    );
}

#[test]
fn test_transform2d_builder_with_rotation_null_returns_null() {
    // SAFETY: Passing null to a function documented to accept null.
    let result = unsafe { goud_transform2d_builder_with_rotation(std::ptr::null_mut(), 0.5) };
    assert!(
        result.is_null(),
        "with_rotation(null) should return null, got {:?}",
        result
    );
}

#[test]
fn test_transform2d_builder_with_scale_null_returns_null() {
    // SAFETY: Passing null to a function documented to accept null.
    let result = unsafe { goud_transform2d_builder_with_scale(std::ptr::null_mut(), 2.0, 3.0) };
    assert!(
        result.is_null(),
        "with_scale(null) should return null, got {:?}",
        result
    );
}

#[test]
fn test_sprite_builder_with_alpha_null_returns_null() {
    // SAFETY: Passing null to a function documented to accept null.
    let result = unsafe { goud_sprite_builder_with_alpha(std::ptr::null_mut(), 0.5) };
    assert!(
        result.is_null(),
        "with_alpha(null) should return null, got {:?}",
        result
    );
}

// ===========================================================================
// Spawn / despawn with INVALID_CONTEXT
// ===========================================================================

#[test]
fn test_spawn_empty_with_invalid_context_returns_sentinel() {
    let id = goud_entity_spawn_empty(GOUD_INVALID_CONTEXT_ID);
    assert_eq!(
        id, INVALID_ENTITY,
        "spawn_empty with INVALID_CONTEXT should return INVALID_ENTITY"
    );
}

#[test]
fn test_entity_count_with_invalid_context_returns_zero() {
    let count = goud_entity_count(GOUD_INVALID_CONTEXT_ID);
    assert_eq!(
        count, 0,
        "entity_count with INVALID_CONTEXT should return 0"
    );
}

#[test]
fn test_is_alive_with_invalid_context_returns_false() {
    let alive = goud_entity_is_alive(GOUD_INVALID_CONTEXT_ID, 0);
    assert!(!alive, "is_alive with INVALID_CONTEXT should return false");
}

#[test]
fn test_despawn_with_invalid_context_returns_error() {
    let result = goud_entity_despawn(GOUD_INVALID_CONTEXT_ID, 0);
    assert!(result.is_err(), "despawn with INVALID_CONTEXT should fail");
}

#[test]
fn test_despawn_with_invalid_entity_returns_error() {
    let ctx = create_test_context();
    let result = goud_entity_despawn(ctx, INVALID_ENTITY);
    assert!(result.is_err(), "despawn with INVALID_ENTITY should fail");
    cleanup_context(ctx);
}

// ===========================================================================
// Batch operations with null pointers
// ===========================================================================

#[test]
fn test_spawn_batch_null_output_returns_zero() {
    let ctx = create_test_context();
    // SAFETY: Null output pointer -- function should handle gracefully.
    let spawned = unsafe { goud_entity_spawn_batch(ctx, 5, std::ptr::null_mut()) };
    assert_eq!(spawned, 0, "spawn_batch with null output should return 0");
    cleanup_context(ctx);
}

#[test]
fn test_spawn_batch_zero_count_returns_zero() {
    let ctx = create_test_context();
    let mut buf = [0u64; 1];
    // SAFETY: Buffer is valid but count is 0 so buffer is not written.
    let spawned = unsafe { goud_entity_spawn_batch(ctx, 0, buf.as_mut_ptr()) };
    assert_eq!(spawned, 0, "spawn_batch with count=0 should return 0");
    cleanup_context(ctx);
}

#[test]
fn test_despawn_batch_null_ids_returns_zero() {
    let ctx = create_test_context();
    // SAFETY: Null pointer for ids -- function should handle gracefully.
    let despawned = unsafe { goud_entity_despawn_batch(ctx, std::ptr::null(), 3) };
    assert_eq!(despawned, 0, "despawn_batch with null ids should return 0");
    cleanup_context(ctx);
}

#[test]
fn test_is_alive_batch_null_ids_returns_zero() {
    let ctx = create_test_context();
    let mut results = [0u8; 3];
    // SAFETY: Null ids pointer -- function should handle gracefully.
    let written =
        unsafe { goud_entity_is_alive_batch(ctx, std::ptr::null(), 3, results.as_mut_ptr()) };
    assert_eq!(written, 0, "is_alive_batch with null ids should return 0");
    cleanup_context(ctx);
}

#[test]
fn test_is_alive_batch_null_results_returns_zero() {
    let ctx = create_test_context();
    let ids = [0u64; 3];
    // SAFETY: Null results pointer -- function should handle gracefully.
    let written = unsafe { goud_entity_is_alive_batch(ctx, ids.as_ptr(), 3, std::ptr::null_mut()) };
    assert_eq!(
        written, 0,
        "is_alive_batch with null results should return 0"
    );
    cleanup_context(ctx);
}

// ===========================================================================
// Error query edge cases
// ===========================================================================

#[test]
fn test_last_error_message_null_buffer_returns_negative_or_zero() {
    // SAFETY: Null buffer is explicitly documented as valid input.
    let result = unsafe { goud_last_error_message(std::ptr::null_mut(), 0) };
    // Returns 0 (no error) or negative (required size).
    assert!(
        result <= 0,
        "last_error_message(null, 0) should return <= 0, got {result}"
    );
}

#[test]
fn test_last_error_code_after_clear_is_zero() {
    goud_clear_last_error();
    let code = goud_last_error_code();
    assert_eq!(code, 0, "error code should be 0 after clear");
}

// ===========================================================================
// Component ops with invalid context / entity
// ===========================================================================

#[test]
fn test_component_has_invalid_context_returns_false() {
    let entity_id = GoudEntityId::new(0);
    let has = goud_component_has(GOUD_INVALID_CONTEXT_ID, entity_id, 12345);
    assert!(!has, "component_has with INVALID_CONTEXT should be false");
}

#[test]
fn test_component_get_invalid_context_returns_null() {
    let entity_id = GoudEntityId::new(0);
    let ptr = goud_component_get(GOUD_INVALID_CONTEXT_ID, entity_id, 12345);
    assert!(
        ptr.is_null(),
        "component_get with INVALID_CONTEXT should return null"
    );
}

#[test]
fn test_component_get_mut_invalid_context_returns_null() {
    let entity_id = GoudEntityId::new(0);
    let ptr = goud_component_get_mut(GOUD_INVALID_CONTEXT_ID, entity_id, 12345);
    assert!(
        ptr.is_null(),
        "component_get_mut with INVALID_CONTEXT should return null"
    );
}

#[test]
fn test_component_remove_invalid_context_returns_error() {
    let entity_id = GoudEntityId::new(0);
    let result = goud_component_remove(GOUD_INVALID_CONTEXT_ID, entity_id, 12345);
    assert!(
        result.is_err(),
        "component_remove with INVALID_CONTEXT should fail"
    );
}
