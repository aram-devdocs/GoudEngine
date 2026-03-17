//! Game configuration and runtime context types.
//!
//! Contains [`GameConfig`] for initialization settings and [`GameContext`]
//! for per-frame runtime state passed to update callbacks.

use crate::core::debugger::DebuggerConfig;
pub use crate::libs::platform::{RenderBackendKind, WindowBackendKind};

// =============================================================================
// Game Configuration
// =============================================================================

/// Configuration for creating a GoudGame instance.
///
/// This struct holds all the settings needed to initialize the game engine,
/// including window properties, rendering options, and engine settings.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::GameConfig;
///
/// let config = GameConfig {
///     title: "My Awesome Game".to_string(),
///     width: 1280,
///     height: 720,
///     vsync: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicsDebugConfig {
    /// Enables runtime physics debug visualization and shape collection.
    pub enabled: bool,
}

/// Configuration for creating a GoudGame instance.
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Window title displayed in the title bar.
    pub title: String,

    /// Window width in pixels.
    pub width: u32,

    /// Window height in pixels.
    pub height: u32,

    /// Enable vertical sync to prevent screen tearing.
    pub vsync: bool,

    /// Enable fullscreen mode.
    pub fullscreen: bool,

    /// Enable window resizing.
    pub resizable: bool,

    /// Native render backend selection.
    pub render_backend: RenderBackendKind,

    /// Native window backend selection.
    pub window_backend: WindowBackendKind,

    /// Target frames per second (0 = unlimited).
    pub target_fps: u32,

    /// Enable debug rendering (collision boxes, etc.).
    pub debug_rendering: bool,

    /// Show the FPS stats overlay.
    pub show_fps_overlay: bool,

    /// Physics debug visualization settings.
    pub physics_debug: PhysicsDebugConfig,

    /// How often (in seconds) the FPS overlay recomputes statistics.
    pub fps_update_interval: f32,

    /// Enable diagnostic mode for detailed engine telemetry and error reporting.
    pub diagnostic_mode: bool,

    /// Debugger runtime configuration.
    pub debugger: DebuggerConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "GoudEngine Game".to_string(),
            width: 800,
            height: 600,
            vsync: true,
            fullscreen: false,
            resizable: true,
            render_backend: RenderBackendKind::Wgpu,
            window_backend: WindowBackendKind::Winit,
            target_fps: 60,
            debug_rendering: false,
            show_fps_overlay: false,
            physics_debug: PhysicsDebugConfig::default(),
            fps_update_interval: 0.5,
            diagnostic_mode: false,
            debugger: DebuggerConfig::default(),
        }
    }
}

impl GameConfig {
    /// Creates a new configuration with the given title and dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::sdk::GameConfig;
    ///
    /// let config = GameConfig::new("My Game", 800, 600);
    /// ```
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            ..Default::default()
        }
    }

    /// Sets the window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the window dimensions.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Enables or disables vsync.
    pub fn with_vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Enables or disables fullscreen mode.
    pub fn with_fullscreen(mut self, enabled: bool) -> Self {
        self.fullscreen = enabled;
        self
    }

    /// Selects the native render backend.
    pub fn with_render_backend(mut self, backend: RenderBackendKind) -> Self {
        self.render_backend = backend;
        self
    }

    /// Selects the native window backend.
    pub fn with_window_backend(mut self, backend: WindowBackendKind) -> Self {
        self.window_backend = backend;
        self
    }

    /// Sets the target FPS (0 for unlimited).
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }

    /// Enables or disables the FPS stats overlay.
    pub fn with_fps_overlay(mut self, enabled: bool) -> Self {
        self.show_fps_overlay = enabled;
        self
    }

    /// Enables or disables physics debug visualization.
    pub fn with_physics_debug(mut self, enabled: bool) -> Self {
        self.physics_debug.enabled = enabled;
        self
    }

    /// Sets how often (in seconds) the FPS overlay recomputes statistics.
    pub fn with_fps_update_interval(mut self, interval: f32) -> Self {
        self.fps_update_interval = interval;
        self
    }

    /// Enables or disables diagnostic mode for detailed engine telemetry.
    pub fn with_diagnostic_mode(mut self, enabled: bool) -> Self {
        self.diagnostic_mode = enabled;
        self
    }

    /// Replaces the debugger runtime configuration.
    pub fn with_debugger(mut self, debugger: DebuggerConfig) -> Self {
        self.debugger = debugger;
        self
    }
}

// =============================================================================
// Game Context (passed to update callback)
// =============================================================================

/// Runtime context passed to the game update callback.
///
/// This struct provides access to frame timing, input state, and other
/// runtime information needed during the game loop.
///
/// # Example
///
/// ```rust,ignore
/// game.run(|ctx| {
///     let dt = ctx.delta_time();
///     let fps = ctx.fps();
///
///     // Move something based on time
///     position.x += velocity * dt;
/// });
/// ```
#[derive(Debug)]
pub struct GameContext {
    /// Time elapsed since last frame in seconds.
    delta_time: f32,

    /// Total time elapsed since game start in seconds.
    total_time: f32,

    /// Current frames per second.
    fps: f32,

    /// Current frame number.
    frame_count: u64,

    /// Window dimensions.
    window_size: (u32, u32),

    /// Whether the game should continue running.
    running: bool,
}

impl GameContext {
    /// Creates a new game context with default values.
    pub(crate) fn new(window_size: (u32, u32)) -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
            fps: 0.0,
            frame_count: 0,
            window_size,
            running: true,
        }
    }

    /// Returns the time elapsed since the last frame in seconds.
    ///
    /// Use this for frame-rate independent movement and animations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Move at 100 pixels per second regardless of frame rate
    /// position.x += 100.0 * ctx.delta_time();
    /// ```
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Returns the total time elapsed since game start in seconds.
    #[inline]
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Returns the current frames per second.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Returns the current frame number (0-indexed).
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the window dimensions as (width, height).
    #[inline]
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    /// Returns the window width in pixels.
    #[inline]
    pub fn window_width(&self) -> u32 {
        self.window_size.0
    }

    /// Returns the window height in pixels.
    #[inline]
    pub fn window_height(&self) -> u32 {
        self.window_size.1
    }

    /// Returns true if the game is still running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Signals the game to exit after the current frame.
    #[inline]
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Updates the context for a new frame.
    pub(crate) fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.total_time += delta_time;
        self.frame_count += 1;

        // Simple FPS calculation (could be smoothed)
        if delta_time > 0.0 {
            self.fps = 1.0 / delta_time;
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
            .with_render_backend(RenderBackendKind::OpenGlLegacy)
            .with_window_backend(WindowBackendKind::GlfwLegacy)
            .with_target_fps(144);

        assert_eq!(config.title, "Builder Game");
        assert_eq!(config.width, 640);
        assert_eq!(config.height, 480);
        assert!(!config.vsync);
        assert!(config.fullscreen);
        assert_eq!(config.render_backend, RenderBackendKind::OpenGlLegacy);
        assert_eq!(config.window_backend, WindowBackendKind::GlfwLegacy);
        assert_eq!(config.target_fps, 144);
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
}
