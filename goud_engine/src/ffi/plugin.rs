//! Plugin registration FFI functions.
//!
//! Provides C-compatible functions for registering, unregistering, and
//! querying plugins within an engine context. Plugins are identified by
//! string IDs and tracked in a per-context runtime registry.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

// ============================================================================
// Plugin Registration
// ============================================================================

/// Registers a plugin by its string ID.
///
/// # Arguments
///
/// * `context_id` - The context to register the plugin in
/// * `plugin_id_ptr` - Pointer to UTF-8 encoded plugin ID bytes
/// * `plugin_id_len` - Length of the plugin ID in bytes
///
/// # Returns
///
/// `0` on success. A negative error code if the plugin is already registered
/// or inputs are invalid. Call `goud_get_last_error_message()` for details.
///
/// # Safety
///
/// Caller must ensure `plugin_id_ptr` points to valid UTF-8 data of at least
/// `plugin_id_len` bytes. Ownership is NOT transferred -- the caller retains
/// ownership of the buffer.
#[no_mangle]
pub unsafe extern "C" fn goud_plugin_register(
    context_id: GoudContextId,
    plugin_id_ptr: *const u8,
    plugin_id_len: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -(GoudError::InvalidContext.error_code());
    }

    if plugin_id_ptr.is_null() {
        set_last_error(GoudError::InvalidState("plugin_id_ptr is null".to_string()));
        return -ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees plugin_id_ptr is valid for plugin_id_len bytes.
    let id_bytes = std::slice::from_raw_parts(plugin_id_ptr, plugin_id_len as usize);
    let plugin_id = match std::str::from_utf8(id_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "plugin_id is not valid UTF-8".to_string(),
            ));
            return -ERR_INVALID_STATE;
        }
    };

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -(GoudError::InvalidContext.error_code());
        }
    };

    if !context.register_plugin(plugin_id) {
        set_last_error(GoudError::ResourceAlreadyExists(format!(
            "Plugin '{}' is already registered",
            plugin_id
        )));
        return -(GoudError::ResourceAlreadyExists(String::new()).error_code());
    }

    0 // SUCCESS
}

/// Unregisters a plugin by its string ID.
///
/// # Arguments
///
/// * `context_id` - The context to unregister the plugin from
/// * `plugin_id_ptr` - Pointer to UTF-8 encoded plugin ID bytes
/// * `plugin_id_len` - Length of the plugin ID in bytes
///
/// # Returns
///
/// `0` on success. A negative error code if the plugin was not registered
/// or inputs are invalid.
///
/// # Safety
///
/// Caller must ensure `plugin_id_ptr` points to valid UTF-8 data of at least
/// `plugin_id_len` bytes. Ownership is NOT transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_plugin_unregister(
    context_id: GoudContextId,
    plugin_id_ptr: *const u8,
    plugin_id_len: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -(GoudError::InvalidContext.error_code());
    }

    if plugin_id_ptr.is_null() {
        set_last_error(GoudError::InvalidState("plugin_id_ptr is null".to_string()));
        return -ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees plugin_id_ptr is valid for plugin_id_len bytes.
    let id_bytes = std::slice::from_raw_parts(plugin_id_ptr, plugin_id_len as usize);
    let plugin_id = match std::str::from_utf8(id_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "plugin_id is not valid UTF-8".to_string(),
            ));
            return -ERR_INVALID_STATE;
        }
    };

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -(GoudError::InvalidContext.error_code());
        }
    };

    if !context.unregister_plugin(plugin_id) {
        set_last_error(GoudError::ResourceNotFound(format!(
            "Plugin '{}' is not registered",
            plugin_id
        )));
        return -(GoudError::ResourceNotFound(String::new()).error_code());
    }

    0 // SUCCESS
}

// ============================================================================
// Plugin Queries
// ============================================================================

/// Checks whether a plugin with the given ID is registered.
///
/// # Arguments
///
/// * `context_id` - The context to query
/// * `plugin_id_ptr` - Pointer to UTF-8 encoded plugin ID bytes
/// * `plugin_id_len` - Length of the plugin ID in bytes
///
/// # Returns
///
/// `1` if the plugin is registered, `0` if not registered, or a negative
/// error code on failure.
///
/// # Safety
///
/// Caller must ensure `plugin_id_ptr` points to valid UTF-8 data of at least
/// `plugin_id_len` bytes. Ownership is NOT transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_plugin_is_registered(
    context_id: GoudContextId,
    plugin_id_ptr: *const u8,
    plugin_id_len: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -(GoudError::InvalidContext.error_code());
    }

    if plugin_id_ptr.is_null() {
        set_last_error(GoudError::InvalidState("plugin_id_ptr is null".to_string()));
        return -ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees plugin_id_ptr is valid for plugin_id_len bytes.
    let id_bytes = std::slice::from_raw_parts(plugin_id_ptr, plugin_id_len as usize);
    let plugin_id = match std::str::from_utf8(id_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "plugin_id is not valid UTF-8".to_string(),
            ));
            return -ERR_INVALID_STATE;
        }
    };

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -(GoudError::InvalidContext.error_code());
        }
    };

    if context.is_plugin_registered(plugin_id) {
        1
    } else {
        0
    }
}

