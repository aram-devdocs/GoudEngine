use super::{debugger, ContextConfig, EngineConfig, EngineConfigHandle, RuntimeSurfaceKind};
use crate::core::error::{set_last_error, GoudError};
use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::{set_window_state, WindowState};
use crate::libs::platform::native_runtime::create_native_runtime;
use crate::libs::platform::WindowConfig;

/// Consumes an `EngineConfig` handle and creates a windowed engine context.
///
/// This follows the same pattern as
/// [`goud_window_create`](crate::ffi::window::goud_window_create)
/// but uses the configuration from the builder instead of explicit parameters.
///
/// # Ownership
/// The `handle` is consumed (freed) by this call. The caller must NOT use it
/// again or call [`super::goud_engine_config_destroy`] on it.
///
/// # Returns
/// A context ID on success, or `GOUD_INVALID_CONTEXT_ID` on failure.
///
/// # Safety
/// `handle` must be a valid `EngineConfig` handle that has not been freed.
#[no_mangle]
pub unsafe extern "C" fn goud_engine_create(handle: EngineConfigHandle) -> GoudContextId {
    if handle.is_null() {
        set_last_error(GoudError::InternalError(
            "Null EngineConfig handle".to_string(),
        ));
        return GOUD_INVALID_CONTEXT_ID;
    }

    // SAFETY: Caller guarantees handle is valid. We take ownership here.
    let engine_config = unsafe { *Box::from_raw(handle as *mut EngineConfig) };
    let (game_config, providers) = engine_config.build();

    let window_config = WindowConfig {
        width: game_config.width,
        height: game_config.height,
        title: game_config.title.clone(),
        vsync: game_config.vsync,
        resizable: game_config.resizable,
        msaa_samples: game_config.msaa_samples,
        fullscreen_mode: game_config.fullscreen_mode,
    };

    let native_runtime = match create_native_runtime(
        &window_config,
        game_config.window_backend,
        game_config.render_backend,
    ) {
        Ok(runtime) => runtime,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

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

        match registry.create_with_config(
            ContextConfig {
                debugger: game_config.debugger.clone(),
            },
            RuntimeSurfaceKind::WindowedGame,
        ) {
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

            if let Ok(am) = crate::assets::AudioManager::new() {
                context.world_mut().insert_resource(am);
            }
        }
    }

    if !crate::ffi::providers::provider_registry_store(context_id, providers) {
        if let Ok(mut registry) = get_context_registry().lock() {
            let _ = registry.destroy(context_id);
        }
        set_last_error(GoudError::InternalError(
            "Failed to store provider registry".to_string(),
        ));
        return GOUD_INVALID_CONTEXT_ID;
    }

    let mut window_state = WindowState::new(
        native_runtime.platform,
        native_runtime.render_backend,
        game_config.physics_debug.enabled,
        debugger::route_for_context(context_id),
        None,
    );
    window_state.fixed_timestep = game_config.fixed_timestep.max(0.0);
    window_state.max_fixed_steps = game_config.max_fixed_steps_per_frame.max(1);

    if let Err(e) = set_window_state(context_id, window_state) {
        if let Ok(mut registry) = get_context_registry().lock() {
            let _ = registry.destroy(context_id);
        }
        set_last_error(e);
        return GOUD_INVALID_CONTEXT_ID;
    }

    context_id
}
