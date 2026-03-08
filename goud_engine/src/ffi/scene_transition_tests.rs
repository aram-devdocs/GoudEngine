use super::*;
use crate::ffi::context::{goud_context_create, goud_context_destroy, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::scene::goud_scene_create;

/// Helper: creates a fresh context for testing.
fn setup_context() -> GoudContextId {
    goud_context_create()
}

/// Helper: tears down a context after testing.
fn teardown_context(id: GoudContextId) {
    goud_context_destroy(id);
}

/// Helper: creates a named scene and returns its ID.
///
/// # Safety
///
/// Name bytes must be valid UTF-8.
unsafe fn create_scene(ctx: GoudContextId, name: &[u8]) -> u32 {
    goud_scene_create(ctx, name.as_ptr(), name.len() as u32)
}

// ----- transition_to and progress ----------------------------------------

#[test]
fn test_ffi_transition_to_and_progress() {
    let ctx = setup_context();

    // SAFETY: name bytes are valid UTF-8 literals.
    let scene_a = unsafe { create_scene(ctx, b"scene_a") };
    let scene_b = unsafe { create_scene(ctx, b"scene_b") };
    assert_ne!(scene_a, u32::MAX);
    assert_ne!(scene_b, u32::MAX);

    // Start a fade transition lasting 2 seconds.
    let result = goud_scene_transition_to(ctx, scene_a, scene_b, 1, 2.0);
    assert!(result.is_ok());

    // Progress should be 0.0 initially.
    let progress = goud_scene_transition_progress(ctx);
    assert!((progress - 0.0).abs() < f32::EPSILON);

    teardown_context(ctx);
}

// ----- transition tick completes -----------------------------------------

#[test]
fn test_ffi_transition_tick_completes() {
    let ctx = setup_context();

    // SAFETY: name bytes are valid UTF-8 literals.
    let scene_a = unsafe { create_scene(ctx, b"tick_a") };
    let scene_b = unsafe { create_scene(ctx, b"tick_b") };

    let result = goud_scene_transition_to(ctx, scene_a, scene_b, 1, 1.0);
    assert!(result.is_ok());
    assert!(goud_scene_transition_is_active(ctx));

    // Tick half-way.
    let result = goud_scene_transition_tick(ctx, 0.5);
    assert!(result.is_ok());
    let progress = goud_scene_transition_progress(ctx);
    assert!((progress - 0.5).abs() < f32::EPSILON);

    // Tick past the end.
    let result = goud_scene_transition_tick(ctx, 0.6);
    assert!(result.is_ok());

    // Transition should now be complete.
    assert!(!goud_scene_transition_is_active(ctx));
    assert!((goud_scene_transition_progress(ctx) - (-1.0)).abs() < f32::EPSILON);

    teardown_context(ctx);
}

// ----- is_active ---------------------------------------------------------

#[test]
fn test_ffi_transition_is_active() {
    let ctx = setup_context();

    // No transition yet.
    assert!(!goud_scene_transition_is_active(ctx));

    // SAFETY: name bytes are valid UTF-8 literals.
    let scene_a = unsafe { create_scene(ctx, b"active_a") };
    let scene_b = unsafe { create_scene(ctx, b"active_b") };

    let result = goud_scene_transition_to(ctx, scene_a, scene_b, 1, 1.0);
    assert!(result.is_ok());
    assert!(goud_scene_transition_is_active(ctx));

    teardown_context(ctx);
}

// ----- invalid context ---------------------------------------------------

#[test]
fn test_ffi_transition_invalid_context() {
    let bad = GOUD_INVALID_CONTEXT_ID;

    let result = goud_scene_transition_to(bad, 0, 1, 0, 1.0);
    assert!(result.is_err());

    assert!((goud_scene_transition_progress(bad) - (-1.0)).abs() < f32::EPSILON);
    assert!(!goud_scene_transition_is_active(bad));

    let result = goud_scene_transition_tick(bad, 0.1);
    assert!(result.is_err());
}

// ----- invalid transition type -------------------------------------------

#[test]
fn test_ffi_transition_invalid_type() {
    let ctx = setup_context();

    // SAFETY: name bytes are valid UTF-8 literals.
    let scene_a = unsafe { create_scene(ctx, b"type_a") };
    let scene_b = unsafe { create_scene(ctx, b"type_b") };

    // Transition type 255 is invalid.
    let result = goud_scene_transition_to(ctx, scene_a, scene_b, 255, 1.0);
    assert!(result.is_err());

    teardown_context(ctx);
}
