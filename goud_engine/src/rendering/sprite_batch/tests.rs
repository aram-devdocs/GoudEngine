//! Unit tests for the sprite batch renderer that do not require an OpenGL context.

use crate::assets::AssetHandle;
use crate::core::math::{Color, Rect, Vec2};
use crate::ecs::components::Mat3x3;
use crate::ecs::Entity;
use crate::rendering::sprite_batch::config::SpriteBatchConfig;
use crate::rendering::sprite_batch::types::{SpriteBatchEntry, SpriteInstance, SpriteVertex};

#[test]
fn test_sprite_batch_config_default() {
    let config = SpriteBatchConfig::default();
    assert_eq!(config.initial_capacity, 1024);
    assert_eq!(config.max_batch_size, 10000);
    assert!(config.enable_z_sorting);
    assert!(config.enable_batching);
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
        z_layer: 100.0,
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
