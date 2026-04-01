//! Spatial hash mutation FFI functions: insert, remove, update.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::core::math::Rect;
use crate::ecs::Entity;

use super::registry;
use super::GOUD_INVALID_SPATIAL_HASH_HANDLE;

/// Converts center-based (x, y, half_w, half_h) to top-left-based Rect.
#[inline]
fn aabb_from_center(x: f32, y: f32, half_w: f32, half_h: f32) -> Rect {
    Rect::new(x - half_w, y - half_h, half_w * 2.0, half_h * 2.0)
}

/// Inserts an entity with an AABB into the spatial hash.
///
/// If the entity already exists, its AABB is overwritten.
///
/// # Arguments
///
/// * `handle` - The hash handle
/// * `entity_id` - The entity ID (as u64 bits)
/// * `x` - Center X position of the AABB
/// * `y` - Center Y position of the AABB
/// * `half_w` - Half-width of the AABB
/// * `half_h` - Half-height of the AABB
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_insert(
    handle: u32,
    entity_id: u64,
    x: f32,
    y: f32,
    half_w: f32,
    half_h: f32,
) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_HASH_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial hash handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let entity = Entity::from_bits(entity_id);
    let aabb = aabb_from_center(x, y, half_w, half_h);

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
            hash.insert(entity, aabb);
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

/// Removes an entity from the spatial hash.
///
/// # Arguments
///
/// * `handle` - The hash handle
/// * `entity_id` - The entity ID (as u64 bits)
///
/// # Returns
///
/// 0 on success (including if entity was not present), negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_remove(handle: u32, entity_id: u64) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_HASH_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial hash handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let entity = Entity::from_bits(entity_id);

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
            // Idempotent: removing a nonexistent entity is a no-op (returns success).
            hash.remove(entity);
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

/// Updates an entity's AABB in the spatial hash.
///
/// # Arguments
///
/// * `handle` - The hash handle
/// * `entity_id` - The entity ID (as u64 bits)
/// * `x` - New center X position of the AABB
/// * `y` - New center Y position of the AABB
/// * `half_w` - New half-width of the AABB
/// * `half_h` - New half-height of the AABB
///
/// # Returns
///
/// 0 on success, negative error code on failure.
/// Returns an error if the entity was not in the hash.
#[no_mangle]
pub extern "C" fn goud_spatial_hash_update(
    handle: u32,
    entity_id: u64,
    x: f32,
    y: f32,
    half_w: f32,
    half_h: f32,
) -> i32 {
    if handle == GOUD_INVALID_SPATIAL_HASH_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid spatial hash handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let entity = Entity::from_bits(entity_id);
    let aabb = aabb_from_center(x, y, half_w, half_h);

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
            if hash.update(entity, aabb) {
                0
            } else {
                set_last_error(GoudError::InvalidState(
                    "entity not found in spatial hash".to_string(),
                ));
                GoudError::InvalidState(String::new()).error_code()
            }
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "spatial hash handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}
