//! Provider capability query and hot-swap methods for [`GoudGame`].

use super::GoudGame;
use crate::core::providers::input_types::InputCapabilities;
use crate::core::providers::network_types::NetworkCapabilities;
use crate::core::providers::types::{AudioCapabilities, PhysicsCapabilities, RenderCapabilities};

#[cfg(debug_assertions)]
use crate::core::error::GoudResult;
#[cfg(debug_assertions)]
use crate::core::providers::audio::AudioProvider;
#[cfg(debug_assertions)]
use crate::core::providers::input::InputProvider;
#[cfg(debug_assertions)]
use crate::core::providers::physics::PhysicsProvider;
#[cfg(debug_assertions)]
use crate::core::providers::render::RenderProvider;

impl GoudGame {
    // =========================================================================
    // Capability Queries
    // =========================================================================

    /// Returns the render provider's capabilities.
    #[inline]
    pub fn render_capabilities(&self) -> &RenderCapabilities {
        self.providers.render_capabilities()
    }

    /// Returns the physics provider's capabilities.
    #[inline]
    pub fn physics_capabilities(&self) -> &PhysicsCapabilities {
        self.providers.physics_capabilities()
    }

    /// Returns the audio provider's capabilities.
    #[inline]
    pub fn audio_capabilities(&self) -> &AudioCapabilities {
        self.providers.audio_capabilities()
    }

    /// Returns the input provider's capabilities.
    #[inline]
    pub fn input_capabilities(&self) -> &InputCapabilities {
        self.providers.input_capabilities()
    }

    /// Returns the network provider's capabilities, if installed.
    #[inline]
    pub fn network_capabilities(&self) -> Option<&NetworkCapabilities> {
        self.providers.network_capabilities()
    }

    // =========================================================================
    // Hot-Swap (debug builds only)
    // =========================================================================

    /// Swap the render provider at runtime (debug builds only).
    #[cfg(debug_assertions)]
    pub fn hot_swap_render_provider(
        &mut self,
        new: Box<dyn RenderProvider>,
    ) -> GoudResult<Box<dyn RenderProvider>> {
        self.providers.swap_render(new)
    }

    /// Swap the physics provider at runtime (debug builds only).
    #[cfg(debug_assertions)]
    pub fn hot_swap_physics_provider(
        &mut self,
        new: Box<dyn PhysicsProvider>,
    ) -> GoudResult<Box<dyn PhysicsProvider>> {
        self.providers.swap_physics(new)
    }

    /// Swap the audio provider at runtime (debug builds only).
    #[cfg(debug_assertions)]
    pub fn hot_swap_audio_provider(
        &mut self,
        new: Box<dyn AudioProvider>,
    ) -> GoudResult<Box<dyn AudioProvider>> {
        self.providers.swap_audio(new)
    }

    /// Swap the input provider at runtime (debug builds only).
    #[cfg(debug_assertions)]
    pub fn hot_swap_input_provider(
        &mut self,
        new: Box<dyn InputProvider>,
    ) -> GoudResult<Box<dyn InputProvider>> {
        self.providers.swap_input(new)
    }
}
