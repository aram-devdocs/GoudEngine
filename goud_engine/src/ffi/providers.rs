//! Provider capability query and hot-swap FFI functions.
//!
//! Provides C-compatible functions for querying provider capabilities
//! and (in debug builds) hot-swapping providers at runtime.
//!
//! Provider registries are stored in a global registry keyed by context ID.
//! The `goud_engine_create` function stores the registry at creation time;
//! `provider_registry_remove` cleans it up at context destruction.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::error::{
    set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE, ERR_PROVIDER_NOT_FOUND,
    SUCCESS,
};
use crate::core::providers::input_types::InputCapabilities;
use crate::core::providers::network_types::NetworkCapabilities;
use crate::core::providers::types::{
    AudioCapabilities, DebugShape, PhysicsCapabilities, RenderCapabilities,
};
use crate::core::providers::ProviderRegistry;
use crate::ffi::context::GoudContextId;

// ============================================================================
// Provider Registry Storage
// ============================================================================

static PROVIDER_REGISTRIES: Mutex<Option<HashMap<GoudContextId, ProviderRegistry>>> =
    Mutex::new(None);

/// Helper to access the provider registries map under the lock.
fn with_registries<F, R>(f: F) -> Result<R, i32>
where
    F: FnOnce(&mut HashMap<GoudContextId, ProviderRegistry>) -> Result<R, i32>,
{
    let mut guard = PROVIDER_REGISTRIES.lock().map_err(|_| {
        set_last_error(GoudError::InternalError(
            "Failed to lock provider registry".to_string(),
        ));
        ERR_INTERNAL_ERROR
    })?;
    let map = guard.get_or_insert_with(HashMap::new);
    f(map)
}

/// Helper to access a specific provider registry by context ID.
fn with_registry<F, R>(context_id: GoudContextId, f: F) -> Result<R, i32>
where
    F: FnOnce(&mut ProviderRegistry) -> Result<R, i32>,
{
    with_registries(|map| {
        let reg = map.get_mut(&context_id).ok_or_else(|| {
            set_last_error(GoudError::InvalidState(format!(
                "No provider registry for context {}",
                context_id
            )));
            ERR_PROVIDER_NOT_FOUND
        })?;
        f(reg)
    })
}

// ============================================================================
// Internal Management (called from engine_config.rs)
// ============================================================================

/// Stores a `ProviderRegistry` for the given context ID.
///
/// Called during engine creation to make providers accessible via FFI.
/// Returns `false` if the lock is poisoned (should not happen in practice).
pub fn provider_registry_store(context_id: GoudContextId, registry: ProviderRegistry) -> bool {
    if let Ok(mut guard) = PROVIDER_REGISTRIES.lock() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(context_id, registry);
        true
    } else {
        false
    }
}

/// Removes the `ProviderRegistry` for the given context ID.
///
/// Called during context destruction to free provider resources.
pub fn provider_registry_remove(context_id: GoudContextId) {
    if let Ok(mut guard) = PROVIDER_REGISTRIES.lock() {
        if let Some(map) = guard.as_mut() {
            map.remove(&context_id);
        }
    }
}

/// Returns cached physics debug shapes for the given context.
///
/// Missing registries are treated as "no overlay data" so rendering callers can
/// stay on the fast path without surfacing context-management errors.
pub(crate) fn physics_debug_shapes(context_id: GoudContextId) -> Vec<DebugShape> {
    with_registry(context_id, |reg| Ok(reg.physics.debug_shapes())).unwrap_or_default()
}

// ============================================================================
// Capability Query FFI Functions
// ============================================================================

