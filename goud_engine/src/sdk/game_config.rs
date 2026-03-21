//! Game configuration and runtime context types.
//!
//! Contains [`GameConfig`] for initialization settings and [`GameContext`]
//! for per-frame runtime state passed to update callbacks.

use crate::core::debugger::DebuggerConfig;
use crate::libs::graphics::AntiAliasingMode;
pub use crate::libs::platform::{FullscreenMode, RenderBackendKind, WindowBackendKind};
use crate::rendering::AspectRatioLock;

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

    /// Fullscreen mode for the window.
    pub fullscreen_mode: FullscreenMode,

    /// Enable window resizing.
    pub resizable: bool,

    /// Runtime anti-aliasing mode for 3D rendering.
    pub anti_aliasing_mode: AntiAliasingMode,

    /// Requested MSAA sample count (1, 2, 4, or 8).
    pub msaa_samples: u32,

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

    /// Whether Lua script hot-reload is enabled.
    ///
    /// Defaults to `true` in debug builds and `false` in release builds.
    pub lua_hot_reload: bool,

    /// Viewport aspect ratio lock.
    pub aspect_ratio_lock: AspectRatioLock,

    /// Fixed timestep size in seconds (0.0 = disabled).
    ///
    /// When enabled, the engine runs simulation updates at a fixed rate
    /// independent of the rendering frame rate.
    pub fixed_timestep: f32,

    /// Maximum fixed steps per frame to prevent spiral of death.
    pub max_fixed_steps_per_frame: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "GoudEngine Game".to_string(),
            width: 800,
            height: 600,
            vsync: true,
            fullscreen_mode: FullscreenMode::Windowed,
            resizable: true,
            anti_aliasing_mode: AntiAliasingMode::Off,
            msaa_samples: 1,
            render_backend: RenderBackendKind::Wgpu,
            window_backend: WindowBackendKind::Winit,
            target_fps: 60,
            debug_rendering: false,
            show_fps_overlay: false,
            physics_debug: PhysicsDebugConfig::default(),
            fps_update_interval: 0.5,
            diagnostic_mode: false,
            debugger: DebuggerConfig::default(),
            lua_hot_reload: cfg!(debug_assertions),
            aspect_ratio_lock: AspectRatioLock::Free,
            fixed_timestep: 0.0,
            max_fixed_steps_per_frame: 8,
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

    /// Enables or disables borderless fullscreen mode (compatibility helper).
    ///
    /// `true` maps to [`FullscreenMode::Borderless`], `false` to
    /// [`FullscreenMode::Windowed`].
    pub fn with_fullscreen(mut self, enabled: bool) -> Self {
        self.fullscreen_mode = if enabled {
            FullscreenMode::Borderless
        } else {
            FullscreenMode::Windowed
        };
        self
    }

    /// Sets the fullscreen mode explicitly.
    pub fn with_fullscreen_mode(mut self, mode: FullscreenMode) -> Self {
        self.fullscreen_mode = mode;
        self
    }

    /// Sets the viewport aspect ratio lock.
    pub fn with_aspect_ratio_lock(mut self, lock: AspectRatioLock) -> Self {
        self.aspect_ratio_lock = lock;
        self
    }

    /// Sets the 3D anti-aliasing mode.
    pub fn with_anti_aliasing_mode(mut self, mode: AntiAliasingMode) -> Self {
        self.anti_aliasing_mode = mode;
        self
    }

    /// Sets the requested MSAA sample count.
    pub fn with_msaa_samples(mut self, samples: u32) -> Self {
        self.msaa_samples = sanitize_msaa_samples(samples);
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

    /// Enables or disables Lua script hot-reload.
    pub fn with_lua_hot_reload(mut self, enabled: bool) -> Self {
        self.lua_hot_reload = enabled;
        self
    }

    /// Sets the fixed timestep size in seconds (e.g. `1.0 / 60.0` for 60 Hz).
    ///
    /// Pass `0.0` to disable fixed timestep mode.
    pub fn with_fixed_timestep(mut self, step: f32) -> Self {
        self.fixed_timestep = step.max(0.0);
        self
    }

    /// Sets the maximum number of fixed steps per frame.
    ///
    /// Caps the accumulator to prevent a spiral of death when the frame
    /// rate drops significantly below the fixed update rate.
    pub fn with_max_fixed_steps_per_frame(mut self, max: u32) -> Self {
        self.max_fixed_steps_per_frame = max.max(1);
        self
    }
}

