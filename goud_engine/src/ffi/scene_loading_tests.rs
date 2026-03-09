use super::*;
use crate::core::error::SUCCESS;
use crate::ffi::context::{goud_context_create, goud_context_destroy, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::error::{goud_clear_last_error, goud_last_error_code};
use crate::ffi::scene::goud_scene_get_by_name;

/// Helper: creates a fresh context for testing.
fn setup_context() -> GoudContextId {
    goud_context_create()
}

/// Helper: tears down a context after testing.
fn teardown_context(id: GoudContextId) {
    goud_context_destroy(id);
}

// ----- load / unload success path ------------------------------------------

#[test]
fn test_scene_load_and_unload_success_path() {
    let ctx = setup_context();
    let name = b"ffi_level";
    let json = br#"{"name":"ffi_level_data","entities":[]}"#;

    // SAFETY: `name` and `json` are valid pointers to UTF-8 byte slices.
    let loaded_id = unsafe {
        goud_scene_load(
            ctx,
            name.as_ptr(),
            name.len() as u32,
            json.as_ptr(),
            json.len() as u32,
        )
    };

    assert_ne!(loaded_id, u32::MAX, "scene load should succeed");

    // SAFETY: `name` points to valid UTF-8.
    let found_id = unsafe { goud_scene_get_by_name(ctx, name.as_ptr(), name.len() as u32) };
    assert_eq!(
        found_id, loaded_id,
        "loaded scene should be queryable by name"
    );

    // SAFETY: `name` points to valid UTF-8.
    let unload_result = unsafe { goud_scene_unload(ctx, name.as_ptr(), name.len() as u32) };
    assert!(unload_result.is_ok(), "scene unload should succeed");

    // SAFETY: `name` points to valid UTF-8.
    let missing_id = unsafe { goud_scene_get_by_name(ctx, name.as_ptr(), name.len() as u32) };
    assert_eq!(
        missing_id,
        u32::MAX,
        "unloaded scene should no longer be queryable"
    );

    teardown_context(ctx);
}

// ----- invalid context ------------------------------------------------------

#[test]
fn test_scene_load_invalid_context_returns_invalid_scene_id_and_sets_error() {
    goud_clear_last_error();

    let name = b"bad_ctx";
    let json = br#"{"name":"bad_ctx_data","entities":[]}"#;

    // SAFETY: pointers reference valid byte slices.
    let scene_id = unsafe {
        goud_scene_load(
            GOUD_INVALID_CONTEXT_ID,
            name.as_ptr(),
            name.len() as u32,
            json.as_ptr(),
            json.len() as u32,
        )
    };

    assert_eq!(scene_id, u32::MAX);
    assert_ne!(goud_last_error_code(), SUCCESS);
}

#[test]
fn test_scene_unload_invalid_context_fails_and_sets_error() {
    goud_clear_last_error();

    let name = b"bad_ctx_unload";

    // SAFETY: pointer references valid byte slice.
    let result =
        unsafe { goud_scene_unload(GOUD_INVALID_CONTEXT_ID, name.as_ptr(), name.len() as u32) };

    assert!(result.is_err());
    assert_ne!(goud_last_error_code(), SUCCESS);
}

// ----- null pointer validation ---------------------------------------------

#[test]
fn test_scene_load_null_name_ptr_fails() {
    let ctx = setup_context();
    let json = br#"{"name":"json_only","entities":[]}"#;

    // SAFETY: null pointer is intentional for validation test.
    let scene_id =
        unsafe { goud_scene_load(ctx, std::ptr::null(), 4, json.as_ptr(), json.len() as u32) };

    assert_eq!(scene_id, u32::MAX);
    teardown_context(ctx);
}

#[test]
fn test_scene_load_null_json_ptr_fails() {
    let ctx = setup_context();
    let name = b"name_only";

    // SAFETY: null pointer is intentional for validation test.
    let scene_id =
        unsafe { goud_scene_load(ctx, name.as_ptr(), name.len() as u32, std::ptr::null(), 8) };

    assert_eq!(scene_id, u32::MAX);
    teardown_context(ctx);
}

#[test]
fn test_scene_unload_null_name_ptr_fails() {
    let ctx = setup_context();

    // SAFETY: null pointer is intentional for validation test.
    let result = unsafe { goud_scene_unload(ctx, std::ptr::null(), 4) };

    assert!(result.is_err());
    teardown_context(ctx);
}

// ----- UTF-8 validation -----------------------------------------------------

#[test]
fn test_scene_load_invalid_utf8_name_fails() {
    let ctx = setup_context();
    let invalid_name = [0xff, 0xfe, 0xfd];
    let json = br#"{"name":"utf8_scene","entities":[]}"#;

    // SAFETY: byte pointers are valid; name bytes are intentionally invalid UTF-8.
    let scene_id = unsafe {
        goud_scene_load(
            ctx,
            invalid_name.as_ptr(),
            invalid_name.len() as u32,
            json.as_ptr(),
            json.len() as u32,
        )
    };

    assert_eq!(scene_id, u32::MAX);
    teardown_context(ctx);
}

#[test]
fn test_scene_load_invalid_utf8_json_fails() {
    let ctx = setup_context();
    let name = b"utf8_json_fail";
    let invalid_json = [0xff, 0x7b, 0x7d];

    // SAFETY: byte pointers are valid; JSON bytes are intentionally invalid UTF-8.
    let scene_id = unsafe {
        goud_scene_load(
            ctx,
            name.as_ptr(),
            name.len() as u32,
            invalid_json.as_ptr(),
            invalid_json.len() as u32,
        )
    };

    assert_eq!(scene_id, u32::MAX);
    teardown_context(ctx);
}

#[test]
fn test_scene_unload_invalid_utf8_name_fails() {
    let ctx = setup_context();
    let invalid_name = [0xff, 0xfe, 0xfd];

    // SAFETY: pointer is valid; bytes are intentionally invalid UTF-8.
    let result =
        unsafe { goud_scene_unload(ctx, invalid_name.as_ptr(), invalid_name.len() as u32) };

    assert!(result.is_err());
    teardown_context(ctx);
}
