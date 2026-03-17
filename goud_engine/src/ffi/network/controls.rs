use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::core::providers::network_types::NetworkSimulationConfig;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::with_window_state;

use super::overlay::network_overlay_set_active_handle_override;
use super::registry::{with_instance, with_registry};

/// Returns the number of active connections, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_network_peer_count(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| Ok(inst.provider.connections().len() as i32));
    result.unwrap_or_else(|e| e)
}

/// Sets an explicit overlay-handle override for this context.
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_network_set_overlay_handle(context_id: GoudContextId, handle: i64) -> i32 {
    match with_registry(|reg| Ok(reg.instances.contains_key(&handle))) {
        Ok(true) => {
            if network_overlay_set_active_handle_override(context_id, Some(handle)) {
                if with_window_state(context_id, |state| {
                    state.network_overlay.set_active_handle(Some(handle));
                })
                .is_none()
                {
                    log::warn!(
                        "Failed to sync network overlay state for context {} after setting handle {}",
                        context_id,
                        handle
                    );
                }
                0
            } else {
                ERR_INTERNAL_ERROR
            }
        }
        Ok(false) => {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown network handle {}",
                handle
            )));
            ERR_INVALID_STATE
        }
        Err(code) => code,
    }
}

/// Clears any explicit overlay-handle override for this context.
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_network_clear_overlay_handle(context_id: GoudContextId) -> i32 {
    if network_overlay_set_active_handle_override(context_id, None) {
        if with_window_state(context_id, |state| {
            state.network_overlay.set_active_handle(None);
        })
        .is_none()
        {
            log::warn!(
                "Failed to sync network overlay state for context {} after clearing handle",
                context_id
            );
        }
        0
    } else {
        ERR_INTERNAL_ERROR
    }
}

#[cfg_attr(any(debug_assertions, test), allow(dead_code))]
pub(super) fn simulation_controls_unavailable() -> i32 {
    set_last_error(GoudError::InvalidState(
        "Network simulation controls are only available in debug/test builds".to_string(),
    ));
    ERR_INVALID_STATE
}

/// Applies a debug-only network simulation config to the provider handle.
/// Returns 0 on success, negative error code on failure.
#[cfg(any(debug_assertions, test))]
#[no_mangle]
pub extern "C" fn goud_network_set_simulation(
    _context_id: GoudContextId,
    handle: i64,
    config: NetworkSimulationConfig,
) -> i32 {
    if let Err(message) = config.validate() {
        set_last_error(GoudError::InvalidState(message));
        return ERR_INVALID_STATE;
    }

    let result = with_instance(handle, |inst| {
        inst.provider.set_simulation_config(config).map_err(|e| {
            let code = e.error_code();
            set_last_error(e);
            code
        })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Applies a debug-only network simulation config to the provider handle.
/// Returns `ERR_INVALID_STATE` in release builds because simulation hooks are
/// not compiled into release networking providers.
/// cbindgen:ignore
#[cfg(not(any(debug_assertions, test)))]
#[no_mangle]
pub extern "C" fn goud_network_set_simulation(
    _context_id: GoudContextId,
    handle: i64,
    config: NetworkSimulationConfig,
) -> i32 {
    let _ = (handle, config);
    simulation_controls_unavailable()
}

/// Clears any debug-only network simulation config from the provider handle.
/// Returns 0 on success, negative error code on failure.
#[cfg(any(debug_assertions, test))]
#[no_mangle]
pub extern "C" fn goud_network_clear_simulation(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| {
        inst.provider.clear_simulation_config().map_err(|e| {
            let code = e.error_code();
            set_last_error(e);
            code
        })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Clears any debug-only network simulation config from the provider handle.
/// Returns `ERR_INVALID_STATE` in release builds because simulation hooks are
/// not compiled into release networking providers.
/// cbindgen:ignore
#[cfg(not(any(debug_assertions, test)))]
#[no_mangle]
pub extern "C" fn goud_network_clear_simulation(_context_id: GoudContextId, handle: i64) -> i32 {
    let _ = handle;
    simulation_controls_unavailable()
}