/// Lists all registered plugin IDs as a newline-separated UTF-8 string.
///
/// Plugin IDs are written into `out_buf` separated by newline characters (`\n`).
/// The list is sorted alphabetically for deterministic output.
///
/// # Arguments
///
/// * `context_id` - The context to query
/// * `out_buf` - Pointer to a caller-allocated buffer for the result
/// * `buf_len` - Length of the output buffer in bytes
///
/// # Returns
///
/// The number of bytes written on success (which may be `0` if no plugins
/// are registered), or a negative error code on failure. If the buffer is
/// too small, returns `-ERR_INVALID_STATE` and sets the last error.
///
/// # Safety
///
/// Caller must ensure `out_buf` points to a writable buffer of at least
/// `buf_len` bytes. Ownership is NOT transferred -- the caller retains
/// ownership of the buffer.
#[no_mangle]
pub unsafe extern "C" fn goud_plugin_list(
    context_id: GoudContextId,
    out_buf: *mut u8,
    buf_len: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -(GoudError::InvalidContext.error_code());
    }

    if out_buf.is_null() && buf_len > 0 {
        set_last_error(GoudError::InvalidState("out_buf is null".to_string()));
        return -ERR_INVALID_STATE;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -(GoudError::InvalidContext.error_code());
        }
    };

    let plugins = context.registered_plugins();
    if plugins.is_empty() {
        return 0;
    }

    let joined = plugins.join("\n");
    let bytes = joined.as_bytes();

    if bytes.len() > buf_len as usize {
        set_last_error(GoudError::InvalidState(format!(
            "Buffer too small: need {} bytes, have {}",
            bytes.len(),
            buf_len
        )));
        return -ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees out_buf is writable for buf_len bytes,
    // and we verified bytes.len() <= buf_len.
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, bytes.len());

    bytes.len() as i32
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::context::goud_context_create;

    #[test]
    fn test_register_and_query_plugin() {
        let ctx = goud_context_create();
        assert_ne!(ctx, GOUD_INVALID_CONTEXT_ID);

        let id = b"my_physics_plugin";
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            let result = goud_plugin_register(ctx, id.as_ptr(), id.len() as u32);
            assert_eq!(result, 0);

            let check = goud_plugin_is_registered(ctx, id.as_ptr(), id.len() as u32);
            assert_eq!(check, 1);
        }
    }

    #[test]
    fn test_register_duplicate_returns_error() {
        let ctx = goud_context_create();
        let id = b"dup_plugin";
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            let r1 = goud_plugin_register(ctx, id.as_ptr(), id.len() as u32);
            assert_eq!(r1, 0);

            let r2 = goud_plugin_register(ctx, id.as_ptr(), id.len() as u32);
            assert!(r2 < 0); // Already exists
        }
    }

    #[test]
    fn test_unregister_plugin() {
        let ctx = goud_context_create();
        let id = b"removable";
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            goud_plugin_register(ctx, id.as_ptr(), id.len() as u32);

            let result = goud_plugin_unregister(ctx, id.as_ptr(), id.len() as u32);
            assert_eq!(result, 0);

            let check = goud_plugin_is_registered(ctx, id.as_ptr(), id.len() as u32);
            assert_eq!(check, 0);
        }
    }

    #[test]
    fn test_unregister_nonexistent_returns_error() {
        let ctx = goud_context_create();
        let id = b"nonexistent";
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            let result = goud_plugin_unregister(ctx, id.as_ptr(), id.len() as u32);
            assert!(result < 0);
        }
    }

    #[test]
    fn test_is_registered_not_found() {
        let ctx = goud_context_create();
        let id = b"missing";
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            let check = goud_plugin_is_registered(ctx, id.as_ptr(), id.len() as u32);
            assert_eq!(check, 0);
        }
    }

    #[test]
    fn test_list_plugins() {
        let ctx = goud_context_create();
        let p1 = b"beta_plugin";
        let p2 = b"alpha_plugin";
        // SAFETY: Test-controlled valid pointers and lengths.
        unsafe {
            goud_plugin_register(ctx, p1.as_ptr(), p1.len() as u32);
            goud_plugin_register(ctx, p2.as_ptr(), p2.len() as u32);

            let mut buf = [0u8; 256];
            let written = goud_plugin_list(ctx, buf.as_mut_ptr(), buf.len() as u32);
            assert!(written > 0);

            let output = std::str::from_utf8(&buf[..written as usize]).unwrap();
            // Should be sorted alphabetically
            assert_eq!(output, "alpha_plugin\nbeta_plugin");
        }
    }

    #[test]
    fn test_list_empty() {
        let ctx = goud_context_create();
        let mut buf = [0u8; 64];
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            let written = goud_plugin_list(ctx, buf.as_mut_ptr(), buf.len() as u32);
            assert_eq!(written, 0);
        }
    }

    #[test]
    fn test_list_buffer_too_small() {
        let ctx = goud_context_create();
        let id = b"a_long_plugin_name";
        // SAFETY: Test-controlled valid pointers and lengths.
        unsafe {
            goud_plugin_register(ctx, id.as_ptr(), id.len() as u32);

            let mut buf = [0u8; 2]; // Too small
            let result = goud_plugin_list(ctx, buf.as_mut_ptr(), buf.len() as u32);
            assert!(result < 0); // Error
        }
    }

    #[test]
    fn test_invalid_context() {
        let id = b"test";
        // SAFETY: Test-controlled valid pointer and length.
        unsafe {
            let r = goud_plugin_register(GOUD_INVALID_CONTEXT_ID, id.as_ptr(), id.len() as u32);
            assert!(r < 0);

            let r =
                goud_plugin_is_registered(GOUD_INVALID_CONTEXT_ID, id.as_ptr(), id.len() as u32);
            assert!(r < 0);
        }
    }

    #[test]
    fn test_null_pointer() {
        let ctx = goud_context_create();
        // SAFETY: Deliberately testing null pointer handling.
        unsafe {
            let r = goud_plugin_register(ctx, std::ptr::null(), 5);
            assert!(r < 0);

            let r = goud_plugin_is_registered(ctx, std::ptr::null(), 5);
            assert!(r < 0);
        }
    }
}
