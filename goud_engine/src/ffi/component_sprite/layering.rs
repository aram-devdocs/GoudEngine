//! FFI accessors and builders for explicit sprite render layering.

use crate::core::error::{set_last_error, GoudError};
use crate::core::types::FfiSprite;

/// Sets the explicit render-order layer.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_z_layer(sprite: *mut FfiSprite, z_layer: i32) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    (*sprite).z_layer = z_layer;
}

/// Gets the explicit render-order layer.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_z_layer(sprite: *const FfiSprite) -> i32 {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return 0;
    }
    (*sprite).z_layer
}

/// Returns a copy with a modified render-order layer.
#[no_mangle]
pub extern "C" fn goud_sprite_with_z_layer(sprite: FfiSprite, z_layer: i32) -> FfiSprite {
    let mut result = sprite;
    result.z_layer = z_layer;
    result
}
