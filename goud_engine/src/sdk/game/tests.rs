//! Tests for [`GoudGame`].

use crate::core::math::Vec2;
use crate::sdk::components::{GlobalTransform2D, Transform2D};
use crate::sdk::game::GoudGame;
use crate::sdk::game_config::GameConfig;

#[test]
fn test_goud_game_new() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert_eq!(game.entity_count(), 0);
    assert!(!game.is_initialized());
}

#[test]
fn test_goud_game_default() {
    let game = GoudGame::default();
    assert_eq!(game.title(), "GoudEngine Game");
    assert_eq!(game.window_size(), (800, 600));
}

#[test]
fn test_goud_game_spawn() {
    let mut game = GoudGame::default();

    let entity = game
        .spawn()
        .with(Transform2D::from_position(Vec2::new(100.0, 100.0)))
        .build();

    assert!(game.is_alive(entity));
    assert!(game.has::<Transform2D>(entity));
    assert_eq!(game.entity_count(), 1);
}

#[test]
fn test_goud_game_spawn_empty() {
    let mut game = GoudGame::default();

    let entity = game.spawn_empty();
    assert!(game.is_alive(entity));
    assert_eq!(game.entity_count(), 1);
}

#[test]
fn test_goud_game_spawn_batch() {
    let mut game = GoudGame::default();

    let entities = game.spawn_batch(100);
    assert_eq!(entities.len(), 100);
    assert_eq!(game.entity_count(), 100);

    for entity in entities {
        assert!(game.is_alive(entity));
    }
}

#[test]
fn test_goud_game_despawn() {
    let mut game = GoudGame::default();

    let entity = game.spawn_empty();
    assert!(game.is_alive(entity));

    let despawned = game.despawn(entity);
    assert!(despawned);
    assert!(!game.is_alive(entity));
}

#[test]
fn test_goud_game_component_operations() {
    let mut game = GoudGame::default();
    let entity = game.spawn_empty();

    // Insert
    game.insert(entity, Transform2D::from_position(Vec2::new(5.0, 10.0)));
    assert!(game.has::<Transform2D>(entity));

    // Get
    let transform = game.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(5.0, 10.0));

    // Get mut
    {
        let transform = game.get_mut::<Transform2D>(entity).unwrap();
        transform.position.x = 100.0;
    }
    let transform = game.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position.x, 100.0);

    // Remove
    let removed = game.remove::<Transform2D>(entity);
    assert!(removed.is_some());
    assert!(!game.has::<Transform2D>(entity));
}

#[test]
fn test_goud_game_world_access() {
    let mut game = GoudGame::default();

    // Immutable access
    assert_eq!(game.world().entity_count(), 0);

    // Mutable access
    let entity = game.world_mut().spawn_empty();
    assert!(game.world().is_alive(entity));
}

#[test]
fn test_goud_game_update_frame() {
    let mut game = GoudGame::default();
    let mut update_count = 0;

    game.update_frame(0.016, |_ctx, _world| {
        update_count += 1;
    });

    assert_eq!(update_count, 1);
    assert_eq!(game.frame_count(), 1);
}

#[test]
fn test_goud_game_run_with_quit() {
    let mut game = GoudGame::default();
    let mut frame_count = 0;

    game.run(|ctx, _world| {
        frame_count += 1;
        if frame_count >= 5 {
            ctx.quit();
        }
    });

    assert_eq!(frame_count, 5);
    assert!(game.is_initialized());
}

#[test]
fn test_goud_game_debug() {
    let game = GoudGame::default();
    let debug = format!("{:?}", game);

    assert!(debug.contains("GoudGame"));
    assert!(debug.contains("config"));
    assert!(debug.contains("entity_count"));
}

// =========================================================================
// Integration Tests
// =========================================================================

#[test]
fn test_full_game_workflow() {
    // Create game
    let mut game = GoudGame::new(GameConfig::new("Test", 800, 600)).unwrap();

    // Spawn player with components
    let player = game
        .spawn()
        .with(Transform2D::from_position(Vec2::new(400.0, 300.0)))
        .with(GlobalTransform2D::IDENTITY)
        .build();

    // Spawn enemies
    let mut enemies = Vec::new();
    for i in 0..10 {
        let enemy = game
            .spawn()
            .with(Transform2D::from_position(Vec2::new(
                i as f32 * 50.0,
                100.0,
            )))
            .build();
        enemies.push(enemy);
    }

    assert_eq!(game.entity_count(), 11); // 1 player + 10 enemies

    // Run a few frames
    let mut player_moved = false;
    game.run(|ctx, world| {
        // Move player
        if let Some(transform) = world.get_mut::<Transform2D>(player) {
            transform.position.x += 10.0 * ctx.delta_time();
            player_moved = true;
        }

        // Quit after a few frames
        if ctx.frame_count() >= 3 {
            ctx.quit();
        }
    });

    assert!(player_moved);
    assert!(game.is_alive(player));
}
