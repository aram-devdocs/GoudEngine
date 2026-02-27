//! # FFI Functions for Sprite Component
//!
//! This module provides C-compatible functions for manipulating Sprite components.
//! These functions allow language bindings to perform sprite operations without
//! duplicating logic across SDKs.
//!
//! ## Design Philosophy
//!
//! All sprite manipulation logic lives in Rust. SDKs (C#, Python, etc.) are thin
//! wrappers that marshal data and call these FFI functions. This ensures:
//! - Single source of truth for sprite operations
//! - Consistent behavior across all language bindings
//! - No logic duplication in SDK code
//!
//! ## Usage (C#)
//!
//! ```csharp
//! // Create a sprite with a texture handle
//! var sprite = goud_sprite_new(textureHandle);
//!
//! // Set color tint (red with 50% alpha)
//! goud_sprite_set_color(ref sprite, 1.0f, 0.0f, 0.0f, 0.5f);
//!
//! // Flip horizontally
//! goud_sprite_set_flip_x(ref sprite, true);
//!
//! // Set anchor to bottom-center
//! goud_sprite_set_anchor(ref sprite, 0.5f, 1.0f);
//! ```
//!
//! ## Thread Safety
//!
//! These functions operate on raw pointers and are not thread-safe. The caller
//! must ensure exclusive access to the Sprite during mutation.

use crate::core::math::{Color, Rect};
use crate::ffi::types::FfiVec2;

// =============================================================================
// FFI-Safe Sprite Type
// =============================================================================

/// FFI-safe Sprite representation.
///
/// This is a simplified version of the Sprite component suitable for FFI.
/// It uses raw u64 texture handles instead of generic AssetHandle types.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiSprite {
    /// Texture handle (index and generation packed as u64).
    pub texture_handle: u64,

    /// Color tint red component (0.0 - 1.0).
    pub color_r: f32,
    /// Color tint green component (0.0 - 1.0).
    pub color_g: f32,
    /// Color tint blue component (0.0 - 1.0).
    pub color_b: f32,
    /// Color tint alpha component (0.0 - 1.0).
    pub color_a: f32,

    /// Source rectangle X position (if has_source_rect is true).
    pub source_rect_x: f32,
    /// Source rectangle Y position.
    pub source_rect_y: f32,
    /// Source rectangle width.
    pub source_rect_width: f32,
    /// Source rectangle height.
    pub source_rect_height: f32,
    /// Whether source_rect is set.
    pub has_source_rect: bool,

    /// Flip horizontally flag.
    pub flip_x: bool,
    /// Flip vertically flag.
    pub flip_y: bool,

    /// Anchor point X (normalized 0-1).
    pub anchor_x: f32,
    /// Anchor point Y (normalized 0-1).
    pub anchor_y: f32,

    /// Custom size width (if has_custom_size is true).
    pub custom_size_x: f32,
    /// Custom size height.
    pub custom_size_y: f32,
    /// Whether custom_size is set.
    pub has_custom_size: bool,
}

/// FFI-safe Color representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiColor {
    /// Red component (0.0 - 1.0).
    pub r: f32,
    /// Green component (0.0 - 1.0).
    pub g: f32,
    /// Blue component (0.0 - 1.0).
    pub b: f32,
    /// Alpha component (0.0 - 1.0).
    pub a: f32,
}

