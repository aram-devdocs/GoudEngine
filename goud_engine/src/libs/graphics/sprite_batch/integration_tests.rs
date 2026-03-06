//! Integration tests for the sprite batch renderer that verify ECS world interaction.
//!
//! All tests here are marked `#[ignore]` because they need a live OpenGL context.

#[cfg(test)]
mod integration_tests {
    use crate::core::math::{Color, Vec2};
    use crate::ecs::Entity;
    use crate::libs::graphics::backend::opengl::OpenGLBackend;
    use crate::libs::graphics::sprite_batch::batch::SpriteBatch;
    use crate::libs::graphics::sprite_batch::config::SpriteBatchConfig;

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_gather_sprites_from_world() {
        use crate::assets::AssetServer;
        use crate::core::math::Rect;
        use crate::ecs::components::{Sprite, Transform2D};
        use crate::ecs::World;

        let mut world = World::new();
        let mut asset_server = AssetServer::new();

        let texture = asset_server.load::<crate::assets::loaders::TextureAsset>("test.png");

        let e1 = world.spawn_empty();
        world.insert(e1, Transform2D::from_position(Vec2::new(10.0, 20.0)));
        world.insert(e1, Sprite::new(texture));

        let e2 = world.spawn_empty();
        world.insert(e2, Transform2D::from_position(Vec2::new(30.0, 40.0)));
        world.insert(e2, Sprite::new(texture).with_color(Color::RED));

        let e3 = world.spawn_empty();
        world.insert(e3, Transform2D::from_position(Vec2::new(50.0, 60.0)));
        world.insert(
            e3,
            Sprite::new(texture).with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0)),
        );

        // Entity without Sprite — should be ignored by the gather pass
        let e4 = world.spawn_empty();
        world.insert(e4, Transform2D::from_position(Vec2::new(70.0, 80.0)));

        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default())
            .expect("Failed to create sprite batch");

        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");

        assert_eq!(batch.sprite_count(), 3);

        let entities: Vec<Entity> = batch.sprites.iter().map(|s| s.entity).collect();
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
        assert!(!entities.contains(&e4));

        let sprite1 = batch.sprites.iter().find(|s| s.entity == e1).unwrap();
        assert_eq!(sprite1.color, Color::WHITE);

        let sprite2 = batch.sprites.iter().find(|s| s.entity == e2).unwrap();
        assert_eq!(sprite2.color, Color::RED);

        let sprite3 = batch.sprites.iter().find(|s| s.entity == e3).unwrap();
        assert!(sprite3.source_rect.is_some());
        let source = sprite3.source_rect.unwrap();
        assert_eq!(source.x, 0.0);
        assert_eq!(source.y, 0.0);
        assert_eq!(source.width, 32.0);
        assert_eq!(source.height, 32.0);

        // Z-layers derive from Y position
        assert_eq!(sprite1.z_layer, 20.0);
        assert_eq!(sprite2.z_layer, 40.0);
        assert_eq!(sprite3.z_layer, 60.0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_gather_sprites_empty_world() {
        use crate::assets::AssetServer;
        use crate::ecs::World;

        let world = World::new();
        let _asset_server = AssetServer::new();

        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default())
            .expect("Failed to create sprite batch");

        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");
        assert_eq!(batch.sprite_count(), 0);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_gather_sprites_clears_previous_frame() {
        use crate::assets::AssetServer;
        use crate::ecs::components::{Sprite, Transform2D};
        use crate::ecs::World;

        let mut world = World::new();
        let mut asset_server = AssetServer::new();

        let texture = asset_server.load::<crate::assets::loaders::TextureAsset>("test.png");

        let backend = OpenGLBackend::new().expect("Failed to create OpenGL backend");
        let mut batch = SpriteBatch::new(backend, SpriteBatchConfig::default())
            .expect("Failed to create sprite batch");

        // First frame: 2 sprites
        let e1 = world.spawn_empty();
        world.insert(e1, Transform2D::from_position(Vec2::new(10.0, 20.0)));
        world.insert(e1, Sprite::new(texture));

        let e2 = world.spawn_empty();
        world.insert(e2, Transform2D::from_position(Vec2::new(30.0, 40.0)));
        world.insert(e2, Sprite::new(texture));

        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");
        assert_eq!(batch.sprite_count(), 2);

        // Second frame: despawn one entity
        world.despawn(e2);

        batch
            .gather_sprites(&world)
            .expect("Failed to gather sprites");
        // Should only have 1 sprite, not accumulate across frames
        assert_eq!(batch.sprite_count(), 1);
    }
}
