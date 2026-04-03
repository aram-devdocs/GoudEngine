use super::*;
use crate::sdk::AntiAliasingMode;
use crate::sdk::GameConfig;

#[test]
fn test_create_cube_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert_eq!(game.create_cube(0, 1.0, 1.0, 1.0), u32::MAX);
}

#[test]
fn test_set_object_position_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.set_object_position(0, 1.0, 2.0, 3.0));
}

#[test]
fn test_set_camera_position_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.set_camera_position(0.0, 5.0, -10.0));
}

#[test]
fn test_render_3d_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.render());
}

#[test]
fn test_has_3d_renderer_headless() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.has_3d_renderer());
}

#[test]
fn test_add_light_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    let id = game.add_light(
        0, 0.0, 5.0, 0.0, 0.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 10.0, 0.0,
    );
    assert_eq!(id, u32::MAX);
}

#[test]
fn test_render_all_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.render_all());
}

#[test]
fn test_configure_grid_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.configure_grid(true, 10.0, 10));
}

#[test]
fn test_configure_skybox_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.configure_skybox(true, 0.5, 0.5, 0.8, 1.0));
}

#[test]
fn test_configure_fog_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.configure_fog(true, 0.5, 0.5, 0.5, 0.01));
}

#[test]
fn test_configure_fog_linear_headless() {
    // Without a renderer, configure_fog_linear should return false gracefully
    // (no panic, no UB) -- the SDK wrapper handles missing renderer state.
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.configure_fog_linear(true, 80.0, 200.0, 0.5, 0.5, 0.5));
}

#[test]
fn test_set_fog_enabled_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.set_fog_enabled(true));
}

#[test]
fn test_destroy_object_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.destroy_object(0));
}

#[test]
fn test_runtime_anti_aliasing_mode_updates_config_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.set_anti_aliasing_mode(AntiAliasingMode::Fxaa));
    assert_eq!(game.config.anti_aliasing_mode, AntiAliasingMode::Fxaa);
}

#[test]
fn test_msaa_samples_are_sanitized_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    game.set_msaa_samples(3);
    assert_eq!(game.msaa_samples(), 1);
    game.set_msaa_samples(8);
    assert_eq!(game.msaa_samples(), 8);
}

#[test]
fn test_shadow_bias_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.set_shadow_bias(0.02));
    assert_eq!(game.shadow_bias(), None);
}