impl From<Color> for FfiColor {
    fn from(c: Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

impl From<FfiColor> for Color {
    fn from(c: FfiColor) -> Self {
        Color::rgba(c.r, c.g, c.b, c.a)
    }
}

/// FFI-safe Rect representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiRect {
    /// X position of the rectangle.
    pub x: f32,
    /// Y position of the rectangle.
    pub y: f32,
    /// Width of the rectangle.
    pub width: f32,
    /// Height of the rectangle.
    pub height: f32,
}

impl From<Rect> for FfiRect {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

impl From<FfiRect> for Rect {
    fn from(r: FfiRect) -> Self {
        Rect::new(r.x, r.y, r.width, r.height)
    }
}

// =============================================================================
// Factory Functions
// =============================================================================

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

// =============================================================================
// Color Methods
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

// =============================================================================
// Builder Pattern (Heap-Allocated)
// =============================================================================
//
// The builder pattern provides a heap-allocated mutable builder that allows
// chaining operations without copying the full struct on each call. This is
// useful when constructing complex sprites with many properties.
//
// ## Usage (C#)
//
// ```csharp
// var builder = goud_sprite_builder_new(textureHandle);
// builder = goud_sprite_builder_with_color(builder, 1.0f, 0.0f, 0.0f, 1.0f);
// builder = goud_sprite_builder_with_flip_x(builder, true);
// builder = goud_sprite_builder_with_anchor(builder, 0.5f, 1.0f);
// var sprite = goud_sprite_builder_build(builder); // Consumes builder
// ```
//
// ## Memory Management
//
// - `goud_sprite_builder_new()` allocates a builder on the heap
// - `goud_sprite_builder_build()` consumes the builder and frees memory
// - If you don't call build(), call `goud_sprite_builder_free()` to clean up
// - Builder functions return the same pointer for chaining

/// Heap-allocated sprite builder for FFI use.
///
/// This builder allows constructing a sprite by setting properties one at a time
/// without copying the entire struct on each modification.
#[repr(C)]
pub struct FfiSpriteBuilder {
    sprite: FfiSprite,
}

/// Creates a new sprite builder with a texture handle.
///
/// The builder is heap-allocated and must be either:
/// - Consumed with `goud_sprite_builder_build()`, or
/// - Freed with `goud_sprite_builder_free()`
///
/// # Parameters
///
/// - `texture_handle`: The texture asset handle (packed as u64)
///
/// # Returns
///
/// A pointer to a new FfiSpriteBuilder, or null if allocation fails.
///
/// # Safety
///
/// The returned pointer must be managed by the caller.
#[no_mangle]
pub extern "C" fn goud_sprite_builder_new(texture_handle: u64) -> *mut FfiSpriteBuilder {
    let builder = FfiSpriteBuilder {
        sprite: goud_sprite_new(texture_handle),
    };
    Box::into_raw(Box::new(builder))
}

/// Creates a new sprite builder with default values.
///
/// The texture handle will be invalid (u64::MAX) and must be set
/// before the sprite is usable.
///
/// # Returns
///
/// A pointer to a new FfiSpriteBuilder with default values.
#[no_mangle]
pub extern "C" fn goud_sprite_builder_default() -> *mut FfiSpriteBuilder {
    let builder = FfiSpriteBuilder {
        sprite: goud_sprite_default(),
    };
    Box::into_raw(Box::new(builder))
}

/// Sets the texture handle on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `texture_handle`: The texture asset handle
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()`
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_texture(
    builder: *mut FfiSpriteBuilder,
    texture_handle: u64,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).sprite.texture_handle = texture_handle;
    builder
}

/// Sets the color tint on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `r`: Red component (0.0 - 1.0)
/// - `g`: Green component (0.0 - 1.0)
/// - `b`: Blue component (0.0 - 1.0)
/// - `a`: Alpha component (0.0 - 1.0)
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()`
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_color(
    builder: *mut FfiSpriteBuilder,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    let s = &mut (*builder).sprite;
    s.color_r = r;
    s.color_g = g;
    s.color_b = b;
    s.color_a = a;
    builder
}

/// Sets the alpha (transparency) on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `alpha`: Alpha component (0.0 = transparent, 1.0 = opaque)
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_alpha(
    builder: *mut FfiSpriteBuilder,
    alpha: f32,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).sprite.color_a = alpha;
    builder
}

/// Sets the source rectangle on the builder for sprite sheet rendering.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `x`: X position of the source rectangle
/// - `y`: Y position of the source rectangle
/// - `width`: Width of the source rectangle
/// - `height`: Height of the source rectangle
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_source_rect(
    builder: *mut FfiSpriteBuilder,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    let s = &mut (*builder).sprite;
    s.source_rect_x = x;
    s.source_rect_y = y;
    s.source_rect_width = width;
    s.source_rect_height = height;
    s.has_source_rect = true;
    builder
}

/// Sets the horizontal flip flag on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `flip`: Whether to flip horizontally
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_flip_x(
    builder: *mut FfiSpriteBuilder,
    flip: bool,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).sprite.flip_x = flip;
    builder
}

