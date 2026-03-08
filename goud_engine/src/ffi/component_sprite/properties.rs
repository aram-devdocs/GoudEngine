//! # Sprite Property Methods
//!
//! Provides FFI functions for reading and writing sprite properties:
//! source rectangle, flip flags, anchor point, custom size, texture handle,
//! and the utility `goud_sprite_size_or_rect` function.

use crate::core::error::{set_last_error, GoudError};
use crate::core::types::{FfiRect, FfiSprite};
use crate::ffi::types::FfiVec2;

// =============================================================================
// Source Rectangle Methods
// =============================================================================

/// Sets the source rectangle for sprite sheet rendering.
///
/// The rectangle is specified in pixel coordinates relative to the
/// top-left corner of the texture.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `x`: X position of the source rectangle
/// - `y`: Y position of the source rectangle
/// - `width`: Width of the source rectangle
/// - `height`: Height of the source rectangle
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_source_rect(
    sprite: *mut FfiSprite,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let s = &mut *sprite;
    s.source_rect_x = x;
    s.source_rect_y = y;
    s.source_rect_width = width;
    s.source_rect_height = height;
    s.has_source_rect = true;
}

/// Removes the source rectangle, rendering the full texture.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_clear_source_rect(sprite: *mut FfiSprite) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    (*sprite).has_source_rect = false;
}

/// Gets the source rectangle of the sprite.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to read
/// - `out_rect`: Pointer to store the source rectangle
///
/// # Returns
///
/// `true` if the sprite has a source rectangle, `false` otherwise.
///
/// # Safety
///
/// - `sprite` and `out_rect` must be valid, non-null pointers
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_source_rect(
    sprite: *const FfiSprite,
    out_rect: *mut FfiRect,
) -> bool {
    if sprite.is_null() || out_rect.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    let s = &*sprite;
    if s.has_source_rect {
        *out_rect = FfiRect {
            x: s.source_rect_x,
            y: s.source_rect_y,
            width: s.source_rect_width,
            height: s.source_rect_height,
        };
        true
    } else {
        false
    }
}

/// Returns whether the sprite has a source rectangle set.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_has_source_rect(sprite: *const FfiSprite) -> bool {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    (*sprite).has_source_rect
}

/// Creates a new sprite with the specified source rectangle (builder pattern).
///
/// # Parameters
///
/// - `sprite`: The sprite to copy
/// - `x`: X position of the source rectangle
/// - `y`: Y position of the source rectangle
/// - `width`: Width of the source rectangle
/// - `height`: Height of the source rectangle
///
/// # Returns
///
/// A new FfiSprite with the source rectangle set.
#[no_mangle]
pub extern "C" fn goud_sprite_with_source_rect(
    sprite: FfiSprite,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FfiSprite {
    let mut result = sprite;
    result.source_rect_x = x;
    result.source_rect_y = y;
    result.source_rect_width = width;
    result.source_rect_height = height;
    result.has_source_rect = true;
    result
}

// =============================================================================
// Flip Methods
// =============================================================================

/// Sets the horizontal flip flag.
///
/// When true, the sprite is mirrored along the Y-axis.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `flip`: Whether to flip horizontally
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_flip_x(sprite: *mut FfiSprite, flip: bool) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    (*sprite).flip_x = flip;
}

/// Gets the horizontal flip flag.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_flip_x(sprite: *const FfiSprite) -> bool {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    (*sprite).flip_x
}

/// Sets the vertical flip flag.
///
/// When true, the sprite is mirrored along the X-axis.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `flip`: Whether to flip vertically
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_flip_y(sprite: *mut FfiSprite, flip: bool) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    (*sprite).flip_y = flip;
}

/// Gets the vertical flip flag.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_flip_y(sprite: *const FfiSprite) -> bool {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    (*sprite).flip_y
}

/// Sets both flip flags at once.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `flip_x`: Whether to flip horizontally
/// - `flip_y`: Whether to flip vertically
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_flip(sprite: *mut FfiSprite, flip_x: bool, flip_y: bool) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let s = &mut *sprite;
    s.flip_x = flip_x;
    s.flip_y = flip_y;
}

