//! Pointer-based get/set operations on sprites.

use crate::core::types::{FfiColor, FfiRect, FfiSprite, FfiVec2};

/// Zero-sized type for Sprite pointer operations.
pub struct SpritePtrOps;

#[goud_engine_macros::goud_api(module = "sprite")]
impl SpritePtrOps {
    /// Sets the color tint for the sprite.
    pub fn set_color(
        sprite: *mut FfiSprite,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if sprite.is_null() {
            return;
        }
        // SAFETY: Caller guarantees pointer is valid.
        let s = unsafe { &mut *sprite };
        s.color_r = r;
        s.color_g = g;
        s.color_b = b;
        s.color_a = a;
    }

    /// Gets the color tint of the sprite.
    pub fn get_color(sprite: *const FfiSprite) -> FfiColor {
        if sprite.is_null() {
            return FfiColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
        }
        let s = unsafe { &*sprite };
        FfiColor {
            r: s.color_r,
            g: s.color_g,
            b: s.color_b,
            a: s.color_a,
        }
    }

    /// Sets the alpha (transparency).
    pub fn set_alpha(sprite: *mut FfiSprite, alpha: f32) {
        if sprite.is_null() {
            return;
        }
        unsafe { (*sprite).color_a = alpha };
    }

    /// Gets the alpha (transparency).
    pub fn get_alpha(sprite: *const FfiSprite) -> f32 {
        if sprite.is_null() {
            return 1.0;
        }
        unsafe { (*sprite).color_a }
    }

    /// Sets the source rectangle for sprite sheet rendering.
    pub fn set_source_rect(
        sprite: *mut FfiSprite,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if sprite.is_null() {
            return;
        }
        let s = unsafe { &mut *sprite };
        s.source_rect_x = x;
        s.source_rect_y = y;
        s.source_rect_width = width;
        s.source_rect_height = height;
        s.has_source_rect = true;
    }

    /// Removes the source rectangle.
    pub fn clear_source_rect(sprite: *mut FfiSprite) {
        if sprite.is_null() {
            return;
        }
        unsafe { (*sprite).has_source_rect = false };
    }

    /// Gets the source rectangle. Returns true if set.
    pub fn get_source_rect(
        sprite: *const FfiSprite,
        out_rect: *mut FfiRect,
    ) -> bool {
        if sprite.is_null() || out_rect.is_null() {
            return false;
        }
        let s = unsafe { &*sprite };
        if s.has_source_rect {
            // SAFETY: Checked non-null above.
            unsafe {
                *out_rect = FfiRect {
                    x: s.source_rect_x,
                    y: s.source_rect_y,
                    width: s.source_rect_width,
                    height: s.source_rect_height,
                };
            }
            true
        } else {
            false
        }
    }

    /// Returns whether the sprite has a source rectangle set.
    pub fn has_source_rect(sprite: *const FfiSprite) -> bool {
        if sprite.is_null() {
            return false;
        }
        unsafe { (*sprite).has_source_rect }
    }

    /// Sets the horizontal flip flag.
    pub fn set_flip_x(sprite: *mut FfiSprite, flip: bool) {
        if sprite.is_null() {
            return;
        }
        unsafe { (*sprite).flip_x = flip };
    }

    /// Gets the horizontal flip flag.
    pub fn get_flip_x(sprite: *const FfiSprite) -> bool {
        if sprite.is_null() {
            return false;
        }
        unsafe { (*sprite).flip_x }
    }

    /// Sets the vertical flip flag.
    pub fn set_flip_y(sprite: *mut FfiSprite, flip: bool) {
        if sprite.is_null() {
            return;
        }
        unsafe { (*sprite).flip_y = flip };
    }

    /// Gets the vertical flip flag.
    pub fn get_flip_y(sprite: *const FfiSprite) -> bool {
        if sprite.is_null() {
            return false;
        }
        unsafe { (*sprite).flip_y }
    }

    /// Sets both flip flags at once.
    pub fn set_flip(
        sprite: *mut FfiSprite,
        flip_x: bool,
        flip_y: bool,
    ) {
        if sprite.is_null() {
            return;
        }
        let s = unsafe { &mut *sprite };
        s.flip_x = flip_x;
        s.flip_y = flip_y;
    }

    /// Returns true if the sprite is flipped on either axis.
    pub fn is_flipped(sprite: *const FfiSprite) -> bool {
        if sprite.is_null() {
            return false;
        }
        let s = unsafe { &*sprite };
        s.flip_x || s.flip_y
    }

    /// Sets the anchor point.
    pub fn set_anchor(sprite: *mut FfiSprite, x: f32, y: f32) {
        if sprite.is_null() {
            return;
        }
        let s = unsafe { &mut *sprite };
        s.anchor_x = x;
        s.anchor_y = y;
    }

    /// Gets the anchor point.
    pub fn get_anchor(sprite: *const FfiSprite) -> FfiVec2 {
        if sprite.is_null() {
            return FfiVec2 { x: 0.5, y: 0.5 };
        }
        let s = unsafe { &*sprite };
        FfiVec2 {
            x: s.anchor_x,
            y: s.anchor_y,
        }
    }

    /// Sets a custom size for the sprite.
    pub fn set_custom_size(
        sprite: *mut FfiSprite,
        width: f32,
        height: f32,
    ) {
        if sprite.is_null() {
            return;
        }
        let s = unsafe { &mut *sprite };
        s.custom_size_x = width;
        s.custom_size_y = height;
        s.has_custom_size = true;
    }

    /// Removes the custom size.
    pub fn clear_custom_size(sprite: *mut FfiSprite) {
        if sprite.is_null() {
            return;
        }
        unsafe { (*sprite).has_custom_size = false };
    }

    /// Gets the custom size. Returns true if set.
    pub fn get_custom_size(
        sprite: *const FfiSprite,
        out_size: *mut FfiVec2,
    ) -> bool {
        if sprite.is_null() || out_size.is_null() {
            return false;
        }
        let s = unsafe { &*sprite };
        if s.has_custom_size {
            // SAFETY: Checked non-null above.
            unsafe {
                *out_size = FfiVec2 {
                    x: s.custom_size_x,
                    y: s.custom_size_y,
                };
            }
            true
        } else {
            false
        }
    }

    /// Returns whether the sprite has a custom size set.
    pub fn has_custom_size(sprite: *const FfiSprite) -> bool {
        if sprite.is_null() {
            return false;
        }
        unsafe { (*sprite).has_custom_size }
    }

    /// Sets the texture handle.
    pub fn set_texture(sprite: *mut FfiSprite, handle: u64) {
        if sprite.is_null() {
            return;
        }
        unsafe { (*sprite).texture_handle = handle };
    }

    /// Gets the texture handle.
    pub fn get_texture(sprite: *const FfiSprite) -> u64 {
        if sprite.is_null() {
            return u64::MAX;
        }
        unsafe { (*sprite).texture_handle }
    }

    /// Gets the effective size (custom > source_rect > zero).
    pub fn size_or_rect(sprite: *const FfiSprite) -> FfiVec2 {
        if sprite.is_null() {
            return FfiVec2 { x: 0.0, y: 0.0 };
        }
        let s = unsafe { &*sprite };
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
}
