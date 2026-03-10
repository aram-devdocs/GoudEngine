//! Tests for [`GoudGame`].

use crate::core::math::Vec2;
use crate::sdk::components::{GlobalTransform2D, Transform2D};
use crate::sdk::engine_config::EngineConfig;
use crate::sdk::game::GoudGame;
use crate::sdk::game_config::GameConfig;
use crate::ui::{UiAnchor, UiButton, UiComponent, UiImage, UiLabel};
use glfw::{Key, MouseButton};

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
// Active Scene Update Tests
// =========================================================================

#[test]
fn test_run_updates_all_active_scenes() {
    let mut game = GoudGame::default();
    let scene_b = game.create_scene("b").unwrap();
    game.set_scene_active(scene_b, true).unwrap();

    // Both default and scene_b are active.
    assert_eq!(game.scene_manager().active_scenes().len(), 2);

    // Each callback invocation spawns an entity in the world it receives.
    game.run(|ctx, world| {
        world.spawn_empty();
        if ctx.frame_count() >= 1 {
            ctx.quit();
        }
    });

    // After 1 frame, both scenes should have 1 entity each.
    let default_id = game.scene_manager().default_scene();
    assert_eq!(game.scene(default_id).unwrap().entity_count(), 1);
    assert_eq!(game.scene(scene_b).unwrap().entity_count(), 1);
}

#[test]
fn test_inactive_scene_not_updated() {
    let mut game = GoudGame::default();
    let scene_b = game.create_scene("b").unwrap();
    // scene_b is NOT activated -- only default is active.

    game.update_frame(0.016, |_ctx, world| {
        world.spawn_empty();
    });

    let default_id = game.scene_manager().default_scene();
    assert_eq!(game.scene(default_id).unwrap().entity_count(), 1);
    assert_eq!(game.scene(scene_b).unwrap().entity_count(), 0);
}

#[test]
fn test_game_hud_simultaneous() {
    let mut game = GoudGame::default();
    let hud = game.create_scene("hud").unwrap();
    game.set_scene_active(hud, true).unwrap();

    // Run 3 frames, each spawns an entity per active world.
    game.run(|ctx, world| {
        world.spawn_empty();
        if ctx.frame_count() >= 3 {
            ctx.quit();
        }
    });

    let default_id = game.scene_manager().default_scene();
    // Each scene gets 3 entities (one per frame).
    assert_eq!(game.scene(default_id).unwrap().entity_count(), 3);
    assert_eq!(game.scene(hud).unwrap().entity_count(), 3);
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

// =========================================================================
// EngineConfig Integration Tests
// =========================================================================

#[test]
fn test_goud_game_from_engine_config() {
    let config = EngineConfig::new()
        .with_title("Config Game")
        .with_size(1024, 768);
    let game = GoudGame::from_engine_config(config).unwrap();
    assert_eq!(game.config().title, "Config Game");
    assert_eq!(game.config().width, 1024);
}

#[test]
fn test_goud_game_from_engine_config_preserves_settings() {
    let config = EngineConfig::new()
        .with_title("Settings Test")
        .with_size(640, 480)
        .with_vsync(false)
        .with_fullscreen(true)
        .with_target_fps(120);
    let game = GoudGame::from_engine_config(config).unwrap();
    assert_eq!(game.config().title, "Settings Test");
    assert_eq!(game.window_size(), (640, 480));
    assert!(!game.config().vsync);
    assert!(game.config().fullscreen);
    assert_eq!(game.config().target_fps, 120);
}

#[test]
fn test_goud_game_providers_accessible() {
    let game = GoudGame::from_engine_config(EngineConfig::new()).unwrap();
    let providers = game.providers();
    assert_eq!(providers.render.name(), "null");
    assert_eq!(providers.physics.name(), "null");
    assert_eq!(providers.audio.name(), "null");
    assert_eq!(providers.input.name(), "null");
}

#[test]
fn test_backward_compat_new() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert_eq!(game.config().title, "GoudEngine Game");
    assert_eq!(game.providers().render.name(), "null");
}

#[test]
fn test_backward_compat_default() {
    let game = GoudGame::default();
    assert_eq!(game.config().width, 800);
    assert_eq!(game.config().height, 600);
    assert_eq!(game.providers().render.name(), "null");
}

#[test]
fn test_update_frame_ui_consumes_mouse_event_before_game_queries() {
    let mut game = GoudGame::default();
    let button = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = game.ui_manager_mut().get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(100.0, 40.0));
    }

    game.input_mut().set_mouse_position(Vec2::new(10.0, 10.0));
    game.input_mut().press_mouse_button(MouseButton::Button1);

    game.update_frame(0.016, |_ctx, _world| {});

    assert!(!game.is_mouse_button_just_pressed(MouseButton::Button1));
    assert!(!game.is_mouse_button_pressed(MouseButton::Button1));
}

#[test]
fn test_update_frame_ui_consumes_tab_and_enter_activation() {
    let mut game = GoudGame::default();
    let button = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Button(UiButton::default())));

    {
        let node = game.ui_manager_mut().get_node_mut(button).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(100.0, 40.0));
    }

    game.input_mut().press_key(Key::Tab);
    game.update_frame(0.016, |_ctx, _world| {});
    assert!(!game.is_key_just_pressed(Key::Tab));
    game.ui_manager_mut().take_events();

    game.input_mut().update();
    game.input_mut().release_key(Key::Tab);
    game.update_frame(0.016, |_ctx, _world| {});
    game.ui_manager_mut().take_events();

    game.input_mut().update();
    game.input_mut().press_key(Key::Enter);
    game.update_frame(0.016, |_ctx, _world| {});

    assert!(!game.is_key_just_pressed(Key::Enter));
    let events = game.ui_manager_mut().take_events();
    assert!(events
        .iter()
        .any(|event| matches!(event, crate::ui::UiEvent::Click(id) if *id == button)));
}

#[test]
fn test_update_frame_headless_with_ui_render_commands_is_safe() {
    let mut game = GoudGame::default();

    let panel = game.ui_manager_mut().create_node(Some(UiComponent::Panel));
    let label = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Label(UiLabel::new("Headless UI"))));
    let image = game
        .ui_manager_mut()
        .create_node(Some(UiComponent::Image(UiImage::new(
            "ui://fixture-checker",
        ))));

    game.ui_manager_mut()
        .set_parent(label, Some(panel))
        .unwrap();
    game.ui_manager_mut()
        .set_parent(image, Some(panel))
        .unwrap();

    game.ui_manager_mut()
        .get_node_mut(panel)
        .unwrap()
        .set_size(Vec2::new(320.0, 100.0));
    game.ui_manager_mut()
        .get_node_mut(label)
        .unwrap()
        .set_size(Vec2::new(220.0, 24.0));
    game.ui_manager_mut()
        .get_node_mut(image)
        .unwrap()
        .set_size(Vec2::new(64.0, 64.0));

    // No platform/render backend in this test; update_frame must remain safe.
    game.update_frame(0.016, |_ctx, _world| {});

    let commands = game.ui_manager_mut().build_render_commands();
    assert!(!commands.is_empty());
}
