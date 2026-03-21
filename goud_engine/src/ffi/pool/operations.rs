//! Entity pool mutation FFI functions: acquire, release, batch variants.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};

use super::registry;
use super::GOUD_INVALID_POOL_HANDLE;

/// Acquires a single entity from the pool.
///
/// # Arguments
///
/// * `handle` - The pool handle.
///
/// # Returns
///
/// The entity ID (as u64) on success, or `u64::MAX` if the pool is exhausted
/// or the handle is invalid. Call `goud_last_error_message()` for error details.
#[no_mangle]
pub extern "C" fn goud_entity_pool_acquire(handle: u32) -> u64 {
    if handle == GOUD_INVALID_POOL_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid pool handle".to_string(),
        ));
        return u64::MAX;
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock pool registry".to_string(),
            ));
            return u64::MAX;
        }
    };

    match reg.pools.get_mut(&handle) {
        Some(pool) => match pool.acquire() {
            Some((_slot_index, entity_id)) => entity_id,
            None => {
                set_last_error(GoudError::InvalidState(
                    "pool exhausted: no available slots".to_string(),
                ));
                u64::MAX
            }
        },
        None => {
            set_last_error(GoudError::InvalidState(
                "pool handle not found".to_string(),
            ));
            u64::MAX
        }
    }
}

/// Acquires multiple entities from the pool in a single call.
///
/// Results are written to the caller-provided `out_entities` buffer.
///
/// # Arguments
///
/// * `handle` - The pool handle.
/// * `count` - Number of entities to acquire.
/// * `out_entities` - Caller-allocated buffer for entity IDs (u64).
///
/// # Returns
///
/// The number of entities actually acquired (may be less than `count`
/// if the pool does not have enough available slots).
///
/// # Safety
///
/// `out_entities` must point to a buffer of at least `count` u64 values,
/// or be null if `count` is 0.
#[no_mangle]
pub unsafe extern "C" fn goud_entity_pool_acquire_batch(
    handle: u32,
    count: u32,
    out_entities: *mut u64,
) -> u32 {
    if handle == GOUD_INVALID_POOL_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid pool handle".to_string(),
        ));
        return 0;
    }

    if count > 0 && out_entities.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_entities is null but count > 0".to_string(),
        ));
        return 0;
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock pool registry".to_string(),
            ));
            return 0;
        }
    };

    let pool = match reg.pools.get_mut(&handle) {
        Some(p) => p,
        None => {
            set_last_error(GoudError::InvalidState(
                "pool handle not found".to_string(),
            ));
            return 0;
        }
    };

    let batch = pool.acquire_batch(count as usize);
    let acquired = batch.len();

    if acquired > 0 {
        // SAFETY: Caller guarantees out_entities points to at least `count` u64 values.
        // We write at most `acquired` entries where acquired <= count.
        let slice = std::slice::from_raw_parts_mut(out_entities, acquired);
        for (i, (_slot_index, entity_id)) in batch.iter().enumerate() {
            slice[i] = *entity_id;
        }
    }

    acquired as u32
}

/// Releases an entity back to the pool by slot index.
///
/// # Arguments
///
/// * `handle` - The pool handle.
/// * `slot_index` - The slot index to release.
///
/// # Returns
///
/// 0 on success, negative error code on failure (e.g., double release,
/// out-of-range index, or invalid handle).
#[no_mangle]
pub extern "C" fn goud_entity_pool_release(handle: u32, slot_index: u32) -> i32 {
    if handle == GOUD_INVALID_POOL_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid pool handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock pool registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    match reg.pools.get_mut(&handle) {
        Some(pool) => {
            if pool.release(slot_index as usize) {
                0
            } else {
                set_last_error(GoudError::InvalidState(
                    "slot index out of range or already released".to_string(),
                ));
                GoudError::InvalidState(String::new()).error_code()
            }
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "pool handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}

/// Releases multiple slots back to the pool in a single call.
///
/// # Arguments
///
/// * `handle` - The pool handle.
/// * `slot_indices` - Pointer to an array of slot indices to release.
/// * `count` - Number of indices in the array.
///
/// # Returns
///
/// The number of slots that were successfully released. Slots that are
/// out of range or already inactive are silently skipped.
///
/// # Safety
///
/// `slot_indices` must point to a buffer of at least `count` u32 values,
/// or be null if `count` is 0.
#[no_mangle]
pub unsafe extern "C" fn goud_entity_pool_release_batch(
    handle: u32,
    slot_indices: *const u32,
    count: u32,
) -> u32 {
    if handle == GOUD_INVALID_POOL_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid pool handle".to_string(),
        ));
        return 0;
    }

    if count > 0 && slot_indices.is_null() {
        set_last_error(GoudError::InvalidState(
            "slot_indices is null but count > 0".to_string(),
        ));
        return 0;
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock pool registry".to_string(),
            ));
            return 0;
        }
    };

    let pool = match reg.pools.get_mut(&handle) {
        Some(p) => p,
        None => {
            set_last_error(GoudError::InvalidState(
                "pool handle not found".to_string(),
            ));
            return 0;
        }
    };

    if count == 0 {
        return 0;
    }

    // SAFETY: Caller guarantees slot_indices points to at least `count` u32 values.
    let indices = std::slice::from_raw_parts(slot_indices, count as usize);
    let usize_indices: Vec<usize> = indices.iter().map(|&i| i as usize).collect();
    pool.release_batch(&usize_indices) as u32
}
