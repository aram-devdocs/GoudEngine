//! Entity pool lifecycle FFI functions: create, destroy.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::pool::EntityPool;

use super::registry;
use super::GOUD_INVALID_POOL_HANDLE;

/// Creates a new entity pool with the specified capacity.
///
/// # Arguments
///
/// * `capacity` - Number of pre-allocated slots in the pool.
///
/// # Returns
///
/// A valid pool handle on success, or `GOUD_INVALID_POOL_HANDLE` on failure.
/// Call `goud_last_error_message()` for error details.
///
/// # Cleanup
///
/// The caller MUST call `goud_entity_pool_destroy` when the pool is no longer needed.
#[no_mangle]
pub extern "C" fn goud_entity_pool_create(capacity: u32) -> u32 {
    let pool = EntityPool::new(capacity as usize);

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock pool registry".to_string(),
            ));
            return GOUD_INVALID_POOL_HANDLE;
        }
    };

    let handle = match registry::allocate_handle(&mut reg) {
        Some(h) => h,
        None => {
            set_last_error(GoudError::InternalError(
                "pool handle space exhausted".to_string(),
            ));
            return GOUD_INVALID_POOL_HANDLE;
        }
    };
    reg.pools.insert(handle, pool);

    handle
}

/// Destroys an entity pool and frees its resources.
///
/// # Arguments
///
/// * `handle` - The pool handle returned by `goud_entity_pool_create`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_entity_pool_destroy(handle: u32) -> i32 {
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

    if reg.pools.remove(&handle).is_none() {
        set_last_error(GoudError::InvalidState(
            "pool handle not found".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    0
}
