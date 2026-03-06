//! Builder pattern for constructing a [`ProviderRegistry`].

use super::audio::AudioProvider;
use super::impls::{NullAudioProvider, NullInputProvider, NullPhysicsProvider, NullRenderProvider};
use super::input::InputProvider;
use super::physics::PhysicsProvider;
use super::registry::ProviderRegistry;
use super::render::RenderProvider;

/// Builder for constructing a [`ProviderRegistry`] with optional overrides.
///
/// Any provider slot left unconfigured defaults to its null (no-op)
/// implementation. This enables incremental adoption: configure only
/// the providers your game needs.
///
/// # Example
///
/// ```ignore
/// let registry = ProviderRegistryBuilder::new()
///     .with_renderer(my_opengl_renderer)
///     .with_audio(my_rodio_audio)
///     .build();
/// // physics and input default to null providers
/// ```
pub struct ProviderRegistryBuilder {
    render: Option<Box<dyn RenderProvider>>,
    physics: Option<Box<dyn PhysicsProvider>>,
    audio: Option<Box<dyn AudioProvider>>,
    input: Option<Box<dyn InputProvider>>,
}

impl ProviderRegistryBuilder {
    /// Create a new builder with no providers configured.
    pub fn new() -> Self {
        Self {
            render: None,
            physics: None,
            audio: None,
            input: None,
        }
    }

    /// Set the render provider.
    pub fn with_renderer(mut self, renderer: impl RenderProvider + 'static) -> Self {
        self.render = Some(Box::new(renderer));
        self
    }

    /// Set the physics provider.
    pub fn with_physics(mut self, physics: impl PhysicsProvider + 'static) -> Self {
        self.physics = Some(Box::new(physics));
        self
    }

    /// Set the audio provider.
    pub fn with_audio(mut self, audio: impl AudioProvider + 'static) -> Self {
        self.audio = Some(Box::new(audio));
        self
    }

    /// Set the input provider.
    pub fn with_input(mut self, input: impl InputProvider + 'static) -> Self {
        self.input = Some(Box::new(input));
        self
    }

    /// Build the [`ProviderRegistry`], filling unconfigured slots with
    /// null providers.
    pub fn build(self) -> ProviderRegistry {
        ProviderRegistry {
            render: self
                .render
                .unwrap_or_else(|| Box::new(NullRenderProvider::new())),
            physics: self
                .physics
                .unwrap_or_else(|| Box::new(NullPhysicsProvider::new())),
            audio: self
                .audio
                .unwrap_or_else(|| Box::new(NullAudioProvider::new())),
            input: self
                .input
                .unwrap_or_else(|| Box::new(NullInputProvider::new())),
        }
    }
}

impl Default for ProviderRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_all_defaults() {
        let registry = ProviderRegistryBuilder::new().build();
        assert_eq!(registry.render.name(), "null");
        assert_eq!(registry.physics.name(), "null");
        assert_eq!(registry.audio.name(), "null");
        assert_eq!(registry.input.name(), "null");
    }

    #[test]
    fn test_builder_with_custom_renderer() {
        let custom_render = NullRenderProvider::new();
        let registry = ProviderRegistryBuilder::new()
            .with_renderer(custom_render)
            .build();
        // Even though it's a NullRenderProvider, the builder accepted it
        assert_eq!(registry.render.name(), "null");
        // Other slots remain null
        assert_eq!(registry.physics.name(), "null");
        assert_eq!(registry.audio.name(), "null");
        assert_eq!(registry.input.name(), "null");
    }

    #[test]
    fn test_builder_partial_configuration() {
        let registry = ProviderRegistryBuilder::new()
            .with_audio(NullAudioProvider::new())
            .with_physics(NullPhysicsProvider::new())
            .build();
        // Configured slots work
        assert_eq!(registry.audio.name(), "null");
        assert_eq!(registry.physics.name(), "null");
        // Unconfigured slots default to null
        assert_eq!(registry.render.name(), "null");
        assert_eq!(registry.input.name(), "null");
    }

    #[test]
    fn test_builder_default_impl() {
        let builder = ProviderRegistryBuilder::default();
        let registry = builder.build();
        assert_eq!(registry.render.name(), "null");
    }

    #[test]
    fn test_builder_all_providers_configured() {
        let registry = ProviderRegistryBuilder::new()
            .with_renderer(NullRenderProvider::new())
            .with_physics(NullPhysicsProvider::new())
            .with_audio(NullAudioProvider::new())
            .with_input(NullInputProvider::new())
            .build();
        assert_eq!(registry.render.name(), "null");
        assert_eq!(registry.physics.name(), "null");
        assert_eq!(registry.audio.name(), "null");
        assert_eq!(registry.input.name(), "null");
    }
}
