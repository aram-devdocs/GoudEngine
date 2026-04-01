//! Spatial hash lifecycle FFI functions: create, destroy, clear.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::ecs::broad_phase::SpatialHash;

use super::registry;
use super::GOUD_INVALID_SPATIAL_HASH_HANDLE;

/// Creates a new spatial hash with the specified cell size.
///
/// # Arguments
///
/// * `cell_size` - Size of each grid cell in world units. Must be positive and finite.
///
/// # Returns
///
/// A valid hash handle on success, or `GOUD_INVALID_SPATIAL_HASH_HANDLE` on failure.
/// Call `goud_last_error_message()` for error details.
///
/// # Cleanup
///
/// The caller MUST call `goud_spatial_hash_destroy` when the hash is no longer needed.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_create(cell_size: f32) -> u32 {
    if !(cell_size > 0.0 && cell_size.is_finite()) {
        set_last_error(GoudError::InvalidState(
            "cell_size must be positive and finite".to_string(),
        ));
        return GOUD_INVALID_SPATIAL_HASH_HANDLE;
    }

    let hash = SpatialHash::new(cell_size);

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial hash registry".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_HASH_HANDLE;
        }
    };

    let handle = match registry::allocate_handle(&mut reg) {
        Some(h) => h,
        None => {
            set_last_error(GoudError::InternalError(
                "spatial hash handle space exhausted".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_HASH_HANDLE;
        }
    };
    reg.hashes.insert(handle, hash);

    handle
}

/// Creates a new spatial hash with pre-allocated capacity.
///
/// # Arguments
///
/// * `cell_size` - Size of each grid cell in world units. Must be positive and finite.
/// * `capacity` - Expected number of entities (pre-allocates internal storage).
///
/// # Returns
///
/// A valid hash handle on success, or `GOUD_INVALID_SPATIAL_HASH_HANDLE` on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_create_with_capacity(cell_size: f32, capacity: u32) -> u32 {
    if !(cell_size > 0.0 && cell_size.is_finite()) {
        set_last_error(GoudError::InvalidState(
            "cell_size must be positive and finite".to_string(),
        ));
        return GOUD_INVALID_SPATIAL_HASH_HANDLE;
    }

    let hash = SpatialHash::with_capacity(cell_size, capacity as usize);

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial hash registry".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_HASH_HANDLE;
        }
    };

    let handle = match registry::allocate_handle(&mut reg) {
        Some(h) => h,
        None => {
            set_last_error(GoudError::InternalError(
                "spatial hash handle space exhausted".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_HASH_HANDLE;
        }
    };
    reg.hashes.insert(handle, hash);

    handle
}

/// Destroys a spatial hash and frees its resources.
///
/// # Arguments
///
/// * `handle` - The hash handle returned by `goud_spatial_hash_create`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_destroy(handle: u32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_HASH_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial hash handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial hash registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    if reg.hashes.remove(&handle).is_none() {
        set_last_error(GoudError::InvalidState(
            "spatial hash handle not found".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    0
}

/// Clears all entities from a spatial hash without destroying it.
///
/// # Arguments
///
/// * `handle` - The hash handle.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_clear(handle: u32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_HASH_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial hash handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial hash registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    match reg.hashes.get_mut(&handle) {
        Some(hash) => {
            hash.clear();
            0
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial hash handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}
