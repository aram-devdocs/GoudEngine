//! Spatial grid mutation FFI functions: insert, remove, update.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::math::Vec2;
use crate::ecs::Entity;

use super::registry;
use super::GOUD_INVALID_SPATIAL_GRID_HANDLE;

/// Inserts an entity at a position into the spatial grid.
///
/// If the entity already exists in the grid, it is moved to the new position.
///
/// # Arguments
///
/// * `handle` - The grid handle
/// * `entity_id` - The entity ID (as u64 bits)
/// * `x` - World X position
/// * `y` - World Y position
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_insert(handle: u32, entity_id: u64, x: f32, y: f32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let entity = Entity::from_bits(entity_id);

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
            grid.insert(entity, Vec2::new(x, y));
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

/// Removes an entity from the spatial grid.
///
/// # Arguments
///
/// * `handle` - The grid handle
/// * `entity_id` - The entity ID (as u64 bits)
///
/// # Returns
///
/// 0 on success (entity was present and removed), negative error code on failure.
/// Returns an error if the entity was not in the grid.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_remove(handle: u32, entity_id: u64) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let entity = Entity::from_bits(entity_id);

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
            if grid.remove(entity) {
                0
            } else {
                set_last_error(GoudError::InvalidState(
                    "entity not found in spatial grid".to_string(),
                ));
                GoudError::InvalidState(String::new()).error_code()
            }
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial grid handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}

/// Updates an entity's position in the spatial grid.
///
/// # Arguments
///
/// * `handle` - The grid handle
/// * `entity_id` - The entity ID (as u64 bits)
/// * `x` - New world X position
/// * `y` - New world Y position
///
/// # Returns
///
/// 0 on success, negative error code on failure.
/// Returns an error if the entity was not in the grid.
#[no_mangle]
pub extern "C" fn goud_spatial_grid_update(handle: u32, entity_id: u64, x: f32, y: f32) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_GRID_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial grid handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let entity = Entity::from_bits(entity_id);

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
            if grid.update(entity, Vec2::new(x, y)) {
                0
            } else {
                set_last_error(GoudError::InvalidState(
                    "entity not found in spatial grid".to_string(),
                ));
                GoudError::InvalidState(String::new()).error_code()
            }
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial grid handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}
