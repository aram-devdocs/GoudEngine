//! # Texture Handle and Utility Methods for Sprite Component
//!
//! Provides FFI functions for reading and writing the sprite's texture handle,
//! and a utility function for querying effective sprite size.

use crate::core::types::FfiSprite;
use crate::ffi::types::FfiVec2;

// =============================================================================
// Texture Handle Methods
// =============================================================================

/// Sets the texture handle for the sprite.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `handle`: The new texture handle
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_texture(sprite: *mut FfiSprite, handle: u64) {
    if sprite.is_null() {
        return;
    }
    (*sprite).texture_handle = handle;
}

/// Gets the texture handle of the sprite.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_texture(sprite: *const FfiSprite) -> u64 {
    if sprite.is_null() {
        return u64::MAX;
    }
    (*sprite).texture_handle
}

// =============================================================================
// Utility Methods
// =============================================================================

/// Gets the effective size of the sprite.
///
/// Returns the custom size if set, otherwise the source rect size if set,
/// otherwise returns (0, 0) indicating the caller should query texture dimensions.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_size_or_rect(sprite: *const FfiSprite) -> FfiVec2 {
    if sprite.is_null() {
        return FfiVec2 { x: 0.0, y: 0.0 };
    }
    let s = &*sprite;
    if s.has_custom_size {
        FfiVec2 {
            x: s.custom_size_x,
            y: s.custom_size_y,
        }
    } else if s.has_source_rect {
        FfiVec2 {
            x: s.source_rect_width,
            y: s.source_rect_height,
        }
    } else {
        FfiVec2 { x: 0.0, y: 0.0 }
    }
}
