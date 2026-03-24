//! FFI functions for 3D scene management.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use std::ffi::CStr;
use std::os::raw::c_char;

/// Sentinel value returned when no scene is active or an operation fails.
pub const GOUD_INVALID_SCENE: u32 = u32::MAX;

// ============================================================================
// FFI: Scene Management
// ============================================================================

/// Creates a new named 3D scene and returns its ID.
///
/// Returns [`GOUD_INVALID_SCENE`] on failure.
///
/// # Safety
///
/// `name_ptr` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_create_scene(
    context_id: GoudContextId,
    name_ptr: *const c_char,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_SCENE;
    }

    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState("name_ptr is null".to_string()));
        return GOUD_INVALID_SCENE;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_SCENE;
    }

    // SAFETY: name_ptr was checked for null above; the caller guarantees it is
    // a valid NUL-terminated C string.
    let name = CStr::from_ptr(name_ptr);
    let name_str = name.to_string_lossy();

    with_renderer(context_id, |renderer| renderer.create_scene(&name_str))
        .unwrap_or(GOUD_INVALID_SCENE)
}

/// Destroys a scene by ID. Returns `true` if the scene existed.
#[no_mangle]
pub extern "C" fn goud_renderer3d_destroy_scene(context_id: GoudContextId, scene_id: u32) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| renderer.destroy_scene(scene_id)).unwrap_or(false)
}

/// Sets the current active scene. Returns `true` if the scene exists.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_current_scene(
    context_id: GoudContextId,
    scene_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| renderer.set_current_scene(scene_id)).unwrap_or(false)
}

/// Returns the current scene ID, or [`GOUD_INVALID_SCENE`] if none is active.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_current_scene(context_id: GoudContextId) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_SCENE;
    }

    with_renderer(context_id, |renderer| {
        renderer.get_current_scene().unwrap_or(GOUD_INVALID_SCENE)
    })
    .unwrap_or(GOUD_INVALID_SCENE)
}

/// Clears the current scene so that all objects are rendered.
#[no_mangle]
pub extern "C" fn goud_renderer3d_clear_current_scene(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.clear_current_scene();
        true
    })
    .unwrap_or(false)
}

/// Adds an object to a scene. Returns `true` on success.
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_object_to_scene(
    context_id: GoudContextId,
    scene_id: u32,
    object_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.add_object_to_scene(scene_id, object_id)
    })
    .unwrap_or(false)
}

/// Removes an object from a scene. Returns `true` if the scene contained it.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_object_from_scene(
    context_id: GoudContextId,
    scene_id: u32,
    object_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.remove_object_from_scene(scene_id, object_id)
    })
    .unwrap_or(false)
}

/// Adds a model to a scene. Returns `true` on success.
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_model_to_scene(
    context_id: GoudContextId,
    scene_id: u32,
    model_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.add_model_to_scene(scene_id, model_id)
    })
    .unwrap_or(false)
}

/// Removes a model from a scene. Returns `true` if the scene contained it.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_model_from_scene(
    context_id: GoudContextId,
    scene_id: u32,
    model_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.remove_model_from_scene(scene_id, model_id)
    })
    .unwrap_or(false)
}

/// Adds a light to a scene. Returns `true` on success.
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_light_to_scene(
    context_id: GoudContextId,
    scene_id: u32,
    light_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.add_light_to_scene(scene_id, light_id)
    })
    .unwrap_or(false)
}

/// Removes a light from a scene. Returns `true` if the scene contained it.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_light_from_scene(
    context_id: GoudContextId,
    scene_id: u32,
    light_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.remove_light_from_scene(scene_id, light_id)
    })
    .unwrap_or(false)
}
