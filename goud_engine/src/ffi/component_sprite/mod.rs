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
//!
//! ## Module Layout
//!
//! - `factory`    — `goud_sprite_new`, `goud_sprite_default`
//! - `color`      — sprite color/alpha methods and color constant constructors
//! - `properties` — source rect, flip, anchor, custom size, texture, utility
//! - `builder`    — heap-allocated `FfiSpriteBuilder` with chaining API

pub mod builder;
pub mod color;
pub mod factory;
pub mod properties;
pub mod texture;

#[cfg(test)]
mod tests;

// Re-export types from core for backward compatibility
pub use crate::core::types::{FfiColor, FfiRect, FfiSprite, FfiSpriteBuilder};

// Re-export all public FFI functions so external consumers see a flat namespace.

// Factory
pub use factory::{goud_sprite_default, goud_sprite_new};

// Color methods
pub use color::{
    goud_color_black, goud_color_blue, goud_color_from_hex, goud_color_from_u8, goud_color_green,
    goud_color_lerp, goud_color_red, goud_color_rgb, goud_color_rgba, goud_color_transparent,
    goud_color_white, goud_color_with_alpha, goud_color_yellow, goud_sprite_get_alpha,
    goud_sprite_get_color, goud_sprite_set_alpha, goud_sprite_set_color, goud_sprite_with_color,
};

// Properties
pub use properties::{
    goud_sprite_clear_custom_size, goud_sprite_clear_source_rect, goud_sprite_get_anchor,
    goud_sprite_get_custom_size, goud_sprite_get_flip_x, goud_sprite_get_flip_y,
    goud_sprite_get_source_rect, goud_sprite_has_custom_size, goud_sprite_has_source_rect,
    goud_sprite_is_flipped, goud_sprite_set_anchor, goud_sprite_set_custom_size,
    goud_sprite_set_flip, goud_sprite_set_flip_x, goud_sprite_set_flip_y,
    goud_sprite_set_source_rect, goud_sprite_with_anchor, goud_sprite_with_custom_size,
    goud_sprite_with_flip, goud_sprite_with_flip_x, goud_sprite_with_flip_y,
    goud_sprite_with_source_rect,
};

// Texture handle and utility
pub use texture::{goud_sprite_get_texture, goud_sprite_set_texture, goud_sprite_size_or_rect};

// Builder
pub use builder::{
    goud_sprite_builder_build, goud_sprite_builder_default, goud_sprite_builder_free,
    goud_sprite_builder_new, goud_sprite_builder_with_alpha, goud_sprite_builder_with_anchor,
    goud_sprite_builder_with_color, goud_sprite_builder_with_custom_size,
    goud_sprite_builder_with_flip, goud_sprite_builder_with_flip_x,
    goud_sprite_builder_with_flip_y, goud_sprite_builder_with_source_rect,
    goud_sprite_builder_with_texture,
};
