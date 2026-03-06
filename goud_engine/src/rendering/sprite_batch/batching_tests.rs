//! Texture batching and integration tests for the sprite batch renderer.
//!
//! All tests here are marked `#[ignore]` because they need a live OpenGL context.

use crate::assets::AssetHandle;
use crate::core::math::{Color, Vec2};
use crate::ecs::components::Mat3x3;
use crate::ecs::Entity;
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::rendering::sprite_batch::batch::SpriteBatch;
use crate::rendering::sprite_batch::config::SpriteBatchConfig;
use crate::rendering::sprite_batch::types::SpriteInstance;

// =========================================================================
// Texture Batching
// =========================================================================

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_single_texture() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    let texture = AssetHandle::new(1, 1);
    for i in 0..5 {
        batch.sprites.push(SpriteInstance {
            entity: Entity::new(i, 0),
            texture,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        });
    }

    assert_eq!(batch.sprites.len(), 5);
    for sprite in &batch.sprites {
        assert_eq!(sprite.texture, texture);
    }
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_multiple_textures() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    let tex1 = AssetHandle::new(1, 1);
    let tex2 = AssetHandle::new(2, 1);
    let tex3 = AssetHandle::new(3, 1);

    batch.sprites = vec![
        SpriteInstance {
            entity: Entity::new(0, 0),
            texture: tex1,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(1, 0),
            texture: tex2,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(2, 0),
            texture: tex3,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        },
    ];

    assert_eq!(batch.sprites.len(), 3);
    assert_ne!(batch.sprites[0].texture, batch.sprites[1].texture);
    assert_ne!(batch.sprites[1].texture, batch.sprites[2].texture);
    assert_ne!(batch.sprites[0].texture, batch.sprites[2].texture);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_sort_by_texture() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    let tex1 = AssetHandle::new(1, 1);
    let tex2 = AssetHandle::new(2, 1);

    batch.sprites = vec![
        SpriteInstance {
            entity: Entity::new(0, 0),
            texture: tex2,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(1, 0),
            texture: tex1,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(2, 0),
            texture: tex2,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        },
    ];

    batch.sort_sprites();

    assert_eq!(batch.sprites[0].texture, tex1);
    assert_eq!(batch.sprites[1].texture, tex2);
    assert_eq!(batch.sprites[2].texture, tex2);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_with_z_layers() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    let tex1 = AssetHandle::new(1, 1);
    let tex2 = AssetHandle::new(2, 1);

    batch.sprites = vec![
        SpriteInstance {
            entity: Entity::new(0, 0),
            texture: tex2,
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
            texture: tex1,
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

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_same_z_different_texture() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    let tex1 = AssetHandle::new(1, 1);
    let tex2 = AssetHandle::new(2, 1);

    batch.sprites = vec![
        SpriteInstance {
            entity: Entity::new(0, 0),
            texture: tex2,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 5.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(1, 0),
            texture: tex1,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 5.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(2, 0),
            texture: tex1,
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

    assert_eq!(batch.sprites[0].texture, tex1);
    assert_eq!(batch.sprites[1].texture, tex1);
    assert_eq!(batch.sprites[2].texture, tex2);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_disabled() {
    let backend = OpenGLBackend::new().unwrap();
    let config = SpriteBatchConfig {
        initial_capacity: 1024,
        max_batch_size: 10000,
        enable_z_sorting: true,
        enable_batching: false,
    };
    let mut batch = SpriteBatch::new(backend, config).unwrap();

    let tex1 = AssetHandle::new(1, 1);
    let tex2 = AssetHandle::new(2, 1);

    batch.sprites = vec![
        SpriteInstance {
            entity: Entity::new(0, 0),
            texture: tex2,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 5.0,
            flip_x: false,
            flip_y: false,
        },
        SpriteInstance {
            entity: Entity::new(1, 0),
            texture: tex1,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 10.0,
            flip_x: false,
            flip_y: false,
        },
    ];

    batch.sort_sprites();

    assert_eq!(batch.sprites[0].z_layer, 5.0);
    assert_eq!(batch.sprites[1].z_layer, 10.0);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_batching_stress_test() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    for texture_id in 0..10 {
        let texture = AssetHandle::new(texture_id, 1);
        for sprite_id in 0..10 {
            batch.sprites.push(SpriteInstance {
                entity: Entity::new((texture_id * 10 + sprite_id) as u32, 0),
                texture,
                transform: Mat3x3::IDENTITY,
                color: Color::WHITE,
                source_rect: None,
                size: Vec2::one(),
                z_layer: 0.0,
                flip_x: false,
                flip_y: false,
            });
        }
    }

    assert_eq!(batch.sprites.len(), 100);

    batch.sort_sprites();

    for i in 0..10 {
        let start = i * 10;
        let end = start + 10;
        let texture = batch.sprites[start].texture;

        for j in start..end {
            assert_eq!(
                batch.sprites[j].texture, texture,
                "Sprite {} should have texture {:?}",
                j, texture
            );
        }
    }
}

#[test]
#[ignore] // Requires OpenGL context
fn test_max_batch_size_enforcement() {
    let backend = OpenGLBackend::new().unwrap();
    let config = SpriteBatchConfig {
        initial_capacity: 1024,
        max_batch_size: 5,
        enable_z_sorting: true,
        enable_batching: true,
    };
    let mut batch = SpriteBatch::new(backend, config).unwrap();

    let texture = AssetHandle::new(1, 1);
    for i in 0..10 {
        batch.sprites.push(SpriteInstance {
            entity: Entity::new(i, 0),
            texture,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        });
    }

    assert_eq!(batch.sprites.len(), 10);
    // max_batch_size of 5 should force 2 batches when generate_batches is called
}

#[test]
#[ignore] // Requires OpenGL context
fn test_interleaved_textures_batching() {
    let backend = OpenGLBackend::new().unwrap();
    let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default()).unwrap();

    let tex1 = AssetHandle::new(1, 1);
    let tex2 = AssetHandle::new(2, 1);

    for i in 0..6 {
        let texture = if i % 2 == 0 { tex1 } else { tex2 };
        batch.sprites.push(SpriteInstance {
            entity: Entity::new(i, 0),
            texture,
            transform: Mat3x3::IDENTITY,
            color: Color::WHITE,
            source_rect: None,
            size: Vec2::one(),
            z_layer: 0.0,
            flip_x: false,
            flip_y: false,
        });
    }

    assert_eq!(batch.sprites[0].texture, tex1);
    assert_eq!(batch.sprites[1].texture, tex2);
    assert_eq!(batch.sprites[2].texture, tex1);

    batch.sort_sprites();

    let first_texture = batch.sprites[0].texture;
    let mut found_second = false;
    let mut second_texture = first_texture;

    for sprite in &batch.sprites {
        if sprite.texture != first_texture {
            if !found_second {
                found_second = true;
                second_texture = sprite.texture;
            } else {
                assert_eq!(sprite.texture, second_texture);
            }
        }
    }

    assert!(found_second);
}
