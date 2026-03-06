//! # Tests for Sprite Component FFI
//!
//! Unit tests covering factory functions, color methods, properties,
//! builder pattern, and null-safety guarantees.

use crate::core::types::{FfiRect, FfiSpriteBuilder};
use crate::ffi::component_sprite::{
    builder::{
        goud_sprite_builder_build, goud_sprite_builder_default, goud_sprite_builder_free,
        goud_sprite_builder_new, goud_sprite_builder_with_alpha, goud_sprite_builder_with_anchor,
        goud_sprite_builder_with_color, goud_sprite_builder_with_custom_size,
        goud_sprite_builder_with_flip, goud_sprite_builder_with_flip_x,
        goud_sprite_builder_with_flip_y, goud_sprite_builder_with_source_rect,
        goud_sprite_builder_with_texture,
    },
    color::{
        goud_color_black, goud_color_from_hex, goud_color_lerp, goud_color_red,
        goud_color_transparent, goud_color_white, goud_sprite_get_color, goud_sprite_set_color,
        goud_sprite_with_color,
    },
    factory::goud_sprite_new,
    properties::{
        goud_sprite_get_anchor, goud_sprite_get_custom_size, goud_sprite_get_flip_x,
        goud_sprite_get_flip_y, goud_sprite_get_source_rect, goud_sprite_has_custom_size,
        goud_sprite_has_source_rect, goud_sprite_is_flipped, goud_sprite_set_anchor,
        goud_sprite_set_custom_size, goud_sprite_set_flip, goud_sprite_set_flip_x,
        goud_sprite_set_source_rect, goud_sprite_with_anchor, goud_sprite_with_custom_size,
        goud_sprite_with_flip_x,
    },
    texture::goud_sprite_size_or_rect,
};
use crate::ffi::types::FfiVec2;

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

        crate::ffi::component_sprite::properties::goud_sprite_clear_source_rect(&mut s);
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
    let size = std::mem::size_of::<crate::core::types::FfiSprite>();
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
    let builder = unsafe { goud_sprite_builder_with_source_rect(builder, 10.0, 20.0, 32.0, 64.0) };
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
    let builder = unsafe { goud_sprite_builder_with_source_rect(builder, 0.0, 0.0, 32.0, 32.0) };
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
        assert!(goud_sprite_builder_with_source_rect(null_builder, 0.0, 0.0, 32.0, 32.0).is_null());
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
