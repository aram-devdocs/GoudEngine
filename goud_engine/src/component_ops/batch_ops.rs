//! Batch component operations.
//!
//! Contains the `_impl` functions for adding, removing, and querying
//! components across multiple entities at once.

use std::collections::HashMap;

use crate::context_registry::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::core::error::{set_last_error, GoudError};

use super::helpers::context_key;
use super::storage::{get_context_storage_map, get_type_registry};

/// Adds the same component type to multiple entities in a batch.
///
/// # Safety
///
/// - `entity_ids` must point to valid memory with `count` u64 values
/// - `data_ptr` must point to `count * component_size` bytes of data
/// - `component_size` must match the registered component type size
pub unsafe fn component_add_batch_impl(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    data_ptr: *const u8,
    component_size: usize,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    if entity_ids.is_null() {
        set_last_error(GoudError::InvalidState(
            "entity_ids pointer is null".to_string(),
        ));
        return 0;
    }

    if data_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "data_ptr pointer is null".to_string(),
        ));
        return 0;
    }

    if count == 0 {
        return 0;
    }

    let type_info = {
        let type_registry = get_type_registry();
        let registry_map = match type_registry.as_ref() {
            Some(map) => map,
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {} not registered",
                    type_id_hash
                )));
                return 0;
            }
        };
        match registry_map.get(&type_id_hash) {
            Some(info) => info.clone(),
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {} not registered",
                    type_id_hash
                )));
                return 0;
            }
        }
    };

    if component_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component size mismatch: expected {}, got {}",
            type_info.size, component_size
        )));
        return 0;
    }

    let mut storage_map = get_context_storage_map();
    let map = storage_map.get_or_insert_with(HashMap::new);

    let key = context_key(context_id);
    let context_storage = map.entry(key).or_default();
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    // SAFETY: Caller guarantees entity_ids points to count u64 values.
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let mut success_count = 0u32;

    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        let component_data = data_ptr.add(i * component_size);
        if storage.insert(entity_bits, component_data) {
            success_count += 1;
        }
    }

    success_count
}

/// Removes the same component type from multiple entities in a batch.
///
/// # Safety
///
/// `entity_ids` must point to valid memory with `count` u64 values.
pub unsafe fn component_remove_batch_impl(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    if entity_ids.is_null() {
        set_last_error(GoudError::InvalidState(
            "entity_ids pointer is null".to_string(),
        ));
        return 0;
    }

    if count == 0 {
        return 0;
    }

    {
        let type_registry = get_type_registry();
        let registry_map = match type_registry.as_ref() {
            Some(map) => map,
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {} not registered",
                    type_id_hash
                )));
                return 0;
            }
        };
        if !registry_map.contains_key(&type_id_hash) {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return 0;
        }
    }

    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return 0,
    };

    let key = context_key(context_id);
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return 0,
    };

    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return 0,
    };

    // SAFETY: Caller guarantees entity_ids points to count u64 values.
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let mut success_count = 0u32;

    for &entity_bits in entity_slice {
        if storage.remove(entity_bits) {
            success_count += 1;
        }
    }

    success_count
}

/// Checks if multiple entities have a specific component type.
///
/// # Safety
///
/// - `entity_ids` must point to valid memory with `count` u64 values
/// - `out_results` must point to valid memory with `count` u8 values
pub unsafe fn component_has_batch_impl(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    out_results: *mut u8,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    if entity_ids.is_null() {
        set_last_error(GoudError::InvalidState(
            "entity_ids pointer is null".to_string(),
        ));
        return 0;
    }

    if out_results.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_results pointer is null".to_string(),
        ));
        return 0;
    }

    if count == 0 {
        return 0;
    }

    {
        let type_registry = get_type_registry();
        let registry_map = match type_registry.as_ref() {
            Some(map) => map,
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {} not registered",
                    type_id_hash
                )));
                return 0;
            }
        };
        if !registry_map.contains_key(&type_id_hash) {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return 0;
        }
    }

    let storage_map = get_context_storage_map();

    let storage_exists = storage_map.as_ref().is_some_and(|map| {
        let key = context_key(context_id);
        map.get(&key)
            .and_then(|cs| cs.get_storage(type_id_hash))
            .is_some()
    });

    // SAFETY: Caller guarantees these pointers are valid for count elements.
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);

    if !storage_exists {
        for result in results_slice.iter_mut() {
            *result = 0;
        }
        return count;
    }

    let map = storage_map.as_ref().unwrap();
    let key = context_key(context_id);
    let context_storage = map.get(&key).unwrap();
    let storage = context_storage.get_storage(type_id_hash).unwrap();

    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        results_slice[i] = if storage.contains(entity_bits) { 1 } else { 0 };
    }

    count
}
