//! OpenGL-context-requiring lifecycle and sorting tests for the sprite batch renderer.
//!
//! All tests here are marked `#[ignore]` because they need a live OpenGL context.

use crate::assets::AssetHandle;
use crate::core::math::{Color, Vec2};
use crate::ecs::components::Mat3x3;
use crate::ecs::Entity;
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::libs::graphics::sprite_batch::batch::SpriteBatch;
use crate::libs::graphics::sprite_batch::config::SpriteBatchConfig;
use crate::libs::graphics::sprite_batch::types::SpriteInstance;

// =========================================================================
// Lifecycle
// =========================================================================

#[test]
#[ignore] // Requires OpenGL context
fn test_sprite_batch_new() {
    let backend = OpenGLBackend::new().unwrap();
    let config = SpriteBatchConfig::default();
    let batch = SpriteBatch::new(backend, config);
    assert!(batch.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_sprite_batch_begin_end() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    batch.begin();
    assert_eq!(batch.sprite_count(), 0);
    assert_eq!(batch.batch_count(), 0);

    let result = batch.end();
    assert!(result.is_ok());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_sprite_batch_gather_empty_world() {
    use crate::ecs::World;

    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();
    let world = World::new();

    batch.begin();
    let result = batch.gather_sprites(&world);
    assert!(result.is_ok());
    assert_eq!(batch.sprite_count(), 0);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_sprite_batch_statistics() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    assert_eq!(batch.frame_count(), 0);
    batch.begin();
    assert_eq!(batch.frame_count(), 1);

    assert_eq!(batch.batch_ratio(), 0.0);
}

// =========================================================================
// Sorting
// =========================================================================

#[test]
#[ignore] // Requires OpenGL context
fn test_sprite_batch_sort_z_layer() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    batch.sprites = vec![
        SpriteInstance {
            entity: Entity::new(0, 0),
            texture: AssetHandle::new(0, 0),
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 10.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(1, 0),
            texture: AssetHandle::new(0, 0),
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 5.0,
            flip_x: false,
            flip_y: false,
        },
    ];

    batch.sort_sprites();
    assert_eq!(batch.sprites[0].z_layer, 5.0);
    assert_eq!(batch.sprites[1].z_layer, 10.0);
}
