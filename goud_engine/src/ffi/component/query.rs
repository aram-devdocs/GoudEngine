//! Bulk component query FFI functions.
//!
//! Provides `goud_component_count`, `goud_component_get_entities`, and
//! `goud_component_get_all` -- queries that enumerate all entities with a
//! given component type without requiring entity IDs up-front.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::registry::get_component_type_info;
use super::storage::{context_key, get_context_storage_map};

// ============================================================================
// FFI Functions - Bulk Component Queries
// ============================================================================

/// Returns the number of entities that currently have the given component type.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// The entity count, or 0 if the context/type is invalid, unregistered, or has
/// no storage.  Callers that need to distinguish "empty" from "error" should
/// check `goud_get_last_error()` — an error is set for invalid contexts and
/// unregistered types, but not for legitimately empty queries.
#[no_mangle]
pub extern "C" fn goud_component_count(context_id: GoudContextId, type_id_hash: u64) -> u32 {
    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    // Verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return 0;
    }

    // Get component storage for this context
    let storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to lock component storage".to_string(),
            ));
            return 0;
        }
    };
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return 0,
    };

    let key = context_key(context_id);

    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return 0,
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return 0,
    };

    storage.count() as u32
}

/// Copies entity IDs (as raw `u64` bits) for all entities that have the given
/// component into the caller-provided buffer.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `type_id_hash`: Type ID of the component
/// - `out_entities`: Caller-allocated buffer for entity bits
/// - `max_count`: Maximum number of entries the buffer can hold
///
/// # Returns
///
/// The number of entity IDs actually written, or 0 on error.
///
/// # Safety
///
/// - `out_entities` must point to a valid buffer of at least `max_count * 8` bytes.
/// - The buffer must remain valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn goud_component_get_entities(
    context_id: GoudContextId,
    type_id_hash: u64,
    out_entities: *mut u64,
    max_count: u32,
) -> u32 {
    // Null-check output pointer
    if out_entities.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_entities pointer is null".to_string(),
        ));
        return 0;
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    // Verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return 0;
    }

    // Get component storage for this context
    let storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to lock component storage".to_string(),
            ));
            return 0;
        }
    };
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return 0,
    };

    let key = context_key(context_id);

    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return 0,
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return 0,
    };

    let entities = storage.dense_entities();
    let copy_count = entities.len().min(max_count as usize);

    // SAFETY: out_entities is non-null (checked above) and the caller guarantees
    // the buffer holds at least `max_count` u64 entries. `copy_count <= max_count`.
    std::ptr::copy_nonoverlapping(entities.as_ptr(), out_entities, copy_count);

    copy_count as u32
}

/// Copies entity IDs and data pointers for all entities that have the given
/// component into the caller-provided buffers.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `type_id_hash`: Type ID of the component
/// - `out_entities`: Caller-allocated buffer for entity bits
/// - `out_data_ptrs`: Caller-allocated buffer for data pointers (read-only)
/// - `max_count`: Maximum number of entries each buffer can hold
///
/// # Returns
///
/// The number of entries actually written, or 0 on error.
///
/// # Safety
///
/// - `out_entities` must point to a valid buffer of at least `max_count * 8` bytes.
/// - `out_data_ptrs` must point to a valid buffer of at least `max_count` pointer-sized entries.
/// - The buffers must remain valid for the duration of this call.
/// - Returned data pointers are valid only until the next mutable World operation.
#[no_mangle]
pub unsafe extern "C" fn goud_component_get_all(
    context_id: GoudContextId,
    type_id_hash: u64,
    out_entities: *mut u64,
    out_data_ptrs: *mut *const u8,
    max_count: u32,
) -> u32 {
    // Null-check output pointers
    if out_entities.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_entities pointer is null".to_string(),
        ));
        return 0;
    }
    if out_data_ptrs.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_data_ptrs pointer is null".to_string(),
        ));
        return 0;
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    // Verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return 0;
    }

    // Get component storage for this context
    let storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to lock component storage".to_string(),
            ));
            return 0;
        }
    };
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return 0,
    };

    let key = context_key(context_id);

    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return 0,
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return 0,
    };

    let entities = storage.dense_entities();
    let data = storage.dense_data();
    let copy_count = entities.len().min(max_count as usize);

    // SAFETY: out_entities is non-null (checked above) and the caller guarantees
    // the buffer holds at least `max_count` u64 entries. `copy_count <= max_count`.
    std::ptr::copy_nonoverlapping(entities.as_ptr(), out_entities, copy_count);

    // SAFETY: out_data_ptrs is non-null (checked above) and the caller guarantees
    // the buffer holds at least `max_count` pointer entries.  `copy_count <= max_count`.
    // The cast `*const *mut u8` -> `*const *const u8` is a pointer-mutability
    // covariance cast: both types are pointer-sized with identical representation
    // on all platforms.  The caller receives the pointers as `*const u8` (read-only
    // contract documented in the function's doc comment).
    std::ptr::copy_nonoverlapping(data.as_ptr() as *const *const u8, out_data_ptrs, copy_count);

    copy_count as u32
}
