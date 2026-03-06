use super::*;
use crate::ffi::context::{goud_context_create, goud_context_destroy};

/// Helper: creates a fresh context for testing.
fn setup_context() -> GoudContextId {
    goud_context_create()
}

/// Helper: tears down a context after testing.
fn teardown_context(id: GoudContextId) {
    goud_context_destroy(id);
}

// ----- create ------------------------------------------------------------

#[test]
fn test_create_scene_success() {
    let ctx = setup_context();
    let name = b"level_1";

    // SAFETY: name is a valid UTF-8 byte slice.
    let id = unsafe {
        goud_scene_create(ctx, name.as_ptr(), name.len() as u32)
    };

    assert_ne!(id, INVALID_SCENE_ID);
    teardown_context(ctx);
}

#[test]
fn test_create_scene_duplicate_fails() {
    let ctx = setup_context();
    let name = b"dup";

    // SAFETY: name is a valid UTF-8 byte slice.
    unsafe {
        let id1 = goud_scene_create(ctx, name.as_ptr(), name.len() as u32);
        assert_ne!(id1, INVALID_SCENE_ID);

        let id2 = goud_scene_create(ctx, name.as_ptr(), name.len() as u32);
        assert_eq!(id2, INVALID_SCENE_ID);
    }

    teardown_context(ctx);
}

#[test]
fn test_create_scene_invalid_context() {
    let name = b"test";
    // SAFETY: name is a valid UTF-8 byte slice.
    let id = unsafe {
        goud_scene_create(GOUD_INVALID_CONTEXT_ID, name.as_ptr(), name.len() as u32)
    };
    assert_eq!(id, INVALID_SCENE_ID);
}

#[test]
fn test_create_scene_null_name() {
    let ctx = setup_context();
    // SAFETY: Passing null intentionally to test null-check.
    let id = unsafe {
        goud_scene_create(ctx, std::ptr::null(), 5)
    };
    assert_eq!(id, INVALID_SCENE_ID);
    teardown_context(ctx);
}

// ----- destroy -----------------------------------------------------------

#[test]
fn test_destroy_scene_success() {
    let ctx = setup_context();
    let name = b"temp";

    // SAFETY: name is a valid UTF-8 byte slice.
    let scene_id = unsafe {
        goud_scene_create(ctx, name.as_ptr(), name.len() as u32)
    };
    assert_ne!(scene_id, INVALID_SCENE_ID);

    let result = goud_scene_destroy(ctx, scene_id);
    assert!(result.is_ok());

    teardown_context(ctx);
}

#[test]
fn test_destroy_default_scene_fails() {
    let ctx = setup_context();
    let result = goud_scene_destroy(ctx, 0);
    assert!(result.is_err());
    teardown_context(ctx);
}

#[test]
fn test_destroy_invalid_context() {
    let result = goud_scene_destroy(GOUD_INVALID_CONTEXT_ID, 0);
    assert!(result.is_err());
}

// ----- get_by_name -------------------------------------------------------

#[test]
fn test_get_by_name_success() {
    let ctx = setup_context();
    let name = b"named";

    // SAFETY: name is a valid UTF-8 byte slice.
    let created_id = unsafe {
        goud_scene_create(ctx, name.as_ptr(), name.len() as u32)
    };
    assert_ne!(created_id, INVALID_SCENE_ID);

    // SAFETY: name is a valid UTF-8 byte slice.
    let found_id = unsafe {
        goud_scene_get_by_name(ctx, name.as_ptr(), name.len() as u32)
    };
    assert_eq!(found_id, created_id);

    teardown_context(ctx);
}

#[test]
fn test_get_by_name_not_found() {
    let ctx = setup_context();
    let name = b"nonexistent";

    // SAFETY: name is a valid UTF-8 byte slice.
    let id = unsafe {
        goud_scene_get_by_name(ctx, name.as_ptr(), name.len() as u32)
    };
    assert_eq!(id, INVALID_SCENE_ID);

    teardown_context(ctx);
}

#[test]
fn test_get_by_name_null_ptr() {
    let ctx = setup_context();
    // SAFETY: Passing null intentionally to test null-check.
    let id = unsafe {
        goud_scene_get_by_name(ctx, std::ptr::null(), 3)
    };
    assert_eq!(id, INVALID_SCENE_ID);
    teardown_context(ctx);
}

// ----- active management -------------------------------------------------

#[test]
fn test_set_active_and_check() {
    let ctx = setup_context();
    let name = b"active_test";

    // SAFETY: name is a valid UTF-8 byte slice.
    let scene_id = unsafe {
        goud_scene_create(ctx, name.as_ptr(), name.len() as u32)
    };

    // New scene is not active by default.
    assert!(!goud_scene_is_active(ctx, scene_id));

    let result = goud_scene_set_active(ctx, scene_id, true);
    assert!(result.is_ok());
    assert!(goud_scene_is_active(ctx, scene_id));

    let result = goud_scene_set_active(ctx, scene_id, false);
    assert!(result.is_ok());
    assert!(!goud_scene_is_active(ctx, scene_id));

    teardown_context(ctx);
}

#[test]
fn test_set_active_nonexistent_fails() {
    let ctx = setup_context();
    let result = goud_scene_set_active(ctx, 999, true);
    assert!(result.is_err());
    teardown_context(ctx);
}

#[test]
fn test_is_active_invalid_context() {
    assert!(!goud_scene_is_active(GOUD_INVALID_CONTEXT_ID, 0));
}

// ----- scene count -------------------------------------------------------

#[test]
fn test_scene_count_default() {
    let ctx = setup_context();
    // Default scene is always created.
    assert_eq!(goud_scene_count(ctx), 1);
    teardown_context(ctx);
}

#[test]
fn test_scene_count_after_create() {
    let ctx = setup_context();
    let name = b"extra";

    // SAFETY: name is a valid UTF-8 byte slice.
    unsafe {
        goud_scene_create(ctx, name.as_ptr(), name.len() as u32);
    }

    assert_eq!(goud_scene_count(ctx), 2);
    teardown_context(ctx);
}

#[test]
fn test_scene_count_invalid_context() {
    assert_eq!(goud_scene_count(GOUD_INVALID_CONTEXT_ID), 0);
}

// ----- current scene targeting -------------------------------------------

#[test]
fn test_get_current_default() {
    let ctx = setup_context();
    // Default scene (0) is current on creation.
    assert_eq!(goud_scene_get_current(ctx), 0);
    teardown_context(ctx);
}

#[test]
fn test_set_and_get_current() {
    let ctx = setup_context();
    let name = b"target";

    // SAFETY: name is a valid UTF-8 byte slice.
    let scene_id = unsafe {
        goud_scene_create(ctx, name.as_ptr(), name.len() as u32)
    };

    let result = goud_scene_set_current(ctx, scene_id);
    assert!(result.is_ok());
    assert_eq!(goud_scene_get_current(ctx), scene_id);

    teardown_context(ctx);
}

#[test]
fn test_set_current_nonexistent_fails() {
    let ctx = setup_context();
    let result = goud_scene_set_current(ctx, 999);
    assert!(result.is_err());
    // Current scene should be unchanged.
    assert_eq!(goud_scene_get_current(ctx), 0);
    teardown_context(ctx);
}

#[test]
fn test_get_current_invalid_context() {
    assert_eq!(
        goud_scene_get_current(GOUD_INVALID_CONTEXT_ID),
        INVALID_SCENE_ID
    );
}