/// Sets the vertical flip flag on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `flip`: Whether to flip vertically
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_flip_y(
    builder: *mut FfiSpriteBuilder,
    flip: bool,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    (*builder).sprite.flip_y = flip;
    builder
}

/// Sets both flip flags on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `flip_x`: Whether to flip horizontally
/// - `flip_y`: Whether to flip vertically
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_flip(
    builder: *mut FfiSpriteBuilder,
    flip_x: bool,
    flip_y: bool,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    let s = &mut (*builder).sprite;
    s.flip_x = flip_x;
    s.flip_y = flip_y;
    builder
}

/// Sets the anchor point on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `x`: X anchor coordinate (normalized 0-1)
/// - `y`: Y anchor coordinate (normalized 0-1)
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_anchor(
    builder: *mut FfiSpriteBuilder,
    x: f32,
    y: f32,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    let s = &mut (*builder).sprite;
    s.anchor_x = x;
    s.anchor_y = y;
    builder
}

/// Sets a custom size on the builder.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder
/// - `width`: Custom width
/// - `height`: Custom height
///
/// # Returns
///
/// The same builder pointer for chaining.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_with_custom_size(
    builder: *mut FfiSpriteBuilder,
    width: f32,
    height: f32,
) -> *mut FfiSpriteBuilder {
    if builder.is_null() {
        return builder;
    }
    let s = &mut (*builder).sprite;
    s.custom_size_x = width;
    s.custom_size_y = height;
    s.has_custom_size = true;
    builder
}