fn sanitize_msaa_samples(samples: u32) -> u32 {
    match samples {
        2 | 4 | 8 => samples,
        _ => 1,
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

    /// Fixed timestep size in seconds (0.0 = disabled).
    fixed_timestep: f32,

    /// Accumulated time waiting to be consumed by fixed steps.
    accumulator: f32,

    /// Maximum fixed steps allowed per frame.
    max_fixed_steps: u32,

    /// Number of fixed steps consumed this frame.
    fixed_steps_this_frame: u32,

    /// Interpolation alpha for render smoothing (0.0 to 1.0).
    interpolation_alpha: f32,
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
            fixed_timestep: 0.0,
            accumulator: 0.0,
            max_fixed_steps: 8,
            fixed_steps_this_frame: 0,
            interpolation_alpha: 0.0,
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

    /// Returns the configured fixed timestep size in seconds.
    #[inline]
    pub fn fixed_timestep(&self) -> f32 {
        self.fixed_timestep
    }

    /// Returns the interpolation alpha for render smoothing between fixed steps.
    ///
    /// After all fixed steps have been consumed for a frame, this value
    /// represents how far between the last and next fixed step the current
    /// frame sits. Use it to interpolate visual positions for smooth rendering.
    #[inline]
    pub fn interpolation_alpha(&self) -> f32 {
        self.interpolation_alpha
    }

    /// Returns `true` if fixed timestep mode is enabled.
    #[inline]
    pub fn is_fixed_timestep_enabled(&self) -> bool {
        self.fixed_timestep > 0.0
    }

    /// Configures the fixed timestep parameters. Called once at init.
    pub(crate) fn configure_fixed_timestep(&mut self, step: f32, max_steps: u32) {
        self.fixed_timestep = step.max(0.0);
        self.max_fixed_steps = max_steps.max(1);
    }

    /// Begins the per-frame accumulator cycle. Call once per frame before
    /// consuming fixed steps.
    pub(crate) fn begin_frame_accumulator(&mut self, raw_delta: f32) {
        self.accumulator += raw_delta;
        self.fixed_steps_this_frame = 0;
    }

    /// Attempts to consume one fixed step from the accumulator.
    ///
    /// Returns `true` if a step was consumed (caller should run the fixed
    /// update). Returns `false` when the accumulator is exhausted or the
    /// per-frame cap has been reached.
    pub(crate) fn consume_fixed_step(&mut self) -> bool {
        if self.fixed_timestep <= 0.0 {
            return false;
        }
        if self.accumulator >= self.fixed_timestep
            && self.fixed_steps_this_frame < self.max_fixed_steps
        {
            self.accumulator -= self.fixed_timestep;
            self.fixed_steps_this_frame += 1;
            return true;
        }
        false
    }

    /// Finalizes the accumulator for this frame and computes the
    /// interpolation alpha.
    pub(crate) fn finish_accumulator(&mut self) {
        if self.fixed_timestep > 0.0 {
            self.interpolation_alpha = self.accumulator / self.fixed_timestep;
        } else {
            self.interpolation_alpha = 0.0;
        }
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

    /// Updates the current logical window size.
    pub(crate) fn set_window_size(&mut self, window_size: (u32, u32)) {
        self.window_size = (window_size.0.max(1), window_size.1.max(1));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[path = "game_config_tests.rs"]
mod tests;
