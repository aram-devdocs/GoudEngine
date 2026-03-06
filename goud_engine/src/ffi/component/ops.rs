//! Component write operations for the FFI layer.
//!
//! Provides `goud_component_register_type`, `goud_component_add`, and
//! `goud_component_remove` -- the functions that mutate component state.
//! Read-only queries (`has`, `get`, `get_mut`) live in `access.rs`.

use std::collections::HashMap;

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::get_context_registry;
use crate::ffi::{GoudContextId, GoudEntityId, GoudResult, GOUD_INVALID_CONTEXT_ID};

use super::registry::{get_component_type_info, register_component_type_internal};
use super::storage::{context_key, entity_from_ffi, get_context_storage_map};

// ============================================================================
// FFI Functions - Type Registration
// ============================================================================

/// Registers a component type with the engine.
///
/// This must be called once for each component type before it can be used.
/// The type ID hash should be stable across runs (e.g., from a const in C#).
///
/// # Parameters
///
/// - `type_id_hash`: Unique 64-bit identifier for the component type
/// - `name_ptr`: Null-terminated C string with the type name (for debugging, currently unused)
/// - `name_len`: Length of the name string (excluding null terminator)
/// - `size`: Size of the component in bytes
/// - `align`: Alignment of the component in bytes
///
/// # Returns
///
/// `true` if the type was newly registered, `false` if already registered.
///
/// # Safety
///
/// - `name_ptr` must be a valid pointer to a C string (or null)
/// - `name_len` must match the actual string length
/// - `size` and `align` must match the actual type layout
#[no_mangle]
pub unsafe extern "C" fn goud_component_register_type(
    type_id_hash: u64,
    name_ptr: *const u8,
    name_len: usize,
    size: usize,
    align: usize,
) -> bool {
    // Validate name pointer (kept for API compatibility, name is not stored)
    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component type name pointer is null".to_string(),
        ));
        return false;
    }

    // Validate name is valid UTF-8 (kept for API compatibility)
    let name_slice = std::slice::from_raw_parts(name_ptr, name_len);
    if std::str::from_utf8(name_slice).is_err() {
        set_last_error(GoudError::InvalidState(
            "Component type name is not valid UTF-8".to_string(),
        ));
        return false;
    }

    register_component_type_internal(type_id_hash, size, align)
}

// ============================================================================
// FFI Functions - Component Add/Remove
// ============================================================================

/// Adds a component to an entity.
///
/// If the entity already has this component type, it will be replaced.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to add the component to
/// - `type_id_hash`: Type ID of the component (from registration)
/// - `data_ptr`: Pointer to the component data
/// - `data_size`: Size of the component data in bytes
///
/// # Returns
///
/// `GoudResult` with success=true if component was added, error otherwise.
///
/// # Safety
///
/// - `data_ptr` must point to valid component data
/// - `data_size` must match the registered size
/// - Memory at `data_ptr` must remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn goud_component_add(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
    data_ptr: *const u8,
    data_size: usize,
) -> GoudResult {
    // Validate inputs
    if data_ptr.is_null() {
        return GoudResult::from_error(GoudError::InvalidState(
            "Component data pointer is null".to_string(),
        ));
    }

    // Look up type info
    let type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            return GoudResult::from_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
        }
    };

    // Validate size
    if data_size != type_info.size {
        return GoudResult::from_error(GoudError::InvalidState(format!(
            "Component data size mismatch: expected {}, got {}",
            type_info.size, data_size
        )));
    }

    // Validate context and entity
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return GoudResult::from_error(GoudError::InvalidContext);
    }

    // Check entity is alive using context registry
    {
        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                return GoudResult::from_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
            }
        };
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                return GoudResult::from_error(GoudError::InvalidContext);
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            return GoudResult::from_error(GoudError::EntityNotFound);
        }
    }

    // Get or create component storage for this context
    let mut storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            return GoudResult::from_error(GoudError::InternalError(
                "Failed to access component storage".to_string(),
            ));
        }
    };
    let map = storage_map.get_or_insert_with(HashMap::new);

    let key = context_key(context_id);
    let context_storage = map.entry(key).or_default();
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    // Insert the component data
    if storage.insert(entity_id.bits(), data_ptr) {
        GoudResult::ok()
    } else {
        GoudResult::from_error(GoudError::InternalError(
            "Failed to allocate component storage".to_string(),
        ))
    }
}

/// Removes a component from an entity.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to remove the component from
/// - `type_id_hash`: Type ID of the component to remove
///
/// # Returns
///
/// `GoudResult` with success=true if component was removed, error otherwise.
#[no_mangle]
pub extern "C" fn goud_component_remove(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> GoudResult {
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        return GoudResult::from_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return GoudResult::from_error(GoudError::InvalidContext);
    }

    // Check entity is alive using context registry
    {
        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                return GoudResult::from_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
            }
        };
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                return GoudResult::from_error(GoudError::InvalidContext);
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            return GoudResult::from_error(GoudError::EntityNotFound);
        }
    }

    // Get component storage for this context
    let mut storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            return GoudResult::from_error(GoudError::InternalError(
                "Failed to access component storage".to_string(),
            ));
        }
    };
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => {
            // No component storage exists, so entity can't have the component
            return GoudResult::ok();
        }
    };

    let key = context_key(context_id);

    // Get context storage
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => {
            // No storage for this context, so entity can't have the component
            return GoudResult::ok();
        }
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => {
            // No storage for this type, so entity can't have the component
            return GoudResult::ok();
        }
    };

    // Remove the component
    storage.remove(entity_id.bits());
    GoudResult::ok()
}
