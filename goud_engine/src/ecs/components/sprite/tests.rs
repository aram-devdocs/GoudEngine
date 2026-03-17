//! Tests for the [`Sprite`] component.

use crate::assets::{
    loaders::{SpriteSheetAsset, TextureAsset},
    AssetHandle,
};
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::components::Sprite;
use crate::ecs::Component;

// Helper to create a valid handle for testing
fn dummy_handle() -> AssetHandle<TextureAsset> {
    AssetHandle::new(1, 1)
}

fn dummy_sheet_handle() -> AssetHandle<SpriteSheetAsset> {
    AssetHandle::new(2, 1)
}

#[test]
fn test_sprite_new() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle);

    assert_eq!(sprite.texture, handle);
    assert_eq!(sprite.color, Color::WHITE);
    assert_eq!(sprite.sprite_sheet, AssetHandle::INVALID);
    assert_eq!(sprite.sprite_sheet_path, None);
    assert_eq!(sprite.sprite_frame, None);
    assert_eq!(sprite.source_rect, None);
    assert_eq!(sprite.flip_x, false);
    assert_eq!(sprite.flip_y, false);
    assert_eq!(sprite.z_layer, 0);
    assert_eq!(sprite.anchor, Vec2::new(0.5, 0.5));
    assert_eq!(sprite.custom_size, None);
}

#[test]
fn test_sprite_default() {
    let sprite = Sprite::default();

    assert_eq!(sprite.texture, AssetHandle::INVALID);
    assert_eq!(sprite.color, Color::WHITE);
    assert_eq!(sprite.sprite_sheet, AssetHandle::INVALID);
    assert_eq!(sprite.anchor, Vec2::new(0.5, 0.5));
}

#[test]
fn test_sprite_from_sprite_sheet() {
    let sprite = Sprite::from_sprite_sheet(dummy_sheet_handle(), "idle");

    assert_eq!(sprite.sprite_sheet, dummy_sheet_handle());
    assert_eq!(sprite.sprite_frame.as_deref(), Some("idle"));
    assert!(sprite.has_sprite_sheet());
}

#[test]
fn test_sprite_with_sprite_sheet_path() {
    let sprite = Sprite::default().with_sprite_sheet_path("sprites/player.sheet.json", "run_1");

    assert_eq!(
        sprite.sprite_sheet_path.as_deref(),
        Some("sprites/player.sheet.json")
    );
    assert_eq!(sprite.sprite_frame.as_deref(), Some("run_1"));
    assert!(sprite.has_sprite_sheet());
}

#[test]
fn test_sprite_without_sprite_sheet() {
    let sprite = Sprite::from_sprite_sheet(dummy_sheet_handle(), "idle").without_sprite_sheet();

    assert_eq!(sprite.sprite_sheet, AssetHandle::INVALID);
    assert_eq!(sprite.sprite_sheet_path, None);
    assert_eq!(sprite.sprite_frame, None);
    assert!(!sprite.has_sprite_sheet());
}

#[test]
fn test_sprite_with_color() {
    let handle = dummy_handle();
    let red = Color::rgba(1.0, 0.0, 0.0, 0.5);
    let sprite = Sprite::new(handle).with_color(red);

    assert_eq!(sprite.color, red);
}

#[test]
fn test_sprite_with_source_rect() {
    let handle = dummy_handle();
    let rect = Rect::new(10.0, 20.0, 32.0, 32.0);
    let sprite = Sprite::new(handle).with_source_rect(rect);

    assert_eq!(sprite.source_rect, Some(rect));
    assert!(sprite.has_source_rect());
}

#[test]
fn test_sprite_without_source_rect() {
    let handle = dummy_handle();
    let rect = Rect::new(10.0, 20.0, 32.0, 32.0);
    let sprite = Sprite::new(handle)
        .with_source_rect(rect)
        .without_source_rect();

    assert_eq!(sprite.source_rect, None);
    assert!(!sprite.has_source_rect());
}

#[test]
fn test_sprite_with_flip_x() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle).with_flip_x(true);

    assert_eq!(sprite.flip_x, true);
    assert_eq!(sprite.flip_y, false);
    assert!(sprite.is_flipped());
}

#[test]
fn test_sprite_with_flip_y() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle).with_flip_y(true);

    assert_eq!(sprite.flip_x, false);
    assert_eq!(sprite.flip_y, true);
    assert!(sprite.is_flipped());
}

