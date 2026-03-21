//! Spatial grid lifecycle FFI functions: create, destroy, clear.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::ecs::spatial_grid::SpatialGrid;

use super::registry;
use super::GOUD_INVALID_SPATIAL_GRID_HANDLE;

/// Creates a new spatial grid with the specified cell size.
///
/// # Arguments
///
/// * `cell_size` - Size of each grid cell in world units. Must be positive and finite.
///
/// # Returns
///
/// A valid grid handle on success, or `GOUD_INVALID_SPATIAL_GRID_HANDLE` on failure.
/// Call `goud_last_error_message()` for error details.
///
/// # Cleanup
///
/// The caller MUST call `goud_spatial_grid_destroy` when the grid is no longer needed.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_create(cell_size: f32) -> u32 {
    if !(cell_size > 0.0 && cell_size.is_finite()) {
        set_last_error(GoudError::InvalidState(
            "cell_size must be positive and finite".to_string(),
        ));
        return GOUD_INVALID_SPATIAL_GRID_HANDLE;
    }

    let grid = SpatialGrid::new(cell_size);

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial grid registry".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_GRID_HANDLE;
        }
    };

    // Find a valid handle that is not the sentinel and not already in use.
    let mut handle = reg.next_handle;
    let start = handle;
    loop {
        if handle != GOUD_INVALID_SPATIAL_GRID_HANDLE && !reg.grids.contains_key(&handle) {
            break;
        }
        handle = handle.wrapping_add(1);
        if handle == start {
            // All handles exhausted (extremely unlikely: 2^32 - 1 live grids).
            set_last_error(GoudError::InternalError(
                "spatial grid handle space exhausted".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_GRID_HANDLE;
        }
    }
    reg.next_handle = handle.wrapping_add(1);
    reg.grids.insert(handle, grid);

    handle
}

/// Creates a new spatial grid with pre-allocated capacity.
///
/// # Arguments
///
/// * `cell_size` - Size of each grid cell in world units. Must be positive and finite.
/// * `capacity` - Expected number of entities (pre-allocates internal storage).
///
/// # Returns
///
/// A valid grid handle on success, or `GOUD_INVALID_SPATIAL_GRID_HANDLE` on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_create_with_capacity(cell_size: f32, capacity: u32) -> u32 {
    if !(cell_size > 0.0 && cell_size.is_finite()) {
        set_last_error(GoudError::InvalidState(
            "cell_size must be positive and finite".to_string(),
        ));
        return GOUD_INVALID_SPATIAL_GRID_HANDLE;
    }

    let grid = SpatialGrid::with_capacity(cell_size, capacity as usize);

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial grid registry".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_GRID_HANDLE;
        }
    };

    // Find a valid handle that is not the sentinel and not already in use.
    let mut handle = reg.next_handle;
    let start = handle;
    loop {
        if handle != GOUD_INVALID_SPATIAL_GRID_HANDLE && !reg.grids.contains_key(&handle) {
            break;
        }
        handle = handle.wrapping_add(1);
        if handle == start {
            set_last_error(GoudError::InternalError(
                "spatial grid handle space exhausted".to_string(),
            ));
            return GOUD_INVALID_SPATIAL_GRID_HANDLE;
        }
    }
    reg.next_handle = handle.wrapping_add(1);
    reg.grids.insert(handle, grid);

    handle
}

/// Destroys a spatial grid and frees its resources.
///
/// # Arguments
///
/// * `handle` - The grid handle returned by `goud_spatial_grid_create`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_destroy(handle: u32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial grid registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    if reg.grids.remove(&handle).is_none() {
        set_last_error(GoudError::InvalidState(
            "spatial grid handle not found".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    0
}

/// Clears all entities from a spatial grid without destroying it.
///
/// # Arguments
///
/// * `handle` - The grid handle.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_clear(handle: u32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let mut reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial grid registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    match reg.grids.get_mut(&handle) {
        Some(grid) => {
            grid.clear();
            0
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial grid handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}
