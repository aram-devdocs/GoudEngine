//! High-level engine configuration builder.
//!
//! [`EngineConfig`] combines [`GameConfig`] (window/display settings) with
//! [`ProviderRegistryBuilder`] (subsystem backends) into a single fluent
//! builder. This is the recommended way to configure and launch the engine.
//!
//! # Example
//!
//! ```rust
//! use goud_engine::sdk::EngineConfig;
//!
//! let config = EngineConfig::new()
//!     .with_title("My Game")
//!     .with_size(1280, 720)
//!     .with_vsync(true);
//!
//! let (game_config, providers) = config.build();
//! assert_eq!(game_config.title, "My Game");
//! ```

use crate::core::debugger::DebuggerConfig;
use crate::core::providers::audio::AudioProvider;
use crate::core::providers::input::InputProvider;
use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::render::RenderProvider;
use crate::core::providers::types::PhysicsBackend2D;
use crate::core::providers::ProviderRegistry;
use crate::core::providers::ProviderRegistryBuilder;
use crate::libs::graphics::AntiAliasingMode;
#[cfg(feature = "rapier2d")]
use crate::libs::providers::impls::Rapier2DPhysicsProvider;
use crate::libs::providers::impls::SimplePhysicsProvider;
#[cfg(test)]
use crate::sdk::game_config::FullscreenMode;
use crate::sdk::game_config::{GameConfig, RenderBackendKind, WindowBackendKind};

/// Unified engine configuration combining window settings and provider selection.
///
/// `EngineConfig` delegates window/display settings to [`GameConfig`] and
/// provider selection to [`ProviderRegistryBuilder`]. Call [`build()`](Self::build)
/// to consume the builder and obtain both parts.
pub struct EngineConfig {
    game_config: GameConfig,
    physics_backend_2d: PhysicsBackend2D,
    provider_builder: ProviderRegistryBuilder,
}

impl EngineConfig {
    /// Creates a new `EngineConfig` with default settings and null providers.
    pub fn new() -> Self {
        Self {
            game_config: GameConfig::default(),
            physics_backend_2d: PhysicsBackend2D::Default,
            provider_builder: ProviderRegistryBuilder::new(),
        }
    }

    // Window / Display Settings (delegated to GameConfig)

