//! Unit tests for the sprite batch renderer that do not require an OpenGL context.

use crate::assets::{
    loaders::{SpriteSheetAsset, TextureAsset},
    AssetHandle, AssetServer,
};
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::components::{Mat3x3, Sprite, Transform2D};
use crate::ecs::Entity;
use crate::ecs::World;
use crate::libs::graphics::backend::null::NullBackend;
use crate::rendering::sprite_batch::batch::SpriteBatch;
use crate::rendering::sprite_batch::config::SpriteBatchConfig;
use crate::rendering::sprite_batch::types::{SpriteBatchEntry, SpriteInstance, SpriteVertex};
use crate::rendering::RenderViewport;
use image::{ImageBuffer, ImageFormat, Rgba};

fn create_test_png(width: u32, height: u32) -> Vec<u8> {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 255, 0, 255])
        }
    });
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("test png encoding should succeed");
    bytes
}

#[test]
fn test_sprite_batch_config_default() {
    let config = SpriteBatchConfig::default();
    assert_eq!(config.initial_capacity, 1024);
    assert_eq!(config.max_batch_size, 10000);
    assert!(config.enable_z_sorting);
    assert!(config.enable_batching);
    assert!(config.enable_frustum_culling);
    assert!(!config.shader_asset.is_valid());
    assert!(!config.material_asset.is_valid());
}

#[test]
fn test_sprite_vertex_layout() {
    let layout = SpriteVertex::layout();
    assert_eq!(layout.stride, std::mem::size_of::<SpriteVertex>() as u32);
    assert_eq!(layout.attributes.len(), 3);
}

#[test]
fn test_sprite_instance_creation() {
    let instance = SpriteInstance {
        entity: Entity::new(42, 1),
        texture: AssetHandle::new(1, 1),
        transform: Mat3x3::IDENTITY,
        color: Color::RED,
        source_rect: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
        size: Vec2::new(64.0, 64.0),
        z_layer: 100,
        flip_x: true,
        flip_y: false,
    };

    assert_eq!(instance.entity.index(), 42);
    assert_eq!(instance.color, Color::RED);
    assert!(instance.flip_x);
    assert!(!instance.flip_y);
}

#[test]
fn test_sprite_batch_entry_creation() {
    let entry = SpriteBatchEntry {
        _texture_handle: AssetHandle::new(1, 1),
        gpu_texture: None,
        vertex_start: 0,
        vertex_count: 24,
    };

    assert_eq!(entry.vertex_start, 0);
    assert_eq!(entry.vertex_count, 24);
    assert!(entry.gpu_texture.is_none());
}

#[test]
fn test_gather_sprites_resolves_sprite_sheet_frames() {
    let mut asset_server = AssetServer::new();
    super::ensure_sprite_asset_loaders(&mut asset_server);
    let texture =
        asset_server.load_from_bytes::<TextureAsset>("textures/player.png", &create_test_png(2, 2));
    let sheet = asset_server.load_from_bytes::<SpriteSheetAsset>(
        "sprites/player.sheet.json",
        br#"{
            "texture_path": "textures/player.png",
            "regions": {
                "idle": { "x": 8.0, "y": 4.0, "width": 16.0, "height": 24.0 }
            }
        }"#,
    );

    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Transform2D::default());
    world.insert(entity, Sprite::from_sprite_sheet(sheet, "idle"));

    let mut batch = SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap();
    batch.gather_sprites(&world, &mut asset_server).unwrap();

    assert_eq!(batch.sprites.len(), 1);
    let sprite = &batch.sprites[0];
    assert_eq!(sprite.texture, texture);
    assert_eq!(sprite.source_rect, Some(Rect::new(8.0, 4.0, 16.0, 24.0)));
    assert_eq!(sprite.size, Vec2::new(16.0, 24.0));
}

#[test]
fn test_gather_sprites_culls_entities_outside_viewport() {
    let mut asset_server = AssetServer::new();
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Transform2D::from_position(Vec2::new(512.0, 512.0)));
    world.insert(
        entity,
        Sprite::new(AssetHandle::INVALID).with_custom_size(Vec2::new(32.0, 32.0)),
    );

    let mut batch = SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap();
    batch.set_viewport(RenderViewport::fullscreen((128, 128)));
    batch.gather_sprites(&world, &mut asset_server).unwrap();

    assert!(batch.sprites.is_empty());
    assert_eq!(batch.culled_count(), 1);
}

#[test]
fn test_gather_sprites_keeps_entities_inside_viewport() {
    let mut asset_server = AssetServer::new();
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, Transform2D::from_position(Vec2::new(32.0, 32.0)));
    world.insert(
        entity,
        Sprite::new(AssetHandle::INVALID).with_custom_size(Vec2::new(32.0, 32.0)),
    );

    let mut batch = SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap();
    batch.set_viewport(RenderViewport::fullscreen((128, 128)));
    batch.gather_sprites(&world, &mut asset_server).unwrap();

    assert_eq!(batch.sprites.len(), 1);
    assert_eq!(batch.culled_count(), 0);
}

#[test]
fn test_gather_sprites_culls_outside_viewport() {
    let mut asset_server = AssetServer::new();
    super::ensure_sprite_asset_loaders(&mut asset_server);
    let texture =
        asset_server.load_from_bytes::<TextureAsset>("textures/player.png", &create_test_png(4, 4));

    let mut world = World::new();

    let visible = world.spawn_empty();
    world.insert(visible, Transform2D::from_position(Vec2::new(32.0, 32.0)));
    world.insert(
        visible,
        Sprite::new(texture).with_custom_size(Vec2::new(16.0, 16.0)),
    );

    let culled = world.spawn_empty();
    world.insert(culled, Transform2D::from_position(Vec2::new(400.0, 400.0)));
    world.insert(
        culled,
        Sprite::new(texture).with_custom_size(Vec2::new(16.0, 16.0)),
    );

    let mut batch = SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap();
    batch.set_viewport(RenderViewport {
        x: 0,
        y: 0,
        width: 128,
        height: 128,
        logical_width: 128,
        logical_height: 128,
        scale_factor: 1.0,
    });

    batch.gather_sprites(&world, &mut asset_server).unwrap();

    assert_eq!(batch.sprite_count(), 1);
    assert_eq!(batch.culled_count(), 1);
    assert_eq!(batch.sprites[0].entity, visible);
}
