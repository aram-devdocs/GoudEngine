//! Heap-allocated sprite builder operations.

use crate::core::types::{FfiSprite, FfiSpriteBuilder};

/// Zero-sized type for Sprite builder operations.
pub struct SpriteBuilderOps;

// NOTE: FFI wrappers are hand-written in ffi/component_sprite.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl SpriteBuilderOps {
    /// Creates a new sprite builder with a texture handle.
    pub fn builder_new(texture_handle: u64) -> *mut FfiSpriteBuilder {
        let builder = FfiSpriteBuilder {
            sprite: super::factory::SpriteOps::new_sprite(texture_handle),
        };
        Box::into_raw(Box::new(builder))
    }

    /// Creates a new sprite builder with default values.
    pub fn builder_default() -> *mut FfiSpriteBuilder {
        let builder = FfiSpriteBuilder {
            sprite: super::factory::SpriteOps::new_default(),
        };
        Box::into_raw(Box::new(builder))
    }

    /// Sets the texture handle on the builder.
    pub fn builder_with_texture(
        builder: *mut FfiSpriteBuilder,
        texture_handle: u64,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Caller guarantees pointer from builder_new.
        unsafe { (*builder).sprite.texture_handle = texture_handle };
        builder
    }

    /// Sets the color tint on the builder.
    pub fn builder_with_color(
        builder: *mut FfiSpriteBuilder,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let s = unsafe { &mut (*builder).sprite };
        s.color_r = r;
        s.color_g = g;
        s.color_b = b;
        s.color_a = a;
        builder
    }

    /// Sets the alpha on the builder.
    pub fn builder_with_alpha(builder: *mut FfiSpriteBuilder, alpha: f32) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        unsafe { (*builder).sprite.color_a = alpha };
        builder
    }

    /// Sets the source rectangle on the builder.
    pub fn builder_with_source_rect(
        builder: *mut FfiSpriteBuilder,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let s = unsafe { &mut (*builder).sprite };
        s.source_rect_x = x;
        s.source_rect_y = y;
        s.source_rect_width = width;
        s.source_rect_height = height;
        s.has_source_rect = true;
        builder
    }

    /// Sets the horizontal flip on the builder.
    pub fn builder_with_flip_x(
        builder: *mut FfiSpriteBuilder,
        flip: bool,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        unsafe { (*builder).sprite.flip_x = flip };
        builder
    }

    /// Sets the vertical flip on the builder.
    pub fn builder_with_flip_y(
        builder: *mut FfiSpriteBuilder,
        flip: bool,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        unsafe { (*builder).sprite.flip_y = flip };
        builder
    }

    /// Sets both flip flags on the builder.
    pub fn builder_with_flip(
        builder: *mut FfiSpriteBuilder,
        flip_x: bool,
        flip_y: bool,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let s = unsafe { &mut (*builder).sprite };
        s.flip_x = flip_x;
        s.flip_y = flip_y;
        builder
    }

    /// Sets the anchor point on the builder.
    pub fn builder_with_anchor(
        builder: *mut FfiSpriteBuilder,
        x: f32,
        y: f32,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let s = unsafe { &mut (*builder).sprite };
        s.anchor_x = x;
        s.anchor_y = y;
        builder
    }

    /// Sets the custom size on the builder.
    pub fn builder_with_custom_size(
        builder: *mut FfiSpriteBuilder,
        width: f32,
        height: f32,
    ) -> *mut FfiSpriteBuilder {
        if builder.is_null() {
            return builder;
        }
        // SAFETY: Pointer checked non-null above; allocated by builder_new via Box::into_raw.
        let s = unsafe { &mut (*builder).sprite };
        s.custom_size_x = width;
        s.custom_size_y = height;
        s.has_custom_size = true;
        builder
    }

    /// Builds the sprite, consuming and freeing the builder.
    pub fn builder_build(builder: *mut FfiSpriteBuilder) -> FfiSprite {
        if builder.is_null() {
            return super::factory::SpriteOps::new_default();
        }
        // SAFETY: Takes ownership from builder_new allocation.
        let boxed = unsafe { Box::from_raw(builder) };
        boxed.sprite
    }

    /// Frees a sprite builder without building.
    pub fn builder_free(builder: *mut FfiSpriteBuilder) {
        if !builder.is_null() {
            // SAFETY: Takes ownership and drops.
            drop(unsafe { Box::from_raw(builder) });
        }
    }
}
