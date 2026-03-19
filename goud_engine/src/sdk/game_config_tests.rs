use super::*;
use crate::libs::graphics::AntiAliasingMode;

// =========================================================================
// GameConfig Tests
// =========================================================================

#[test]
fn test_game_config_default() {
    let config = GameConfig::default();
    assert_eq!(config.title, "GoudEngine Game");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.vsync);
    assert!(!config.fullscreen);
    assert_eq!(config.anti_aliasing_mode, AntiAliasingMode::Off);
    assert_eq!(config.msaa_samples, 1);
    assert_eq!(config.render_backend, RenderBackendKind::Wgpu);
    assert_eq!(config.window_backend, WindowBackendKind::Winit);
}

#[test]
fn test_game_config_new() {
    let config = GameConfig::new("Test Game", 1920, 1080);
    assert_eq!(config.title, "Test Game");
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
}

#[test]
fn test_game_config_builder() {
    let config = GameConfig::default()
        .with_title("Builder Game")
        .with_size(640, 480)
        .with_vsync(false)
        .with_fullscreen(true)
        .with_anti_aliasing_mode(AntiAliasingMode::MsaaFxaa)
        .with_msaa_samples(8)
        .with_render_backend(RenderBackendKind::OpenGlLegacy)
        .with_window_backend(WindowBackendKind::GlfwLegacy)
        .with_target_fps(144);

    assert_eq!(config.title, "Builder Game");
    assert_eq!(config.width, 640);
    assert_eq!(config.height, 480);
    assert!(!config.vsync);
    assert!(config.fullscreen);
    assert_eq!(config.anti_aliasing_mode, AntiAliasingMode::MsaaFxaa);
    assert_eq!(config.msaa_samples, 8);
    assert_eq!(config.render_backend, RenderBackendKind::OpenGlLegacy);
    assert_eq!(config.window_backend, WindowBackendKind::GlfwLegacy);
    assert_eq!(config.target_fps, 144);
}

#[test]
fn test_game_config_msaa_samples_are_sanitized() {
    let config = GameConfig::default().with_msaa_samples(3);
    assert_eq!(config.msaa_samples, 1);
}

#[test]
fn test_backend_kind_from_u32() {
    assert_eq!(
        RenderBackendKind::from_u32(0),
        Some(RenderBackendKind::Wgpu)
    );
    assert_eq!(
        RenderBackendKind::from_u32(1),
        Some(RenderBackendKind::OpenGlLegacy)
    );
    assert_eq!(RenderBackendKind::from_u32(99), None);

    assert_eq!(
        WindowBackendKind::from_u32(0),
        Some(WindowBackendKind::Winit)
    );
    assert_eq!(
        WindowBackendKind::from_u32(1),
        Some(WindowBackendKind::GlfwLegacy)
    );
    assert_eq!(WindowBackendKind::from_u32(99), None);
}

// =========================================================================
// GameContext Tests
// =========================================================================

#[test]
fn test_game_context_new() {
    let ctx = GameContext::new((800, 600));
    assert_eq!(ctx.delta_time(), 0.0);
    assert_eq!(ctx.total_time(), 0.0);
    assert_eq!(ctx.frame_count(), 0);
    assert_eq!(ctx.window_size(), (800, 600));
    assert!(ctx.is_running());
}

#[test]
fn test_game_context_update() {
    let mut ctx = GameContext::new((800, 600));
    ctx.update(0.016); // ~60 FPS

    assert!((ctx.delta_time() - 0.016).abs() < 0.001);
    assert!((ctx.total_time() - 0.016).abs() < 0.001);
    assert_eq!(ctx.frame_count(), 1);
    assert!((ctx.fps() - 62.5).abs() < 1.0);
}

#[test]
fn test_game_context_quit() {
    let mut ctx = GameContext::new((800, 600));
    assert!(ctx.is_running());

    ctx.quit();
    assert!(!ctx.is_running());
}
