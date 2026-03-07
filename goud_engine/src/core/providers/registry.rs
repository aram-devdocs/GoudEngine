//! Central registry holding all engine providers.

use super::audio::AudioProvider;
use super::impls::{
    NullAudioProvider, NullInputProvider, NullPhysicsProvider, NullPhysicsProvider3D,
    NullRenderProvider,
};
use super::input::InputProvider;
use super::physics::PhysicsProvider;
use super::physics3d::PhysicsProvider3D;
use super::render::RenderProvider;

/// Central registry holding all engine providers.
///
/// Each slot holds a boxed trait object for the corresponding subsystem.
/// `WindowProvider` is intentionally excluded because it is `!Send + !Sync`
/// (GLFW requires main-thread access) and is stored separately in `GoudGame`.
///
/// All providers default to their null (no-op) implementation, making it
/// safe to construct a `ProviderRegistry` without configuring any backends.
pub struct ProviderRegistry {
    /// The rendering backend (e.g., OpenGL, null).
    pub render: Box<dyn RenderProvider>,
    /// The 2D physics backend (e.g., Rapier2D, null).
    pub physics: Box<dyn PhysicsProvider>,
    /// The 3D physics backend (e.g., Rapier3D, null).
    pub physics3d: Box<dyn PhysicsProvider3D>,
    /// The audio backend (e.g., Rodio, null).
    pub audio: Box<dyn AudioProvider>,
    /// The input backend (e.g., GLFW input, null).
    pub input: Box<dyn InputProvider>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self {
            render: Box::new(NullRenderProvider::new()),
            physics: Box::new(NullPhysicsProvider::new()),
            physics3d: Box::new(NullPhysicsProvider3D::new()),
            audio: Box::new(NullAudioProvider::new()),
            input: Box::new(NullInputProvider::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_uses_null_providers() {
        let registry = ProviderRegistry::default();
        assert_eq!(registry.render.name(), "null");
        assert_eq!(registry.physics.name(), "null");
        assert_eq!(registry.physics3d.name(), "null");
        assert_eq!(registry.audio.name(), "null");
        assert_eq!(registry.input.name(), "null");
    }

    #[test]
    fn test_default_registry_versions_are_null() {
        let registry = ProviderRegistry::default();
        assert_eq!(registry.render.version(), "0.0.0");
        assert_eq!(registry.physics.version(), "0.0.0");
        assert_eq!(registry.physics3d.version(), "0.0.0");
        assert_eq!(registry.audio.version(), "0.0.0");
        assert_eq!(registry.input.version(), "0.0.0");
    }
}