/// Creates a new sprite with the specified horizontal flip (builder pattern).
#[no_mangle]
pub extern "C" fn goud_sprite_with_flip_x(sprite: FfiSprite, flip: bool) -> FfiSprite {
    let mut result = sprite;
    result.flip_x = flip;
    result
}

/// Creates a new sprite with the specified vertical flip (builder pattern).
#[no_mangle]
pub extern "C" fn goud_sprite_with_flip_y(sprite: FfiSprite, flip: bool) -> FfiSprite {
    let mut result = sprite;
    result.flip_y = flip;
    result
}

/// Creates a new sprite with both flip flags set (builder pattern).
#[no_mangle]
pub extern "C" fn goud_sprite_with_flip(
    sprite: FfiSprite,
    flip_x: bool,
    flip_y: bool,
) -> FfiSprite {
    let mut result = sprite;
    result.flip_x = flip_x;
    result.flip_y = flip_y;
    result
}

/// Returns true if the sprite is flipped on either axis.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_is_flipped(sprite: *const FfiSprite) -> bool {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    let s = &*sprite;
    s.flip_x || s.flip_y
}

// =============================================================================
// Anchor Methods
// =============================================================================

/// Sets the anchor point with individual coordinates.
///
/// Coordinates are normalized in range [0.0, 1.0]:
/// - (0.0, 0.0) = Top-left
/// - (0.5, 0.5) = Center (default)
/// - (1.0, 1.0) = Bottom-right
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `x`: X anchor coordinate (normalized)
/// - `y`: Y anchor coordinate (normalized)
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_anchor(sprite: *mut FfiSprite, x: f32, y: f32) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let s = &mut *sprite;
    s.anchor_x = x;
    s.anchor_y = y;
}

/// Gets the anchor point of the sprite.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_anchor(sprite: *const FfiSprite) -> FfiVec2 {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return FfiVec2 { x: 0.5, y: 0.5 };
    }
    let s = &*sprite;
    FfiVec2 {
        x: s.anchor_x,
        y: s.anchor_y,
    }
}

/// Creates a new sprite with the specified anchor (builder pattern).
#[no_mangle]
pub extern "C" fn goud_sprite_with_anchor(sprite: FfiSprite, x: f32, y: f32) -> FfiSprite {
    let mut result = sprite;
    result.anchor_x = x;
    result.anchor_y = y;
    result
}

// =============================================================================
// Custom Size Methods
// =============================================================================

/// Sets a custom size for the sprite.
///
/// When set, the sprite is scaled to this size regardless of the
/// texture dimensions.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `width`: Custom width
/// - `height`: Custom height
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_custom_size(
    sprite: *mut FfiSprite,
    width: f32,
    height: f32,
) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    let s = &mut *sprite;
    s.custom_size_x = width;
    s.custom_size_y = height;
    s.has_custom_size = true;
}

/// Removes the custom size, using texture dimensions.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_clear_custom_size(sprite: *mut FfiSprite) {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return;
    }
    (*sprite).has_custom_size = false;
}

/// Gets the custom size of the sprite.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to read
/// - `out_size`: Pointer to store the custom size
///
/// # Returns
///
/// `true` if the sprite has a custom size, `false` otherwise.
///
/// # Safety
///
/// - `sprite` and `out_size` must be valid, non-null pointers
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_custom_size(
    sprite: *const FfiSprite,
    out_size: *mut FfiVec2,
) -> bool {
    if sprite.is_null() || out_size.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    let s = &*sprite;
    if s.has_custom_size {
        *out_size = FfiVec2 {
            x: s.custom_size_x,
            y: s.custom_size_y,
        };
        true
    } else {
        false
    }
}

/// Returns whether the sprite has a custom size set.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_has_custom_size(sprite: *const FfiSprite) -> bool {
    if sprite.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }
    (*sprite).has_custom_size
}

/// Creates a new sprite with the specified custom size (builder pattern).
#[no_mangle]
pub extern "C" fn goud_sprite_with_custom_size(
    sprite: FfiSprite,
    width: f32,
    height: f32,
) -> FfiSprite {
    let mut result = sprite;
    result.custom_size_x = width;
    result.custom_size_y = height;
    result.has_custom_size = true;
    result
}
