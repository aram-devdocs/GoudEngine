//! Central registry holding all engine providers.

use super::audio::AudioProvider;
use super::impls::{
    NullAudioProvider, NullInputProvider, NullPhysicsProvider, NullPhysicsProvider3D,
    NullRenderProvider,
};
use super::input::InputProvider;
use super::network::NetworkProvider;
use super::physics::PhysicsProvider;
use super::physics3d::PhysicsProvider3D;
use super::render::RenderProvider;
use super::types::InputCapabilities;
use super::types::{
    AudioCapabilities, NetworkCapabilities, PhysicsCapabilities, RenderCapabilities,
};
#[cfg(any(debug_assertions, test))]
use crate::core::error::GoudResult;

/// Central registry holding all engine providers.
///
/// Each slot holds a boxed trait object for the corresponding subsystem.
/// `WindowProvider` is intentionally excluded because it is `!Send + !Sync`
/// and requires main-thread access, so it is stored separately in `GoudGame`.
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
    /// The input backend (e.g., native input, null).
    pub input: Box<dyn InputProvider>,
    /// The network backend (e.g., UDP, WebSocket, null). Optional because
    /// most single-player games do not need networking.
    pub network: Option<Box<dyn NetworkProvider>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self {
            render: Box::new(NullRenderProvider::new()),
            physics: Box::new(NullPhysicsProvider::new()),
            physics3d: Box::new(NullPhysicsProvider3D::new()),
            audio: Box::new(NullAudioProvider::new()),
            input: Box::new(NullInputProvider::new()),
            network: None,
        }
    }
}

// =============================================================================
// Capability Query Convenience Methods
// =============================================================================

impl ProviderRegistry {
    /// Returns the render provider's capabilities.
    pub fn render_capabilities(&self) -> &RenderCapabilities {
        self.render.render_capabilities()
    }

    /// Returns the physics provider's capabilities.
    pub fn physics_capabilities(&self) -> &PhysicsCapabilities {
        self.physics.physics_capabilities()
    }

    /// Returns the audio provider's capabilities.
    pub fn audio_capabilities(&self) -> &AudioCapabilities {
        self.audio.audio_capabilities()
    }

    /// Returns the input provider's capabilities.
    pub fn input_capabilities(&self) -> &InputCapabilities {
        self.input.input_capabilities()
    }

    /// Returns the network provider's capabilities, if a network provider is installed.
    pub fn network_capabilities(&self) -> Option<&NetworkCapabilities> {
        self.network.as_ref().map(|n| n.network_capabilities())
    }

    /// Collect diagnostics from all registered providers as a sorted map.
    ///
    /// Each provider's diagnostics are serialized to a `serde_json::Value`.
    /// Network diagnostics are included only when a network provider is installed.
    pub fn collect_provider_diagnostics(
        &self,
    ) -> std::collections::BTreeMap<String, serde_json::Value> {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "render".into(),
            serde_json::to_value(self.render.render_diagnostics()).unwrap_or_default(),
        );
        map.insert(
            "physics_2d".into(),
            serde_json::to_value(self.physics.physics_diagnostics()).unwrap_or_default(),
        );
        map.insert(
            "physics_3d".into(),
            serde_json::to_value(self.physics3d.physics3d_diagnostics()).unwrap_or_default(),
        );
        map.insert(
            "audio".into(),
            serde_json::to_value(self.audio.audio_diagnostics()).unwrap_or_default(),
        );
        map.insert(
            "input".into(),
            serde_json::to_value(self.input.input_diagnostics()).unwrap_or_default(),
        );
        if let Some(ref net) = self.network {
            map.insert(
                "network".into(),
                serde_json::to_value(net.network_diagnostics()).unwrap_or_default(),
            );
        }
        map
    }
}

// =============================================================================
// Hot-Swap Methods (debug builds only)
// =============================================================================

#[cfg(debug_assertions)]
impl ProviderRegistry {
    /// Swap the render provider at runtime. Shuts down the old provider,
    /// initializes the new one, and returns the old (shut-down) provider.
    ///
    /// If the new provider fails to initialize, a `NullRenderProvider` is
    /// installed as a fallback and the error is returned.
    pub fn swap_render(
        &mut self,
        mut new: Box<dyn RenderProvider>,
    ) -> GoudResult<Box<dyn RenderProvider>> {
        let mut old = std::mem::replace(&mut self.render, Box::new(NullRenderProvider::new()));
        old.shutdown();
        if let Err(e) = new.init() {
            self.render = Box::new(NullRenderProvider::new());
            return Err(e);
        }
        self.render = new;
        Ok(old)
    }

    /// Swap the physics provider at runtime. Shuts down the old provider,
    /// initializes the new one, and returns the old (shut-down) provider.
    ///
    /// If the new provider fails to initialize, a `NullPhysicsProvider` is
    /// installed as a fallback and the error is returned.
    pub fn swap_physics(
        &mut self,
        mut new: Box<dyn PhysicsProvider>,
    ) -> GoudResult<Box<dyn PhysicsProvider>> {
        let mut old = std::mem::replace(&mut self.physics, Box::new(NullPhysicsProvider::new()));
        old.shutdown();
        if let Err(e) = new.init() {
            self.physics = Box::new(NullPhysicsProvider::new());
            return Err(e);
        }
        self.physics = new;
        Ok(old)
    }

    /// Swap the audio provider at runtime. Shuts down the old provider,
    /// initializes the new one, and returns the old (shut-down) provider.
    ///
    /// If the new provider fails to initialize, a `NullAudioProvider` is
    /// installed as a fallback and the error is returned.
    pub fn swap_audio(
        &mut self,
        mut new: Box<dyn AudioProvider>,
    ) -> GoudResult<Box<dyn AudioProvider>> {
        let mut old = std::mem::replace(&mut self.audio, Box::new(NullAudioProvider::new()));
        old.shutdown();
        if let Err(e) = new.init() {
            self.audio = Box::new(NullAudioProvider::new());
            return Err(e);
        }
        self.audio = new;
        Ok(old)
    }

    /// Swap the input provider at runtime. Returns the old provider.
    ///
    /// `InputProvider` does not extend `ProviderLifecycle`, so no
    /// init/shutdown calls are made.
    pub fn swap_input(
        &mut self,
        new: Box<dyn InputProvider>,
    ) -> GoudResult<Box<dyn InputProvider>> {
        let old = std::mem::replace(&mut self.input, new);
        Ok(old)
    }
}

#[cfg(test)]
mod tests;
