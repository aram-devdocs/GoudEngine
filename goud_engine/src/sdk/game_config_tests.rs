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
    assert_eq!(config.fullscreen_mode, FullscreenMode::Windowed);
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
    assert_eq!(config.fullscreen_mode, FullscreenMode::Borderless);
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

// =========================================================================
// Fixed Timestep Tests
// =========================================================================

#[test]
fn test_fixed_timestep_disabled_by_default() {
    let config = GameConfig::default();
    assert_eq!(config.fixed_timestep, 0.0);
    assert_eq!(config.max_fixed_steps_per_frame, 8);

    let ctx = GameContext::new((800, 600));
    assert!(!ctx.is_fixed_timestep_enabled());
    assert_eq!(ctx.fixed_timestep(), 0.0);
}

#[test]
fn test_fixed_timestep_config_builder() {
    let config = GameConfig::default()
        .with_fixed_timestep(1.0 / 60.0)
        .with_max_fixed_steps_per_frame(4);

    assert!((config.fixed_timestep - 1.0 / 60.0).abs() < f32::EPSILON);
    assert_eq!(config.max_fixed_steps_per_frame, 4);
}

#[test]
fn test_fixed_timestep_config_clamps_negative() {
    let config = GameConfig::default().with_fixed_timestep(-1.0);
    assert_eq!(config.fixed_timestep, 0.0);

    let config = GameConfig::default().with_max_fixed_steps_per_frame(0);
    assert_eq!(config.max_fixed_steps_per_frame, 1);
}

#[test]
fn test_accumulator_single_step() {
    let mut ctx = GameContext::new((800, 600));
    ctx.configure_fixed_timestep(1.0 / 60.0, 8);

    // Simulate a frame of ~33ms (30 FPS) — should allow 1 step at 1/60
    ctx.begin_frame_accumulator(0.033);

    assert!(ctx.consume_fixed_step());
    // After one step (0.01667s consumed), remainder ~0.01633
    assert!(!ctx.consume_fixed_step());

    ctx.finish_accumulator();
    // alpha = remainder / step ≈ 0.01633 / 0.01667 ≈ 0.98
    assert!(ctx.interpolation_alpha() > 0.0);
    assert!(ctx.interpolation_alpha() < 1.0);
}

#[test]
fn test_accumulator_multiple_steps() {
    let mut ctx = GameContext::new((800, 600));
    let step = 1.0 / 60.0;
    ctx.configure_fixed_timestep(step, 8);

    // 50ms frame at 60Hz (step ≈ 16.67ms): floor(50/16.67) = 2 steps
    // (floating point: 3*step > 0.05 due to rounding)
    ctx.begin_frame_accumulator(0.05);

    let mut count = 0;
    while ctx.consume_fixed_step() {
        count += 1;
    }
    assert_eq!(count, 2);

    ctx.finish_accumulator();
    assert!(ctx.interpolation_alpha() >= 0.0);
    assert!(ctx.interpolation_alpha() < 1.0);
}

#[test]
fn test_accumulator_max_steps_cap() {
    let mut ctx = GameContext::new((800, 600));
    let step = 1.0 / 60.0;
    ctx.configure_fixed_timestep(step, 8);

    // 1 second spike — would be 60 steps uncapped
    ctx.begin_frame_accumulator(1.0);

    let mut count = 0;
    while ctx.consume_fixed_step() {
        count += 1;
    }
    assert_eq!(count, 8);

    // Alpha must be clamped to 1.0 even though accumulator >> step
    ctx.finish_accumulator();
    assert!(ctx.interpolation_alpha() <= 1.0);
}

#[test]
fn test_accumulator_disabled_returns_false() {
    let mut ctx = GameContext::new((800, 600));
    // fixed_timestep stays 0.0 (disabled)
    ctx.begin_frame_accumulator(0.016);
    assert!(!ctx.consume_fixed_step());
}

#[test]
fn test_accumulator_carries_remainder() {
    let mut ctx = GameContext::new((800, 600));
    let step = 1.0 / 60.0;
    ctx.configure_fixed_timestep(step, 8);

    // Frame 1: slightly less than one step
    ctx.begin_frame_accumulator(0.015);
    assert!(!ctx.consume_fixed_step());

    // Frame 2: another partial — now total accumulated ~0.030 > 1/60
    ctx.begin_frame_accumulator(0.015);
    assert!(ctx.consume_fixed_step());
    assert!(!ctx.consume_fixed_step());
}

#[test]
fn test_interpolation_alpha_zero_when_disabled() {
    let mut ctx = GameContext::new((800, 600));
    ctx.begin_frame_accumulator(0.016);
    ctx.finish_accumulator();
    assert_eq!(ctx.interpolation_alpha(), 0.0);
}
