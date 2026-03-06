//! Scene management FFI functions.
//!
//! Provides C-compatible functions for creating, destroying, and managing
//! scenes within an engine context. Each scene owns an isolated ECS World.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::types::GoudResult;

/// Sentinel value returned when a scene operation fails to produce a valid ID.
const INVALID_SCENE_ID: u32 = u32::MAX;

// ============================================================================
// Scene Lifecycle
// ============================================================================

/// Creates a new scene with the given name.
///
/// # Arguments
///
/// * `context_id` - The context to create the scene in
/// * `name_ptr` - Pointer to UTF-8 encoded scene name bytes
/// * `name_len` - Length of the name in bytes
///
/// # Returns
///
/// The `SceneId` on success, or `u32::MAX` on error.
/// Call `goud_get_last_error_message()` for error details.
///
/// # Safety
///
/// Caller must ensure `name_ptr` points to valid UTF-8 data of at least
/// `name_len` bytes. Ownership is NOT transferred -- the caller retains
/// ownership of the name buffer.
#[no_mangle]
pub unsafe extern "C" fn goud_scene_create(
    context_id: GoudContextId,
    name_ptr: *const u8,
    name_len: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return INVALID_SCENE_ID;
    }

    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "name_ptr is null".to_string(),
        ));
        return INVALID_SCENE_ID;
    }

    // SAFETY: Caller guarantees name_ptr is valid for name_len bytes.
    let name_bytes = std::slice::from_raw_parts(name_ptr, name_len as usize);
    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "scene name is not valid UTF-8".to_string(),
            ));
            return INVALID_SCENE_ID;
        }
    };

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return INVALID_SCENE_ID;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return INVALID_SCENE_ID;
        }
    };

    match context.scene_manager_mut().create_scene(name) {
        Ok(id) => id,
        Err(err) => {
            set_last_error(err);
            INVALID_SCENE_ID
        }
    }
}

/// Destroys a scene and frees its resources.
///
/// The default scene (ID 0) cannot be destroyed.
///
/// # Arguments
///
/// * `context_id` - The context containing the scene
/// * `scene_id` - The scene to destroy
///
/// # Returns
///
/// A `GoudResult` indicating success or failure.
#[no_mangle]
pub extern "C" fn goud_scene_destroy(
    context_id: GoudContextId,
    scene_id: u32,
) -> GoudResult {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return GoudResult::err(ERR_INTERNAL_ERROR),
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    match context.scene_manager_mut().destroy_scene(scene_id) {
        Ok(()) => GoudResult::ok(),
        Err(err) => {
            let code = err.error_code();
            set_last_error(err);
            GoudResult::err(code)
        }
    }
}

// ============================================================================
// Scene Lookup
// ============================================================================

/// Looks up a scene by name, returning its ID.
///
/// # Arguments
///
/// * `context_id` - The context to search in
/// * `name_ptr` - Pointer to UTF-8 encoded scene name bytes
/// * `name_len` - Length of the name in bytes
///
/// # Returns
///
/// The `SceneId` if found, or `u32::MAX` if not found.
///
/// # Safety
///
/// Caller must ensure `name_ptr` points to valid UTF-8 data of at least
/// `name_len` bytes. Ownership is NOT transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_scene_get_by_name(
    context_id: GoudContextId,
    name_ptr: *const u8,
    name_len: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return INVALID_SCENE_ID;
    }

    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "name_ptr is null".to_string(),
        ));
        return INVALID_SCENE_ID;
    }

    // SAFETY: Caller guarantees name_ptr is valid for name_len bytes.
    let name_bytes = std::slice::from_raw_parts(name_ptr, name_len as usize);
    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "scene name is not valid UTF-8".to_string(),
            ));
            return INVALID_SCENE_ID;
        }
    };

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return INVALID_SCENE_ID;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return INVALID_SCENE_ID;
        }
    };

    context
        .scene_manager()
        .get_scene_by_name(name)
        .unwrap_or(INVALID_SCENE_ID)
}

// ============================================================================
// Active Scene Management
// ============================================================================

/// Sets whether a scene is active.
///
/// Active scenes participate in the game loop (update, render, etc.).
///
/// # Arguments
///
/// * `context_id` - The context containing the scene
/// * `scene_id` - The scene to activate or deactivate
/// * `active` - Whether the scene should be active
///
/// # Returns
///
/// A `GoudResult` indicating success or failure.
#[no_mangle]
pub extern "C" fn goud_scene_set_active(
    context_id: GoudContextId,
    scene_id: u32,
    active: bool,
) -> GoudResult {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return GoudResult::err(ERR_INTERNAL_ERROR),
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    match context.scene_manager_mut().set_active(scene_id, active) {
        Ok(()) => GoudResult::ok(),
        Err(err) => {
            let code = err.error_code();
            set_last_error(err);
            GoudResult::err(code)
        }
    }
}

/// Returns whether a scene is currently active.
///
/// # Arguments
///
/// * `context_id` - The context containing the scene
/// * `scene_id` - The scene to check
///
/// # Returns
///
/// `true` if the scene is active, `false` otherwise (including on error).
#[no_mangle]
pub extern "C" fn goud_scene_is_active(
    context_id: GoudContextId,
    scene_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return false,
    };

    context.scene_manager().is_active(scene_id)
}

// ============================================================================
// Scene Queries
// ============================================================================

/// Returns the number of scenes in the context.
///
/// # Arguments
///
/// * `context_id` - The context to query
///
/// # Returns
///
/// The number of occupied scenes, or 0 on error.
#[no_mangle]
pub extern "C" fn goud_scene_count(context_id: GoudContextId) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return 0;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return 0,
    };

    context.scene_manager().scene_count() as u32
}

// ============================================================================
// Current Scene Targeting
// ============================================================================

/// Sets the scene that subsequent entity/component FFI operations target.
///
/// After calling this, `world()` and `world_mut()` on the context will
/// return the World belonging to the specified scene.
///
/// # Arguments
///
/// * `context_id` - The context to modify
/// * `scene_id` - The scene to set as current
///
/// # Returns
///
/// A `GoudResult` indicating success or failure. Fails if the scene
/// does not exist.
#[no_mangle]
pub extern "C" fn goud_scene_set_current(
    context_id: GoudContextId,
    scene_id: u32,
) -> GoudResult {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return GoudResult::err(ERR_INTERNAL_ERROR),
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    // Verify the scene exists before setting it as current.
    if context.scene_manager().get_scene(scene_id).is_none() {
        let err = GoudError::ResourceNotFound(format!(
            "Scene id {} not found",
            scene_id
        ));
        let code = err.error_code();
        set_last_error(err);
        return GoudResult::err(code);
    }

    context.set_current_scene(scene_id);
    GoudResult::ok()
}

/// Returns the ID of the scene currently targeted by entity/component
/// FFI operations.
///
/// # Arguments
///
/// * `context_id` - The context to query
///
/// # Returns
///
/// The current scene ID, or `u32::MAX` on error.
#[no_mangle]
pub extern "C" fn goud_scene_get_current(context_id: GoudContextId) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return INVALID_SCENE_ID;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return INVALID_SCENE_ID,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return INVALID_SCENE_ID,
    };

    context.current_scene()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
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
}
