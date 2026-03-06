//! # Builder Pattern for Sprite Component
//!
//! Provides a heap-allocated `FfiSpriteBuilder` that allows chaining operations
//! without copying the full struct on each call. This is useful when constructing
//! complex sprites with many properties.
//!
//! ## Usage (C#)
//!
//! ```csharp
//! var builder = goud_sprite_builder_new(textureHandle);
//! builder = goud_sprite_builder_with_color(builder, 1.0f, 0.0f, 0.0f, 1.0f);
//! builder = goud_sprite_builder_with_flip_x(builder, true);
//! builder = goud_sprite_builder_with_anchor(builder, 0.5f, 1.0f);
//! var sprite = goud_sprite_builder_build(builder); // Consumes builder
//! ```
//!
//! ## Memory Management
//!
//! - `goud_sprite_builder_new()` allocates a builder on the heap
//! - `goud_sprite_builder_build()` consumes the builder and frees memory
//! - If you don't call build(), call `goud_sprite_builder_free()` to clean up
//! - Builder functions return the same pointer for chaining

use crate::core::types::{FfiSprite, FfiSpriteBuilder};
use crate::ffi::component_sprite::factory::{goud_sprite_default, goud_sprite_new};

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
    // SAFETY: builder is non-null and was allocated with Box::into_raw
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
        // SAFETY: builder is non-null and was allocated with Box::into_raw
        drop(Box::from_raw(builder));
    }
}
