//! # Color Methods and Constants for Sprite Component
//!
//! Provides FFI functions for reading and writing sprite color tints,
//! as well as color constant constructors and utility functions.

use crate::core::math::Color;
use crate::core::types::{FfiColor, FfiSprite};

// =============================================================================
// Sprite Color Methods
// =============================================================================

/// Sets the color tint for this sprite.
///
/// The color is multiplied with each texture pixel. Use white (1, 1, 1, 1)
/// for no tinting.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `r`: Red component (0.0 - 1.0)
/// - `g`: Green component (0.0 - 1.0)
/// - `b`: Blue component (0.0 - 1.0)
/// - `a`: Alpha component (0.0 - 1.0)
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_color(
    sprite: *mut FfiSprite,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    if sprite.is_null() {
        return;
    }
    let s = &mut *sprite;
    s.color_r = r;
    s.color_g = g;
    s.color_b = b;
    s.color_a = a;
}

/// Gets the color tint of the sprite.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to read
///
/// # Returns
///
/// The color tint as FfiColor.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_color(sprite: *const FfiSprite) -> FfiColor {
    if sprite.is_null() {
        return FfiColor {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
    }
    let s = &*sprite;
    FfiColor {
        r: s.color_r,
        g: s.color_g,
        b: s.color_b,
        a: s.color_a,
    }
}

/// Creates a new sprite with the specified color tint (builder pattern).
///
/// # Parameters
///
/// - `sprite`: The sprite to copy
/// - `r`: Red component (0.0 - 1.0)
/// - `g`: Green component (0.0 - 1.0)
/// - `b`: Blue component (0.0 - 1.0)
/// - `a`: Alpha component (0.0 - 1.0)
///
/// # Returns
///
/// A new FfiSprite with the updated color.
#[no_mangle]
pub extern "C" fn goud_sprite_with_color(
    sprite: FfiSprite,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> FfiSprite {
    let mut result = sprite;
    result.color_r = r;
    result.color_g = g;
    result.color_b = b;
    result.color_a = a;
    result
}

/// Sets the alpha (transparency) for this sprite.
///
/// # Parameters
///
/// - `sprite`: Pointer to the sprite to modify
/// - `alpha`: Alpha component (0.0 = transparent, 1.0 = opaque)
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_set_alpha(sprite: *mut FfiSprite, alpha: f32) {
    if sprite.is_null() {
        return;
    }
    (*sprite).color_a = alpha;
}

/// Gets the alpha (transparency) of the sprite.
///
/// # Safety
///
/// - `sprite` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_get_alpha(sprite: *const FfiSprite) -> f32 {
    if sprite.is_null() {
        return 1.0;
    }
    (*sprite).color_a
}

// =============================================================================
// Color Constants
// =============================================================================

/// Returns the white color constant.
#[no_mangle]
pub extern "C" fn goud_color_white() -> FfiColor {
    FfiColor {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }
}

/// Returns the black color constant.
#[no_mangle]
pub extern "C" fn goud_color_black() -> FfiColor {
    FfiColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }
}

/// Returns the red color constant.
#[no_mangle]
pub extern "C" fn goud_color_red() -> FfiColor {
    FfiColor {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }
}

/// Returns the green color constant.
#[no_mangle]
pub extern "C" fn goud_color_green() -> FfiColor {
    FfiColor {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    }
}

/// Returns the blue color constant.
#[no_mangle]
pub extern "C" fn goud_color_blue() -> FfiColor {
    FfiColor {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    }
}

/// Returns the yellow color constant.
#[no_mangle]
pub extern "C" fn goud_color_yellow() -> FfiColor {
    FfiColor {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    }
}

/// Returns the transparent color constant.
#[no_mangle]
pub extern "C" fn goud_color_transparent() -> FfiColor {
    FfiColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    }
}

/// Creates a color from RGBA components.
#[no_mangle]
pub extern "C" fn goud_color_rgba(r: f32, g: f32, b: f32, a: f32) -> FfiColor {
    FfiColor { r, g, b, a }
}

/// Creates a color from RGB components with alpha = 1.0.
#[no_mangle]
pub extern "C" fn goud_color_rgb(r: f32, g: f32, b: f32) -> FfiColor {
    FfiColor { r, g, b, a: 1.0 }
}

/// Creates a color from 8-bit RGBA values (0-255).
#[no_mangle]
pub extern "C" fn goud_color_from_u8(r: u8, g: u8, b: u8, a: u8) -> FfiColor {
    FfiColor {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

/// Creates a color from a hex value (0xRRGGBB or 0xRRGGBBAA).
#[no_mangle]
pub extern "C" fn goud_color_from_hex(hex: u32) -> FfiColor {
    Color::from_hex(hex).into()
}

/// Linearly interpolates between two colors.
#[no_mangle]
pub extern "C" fn goud_color_lerp(from: FfiColor, to: FfiColor, t: f32) -> FfiColor {
    let from_c: Color = from.into();
    let to_c: Color = to.into();
    from_c.lerp(to_c, t).into()
}

/// Returns a new color with the specified alpha.
#[no_mangle]
pub extern "C" fn goud_color_with_alpha(color: FfiColor, alpha: f32) -> FfiColor {
    FfiColor {
        r: color.r,
        g: color.g,
        b: color.b,
        a: alpha,
    }
}
