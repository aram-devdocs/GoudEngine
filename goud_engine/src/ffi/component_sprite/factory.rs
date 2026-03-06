//! # Factory Functions for Sprite Component
//!
//! Provides `goud_sprite_new` and `goud_sprite_default` for creating sprites
//! with sensible defaults.

use crate::core::types::FfiSprite;

/// Creates a new sprite with default settings.
///
/// The sprite will render the entire texture with:
/// - White color tint (no modification)
/// - No source rectangle (full texture)
/// - No flipping
/// - Center anchor point (0.5, 0.5)
/// - No custom size (uses texture dimensions)
///
/// # Parameters
///
/// - `texture_handle`: The texture asset handle (packed as u64)
///
/// # Returns
///
/// A new FfiSprite with default settings.
#[no_mangle]
pub extern "C" fn goud_sprite_new(texture_handle: u64) -> FfiSprite {
    FfiSprite {
        texture_handle,
        color_r: 1.0,
        color_g: 1.0,
        color_b: 1.0,
        color_a: 1.0,
        source_rect_x: 0.0,
        source_rect_y: 0.0,
        source_rect_width: 0.0,
        source_rect_height: 0.0,
        has_source_rect: false,
        flip_x: false,
        flip_y: false,
        anchor_x: 0.5,
        anchor_y: 0.5,
        custom_size_x: 0.0,
        custom_size_y: 0.0,
        has_custom_size: false,
    }
}

/// Creates a default sprite with an invalid texture handle.
///
/// This is primarily useful for deserialization or when the texture
/// will be set later.
///
/// # Returns
///
/// A default FfiSprite with invalid texture handle.
#[no_mangle]
pub extern "C" fn goud_sprite_default() -> FfiSprite {
    goud_sprite_new(u64::MAX) // Invalid handle
}
