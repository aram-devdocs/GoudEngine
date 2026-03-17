use super::*;
use crate::sdk::GameConfig;

#[test]
fn test_begin_2d_render_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(game.begin_2d_render().is_err());
}

#[test]
fn test_end_2d_render_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(game.end_2d_render().is_err());
}

#[test]
fn test_draw_sprites_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(game.draw_sprites().is_err());
}

#[test]
fn test_render_2d_stats_headless() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert_eq!(game.render_2d_stats(), (0, 0, 0.0));
}

#[test]
fn test_has_2d_renderer_headless() {
    let game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.has_2d_renderer());
}

#[test]
fn test_renderer_2d_facade_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    let mut renderer = game.renderer_2d();
    assert!(!renderer.is_available());
    assert!(renderer.begin().is_err());
    assert!(renderer.create_render_target(64, 64, false).is_err());
    assert!(renderer.bind_render_target(1).is_err());
    assert!(renderer.bind_default_render_target().is_err());
    assert_eq!(renderer.render_target_texture(1), None);
    assert!(!renderer.destroy_render_target(1));
    assert_eq!(renderer.stats(), (0, 0, 0.0));
}

#[test]
fn test_draw_sprite_headless_returns_false() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.draw_sprite(0, 0.0, 0.0, 10.0, 10.0, 0.0, 1.0, 1.0, 1.0, 1.0));
}

#[test]
fn test_draw_quad_headless_returns_false() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.draw_quad(0.0, 0.0, 10.0, 10.0, 1.0, 0.0, 0.0, 1.0));
}

#[test]
fn test_begin_render_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.begin_render());
}

#[test]
fn test_end_render_headless() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    assert!(!game.end_render());
}

#[test]
fn test_ortho_matrix_identity_like() {
    let m = immediate::ortho_matrix(0.0, 2.0, 0.0, 2.0);
    assert!((m[0] - 1.0).abs() < 0.001);
    assert!((m[5] - 1.0).abs() < 0.001);
}

#[test]
fn test_model_matrix_no_rotation() {
    let m = immediate::model_matrix(10.0, 20.0, 5.0, 5.0, 0.0);
    assert!((m[12] - 10.0).abs() < 0.001);
    assert!((m[13] - 20.0).abs() < 0.001);
}

#[cfg(feature = "native")]
#[test]
fn test_render_viewport_prefers_bound_render_target_override() {
    let mut game = GoudGame::new(GameConfig::default()).unwrap();
    game.bound_render_target_viewport = Some((7, RenderViewport::fullscreen((64, 32))));

    assert_eq!(game.render_viewport(), RenderViewport::fullscreen((64, 32)));
}
