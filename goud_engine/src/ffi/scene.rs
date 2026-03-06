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
        set_last_error(GoudError::InvalidState("name_ptr is null".to_string()));
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
pub extern "C" fn goud_scene_destroy(context_id: GoudContextId, scene_id: u32) -> GoudResult {
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
        set_last_error(GoudError::InvalidState("name_ptr is null".to_string()));
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
pub extern "C" fn goud_scene_is_active(context_id: GoudContextId, scene_id: u32) -> bool {
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
pub extern "C" fn goud_scene_set_current(context_id: GoudContextId, scene_id: u32) -> GoudResult {
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

    match context.set_current_scene(scene_id) {
        Ok(()) => GoudResult::ok(),
        Err(err) => {
            let code = err.error_code();
            set_last_error(err);
            GoudResult::err(code)
        }
    }
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
#[path = "scene_tests.rs"]
mod tests;
