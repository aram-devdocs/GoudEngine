//! Factory and value-builder operations on sprites.

use crate::core::types::FfiSprite;

/// Zero-sized type hosting Sprite factory operations.
pub struct SpriteOps;

// NOTE: FFI wrappers are hand-written in ffi/component_sprite.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl SpriteOps {
    /// Creates a new sprite with a texture handle.
    pub fn new_sprite(texture_handle: u64) -> FfiSprite {
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
    pub fn new_default() -> FfiSprite {
        SpriteOps::new_sprite(u64::MAX)
    }

    /// Creates a new sprite with the specified color tint.
    pub fn with_color(sprite: FfiSprite, r: f32, g: f32, b: f32, a: f32) -> FfiSprite {
        let mut result = sprite;
        result.color_r = r;
        result.color_g = g;
        result.color_b = b;
        result.color_a = a;
        result
    }

    /// Creates a new sprite with the specified source rectangle.
    pub fn with_source_rect(
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

    /// Creates a new sprite with the specified horizontal flip.
    pub fn with_flip_x(sprite: FfiSprite, flip: bool) -> FfiSprite {
        let mut result = sprite;
        result.flip_x = flip;
        result
    }

    /// Creates a new sprite with the specified vertical flip.
    pub fn with_flip_y(sprite: FfiSprite, flip: bool) -> FfiSprite {
        let mut result = sprite;
        result.flip_y = flip;
        result
    }

    /// Creates a new sprite with both flip flags set.
    pub fn with_flip(sprite: FfiSprite, flip_x: bool, flip_y: bool) -> FfiSprite {
        let mut result = sprite;
        result.flip_x = flip_x;
        result.flip_y = flip_y;
        result
    }

    /// Creates a new sprite with the specified anchor.
    pub fn with_anchor(sprite: FfiSprite, x: f32, y: f32) -> FfiSprite {
        let mut result = sprite;
        result.anchor_x = x;
        result.anchor_y = y;
        result
    }

    /// Creates a new sprite with the specified custom size.
    pub fn with_custom_size(sprite: FfiSprite, width: f32, height: f32) -> FfiSprite {
        let mut result = sprite;
        result.custom_size_x = width;
        result.custom_size_y = height;
        result.has_custom_size = true;
        result
    }
}
