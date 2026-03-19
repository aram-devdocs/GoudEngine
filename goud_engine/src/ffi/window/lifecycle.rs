//! # Window Lifecycle FFI
//!
//! FFI functions for creating and destroying windowed contexts:
//! [`goud_window_create`](self::goud_window_create) and
//! [`goud_window_destroy`](super::destroy::goud_window_destroy).

use crate::assets::AudioManager;
use crate::core::error::{set_last_error, GoudError};
use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::platform::native_runtime::create_native_runtime;
use crate::libs::platform::WindowConfig;
use crate::sdk::game_config::{RenderBackendKind, WindowBackendKind};
use std::ffi::CStr;
use std::os::raw::c_char;

use super::state::{remove_window_state, set_window_state, WindowState};

/// Creates a new windowed context with the default native runtime.
///
/// This creates:
/// - A native window with the specified dimensions and title
/// - An ECS World with InputManager resource
/// - The default `winit + wgpu` rendering runtime
///
/// # Arguments
///
/// * `width` - Window width in pixels
/// * `height` - Window height in pixels
/// * `title` - Window title as a null-terminated C string
///
/// # Returns
///
/// A context ID on success, or `GOUD_INVALID_CONTEXT_ID` on failure.
///
/// # Safety
///
/// The `title` pointer must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn goud_window_create(
    width: u32,
    height: u32,
    title: *const c_char,
) -> GoudContextId {
    // SAFETY: Caller guarantees `title` is a valid C string or null.
    let title_str = if title.is_null() {
        "GoudEngine"
    } else {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Invalid UTF-8 in window title".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        }
    };

    let config = WindowConfig {
        width,
        height,
        title: title_str.to_string(),
        vsync: true,
        resizable: true,
        msaa_samples: 1,
        ..Default::default()
    };

    let native_runtime =
        match create_native_runtime(&config, WindowBackendKind::Winit, RenderBackendKind::Wgpu) {
            Ok(runtime) => runtime,
            Err(e) => {
                set_last_error(e);
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

    let context_id = {
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
            if let Ok(am) = AudioManager::new() {
                context.world_mut().insert_resource(am);
            }
        }
    }

    let window_state = WindowState::new(
        native_runtime.platform,
        native_runtime.render_backend,
        false,
        None,
        None,
    );

    if let Err(e) = set_window_state(context_id, window_state) {
        if let Ok(mut registry) = get_context_registry().lock() {
            let _ = registry.destroy(context_id);
        }
        set_last_error(e);
        return GOUD_INVALID_CONTEXT_ID;
    }

    context_id
}

/// Destroys a windowed context and releases all resources.
///
/// This destroys the window, OpenGL context, and ECS world.
///
/// # Arguments
///
/// * `context_id` - The windowed context to destroy
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_window_destroy(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    crate::ffi::renderer::cleanup_text_state(context_id);

    remove_window_state(context_id);

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return false;
        }
    };

    match registry.destroy(context_id) {
        Ok(()) => true,
        Err(e) => {
            set_last_error(e);
            false
        }
    }
}