/// Builds and consumes the sprite builder, returning the final sprite.
///
/// This function:
/// 1. Extracts the sprite from the builder
/// 2. Frees the builder memory
/// 3. Returns the sprite by value
///
/// After calling this, the builder pointer is invalid.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder to consume
///
/// # Returns
///
/// The constructed FfiSprite. If builder is null, returns a default sprite.
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()`
/// - After this call, the builder pointer must not be used
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_build(builder: *mut FfiSpriteBuilder) -> FfiSprite {
    if builder.is_null() {
        return goud_sprite_default();
    }
    // Take ownership of the box and extract the sprite
    let boxed = Box::from_raw(builder);
    boxed.sprite
}

/// Frees a sprite builder without building.
///
/// Use this if you need to abort sprite construction and clean up memory.
///
/// # Parameters
///
/// - `builder`: Pointer to the builder to free
///
/// # Safety
///
/// - `builder` must be a valid pointer from `goud_sprite_builder_new()` or null
/// - After this call, the builder pointer must not be used
#[no_mangle]
pub unsafe extern "C" fn goud_sprite_builder_free(builder: *mut FfiSpriteBuilder) {
    if !builder.is_null() {
        drop(Box::from_raw(builder));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_sprite_new() {
        let s = goud_sprite_new(42);
        assert_eq!(s.texture_handle, 42);
        assert_eq!(s.color_r, 1.0);
        assert_eq!(s.color_g, 1.0);
        assert_eq!(s.color_b, 1.0);
        assert_eq!(s.color_a, 1.0);
        assert!(!s.has_source_rect);
        assert!(!s.flip_x);
        assert!(!s.flip_y);
        assert_eq!(s.anchor_x, 0.5);
        assert_eq!(s.anchor_y, 0.5);
        assert!(!s.has_custom_size);
    }

    #[test]
    fn test_ffi_sprite_set_color() {
        let mut s = goud_sprite_new(1);
        unsafe {
            goud_sprite_set_color(&mut s, 1.0, 0.0, 0.0, 0.5);
        }
        assert_eq!(s.color_r, 1.0);
        assert_eq!(s.color_g, 0.0);
        assert_eq!(s.color_b, 0.0);
        assert_eq!(s.color_a, 0.5);
    }

    #[test]
    fn test_ffi_sprite_with_color() {
        let s = goud_sprite_new(1);
        let s2 = goud_sprite_with_color(s, 0.0, 1.0, 0.0, 1.0);
        assert_eq!(s2.color_r, 0.0);
        assert_eq!(s2.color_g, 1.0);
        assert_eq!(s2.color_b, 0.0);
        assert_eq!(s2.color_a, 1.0);
    }

    #[test]
    fn test_ffi_sprite_flip() {
        let mut s = goud_sprite_new(1);
        unsafe {
            goud_sprite_set_flip_x(&mut s, true);
            assert!(goud_sprite_get_flip_x(&s));
            assert!(!goud_sprite_get_flip_y(&s));
            assert!(goud_sprite_is_flipped(&s));

            goud_sprite_set_flip(&mut s, false, true);
            assert!(!goud_sprite_get_flip_x(&s));
            assert!(goud_sprite_get_flip_y(&s));
        }
    }

    #[test]
    fn test_ffi_sprite_anchor() {
        let mut s = goud_sprite_new(1);
        unsafe {
            goud_sprite_set_anchor(&mut s, 0.0, 1.0);
            let anchor = goud_sprite_get_anchor(&s);
            assert_eq!(anchor.x, 0.0);
            assert_eq!(anchor.y, 1.0);
        }
    }

    #[test]
    fn test_ffi_sprite_source_rect() {
        let mut s = goud_sprite_new(1);
        unsafe {
            assert!(!goud_sprite_has_source_rect(&s));

            goud_sprite_set_source_rect(&mut s, 10.0, 20.0, 32.0, 32.0);
            assert!(goud_sprite_has_source_rect(&s));

            let mut rect = FfiRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
            assert!(goud_sprite_get_source_rect(&s, &mut rect));
            assert_eq!(rect.x, 10.0);
            assert_eq!(rect.y, 20.0);
            assert_eq!(rect.width, 32.0);
            assert_eq!(rect.height, 32.0);

            goud_sprite_clear_source_rect(&mut s);
            assert!(!goud_sprite_has_source_rect(&s));
        }
    }

    #[test]
    fn test_ffi_sprite_custom_size() {
        let mut s = goud_sprite_new(1);
        unsafe {
            assert!(!goud_sprite_has_custom_size(&s));

            goud_sprite_set_custom_size(&mut s, 64.0, 128.0);
            assert!(goud_sprite_has_custom_size(&s));

            let mut size = FfiVec2 { x: 0.0, y: 0.0 };
            assert!(goud_sprite_get_custom_size(&s, &mut size));
            assert_eq!(size.x, 64.0);
            assert_eq!(size.y, 128.0);

            let effective = goud_sprite_size_or_rect(&s);
            assert_eq!(effective.x, 64.0);
            assert_eq!(effective.y, 128.0);
        }
    }

    #[test]
    fn test_ffi_sprite_builder_chain() {
        let s = goud_sprite_new(1);
        let s = goud_sprite_with_color(s, 1.0, 0.0, 0.0, 1.0);
        let s = goud_sprite_with_flip_x(s, true);
        let s = goud_sprite_with_anchor(s, 0.5, 1.0);
        let s = goud_sprite_with_custom_size(s, 64.0, 64.0);

        assert_eq!(s.color_r, 1.0);
        assert!(s.flip_x);
        assert_eq!(s.anchor_y, 1.0);
        assert!(s.has_custom_size);
    }

    #[test]
    fn test_ffi_sprite_null_safety() {
        unsafe {
            // Test that null pointer functions don't crash
            goud_sprite_set_color(std::ptr::null_mut(), 1.0, 0.0, 0.0, 1.0);
            goud_sprite_set_flip_x(std::ptr::null_mut(), true);
            goud_sprite_set_anchor(std::ptr::null_mut(), 0.5, 0.5);

            let color = goud_sprite_get_color(std::ptr::null());
            assert_eq!(color.r, 1.0);
            assert_eq!(color.a, 1.0);

            let anchor = goud_sprite_get_anchor(std::ptr::null());
            assert_eq!(anchor.x, 0.5);
            assert_eq!(anchor.y, 0.5);

            assert!(!goud_sprite_get_flip_x(std::ptr::null()));
            assert!(!goud_sprite_is_flipped(std::ptr::null()));
        }
    }

    #[test]
    fn test_ffi_color_constants() {
        let white = goud_color_white();
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
        assert_eq!(white.a, 1.0);

        let red = goud_color_red();
        assert_eq!(red.r, 1.0);
        assert_eq!(red.g, 0.0);
        assert_eq!(red.b, 0.0);

        let transparent = goud_color_transparent();
        assert_eq!(transparent.a, 0.0);
    }

    #[test]
    fn test_ffi_color_from_hex() {
        let red = goud_color_from_hex(0xFF0000);
        assert_eq!(red.r, 1.0);
        assert_eq!(red.g, 0.0);
        assert_eq!(red.b, 0.0);
        assert_eq!(red.a, 1.0);

        let red_half = goud_color_from_hex(0xFF000080);
        assert_eq!(red_half.r, 1.0);
        assert!((red_half.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_ffi_color_lerp() {
        let black = goud_color_black();
        let white = goud_color_white();
        let mid = goud_color_lerp(black, white, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_ffi_sprite_size() {
        // Verify FFI type sizes are reasonable
        let size = std::mem::size_of::<FfiSprite>();
        assert!(size > 0);
        // FfiSprite has many fields, so it should be reasonably sized
        // 1 u64 + 4 f32 (color) + 4 f32 (source_rect) + 1 bool + 2 bool + 2 f32 (anchor) + 2 f32 (custom_size) + 1 bool
        // = 8 + 16 + 16 + 1 + 2 + 8 + 8 + 1 = ~60 bytes (with padding)
        assert!(size <= 80); // Allow for padding
    }

    // =========================================================================
    // Builder Pattern Tests
    // =========================================================================

    #[test]
    fn test_builder_new_and_build() {
        let builder = goud_sprite_builder_new(42);
        assert!(!builder.is_null());

        let sprite = unsafe { goud_sprite_builder_build(builder) };
        assert_eq!(sprite.texture_handle, 42);
        assert_eq!(sprite.color_r, 1.0);
        assert_eq!(sprite.color_a, 1.0);
    }

    #[test]
    fn test_builder_default() {
        let builder = goud_sprite_builder_default();
        assert!(!builder.is_null());

        let sprite = unsafe { goud_sprite_builder_build(builder) };
        assert_eq!(sprite.texture_handle, u64::MAX);
    }

    #[test]
    fn test_builder_with_texture() {
        let builder = goud_sprite_builder_default();
        let builder = unsafe { goud_sprite_builder_with_texture(builder, 123) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert_eq!(sprite.texture_handle, 123);
    }

    #[test]
    fn test_builder_with_color() {
        let builder = goud_sprite_builder_new(1);
        let builder = unsafe { goud_sprite_builder_with_color(builder, 1.0, 0.5, 0.25, 0.75) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert_eq!(sprite.color_r, 1.0);
        assert_eq!(sprite.color_g, 0.5);
        assert_eq!(sprite.color_b, 0.25);
        assert_eq!(sprite.color_a, 0.75);
    }

    #[test]
    fn test_builder_with_alpha() {
        let builder = goud_sprite_builder_new(1);
        let builder = unsafe { goud_sprite_builder_with_alpha(builder, 0.5) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert_eq!(sprite.color_a, 0.5);
    }

    #[test]
    fn test_builder_with_source_rect() {
        let builder = goud_sprite_builder_new(1);
        let builder =
            unsafe { goud_sprite_builder_with_source_rect(builder, 10.0, 20.0, 32.0, 64.0) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert!(sprite.has_source_rect);
        assert_eq!(sprite.source_rect_x, 10.0);
        assert_eq!(sprite.source_rect_y, 20.0);
        assert_eq!(sprite.source_rect_width, 32.0);
        assert_eq!(sprite.source_rect_height, 64.0);
    }

    #[test]
    fn test_builder_with_flip() {
        let builder = goud_sprite_builder_new(1);
        let builder = unsafe { goud_sprite_builder_with_flip_x(builder, true) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert!(sprite.flip_x);
        assert!(!sprite.flip_y);

        let builder2 = goud_sprite_builder_new(1);
        let builder2 = unsafe { goud_sprite_builder_with_flip(builder2, true, true) };
        let sprite2 = unsafe { goud_sprite_builder_build(builder2) };

        assert!(sprite2.flip_x);
        assert!(sprite2.flip_y);
    }

    #[test]
    fn test_builder_with_anchor() {
        let builder = goud_sprite_builder_new(1);
        let builder = unsafe { goud_sprite_builder_with_anchor(builder, 0.0, 1.0) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert_eq!(sprite.anchor_x, 0.0);
        assert_eq!(sprite.anchor_y, 1.0);
    }

    #[test]
    fn test_builder_with_custom_size() {
        let builder = goud_sprite_builder_new(1);
        let builder = unsafe { goud_sprite_builder_with_custom_size(builder, 128.0, 256.0) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert!(sprite.has_custom_size);
        assert_eq!(sprite.custom_size_x, 128.0);
        assert_eq!(sprite.custom_size_y, 256.0);
    }

    #[test]
    fn test_builder_chain() {
        let builder = goud_sprite_builder_new(42);
        let builder = unsafe { goud_sprite_builder_with_color(builder, 1.0, 0.0, 0.0, 1.0) };
        let builder = unsafe { goud_sprite_builder_with_flip_x(builder, true) };
        let builder = unsafe { goud_sprite_builder_with_anchor(builder, 0.5, 1.0) };
        let builder = unsafe { goud_sprite_builder_with_custom_size(builder, 64.0, 64.0) };
        let builder =
            unsafe { goud_sprite_builder_with_source_rect(builder, 0.0, 0.0, 32.0, 32.0) };
        let sprite = unsafe { goud_sprite_builder_build(builder) };

        assert_eq!(sprite.texture_handle, 42);
        assert_eq!(sprite.color_r, 1.0);
        assert_eq!(sprite.color_g, 0.0);
        assert!(sprite.flip_x);
        assert_eq!(sprite.anchor_y, 1.0);
        assert!(sprite.has_custom_size);
        assert!(sprite.has_source_rect);
    }

    #[test]
    fn test_builder_free() {
        let builder = goud_sprite_builder_new(1);
        unsafe { goud_sprite_builder_free(builder) };
        // Should not crash - just testing memory is freed properly
    }

    #[test]
    fn test_builder_null_safety() {
        // All builder functions should handle null safely
        unsafe {
            let null_builder: *mut FfiSpriteBuilder = std::ptr::null_mut();

            assert!(goud_sprite_builder_with_texture(null_builder, 1).is_null());
            assert!(goud_sprite_builder_with_color(null_builder, 1.0, 0.0, 0.0, 1.0).is_null());
            assert!(goud_sprite_builder_with_alpha(null_builder, 0.5).is_null());
            assert!(
                goud_sprite_builder_with_source_rect(null_builder, 0.0, 0.0, 32.0, 32.0).is_null()
            );
            assert!(goud_sprite_builder_with_flip_x(null_builder, true).is_null());
            assert!(goud_sprite_builder_with_flip_y(null_builder, true).is_null());
            assert!(goud_sprite_builder_with_flip(null_builder, true, true).is_null());
            assert!(goud_sprite_builder_with_anchor(null_builder, 0.5, 0.5).is_null());
            assert!(goud_sprite_builder_with_custom_size(null_builder, 64.0, 64.0).is_null());

            // Build with null should return default
            let sprite = goud_sprite_builder_build(null_builder);
            assert_eq!(sprite.texture_handle, u64::MAX);

            // Free null should not crash
            goud_sprite_builder_free(null_builder);
        }
    }
}
