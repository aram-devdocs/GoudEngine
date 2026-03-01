//! Single-entity component operations.
//!
//! Contains the `_impl` functions for register, add, remove, has, get, and
//! get_mut on individual entities.

use std::collections::HashMap;

use crate::core::context_registry::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::core::error::{set_last_error, GoudError};
use crate::core::types::{GoudEntityId, GoudResult};

use super::helpers::{context_key, entity_from_ffi};
use super::storage::{
    get_component_type_info, get_context_storage_map, register_component_type_internal,
};

/// Registers a component type with the engine.
///
/// # Safety
///
/// - `name_ptr` must be a valid pointer to UTF-8 data (or null)
/// - `name_len` must match the actual string length
/// - `size` and `align` must match the actual type layout
pub unsafe fn component_register_type_impl(
    type_id_hash: u64,
    name_ptr: *const u8,
    name_len: usize,
    size: usize,
    align: usize,
) -> bool {
    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component type name pointer is null".to_string(),
        ));
        return false;
    }

    // SAFETY: Caller guarantees name_ptr points to valid memory of
    // name_len bytes.
    let name_slice = std::slice::from_raw_parts(name_ptr, name_len);
    if std::str::from_utf8(name_slice).is_err() {
        set_last_error(GoudError::InvalidState(
            "Component type name is not valid UTF-8".to_string(),
        ));
        return false;
    }

    register_component_type_internal(type_id_hash, size, align)
}

/// Adds a component to an entity.
///
/// # Safety
///
/// - `data_ptr` must point to valid component data
/// - `data_size` must match the registered component size
pub unsafe fn component_add_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
    data_ptr: *const u8,
    data_size: usize,
) -> GoudResult {
    if data_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component data pointer is null".to_string(),
        ));
        return GoudResult::err(902);
    }

    let type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return GoudResult::err(101);
        }
    };

    if data_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component data size mismatch: expected {}, got {}",
            type_info.size, data_size
        )));
        return GoudResult::err(902);
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(3);
    }

    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GoudResult::err(3);
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return GoudResult::err(300);
        }
    }

    let mut storage_map = get_context_storage_map();
    let map = storage_map.get_or_insert_with(HashMap::new);

    let key = context_key(context_id);
    let context_storage = map.entry(key).or_default();
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    if storage.insert(entity_id.bits(), data_ptr) {
        GoudResult::ok()
    } else {
        set_last_error(GoudError::InternalError(
            "Failed to allocate component storage".to_string(),
        ));
        GoudResult::err(900)
    }
}

/// Removes a component from an entity.
pub fn component_remove_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> GoudResult {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return GoudResult::err(101);
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(3);
    }

    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GoudResult::err(3);
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return GoudResult::err(300);
        }
    }

    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return GoudResult::ok(),
    };

    let key = context_key(context_id);
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return GoudResult::ok(),
    };

    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return GoudResult::ok(),
    };

    storage.remove(entity_id.bits());
    GoudResult::ok()
}

/// Checks if an entity has a specific component.
pub fn component_has_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> bool {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return false;
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    {
        let registry = get_context_registry().lock().unwrap();
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

    let storage_map = get_context_storage_map();
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return false,
    };

    let key = context_key(context_id);
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return false,
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return false,
    };

    storage.contains(entity_id.bits())
}

/// Gets a read-only pointer to a component on an entity.
pub fn component_get_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *const u8 {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return std::ptr::null();
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null();
    }

    {
        let registry = get_context_registry().lock().unwrap();
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

    let storage_map = get_context_storage_map();
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return std::ptr::null(),
    };

    let key = context_key(context_id);
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return std::ptr::null(),
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null(),
    };

    storage.get(entity_id.bits())
}

/// Gets a mutable pointer to a component on an entity.
pub fn component_get_mut_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *mut u8 {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return std::ptr::null_mut();
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null_mut();
    }

    {
        let registry = get_context_registry().lock().unwrap();
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

    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return std::ptr::null_mut(),
    };

    let key = context_key(context_id);
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    storage.get_mut(entity_id.bits())
}