#[test]
fn test_sprite_with_flip() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle).with_flip(true, true);

    assert_eq!(sprite.flip_x, true);
    assert_eq!(sprite.flip_y, true);
    assert!(sprite.is_flipped());
}

#[test]
fn test_sprite_with_anchor() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle).with_anchor(0.0, 1.0);

    assert_eq!(sprite.anchor, Vec2::new(0.0, 1.0));
}

#[test]
fn test_sprite_with_anchor_vec() {
    let handle = dummy_handle();
    let anchor = Vec2::new(0.25, 0.75);
    let sprite = Sprite::new(handle).with_anchor_vec(anchor);

    assert_eq!(sprite.anchor, anchor);
}

#[test]
fn test_sprite_with_z_layer() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle).with_z_layer(42);

    assert_eq!(sprite.z_layer, 42);
}

#[test]
fn test_sprite_with_custom_size() {
    let handle = dummy_handle();
    let size = Vec2::new(64.0, 64.0);
    let sprite = Sprite::new(handle).with_custom_size(size);

    assert_eq!(sprite.custom_size, Some(size));
    assert!(sprite.has_custom_size());
}

#[test]
fn test_sprite_without_custom_size() {
    let handle = dummy_handle();
    let size = Vec2::new(64.0, 64.0);
    let sprite = Sprite::new(handle)
        .with_custom_size(size)
        .without_custom_size();

    assert_eq!(sprite.custom_size, None);
    assert!(!sprite.has_custom_size());
}

#[test]
fn test_sprite_size_or_rect_custom() {
    let handle = dummy_handle();
    let custom_size = Vec2::new(100.0, 100.0);
    let sprite = Sprite::new(handle)
        .with_custom_size(custom_size)
        .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0));

    // Custom size takes precedence
    assert_eq!(sprite.size_or_rect(), custom_size);
}

#[test]
fn test_sprite_size_or_rect_source() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle).with_source_rect(Rect::new(0.0, 0.0, 32.0, 48.0));

    // Source rect size is used when no custom size
    assert_eq!(sprite.size_or_rect(), Vec2::new(32.0, 48.0));
}

#[test]
fn test_sprite_size_or_rect_none() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle);

    // Returns zero when neither is set (caller should query texture)
    assert_eq!(sprite.size_or_rect(), Vec2::zero());
}

#[test]
fn test_sprite_is_flipped() {
    let handle = dummy_handle();

    let sprite1 = Sprite::new(handle);
    assert!(!sprite1.is_flipped());

    let sprite2 = Sprite::new(handle).with_flip_x(true);
    assert!(sprite2.is_flipped());

    let sprite3 = Sprite::new(handle).with_flip_y(true);
    assert!(sprite3.is_flipped());

    let sprite4 = Sprite::new(handle).with_flip(true, true);
    assert!(sprite4.is_flipped());
}

#[test]
fn test_sprite_builder_chain() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle)
        .with_color(Color::RED)
        .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0))
        .with_flip(true, false)
        .with_z_layer(7)
        .with_anchor(0.5, 1.0)
        .with_custom_size(Vec2::new(64.0, 64.0));

    assert_eq!(sprite.color, Color::RED);
    assert_eq!(sprite.source_rect, Some(Rect::new(0.0, 0.0, 32.0, 32.0)));
    assert_eq!(sprite.flip_x, true);
    assert_eq!(sprite.flip_y, false);
    assert_eq!(sprite.z_layer, 7);
    assert_eq!(sprite.anchor, Vec2::new(0.5, 1.0));
    assert_eq!(sprite.custom_size, Some(Vec2::new(64.0, 64.0)));
}

#[test]
fn test_sprite_clone() {
    let handle = dummy_handle();
    let sprite1 = Sprite::new(handle).with_color(Color::BLUE);
    let sprite2 = sprite1.clone();

    assert_eq!(sprite1, sprite2);
}

#[test]
fn test_sprite_is_component() {
    // Compile-time check that Sprite implements Component
    fn assert_component<T: Component>() {}
    assert_component::<Sprite>();
}

#[test]
fn test_sprite_debug() {
    let handle = dummy_handle();
    let sprite = Sprite::new(handle);
    let debug_str = format!("{:?}", sprite);

    assert!(debug_str.contains("Sprite"));
}

#[test]
fn test_sprite_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Sprite>();
}
