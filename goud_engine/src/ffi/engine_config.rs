//! FFI functions for the [`EngineConfig`] builder.
//!
//! Provides opaque-handle based configuration and engine creation for
//! cross-language callers (C#, Python, TypeScript).

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::types::PhysicsBackend2D;
use crate::ffi::context::GoudDebuggerConfig;
use crate::sdk::engine_config::EngineConfig;
use crate::sdk::game_config::{RenderBackendKind, WindowBackendKind};

#[cfg(feature = "native")]
use crate::core::debugger::{self, ContextConfig, RuntimeSurfaceKind};

#[cfg(feature = "native")]
mod native;
#[cfg(test)]
mod tests;

#[cfg(feature = "native")]
pub use native::goud_engine_create;

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
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
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
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: Caller guarantees title is a valid C string or null.
    let title_str = if title.is_null() {
        "GoudEngine"
    } else {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(GoudError::InvalidState(
                    "title is not valid UTF-8".to_string(),
                ));
                return false;
            }
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
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
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
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().vsync = enabled;
    true
}

/// Sets the fullscreen mode on an `EngineConfig`.
///
/// Mode values:
/// - 0: Windowed
/// - 1: Borderless
/// - 2: Exclusive
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_fullscreen(
    handle: EngineConfigHandle,
    mode: u32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let fullscreen_mode = match crate::libs::platform::FullscreenMode::from_u32(mode) {
        Some(m) => m,
        None => {
            set_last_error(GoudError::InvalidState(
                "invalid fullscreen mode".to_string(),
            ));
            return false;
        }
    };

    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().fullscreen_mode = fullscreen_mode;
    true
}

/// Sets the aspect ratio lock on an `EngineConfig`.
///
/// Lock values:
/// - 0: Free (no lock)
/// - 1: 4:3
/// - 2: 16:9
/// - 3: 16:10
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_aspect_ratio_lock(
    handle: EngineConfigHandle,
    lock: u32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let aspect_lock = match crate::rendering::AspectRatioLock::from_u32(lock) {
        Some(l) => l,
        None => {
            set_last_error(GoudError::InvalidState(
                "invalid aspect ratio lock".to_string(),
            ));
            return false;
        }
    };

    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().aspect_ratio_lock = aspect_lock;
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
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
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
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().show_fps_overlay = enabled;
    true
}

/// Enables or disables the physics debug visualization overlay for physics worlds
/// created through this `EngineConfig` and provider registry. Standalone FFI
/// worlds created directly with `goud_physics_create` or `goud_physics3d_create`
/// are not affected.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_physics_debug(
    handle: EngineConfigHandle,
    enabled: bool,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().physics_debug.enabled = enabled;
    true
}

/// Sets the 2D physics backend used when building providers from `EngineConfig`.
///
/// Backend values:
/// - 0: Default
/// - 1: Rapier
/// - 2: Simple
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_physics_backend_2d(
    handle: EngineConfigHandle,
    backend: u32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let backend = match PhysicsBackend2D::from_u32(backend) {
        Some(backend) => backend,
        None => {
            set_last_error(GoudError::InvalidState(
                "invalid physics backend".to_string(),
            ));
            return false;
        }
    };

    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.set_physics_backend_2d(backend);
    true
}

/// Sets the native render backend used when creating windowed engines.
///
/// Backend values:
/// - 0: wgpu
/// - 1: legacy OpenGL
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_render_backend(
    handle: EngineConfigHandle,
    backend: u32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let backend = match RenderBackendKind::from_u32(backend) {
        Some(backend) => backend,
        None => {
            set_last_error(GoudError::InvalidState(
                "invalid render backend".to_string(),
            ));
            return false;
        }
    };

    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().render_backend = backend;
    true
}

/// Sets the native window backend used when creating windowed engines.
///
/// Backend values:
/// - 0: winit
/// - 1: legacy GLFW
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_window_backend(
    handle: EngineConfigHandle,
    backend: u32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let backend = match WindowBackendKind::from_u32(backend) {
        Some(backend) => backend,
        None => {
            set_last_error(GoudError::InvalidState(
                "invalid window backend".to_string(),
            ));
            return false;
        }
    };

    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().window_backend = backend;
    true
}

/// Configures debugger runtime startup for engines built from this `EngineConfig`.
///
/// # Safety
/// - `handle` must be a valid `EngineConfig` handle.
/// - `debugger` must be null or point to a valid [`GoudDebuggerConfig`] for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_debugger(
    handle: EngineConfigHandle,
    debugger: *const GoudDebuggerConfig,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    let debugger = if debugger.is_null() {
        crate::core::debugger::DebuggerConfig::default()
    } else {
        // SAFETY: `debugger` is validated non-null above and only read for the duration of this call.
        let debugger = unsafe { &*debugger };
        let route_label = if debugger.route_label.is_null() {
            None
        } else {
            // SAFETY: Caller guarantees `route_label` is null or a valid null-terminated UTF-8 string.
            match unsafe { CStr::from_ptr(debugger.route_label) }.to_str() {
                Ok(label) => Some(label.to_string()),
                Err(_) => {
                    set_last_error(GoudError::InvalidState(
                        "route_label is not valid UTF-8".to_string(),
                    ));
                    return false;
                }
            }
        };

        crate::core::debugger::DebuggerConfig {
            enabled: debugger.enabled,
            publish_local_attach: debugger.publish_local_attach,
            route_label,
        }
    };

    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = unsafe { &mut *(handle as *mut EngineConfig) };
    config.game_config_mut().debugger = debugger;
    true
}

/// Sets the fixed timestep step size in seconds on an `EngineConfig`.
///
/// Pass `0.0` to disable fixed timestep mode.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_fixed_timestep(
    handle: EngineConfigHandle,
    step: f32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().fixed_timestep = step.max(0.0);
    true
}

/// Sets the maximum fixed steps per frame on an `EngineConfig`.
///
/// Caps the accumulator to prevent a spiral of death.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_config_set_max_fixed_steps(
    handle: EngineConfigHandle,
    max: u32,
) -> bool {
    if handle.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    // SAFETY: Caller guarantees handle points to a valid EngineConfig.
    let config = &mut *(handle as *mut EngineConfig);
    config.game_config_mut().max_fixed_steps_per_frame = max.max(1);
    true
}
