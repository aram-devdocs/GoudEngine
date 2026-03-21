//! Spatial grid query FFI functions.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::math::Vec2;

use super::registry;
use super::GOUD_INVALID_SPATIAL_GRID_HANDLE;

/// Queries for entities within a radius of a center point.
///
/// Results are written to the caller-provided `out_entities` buffer. If the
/// buffer is too small, only `capacity` entities are written but the full
/// count is still returned, allowing the caller to retry with a larger buffer.
///
/// # Arguments
///
/// * `handle` - The grid handle
/// * `center_x` - Query center X position
/// * `center_y` - Query center Y position
/// * `radius` - Query radius
/// * `out_entities` - Caller-allocated buffer for entity IDs (u64 bits)
/// * `capacity` - Number of u64 slots in `out_entities`
///
/// # Returns
///
/// Non-negative: the total number of entities found (may exceed `capacity`).
/// Negative: error code. Call `goud_last_error_message()` for details.
///
/// # Safety
///
/// `out_entities` must point to a buffer of at least `capacity` u64 values,
/// or be null if `capacity` is 0.
#[no_mangle]
pub unsafe extern "C" fn goud_spatial_grid_query_radius(
    handle: u32,
    center_x: f32,
    center_y: f32,
    radius: f32,
    out_entities: *mut u64,
    capacity: u32,
) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    if capacity > 0 && out_entities.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_entities is null but capacity > 0".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial grid registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    let grid = match reg.grids.get(&handle) {
        Some(g) => g,
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial grid handle not found".to_string(),
            ));
            return GoudError::InvalidState(String::new()).error_code();
        }
    };

    let results = grid.query_radius(Vec2::new(center_x, center_y), radius);
    let total = results.len();
    let write_count = total.min(capacity as usize);

    if write_count > 0 {
        // SAFETY: Caller guarantees out_entities points to at least `capacity` u64 values.
        // We write at most `capacity` entries (write_count <= capacity).
        let slice = std::slice::from_raw_parts_mut(out_entities, write_count);
        for (i, entity) in results.iter().take(write_count).enumerate() {
            slice[i] = entity.to_bits();
        }
    }

    i32::try_from(total).unwrap_or(i32::MAX)
}

/// Returns the number of entities in the spatial grid.
///
/// # Arguments
///
/// * `handle` - The grid handle
///
/// # Returns
///
/// Non-negative entity count on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_entity_count(handle: u32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock spatial grid registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    match reg.grids.get(&handle) {
        Some(grid) => i32::try_from(grid.entity_count()).unwrap_or(i32::MAX),
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial grid handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}
