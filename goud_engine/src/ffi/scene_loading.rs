//! Scene load/unload FFI functions.
//!
//! Provides C-compatible exports for loading a scene from JSON and unloading
//! a scene by name.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::types::GoudResult;

/// Sentinel value returned when scene loading fails.
const INVALID_SCENE_ID: u32 = u32::MAX;

/// Validates and decodes a UTF-8 string from an FFI byte pointer.
///
/// On failure, sets last error and returns an error code for callers that
/// need to produce a `GoudResult`.
unsafe fn parse_utf8_arg<'a>(
    ptr: *const u8,
    len: u32,
    null_message: &str,
    utf8_message: &str,
    error_return_code: i32,
) -> Result<&'a str, i32> {
    if ptr.is_null() {
        set_last_error(GoudError::InvalidState(null_message.to_string()));
        return Err(error_return_code);
    }

    // SAFETY: Caller guarantees `ptr` is valid for `len` bytes.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    match std::str::from_utf8(bytes) {
        Ok(value) => Ok(value),
        Err(_) => {
            set_last_error(GoudError::InvalidState(utf8_message.to_string()));
            Err(error_return_code)
        }
    }
}

/// Loads a scene from JSON.
///
/// # Safety
///
/// Caller must ensure both `name_ptr` and `json_ptr` are valid for their
/// respective lengths. Ownership is not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_scene_load(
    context_id: GoudContextId,
    name_ptr: *const u8,
    name_len: u32,
    json_ptr: *const u8,
    json_len: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return INVALID_SCENE_ID;
    }

    let invalid_state_code = GoudError::InvalidState(String::new()).error_code();

    // SAFETY: FFI caller contract guarantees pointer/length validity.
    let name = match unsafe {
        parse_utf8_arg(
            name_ptr,
            name_len,
            "name_ptr is null",
            "scene name is not valid UTF-8",
            invalid_state_code,
        )
    } {
        Ok(value) => value,
        Err(_) => return INVALID_SCENE_ID,
    };

    // SAFETY: FFI caller contract guarantees pointer/length validity.
    let json = match unsafe {
        parse_utf8_arg(
            json_ptr,
            json_len,
            "json_ptr is null",
            "scene json is not valid UTF-8",
            invalid_state_code,
        )
    } {
        Ok(value) => value,
        Err(_) => return INVALID_SCENE_ID,
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

    match context.load_scene_from_json(name, json) {
        Ok(id) => id,
        Err(err) => {
            set_last_error(err);
            INVALID_SCENE_ID
        }
    }
}

/// Unloads a scene by name.
///
/// # Safety
///
/// Caller must ensure `name_ptr` is valid for `name_len` bytes.
/// Ownership is not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_scene_unload(
    context_id: GoudContextId,
    name_ptr: *const u8,
    name_len: u32,
) -> GoudResult {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    let invalid_state_code = GoudError::InvalidState(String::new()).error_code();

    // SAFETY: FFI caller contract guarantees pointer/length validity.
    let name = match unsafe {
        parse_utf8_arg(
            name_ptr,
            name_len,
            "name_ptr is null",
            "scene name is not valid UTF-8",
            invalid_state_code,
        )
    } {
        Ok(value) => value,
        Err(code) => return GoudResult::err(code),
    };

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return GoudResult::err(ERR_INTERNAL_ERROR);
        }
    };

    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    match context.unload_scene_by_name(name) {
        Ok(()) => GoudResult::ok(),
        Err(err) => {
            let code = err.error_code();
            set_last_error(err);
            GoudResult::err(code)
        }
    }
}

#[cfg(test)]
#[path = "scene_loading_tests.rs"]
mod tests;
