//! Read-only component access FFI functions.
//!
//! Provides `goud_component_has`, `goud_component_get`, and
//! `goud_component_get_mut` — queries that inspect component state without
//! structural mutation of the storage.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::get_context_registry;
use crate::ffi::{GoudContextId, GoudEntityId, GOUD_INVALID_CONTEXT_ID};

use super::registry::get_component_type_info;
use super::storage::{context_key, entity_from_ffi, get_context_storage_map};

// ============================================================================
// FFI Functions - Component Query
// ============================================================================

/// Checks if an entity has a specific component.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to check
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// `true` if entity has the component, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_component_has(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> bool {
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return false;
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Check entity is alive using context registry
    {
        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return false;
            }
        };
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return false;
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            return false;
        }
    }

    // Get component storage for this context
    let storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => return false,
    };
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return false, // No storage exists
    };

    let key = context_key(context_id);

    // Get context storage
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return false, // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return false, // No storage for this type
    };

    // Check if entity has the component
    storage.contains(entity_id.bits())
}

/// Gets a read-only pointer to a component on an entity.
///
/// The returned pointer is valid until the next mutable operation on the World.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to get the component from
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// Pointer to the component data, or null if not found.
///
/// # Safety
///
/// - Returned pointer is only valid until next mutable World operation
/// - Caller must not modify data through this pointer
/// - Caller must not retain pointer across FFI calls
#[no_mangle]
pub extern "C" fn goud_component_get(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *const u8 {
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return std::ptr::null();
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null();
    }

    // Check entity is alive using context registry
    {
        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return std::ptr::null();
            }
        };
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return std::ptr::null();
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return std::ptr::null();
        }
    }

    // Get component storage for this context
    let storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => return std::ptr::null(),
    };
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return std::ptr::null(), // No storage exists
    };

    let key = context_key(context_id);

    // Get context storage
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return std::ptr::null(), // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null(), // No storage for this type
    };

    // Get component data pointer
    storage.get(entity_id.bits())
}

/// Gets a mutable pointer to a component on an entity.
///
/// The returned pointer is valid until the next operation on the World.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to get the component from
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// Mutable pointer to the component data, or null if not found.
///
/// # Safety
///
/// - Returned pointer is only valid until next World operation
/// - Caller must ensure exclusive access (no concurrent reads/writes)
/// - Caller must not retain pointer across FFI calls
#[no_mangle]
pub extern "C" fn goud_component_get_mut(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *mut u8 {
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return std::ptr::null_mut();
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null_mut();
    }

    // Check entity is alive using context registry
    {
        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return std::ptr::null_mut();
            }
        };
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return std::ptr::null_mut();
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return std::ptr::null_mut();
        }
    }

    // Get component storage for this context
    let mut storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return std::ptr::null_mut(), // No storage exists
    };

    let key = context_key(context_id);

    // Get context storage
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return std::ptr::null_mut(), // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null_mut(), // No storage for this type
    };

    // Get mutable component data pointer
    storage.get_mut(entity_id.bits())
}
