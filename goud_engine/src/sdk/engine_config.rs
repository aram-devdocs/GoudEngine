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

use crate::core::providers::audio::AudioProvider;
use crate::core::providers::input::InputProvider;
use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::render::RenderProvider;
use crate::core::providers::ProviderRegistry;
use crate::core::providers::ProviderRegistryBuilder;
use crate::sdk::game_config::GameConfig;

/// Unified engine configuration combining window settings and provider selection.
///
/// `EngineConfig` delegates window/display settings to [`GameConfig`] and
/// provider selection to [`ProviderRegistryBuilder`]. Call [`build()`](Self::build)
/// to consume the builder and obtain both parts.
pub struct EngineConfig {
    game_config: GameConfig,
    provider_builder: ProviderRegistryBuilder,
}

impl EngineConfig {
    /// Creates a new `EngineConfig` with default settings and null providers.
    pub fn new() -> Self {
        Self {
            game_config: GameConfig::default(),
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

    /// Replaces the entire [`GameConfig`] with the provided one.
    pub fn with_game_config(mut self, config: GameConfig) -> Self {
        self.game_config = config;
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
    pub fn build(self) -> (GameConfig, ProviderRegistry) {
        (self.game_config, self.provider_builder.build())
    }

    /// Returns a reference to the current game configuration.
    pub fn game_config(&self) -> &GameConfig {
        &self.game_config
    }

    /// Returns a mutable reference to the current game configuration.
    /// Used for direct field mutation in FFI boundary code.
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
    use crate::core::providers::impls::{
        NullAudioProvider, NullInputProvider, NullPhysicsProvider, NullRenderProvider,
    };

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        let (game_config, providers) = config.build();
        assert_eq!(game_config.title, "GoudEngine Game");
        assert_eq!(game_config.width, 800);
        assert_eq!(game_config.height, 600);
        assert_eq!(providers.render.name(), "null");
        assert_eq!(providers.physics.name(), "null");
        assert_eq!(providers.audio.name(), "null");
        assert_eq!(providers.input.name(), "null");
    }

    #[test]
    fn test_engine_config_builder_chain() {
        let config = EngineConfig::new()
            .with_title("Chain Test")
            .with_size(1920, 1080)
            .with_vsync(false)
            .with_fullscreen(true)
            .with_target_fps(144)
            .with_fps_overlay(true);
        let gc = config.game_config();
        assert_eq!(gc.title, "Chain Test");
        assert_eq!(gc.width, 1920);
        assert_eq!(gc.height, 1080);
        assert!(!gc.vsync);
        assert!(gc.fullscreen);
        assert_eq!(gc.target_fps, 144);
        assert!(gc.show_fps_overlay);
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
        assert!(game_config.fullscreen);
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
