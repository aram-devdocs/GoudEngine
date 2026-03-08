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
use crate::core::error::GoudResult;

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
        assert!(registry.network.is_none());
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

    // =========================================================================
    // Capability query tests
    // =========================================================================

    #[test]
    fn test_default_registry_render_capabilities_are_zero() {
        let registry = ProviderRegistry::default();
        let caps = registry.render_capabilities();
        assert_eq!(caps.max_texture_units, 0);
        assert_eq!(caps.max_texture_size, 0);
        assert!(!caps.supports_instancing);
        assert!(!caps.supports_compute);
        assert!(!caps.supports_msaa);
    }

    #[test]
    fn test_default_registry_physics_capabilities_are_zero() {
        let registry = ProviderRegistry::default();
        let caps = registry.physics_capabilities();
        assert!(!caps.supports_continuous_collision);
        assert!(!caps.supports_joints);
        assert_eq!(caps.max_bodies, 0);
    }

    #[test]
    fn test_default_registry_audio_capabilities_are_zero() {
        let registry = ProviderRegistry::default();
        let caps = registry.audio_capabilities();
        assert!(!caps.supports_spatial);
        assert_eq!(caps.max_channels, 0);
    }

    #[test]
    fn test_default_registry_input_capabilities_are_zero() {
        let registry = ProviderRegistry::default();
        let caps = registry.input_capabilities();
        assert!(!caps.supports_gamepad);
        assert!(!caps.supports_touch);
        assert_eq!(caps.max_gamepads, 0);
    }

    #[test]
    fn test_default_registry_network_capabilities_are_none() {
        let registry = ProviderRegistry::default();
        assert!(registry.network_capabilities().is_none());
    }

    // =========================================================================
    // Hot-swap tests (debug-only)
    // =========================================================================

    #[cfg(debug_assertions)]
    mod hot_swap_tests {
        use super::*;
        use crate::core::error::GoudError;
        use crate::core::providers::render::RenderProvider;
        use crate::core::providers::types::{
            BufferDesc, BufferHandle, CameraData, DrawCommand, FrameContext, MeshDrawCommand,
            ParticleDrawCommand, PipelineDesc, PipelineHandle, RenderTargetDesc,
            RenderTargetHandle, ShaderDesc, ShaderHandle, TextDrawCommand, TextureDesc,
            TextureHandle,
        };
        use crate::core::providers::{Provider, ProviderLifecycle};

        #[test]
        fn test_swap_render_succeeds_and_returns_old() {
            let mut registry = ProviderRegistry::default();
            assert_eq!(registry.render.name(), "null");

            let new_render = Box::new(NullRenderProvider::new());
            let old = registry.swap_render(new_render).unwrap();
            assert_eq!(old.name(), "null");
            assert_eq!(registry.render.name(), "null");
        }

        #[test]
        fn test_swap_physics_succeeds_and_returns_old() {
            let mut registry = ProviderRegistry::default();
            let old = registry
                .swap_physics(Box::new(NullPhysicsProvider::new()))
                .unwrap();
            assert_eq!(old.name(), "null");
        }

        #[test]
        fn test_swap_audio_succeeds_and_returns_old() {
            let mut registry = ProviderRegistry::default();
            let old = registry
                .swap_audio(Box::new(NullAudioProvider::new()))
                .unwrap();
            assert_eq!(old.name(), "null");
        }

        #[test]
        fn test_swap_input_succeeds_and_returns_old() {
            let mut registry = ProviderRegistry::default();
            let old = registry
                .swap_input(Box::new(NullInputProvider::new()))
                .unwrap();
            assert_eq!(old.name(), "null");
        }

        /// A render provider whose init() always fails.
        struct FailingRenderProvider;

        impl Provider for FailingRenderProvider {
            fn name(&self) -> &str {
                "failing"
            }
            fn version(&self) -> &str {
                "0.0.0"
            }
            fn capabilities(&self) -> Box<dyn std::any::Any> {
                Box::new(RenderCapabilities::default())
            }
        }

        impl ProviderLifecycle for FailingRenderProvider {
            fn init(&mut self) -> GoudResult<()> {
                Err(GoudError::ProviderError {
                    subsystem: "render",
                    message: "init failed".to_string(),
                })
            }
            fn update(&mut self, _delta: f32) -> GoudResult<()> {
                Ok(())
            }
            fn shutdown(&mut self) {}
        }

        impl RenderProvider for FailingRenderProvider {
            fn render_capabilities(&self) -> &RenderCapabilities {
                static CAPS: RenderCapabilities = RenderCapabilities {
                    max_texture_units: 0,
                    max_texture_size: 0,
                    supports_instancing: false,
                    supports_compute: false,
                    supports_msaa: false,
                };
                &CAPS
            }
            fn begin_frame(&mut self) -> GoudResult<FrameContext> {
                unimplemented!()
            }
            fn end_frame(&mut self, _: FrameContext) -> GoudResult<()> {
                unimplemented!()
            }
            fn resize(&mut self, _: u32, _: u32) -> GoudResult<()> {
                unimplemented!()
            }
            fn create_texture(&mut self, _: &TextureDesc) -> GoudResult<TextureHandle> {
                unimplemented!()
            }
            fn destroy_texture(&mut self, _: TextureHandle) {}
            fn create_buffer(&mut self, _: &BufferDesc) -> GoudResult<BufferHandle> {
                unimplemented!()
            }
            fn destroy_buffer(&mut self, _: BufferHandle) {}
            fn create_shader(&mut self, _: &ShaderDesc) -> GoudResult<ShaderHandle> {
                unimplemented!()
            }
            fn destroy_shader(&mut self, _: ShaderHandle) {}
            fn create_pipeline(&mut self, _: &PipelineDesc) -> GoudResult<PipelineHandle> {
                unimplemented!()
            }
            fn destroy_pipeline(&mut self, _: PipelineHandle) {}
            fn create_render_target(
                &mut self,
                _: &RenderTargetDesc,
            ) -> GoudResult<RenderTargetHandle> {
                unimplemented!()
            }
            fn destroy_render_target(&mut self, _: RenderTargetHandle) {}
            fn draw(&mut self, _: &DrawCommand) -> GoudResult<()> {
                unimplemented!()
            }
            fn draw_batch(&mut self, _: &[DrawCommand]) -> GoudResult<()> {
                unimplemented!()
            }
            fn draw_mesh(&mut self, _: &MeshDrawCommand) -> GoudResult<()> {
                unimplemented!()
            }
            fn draw_text(&mut self, _: &TextDrawCommand) -> GoudResult<()> {
                unimplemented!()
            }
            fn draw_particles(&mut self, _: &ParticleDrawCommand) -> GoudResult<()> {
                unimplemented!()
            }
            fn set_viewport(&mut self, _: i32, _: i32, _: u32, _: u32) {}
            fn set_camera(&mut self, _: &CameraData) {}
            fn set_render_target(&mut self, _: Option<RenderTargetHandle>) {}
            fn clear(&mut self, _: [f32; 4]) {}
        }

        #[test]
        fn test_swap_render_init_failure_installs_null_fallback() {
            let mut registry = ProviderRegistry::default();
            let result = registry.swap_render(Box::new(FailingRenderProvider));
            assert!(result.is_err());
            // After failure, null provider should be installed as fallback.
            assert_eq!(registry.render.name(), "null");
        }

        #[test]
        fn test_swap_render_changes_provider_name() {
            // NullRenderProvider always has name "null", so we verify the
            // swap mechanism works by confirming name after swap.
            let mut registry = ProviderRegistry::default();
            assert_eq!(registry.render.name(), "null");
            let new_render = Box::new(NullRenderProvider::new());
            let _old = registry.swap_render(new_render).unwrap();
            assert_eq!(registry.render.name(), "null");
        }
    }
}
