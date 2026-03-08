//! FFI functions for the [`EngineConfig`] builder.
//!
//! Provides opaque-handle based configuration and engine creation for
//! cross-language callers (C#, Python, TypeScript).

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::sdk::engine_config::EngineConfig;

/// Opaque handle to an `EngineConfig` on the heap.
type EngineConfigHandle = *mut std::ffi::c_void;

/// Creates a new default `EngineConfig` and returns an opaque handle.
///
/// # Ownership
/// The caller owns the returned handle and must free it with
/// [`goud_engine_config_destroy`] or consume it with [`goud_engine_create`].
#[no_mangle]
pub extern "C" fn goud_engine_config_create() -> EngineConfigHandle {
    let config = Box::new(EngineConfig::new());
    Box::into_raw(config) as EngineConfigHandle
}

/// Frees an `EngineConfig` handle without creating a context.
///
/// # Safety
/// `handle` must be a valid pointer returned by [`goud_engine_config_create`],
/// or null (in which case this is a no-op).
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_destroy(handle: EngineConfigHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: Caller guarantees handle is valid and not previously freed.
    let _ = Box::from_raw(handle as *mut EngineConfig);
}

/// Sets the window title on an `EngineConfig`.
///
/// # Safety
/// - `handle` must be a valid `EngineConfig` handle.
/// - `title` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_title(
    handle: EngineConfigHandle,
    title: *const c_char,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees title is a valid C string or null.
    let title_str = if title.is_null() {
        "GoudEngine"
    } else {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
            Err(_) => return false,
        }
    };
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().title = title_str.to_string();
    true
}

/// Sets the window size on an `EngineConfig`.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_size(
    handle: EngineConfigHandle,
    width: u32,
    height: u32,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    let gc = config.game_config_mut();
    gc.width = width;
    gc.height = height;
    true
}

/// Enables or disables vsync on an `EngineConfig`.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_vsync(
    handle: EngineConfigHandle,
    enabled: bool,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().vsync = enabled;
    true
}

/// Enables or disables fullscreen on an `EngineConfig`.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_fullscreen(
    handle: EngineConfigHandle,
    enabled: bool,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().fullscreen = enabled;
    true
}

/// Sets the target FPS on an `EngineConfig`.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_target_fps(
    handle: EngineConfigHandle,
    fps: u32,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().target_fps = fps;
    true
}

/// Enables or disables the FPS overlay on an `EngineConfig`.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_fps_overlay(
    handle: EngineConfigHandle,
    enabled: bool,
) -> bool {
    if handle.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().show_fps_overlay = enabled;
    true
}

/// Consumes an `EngineConfig` handle and creates a windowed engine context.
///
/// This follows the same pattern as [`goud_window_create`] but uses the
/// configuration from the builder instead of explicit parameters.
///
/// # Ownership
/// The `handle` is consumed (freed) by this call. The caller must NOT use it
/// again or call [`goud_engine_config_destroy`] on it.
///
/// # Returns
/// A context ID on success, or `GOUD_INVALID_CONTEXT_ID` on failure.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle that has not been freed.
#[cfg(feature = "native")]
#[no_mangle]
pub unsafe extern "C" fn goud_engine_create(
    handle: EngineConfigHandle,
) -> crate::ffi::context::GoudContextId {
    use crate::core::error::set_last_error;
    use crate::core::error::GoudError;
    use crate::ecs::InputManager;
    use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
    use crate::ffi::window::{set_window_state, WindowState};
    use crate::libs::graphics::backend::opengl::OpenGLBackend;
    use crate::libs::graphics::backend::StateOps;
    use crate::libs::platform::glfw_platform::GlfwPlatform;
    use crate::libs::platform::WindowConfig;

    if handle.is_null() {
        set_last_error(GoudError::InternalError(
            "Null EngineConfig handle".to_string(),
        ));
        return GOUD_INVALID_CONTEXT_ID;
    }

    // SAFETY: Caller guarantees handle is valid. We take ownership here.
    let engine_config = *Box::from_raw(handle as *mut EngineConfig);
    let (game_config, providers) = engine_config.build();

    let window_config = WindowConfig {
        width: game_config.width,
        height: game_config.height,
        title: game_config.title.clone(),
        vsync: game_config.vsync,
        resizable: game_config.resizable,
    };

    let platform = match GlfwPlatform::new(&window_config) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    let mut backend = match OpenGLBackend::new() {
        Ok(b) => b,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    backend.set_viewport(0, 0, game_config.width, game_config.height);

    let context_id: GoudContextId = {
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        match registry.create() {
            Ok(id) => id,
            Err(e) => {
                set_last_error(e);
                return GOUD_INVALID_CONTEXT_ID;
            }
        }
    };

    {
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        if let Some(context) = registry.get_mut(context_id) {
            context.world_mut().insert_resource(InputManager::new());
        }
    }

    // Store the provider registry so capability queries can access it via FFI.
    if !crate::ffi::providers::provider_registry_store(context_id, providers) {
        if let Ok(mut registry) = get_context_registry().lock() {
            let _ = registry.destroy(context_id);
        }
        set_last_error(GoudError::InternalError(
            "Failed to store provider registry".to_string(),
        ));
        return GOUD_INVALID_CONTEXT_ID;
    }

    let window_state = WindowState::new(platform, backend);

    if let Err(e) = set_window_state(context_id, window_state) {
        if let Ok(mut registry) = get_context_registry().lock() {
            let _ = registry.destroy(context_id);
        }
        set_last_error(e);
        return GOUD_INVALID_CONTEXT_ID;
    }

    context_id
}