    /// Sets the window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.game_config = self.game_config.with_title(title);
        self
    }

    /// Sets the window dimensions.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.game_config = self.game_config.with_size(width, height);
        self
    }

    /// Enables or disables vertical sync.
    pub fn with_vsync(mut self, enabled: bool) -> Self {
        self.game_config = self.game_config.with_vsync(enabled);
        self
    }

    /// Enables or disables fullscreen mode.
    pub fn with_fullscreen(mut self, enabled: bool) -> Self {
        self.game_config = self.game_config.with_fullscreen(enabled);
        self
    }

    /// Sets the 3D anti-aliasing mode.
    pub fn with_anti_aliasing_mode(mut self, mode: AntiAliasingMode) -> Self {
        self.game_config = self.game_config.with_anti_aliasing_mode(mode);
        self
    }

    /// Sets the requested MSAA sample count.
    pub fn with_msaa_samples(mut self, samples: u32) -> Self {
        self.game_config = self.game_config.with_msaa_samples(samples);
        self
    }

    /// Selects the native render backend.
    pub fn with_render_backend(mut self, backend: RenderBackendKind) -> Self {
        self.game_config = self.game_config.with_render_backend(backend);
        self
    }

    /// Selects the native window backend.
    pub fn with_window_backend(mut self, backend: WindowBackendKind) -> Self {
        self.game_config = self.game_config.with_window_backend(backend);
        self
    }

    /// Sets the target frames per second (0 = unlimited).
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.game_config = self.game_config.with_target_fps(fps);
        self
    }

    /// Enables or disables the FPS stats overlay.
    pub fn with_fps_overlay(mut self, enabled: bool) -> Self {
        self.game_config = self.game_config.with_fps_overlay(enabled);
        self
    }

    /// Enables or disables runtime physics debug visualization.
    pub fn with_physics_debug(mut self, enabled: bool) -> Self {
        self.game_config = self.game_config.with_physics_debug(enabled);
        self
    }

    /// Selects the 2D physics backend used during engine creation.
    pub fn with_physics_backend_2d(mut self, backend: PhysicsBackend2D) -> Self {
        self.physics_backend_2d = backend;
        self
    }

    /// Replaces the entire [`GameConfig`] with the provided one.
    pub fn with_game_config(mut self, config: GameConfig) -> Self {
        self.game_config = config;
        self
    }

    /// Replaces the debugger runtime configuration carried by the inner [`GameConfig`].
    pub fn with_debugger(mut self, debugger: DebuggerConfig) -> Self {
        self.game_config = self.game_config.with_debugger(debugger);
        self
    }

    // Provider Selection (delegated to ProviderRegistryBuilder)

    /// Sets the render provider.
    pub fn with_render_provider(mut self, provider: impl RenderProvider + 'static) -> Self {
        self.provider_builder = self.provider_builder.with_renderer(provider);
        self
    }

    /// Sets the physics provider.
    pub fn with_physics_provider(mut self, provider: impl PhysicsProvider + 'static) -> Self {
        self.provider_builder = self.provider_builder.with_physics(provider);
        self
    }

    /// Sets the audio provider.
    pub fn with_audio_provider(mut self, provider: impl AudioProvider + 'static) -> Self {
        self.provider_builder = self.provider_builder.with_audio(provider);
        self
    }

    /// Sets the input provider.
    pub fn with_input_provider(mut self, provider: impl InputProvider + 'static) -> Self {
        self.provider_builder = self.provider_builder.with_input(provider);
        self
    }

    // Build & Accessors

    /// Consumes the builder and returns the `GameConfig` and `ProviderRegistry`.
    #[cfg(feature = "native")]
    pub(crate) fn set_physics_backend_2d(&mut self, backend: PhysicsBackend2D) {
        self.physics_backend_2d = backend;
    }

    /// Consumes the builder and returns the `GameConfig` and configured `ProviderRegistry`.
    ///
    /// 2D physics selection is applied during build. `PhysicsBackend2D::Default`
    /// keeps existing defaults, while explicit variants install the corresponding provider.
    pub fn build(self) -> (GameConfig, ProviderRegistry) {
        let mut builder = self.provider_builder;
        match self.physics_backend_2d {
            PhysicsBackend2D::Default => {}
            PhysicsBackend2D::Rapier => {
                #[cfg(feature = "rapier2d")]
                {
                    builder = builder.with_physics(Rapier2DPhysicsProvider::new([0.0, 0.0]));
                }
            }
            PhysicsBackend2D::Simple => {
                builder = builder.with_physics(SimplePhysicsProvider::new([0.0, 0.0]));
            }
        }

        (self.game_config, builder.build())
    }

    /// Returns a reference to the current game configuration.
    pub fn game_config(&self) -> &GameConfig {
        &self.game_config
    }

    /// Returns a mutable reference to the current game configuration.
    /// Used for direct field mutation in FFI boundary code.
    #[cfg(feature = "native")]
    pub(crate) fn game_config_mut(&mut self) -> &mut GameConfig {
        &mut self.game_config
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::debugger::DebuggerConfig;
    use crate::core::providers::impls::{
        NullAudioProvider, NullInputProvider, NullPhysicsProvider, NullRenderProvider,
    };
    use crate::libs::graphics::AntiAliasingMode;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        let (game_config, providers) = config.build();
        assert_eq!(game_config.title, "GoudEngine Game");
        assert_eq!(game_config.width, 800);
        assert_eq!(game_config.height, 600);
        assert_eq!(game_config.render_backend, RenderBackendKind::Wgpu);
        assert_eq!(game_config.window_backend, WindowBackendKind::Winit);
        assert_eq!(providers.render.name(), "null");
        assert_eq!(providers.physics.name(), "null");
        assert_eq!(providers.audio.name(), "null");
        assert_eq!(providers.input.name(), "null");
    }

    #[test]
    fn test_debugger_game_config_default_is_disabled() {
        let config = GameConfig::default();
        assert!(!config.debugger.enabled);
        assert!(!config.debugger.publish_local_attach);
        assert_eq!(config.debugger.route_label, None);
    }

    #[test]
    fn test_debugger_engine_config_default_preserves_game_config_defaults() {
        let config = EngineConfig::new();
        assert!(!config.game_config().debugger.enabled);
        assert!(!config.game_config().debugger.publish_local_attach);
    }

    #[test]
    fn test_debugger_engine_config_with_debugger_propagates_through_build() {
        let debugger = DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("feature-lab".to_string()),
        };

        let (game_config, _) = EngineConfig::new().with_debugger(debugger.clone()).build();

        assert_eq!(game_config.debugger, debugger);
    }

    #[test]
    fn test_engine_config_builder_chain() {
        let config = EngineConfig::new()
            .with_title("Chain Test")
            .with_size(1920, 1080)
            .with_vsync(false)
            .with_fullscreen(true)
            .with_anti_aliasing_mode(AntiAliasingMode::Fxaa)
            .with_msaa_samples(4)
            .with_render_backend(RenderBackendKind::OpenGlLegacy)
            .with_window_backend(WindowBackendKind::GlfwLegacy)
            .with_target_fps(144)
            .with_fps_overlay(true)
            .with_physics_debug(true);
        let gc = config.game_config();
        assert_eq!(gc.title, "Chain Test");
        assert_eq!(gc.width, 1920);
        assert_eq!(gc.height, 1080);
        assert!(!gc.vsync);
        assert_eq!(gc.fullscreen_mode, FullscreenMode::Borderless);
        assert_eq!(gc.anti_aliasing_mode, AntiAliasingMode::Fxaa);
        assert_eq!(gc.msaa_samples, 4);
        assert_eq!(gc.render_backend, RenderBackendKind::OpenGlLegacy);
        assert_eq!(gc.window_backend, WindowBackendKind::GlfwLegacy);
        assert_eq!(gc.target_fps, 144);
        assert!(gc.show_fps_overlay);
        assert!(gc.physics_debug.enabled);
    }

    #[test]
    fn test_engine_config_with_game_config() {
        let gc = GameConfig::new("Custom", 640, 480)
            .with_vsync(false)
            .with_fullscreen(true);
        let config = EngineConfig::new().with_game_config(gc);
        let (game_config, _) = config.build();
        assert_eq!(game_config.title, "Custom");
        assert_eq!(game_config.width, 640);
        assert_eq!(game_config.height, 480);
        assert!(!game_config.vsync);
        assert_eq!(game_config.fullscreen_mode, FullscreenMode::Borderless);
    }

    #[test]
    fn test_engine_config_custom_providers() {
        let config = EngineConfig::new()
            .with_render_provider(NullRenderProvider::new())
            .with_physics_provider(NullPhysicsProvider::new())
            .with_audio_provider(NullAudioProvider::new())
            .with_input_provider(NullInputProvider::new());
        let (_, providers) = config.build();
        assert_eq!(providers.render.name(), "null");
        assert_eq!(providers.physics.name(), "null");
        assert_eq!(providers.audio.name(), "null");
        assert_eq!(providers.input.name(), "null");
    }

    #[test]
    fn test_engine_config_build_returns_parts() {
        let config = EngineConfig::new()
            .with_title("Parts Test")
            .with_size(320, 240);
        let (game_config, providers) = config.build();
        assert_eq!(game_config.title, "Parts Test");
        assert_eq!(game_config.width, 320);
        assert_eq!(game_config.height, 240);
        assert_eq!(providers.render.name(), "null");
        assert_eq!(providers.physics.name(), "null");
    }
}
