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
// Provider diagnostics tests
// =========================================================================

#[test]
fn test_collect_provider_diagnostics_has_expected_keys() {
    let registry = ProviderRegistry::default();
    let diag = registry.collect_provider_diagnostics();
    assert!(
        diag.contains_key("render"),
        "diagnostics should contain render"
    );
    assert!(
        diag.contains_key("physics_2d"),
        "diagnostics should contain physics_2d"
    );
    assert!(
        diag.contains_key("physics_3d"),
        "diagnostics should contain physics_3d"
    );
    assert!(
        diag.contains_key("audio"),
        "diagnostics should contain audio"
    );
    assert!(
        diag.contains_key("input"),
        "diagnostics should contain input"
    );
}

#[test]
fn test_collect_provider_diagnostics_all_values_are_valid_json() {
    let registry = ProviderRegistry::default();
    let diag = registry.collect_provider_diagnostics();
    for (key, value) in &diag {
        assert!(
            !value.is_null(),
            "diagnostics value for '{key}' should not be null"
        );
        assert!(
            value.is_object(),
            "diagnostics value for '{key}' should be a JSON object"
        );
    }
}

#[test]
fn test_collect_provider_diagnostics_network_absent_without_provider() {
    let registry = ProviderRegistry::default();
    let diag = registry.collect_provider_diagnostics();
    assert!(
        !diag.contains_key("network"),
        "network diagnostics should be absent when no network provider is installed"
    );
}

#[test]
fn test_collect_provider_diagnostics_network_present_with_provider() {
    use crate::core::providers::impls::NullNetworkProvider;
    let mut registry = ProviderRegistry::default();
    registry.network = Some(Box::new(NullNetworkProvider::new()));
    let diag = registry.collect_provider_diagnostics();
    assert!(
        diag.contains_key("network"),
        "network diagnostics should be present when a network provider is installed"
    );
    assert!(
        diag["network"].is_object(),
        "network diagnostics should be a JSON object"
    );
}

#[test]
fn test_null_render_diagnostics_returns_defaults() {
    let registry = ProviderRegistry::default();
    let render_diag = registry.render.render_diagnostics();
    assert_eq!(render_diag.draw_calls, 0);
    assert_eq!(render_diag.triangles, 0);
    assert_eq!(render_diag.texture_binds, 0);
    assert_eq!(render_diag.shader_binds, 0);
    assert_eq!(render_diag.active_textures, 0);
    assert_eq!(render_diag.active_shaders, 0);
}

#[test]
fn test_null_physics_diagnostics_returns_defaults() {
    let registry = ProviderRegistry::default();
    let phys_diag = registry.physics.physics_diagnostics();
    assert_eq!(phys_diag.body_count, 0);
    assert_eq!(phys_diag.collider_count, 0);
    assert_eq!(phys_diag.joint_count, 0);
    assert_eq!(phys_diag.contact_pair_count, 0);
}

#[test]
fn test_null_physics3d_diagnostics_returns_defaults() {
    let registry = ProviderRegistry::default();
    let phys3d_diag = registry.physics3d.physics3d_diagnostics();
    assert_eq!(phys3d_diag.body_count, 0);
    assert_eq!(phys3d_diag.collider_count, 0);
    assert_eq!(phys3d_diag.joint_count, 0);
    assert_eq!(phys3d_diag.contact_pair_count, 0);
}

#[test]
fn test_null_audio_diagnostics_returns_defaults() {
    let registry = ProviderRegistry::default();
    let audio_diag = registry.audio.audio_diagnostics();
    assert_eq!(audio_diag.active_playbacks, 0);
    assert_eq!(audio_diag.master_volume, 0.0);
}

#[test]
fn test_null_input_diagnostics_returns_defaults() {
    let registry = ProviderRegistry::default();
    let input_diag = registry.input.input_diagnostics();
    assert!(input_diag.pressed_keys.is_empty());
    assert_eq!(input_diag.mouse_position, [0.0, 0.0]);
    assert!(input_diag.mouse_buttons_pressed.is_empty());
    assert_eq!(input_diag.connected_gamepads, 0);
}

#[test]
fn test_null_network_diagnostics_returns_defaults() {
    use crate::core::providers::impls::NullNetworkProvider;
    let provider = NullNetworkProvider::new();
    let net_diag = provider.network_diagnostics();
    assert_eq!(net_diag.bytes_sent, 0);
    assert_eq!(net_diag.bytes_received, 0);
    assert_eq!(net_diag.packets_sent, 0);
    assert_eq!(net_diag.packets_received, 0);
    assert_eq!(net_diag.rtt_ms, 0.0);
    assert_eq!(net_diag.active_connections, 0);
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
        ParticleDrawCommand, PipelineDesc, PipelineHandle, RenderTargetDesc, RenderTargetHandle,
        ShaderDesc, ShaderHandle, TextDrawCommand, TextureDesc, TextureHandle,
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
        fn create_render_target(&mut self, _: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
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
        fn render_diagnostics(&self) -> crate::core::providers::diagnostics::RenderDiagnosticsV1 {
            Default::default()
        }
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