/// Queries the render provider's capabilities for the given context.
///
/// # Safety
///
/// `out` must point to a valid, writable `RenderCapabilities` struct.
/// Caller owns the memory; this function writes into it.
#[no_mangle]
pub unsafe extern "C" fn goud_provider_render_capabilities(
    context_id: GoudContextId,
    out: *mut RenderCapabilities,
) -> i32 {
    if out.is_null() {
        set_last_error(GoudError::InvalidState("out pointer is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_registry(context_id, |reg| {
        let caps = reg.render_capabilities().clone();
        // SAFETY: Caller guarantees `out` points to valid writable memory
        // for a RenderCapabilities struct.
        std::ptr::write(out, caps);
        Ok(SUCCESS)
    });
    result.unwrap_or_else(|e| e)
}

/// Queries the physics provider's capabilities for the given context.
///
/// # Safety
///
/// `out` must point to a valid, writable `PhysicsCapabilities` struct.
/// Caller owns the memory; this function writes into it.
#[no_mangle]
pub unsafe extern "C" fn goud_provider_physics_capabilities(
    context_id: GoudContextId,
    out: *mut PhysicsCapabilities,
) -> i32 {
    if out.is_null() {
        set_last_error(GoudError::InvalidState("out pointer is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_registry(context_id, |reg| {
        let caps = reg.physics_capabilities().clone();
        // SAFETY: Caller guarantees `out` points to valid writable memory
        // for a PhysicsCapabilities struct.
        std::ptr::write(out, caps);
        Ok(SUCCESS)
    });
    result.unwrap_or_else(|e| e)
}

/// Queries the audio provider's capabilities for the given context.
///
/// # Safety
///
/// `out` must point to a valid, writable `AudioCapabilities` struct.
/// Caller owns the memory; this function writes into it.
#[no_mangle]
pub unsafe extern "C" fn goud_provider_audio_capabilities(
    context_id: GoudContextId,
    out: *mut AudioCapabilities,
) -> i32 {
    if out.is_null() {
        set_last_error(GoudError::InvalidState("out pointer is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_registry(context_id, |reg| {
        let caps = reg.audio_capabilities().clone();
        // SAFETY: Caller guarantees `out` points to valid writable memory
        // for an AudioCapabilities struct.
        std::ptr::write(out, caps);
        Ok(SUCCESS)
    });
    result.unwrap_or_else(|e| e)
}

/// Queries the input provider's capabilities for the given context.
///
/// # Safety
///
/// `out` must point to a valid, writable `InputCapabilities` struct.
/// Caller owns the memory; this function writes into it.
#[no_mangle]
pub unsafe extern "C" fn goud_provider_input_capabilities(
    context_id: GoudContextId,
    out: *mut InputCapabilities,
) -> i32 {
    if out.is_null() {
        set_last_error(GoudError::InvalidState("out pointer is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_registry(context_id, |reg| {
        let caps = reg.input_capabilities().clone();
        // SAFETY: Caller guarantees `out` points to valid writable memory
        // for an InputCapabilities struct.
        std::ptr::write(out, caps);
        Ok(SUCCESS)
    });
    result.unwrap_or_else(|e| e)
}

/// Queries the network provider's capabilities for the given context.
///
/// Returns `ERR_PROVIDER_NOT_FOUND` if no network provider is installed.
///
/// # Safety
///
/// `out` must point to a valid, writable `NetworkCapabilities` struct.
/// Caller owns the memory; this function writes into it.
#[no_mangle]
pub unsafe extern "C" fn goud_provider_network_capabilities(
    context_id: GoudContextId,
    out: *mut NetworkCapabilities,
) -> i32 {
    if out.is_null() {
        set_last_error(GoudError::InvalidState("out pointer is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_registry(context_id, |reg| {
        match reg.network_capabilities() {
            Some(caps) => {
                let caps = caps.clone();
                // SAFETY: Caller guarantees `out` points to valid writable
                // memory for a NetworkCapabilities struct.
                std::ptr::write(out, caps);
                Ok(SUCCESS)
            }
            None => {
                set_last_error(GoudError::InvalidState(
                    "No network provider installed".to_string(),
                ));
                Err(ERR_PROVIDER_NOT_FOUND)
            }
        }
    });
    result.unwrap_or_else(|e| e)
}

// ============================================================================
// Hot-Swap FFI Functions (debug builds only)
// ============================================================================

/// Swaps the render provider to a null (no-op) provider at runtime.
///
/// Only available in debug builds. `provider_type` must be 0 (null provider).
/// Returns 0 on success, negative error code on failure.
#[cfg(debug_assertions)]
#[no_mangle]
pub extern "C" fn goud_provider_hot_swap_render(
    context_id: GoudContextId,
    provider_type: i32,
) -> i32 {
    use crate::core::providers::impls::NullRenderProvider;

    if provider_type != 0 {
        set_last_error(GoudError::InvalidState(format!(
            "Unknown render provider type: {}. Only 0 (null) is supported.",
            provider_type
        )));
        return ERR_INVALID_STATE;
    }

    let result = with_registry(context_id, |reg| {
        reg.swap_render(Box::new(NullRenderProvider::new()))
            .map_err(|e| {
                let code = e.error_code();
                set_last_error(e);
                code
            })?;
        Ok(SUCCESS)
    });
    result.unwrap_or_else(|e| e)
}

/// Stub for release builds -- hot-swap is not available.
/// cbindgen:ignore
#[cfg(not(debug_assertions))]
#[no_mangle]
pub extern "C" fn goud_provider_hot_swap_render(
    _context_id: GoudContextId,
    _provider_type: i32,
) -> i32 {
    set_last_error(GoudError::InvalidState(
        "Hot-swap is only available in debug builds".to_string(),
    ));
    ERR_INVALID_STATE
}

/// Checks if the hot-swap keyboard shortcut (F5) was pressed and
/// cycles the render provider to null. Debug builds only.
///
/// Returns 1 if a swap occurred, 0 if no key press, negative on error.
#[cfg(debug_assertions)]
#[no_mangle]
pub extern "C" fn goud_provider_check_hot_swap_shortcut(context_id: GoudContextId) -> i32 {
    use crate::ffi::input::{goud_input_key_just_pressed, KEY_F5};

    if !goud_input_key_just_pressed(context_id, KEY_F5) {
        return 0;
    }

    let result = goud_provider_hot_swap_render(context_id, 0);
    if result == SUCCESS {
        1
    } else {
        result
    }
}

/// Stub for release builds -- hot-swap shortcut is not available.
/// Always returns 0.
/// cbindgen:ignore
#[cfg(not(debug_assertions))]
#[no_mangle]
pub extern "C" fn goud_provider_check_hot_swap_shortcut(_context_id: GoudContextId) -> i32 {
    0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_registry::GoudContextId;
    use crate::core::providers::types::RenderCapabilities;

    /// Creates a unique test context ID. Uses high index values to avoid
    /// collisions with real contexts (generation=99).
    fn test_ctx(index: u32) -> GoudContextId {
        GoudContextId::new(index, 99)
    }

    #[test]
    fn test_store_and_query_render_capabilities() {
        let ctx_id = test_ctx(9999);
        let registry = ProviderRegistry::default();
        provider_registry_store(ctx_id, registry);

        let mut caps = RenderCapabilities::default();
        let result = unsafe { goud_provider_render_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, SUCCESS);
        assert_eq!(caps.max_texture_units, 0);
        assert!(!caps.supports_instancing);

        provider_registry_remove(ctx_id);
    }

    #[test]
    fn test_store_and_query_physics_capabilities() {
        let ctx_id = test_ctx(9998);
        let registry = ProviderRegistry::default();
        provider_registry_store(ctx_id, registry);

        let mut caps = PhysicsCapabilities::default();
        let result = unsafe { goud_provider_physics_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, SUCCESS);
        assert!(!caps.supports_continuous_collision);
        assert_eq!(caps.max_bodies, 0);

        provider_registry_remove(ctx_id);
    }

    #[test]
    fn test_store_and_query_audio_capabilities() {
        let ctx_id = test_ctx(9997);
        let registry = ProviderRegistry::default();
        provider_registry_store(ctx_id, registry);

        let mut caps = AudioCapabilities::default();
        let result = unsafe { goud_provider_audio_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, SUCCESS);
        assert!(!caps.supports_spatial);
        assert_eq!(caps.max_channels, 0);

        provider_registry_remove(ctx_id);
    }

    #[test]
    fn test_store_and_query_input_capabilities() {
        let ctx_id = test_ctx(9996);
        let registry = ProviderRegistry::default();
        provider_registry_store(ctx_id, registry);

        let mut caps = InputCapabilities::default();
        let result = unsafe { goud_provider_input_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, SUCCESS);
        assert!(!caps.supports_gamepad);
        assert_eq!(caps.max_gamepads, 0);

        provider_registry_remove(ctx_id);
    }

    #[test]
    fn test_network_capabilities_returns_not_found_when_no_provider() {
        let ctx_id = test_ctx(9995);
        let registry = ProviderRegistry::default();
        provider_registry_store(ctx_id, registry);

        let mut caps = NetworkCapabilities::default();
        let result = unsafe { goud_provider_network_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, ERR_PROVIDER_NOT_FOUND);

        provider_registry_remove(ctx_id);
    }

    #[test]
    fn test_null_out_pointer_returns_error() {
        let ctx_id = test_ctx(9994);
        let registry = ProviderRegistry::default();
        provider_registry_store(ctx_id, registry);

        let result = unsafe { goud_provider_render_capabilities(ctx_id, std::ptr::null_mut()) };
        assert_eq!(result, ERR_INVALID_STATE);

        provider_registry_remove(ctx_id);
    }

    #[test]
    fn test_missing_context_returns_error() {
        let ctx_id = test_ctx(88888);
        let mut caps = RenderCapabilities::default();
        let result = unsafe { goud_provider_render_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, ERR_PROVIDER_NOT_FOUND);
    }

    #[test]
    fn test_remove_cleans_up() {
        let ctx_id = test_ctx(9993);
        provider_registry_store(ctx_id, ProviderRegistry::default());
        provider_registry_remove(ctx_id);

        let mut caps = RenderCapabilities::default();
        let result = unsafe { goud_provider_render_capabilities(ctx_id, &mut caps as *mut _) };
        assert_eq!(result, ERR_PROVIDER_NOT_FOUND);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_hot_swap_render_null_provider() {
        let ctx_id = test_ctx(9992);
        provider_registry_store(ctx_id, ProviderRegistry::default());

        let result = goud_provider_hot_swap_render(ctx_id, 0);
        assert_eq!(result, SUCCESS);

        provider_registry_remove(ctx_id);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_hot_swap_render_invalid_type() {
        let ctx_id = test_ctx(9991);
        provider_registry_store(ctx_id, ProviderRegistry::default());

        let result = goud_provider_hot_swap_render(ctx_id, 99);
        assert_eq!(result, ERR_INVALID_STATE);

        provider_registry_remove(ctx_id);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_check_hot_swap_shortcut_returns_zero_when_no_key() {
        let ctx_id = test_ctx(9990);
        provider_registry_store(ctx_id, ProviderRegistry::default());

        // No input state set up, so F5 is not pressed -- should return 0.
        let result = goud_provider_check_hot_swap_shortcut(ctx_id);
        assert_eq!(result, 0);

        provider_registry_remove(ctx_id);
    }
}
