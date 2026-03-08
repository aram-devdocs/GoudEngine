//! Batch component FFI operations.
//!
//! Provides `goud_component_add_batch`, `goud_component_remove_batch`, and
//! `goud_component_has_batch` for operating on many entities in a single call.
//! These are more efficient than the single-entity equivalents when working
//! with large groups of entities.

use std::collections::HashMap;

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::registry::get_type_registry;
use super::storage::{context_key, get_context_storage_map};

// ============================================================================
// Batch Operations
// ============================================================================

/// Adds the same component type to multiple entities in a single batch.
///
/// This is more efficient than calling `goud_component_add()` multiple times
/// when adding the same component type to many entities.
///
/// # Arguments
///
/// * `context_id` - The context containing the entities
/// * `entity_ids` - Pointer to array of entity IDs
/// * `count` - Number of entities
/// * `type_id_hash` - Hash of the component type
/// * `data_ptr` - Pointer to array of component data (size = count * component_size)
/// * `component_size` - Size of a single component instance
///
/// # Returns
///
/// The number of entities that successfully received the component.
///
/// # Safety
///
/// Caller must ensure:
/// - `entity_ids` points to valid memory with `count` u64 values
/// - `data_ptr` points to valid memory with `count * component_size` bytes
/// - `component_size` matches the registered component type size
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `RESOURCE_ERROR_BASE + 100` (ResourceNotFound) - Component type not registered
/// - `INTERNAL_ERROR_BASE + 2` (InvalidState) - Size mismatch
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// Position[] positions = {
///     new Position { x = 0, y = 0 },
///     new Position { x = 10, y = 10 },
///     new Position { x = 20, y = 20 },
///     new Position { x = 30, y = 30 },
///     new Position { x = 40, y = 40 }
/// };
/// fixed (ulong* ePtr = entities)
/// fixed (Position* pPtr = positions) {
///     int added = goud_component_add_batch(
///         contextId, ePtr, 5, positionTypeId,
///         pPtr, sizeof(Position)
///     );
///     Console.WriteLine($"Added components to {added} entities");
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_component_add_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    data_ptr: *const u8,
    component_size: usize,
) -> u32 {
    // Validate inputs
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

    // Verify component type is registered and get info
    let type_info = {
        let type_registry = match get_type_registry() {
            Some(r) => r,
            None => {
                set_last_error(GoudError::InternalError(
                    "Failed to access component registry".to_string(),
                ));
                return 0;
            }
        };
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

    // Validate size
    if component_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component size mismatch: expected {}, got {}",
            type_info.size, component_size
        )));
        return 0;
    }

    // Get component storage for this context
    let mut storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to access component registry".to_string(),
            ));
            return 0;
        }
    };
    let map = storage_map.get_or_insert_with(HashMap::new);

    let key = context_key(context_id);
    let context_storage = map.entry(key).or_default();
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    // Add components for each entity
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

/// Removes the same component type from multiple entities in a single batch.
///
/// This is more efficient than calling `goud_component_remove()` multiple times.
/// Invalid entities or entities without the component are skipped.
///
/// # Arguments
///
/// * `context_id` - The context containing the entities
/// * `entity_ids` - Pointer to array of entity IDs
/// * `count` - Number of entities
/// * `type_id_hash` - Hash of the component type
///
/// # Returns
///
/// The number of entities that successfully had the component removed.
///
/// # Safety
///
/// Caller must ensure `entity_ids` points to valid memory with `count` u64 values.
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `RESOURCE_ERROR_BASE + 100` (ResourceNotFound) - Component type not registered
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// fixed (ulong* ePtr = entities) {
///     int removed = goud_component_remove_batch(
///         contextId, ePtr, 5, positionTypeId
///     );
///     Console.WriteLine($"Removed components from {removed} entities");
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_component_remove_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
) -> u32 {
    // Validate inputs
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

    // Verify component type is registered
    {
        let type_registry = match get_type_registry() {
            Some(r) => r,
            None => {
                set_last_error(GoudError::InternalError(
                    "Failed to access component registry".to_string(),
                ));
                return 0;
            }
        };
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

    // Get component storage for this context
    let mut storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to access component registry".to_string(),
            ));
            return 0;
        }
    };
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to access component registry".to_string(),
            ));
            return 0;
        }
    };

    let key = context_key(context_id);

    // Get context storage
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to access component registry".to_string(),
            ));
            return 0;
        }
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to access component registry".to_string(),
            ));
            return 0;
        }
    };

    // Remove components from each entity
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
/// Results are written to the `out_results` array, where each entry is:
/// - `1` (true) if the entity has the component
/// - `0` (false) if the entity doesn't have the component or is invalid
///
/// # Arguments
///
/// * `context_id` - The context to check
/// * `entity_ids` - Pointer to array of entity IDs
/// * `count` - Number of entities
/// * `type_id_hash` - Hash of the component type
/// * `out_results` - Pointer to array where results will be written (size = count)
///
/// # Returns
///
/// The number of results written (should equal `count` on success).
///
/// # Safety
///
/// Caller must ensure:
/// - `entity_ids` points to valid memory with `count` u64 values
/// - `out_results` points to valid memory with `count` u8 values
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `RESOURCE_ERROR_BASE + 100` (ResourceNotFound) - Component type not registered
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// byte[] results = new byte[5];
/// fixed (ulong* ePtr = entities)
/// fixed (byte* rPtr = results) {
///     int count = goud_component_has_batch(
///         contextId, ePtr, 5, positionTypeId, rPtr
///     );
///     for (int i = 0; i < count; i++) {
///         Console.WriteLine($"Entity {i}: {results[i] != 0}");
///     }
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_component_has_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    out_results: *mut u8,
) -> u32 {
    // Validate inputs
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

    // Verify component type is registered
    {
        let type_registry = match get_type_registry() {
            Some(r) => r,
            None => {
                set_last_error(GoudError::InternalError(
                    "Failed to access component registry".to_string(),
                ));
                return 0;
            }
        };
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

    // Get component storage for this context
    let storage_map = match get_context_storage_map() {
        Some(s) => s,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to access component registry".to_string(),
            ));
            return 0;
        }
    };

    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);

    // Check for storage existence
    let key = context_key(context_id);
    let storage_opt = storage_map
        .as_ref()
        .and_then(|map| map.get(&key))
        .and_then(|cs| cs.get_storage(type_id_hash));

    if storage_opt.is_none() {
        // No storage exists, all results are false
        for result in results_slice.iter_mut() {
            *result = 0;
        }
        return count;
    }

    let storage = storage_opt.unwrap();

    // Check each entity
    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        results_slice[i] = if storage.contains(entity_bits) { 1 } else { 0 };
    }

    count
}
