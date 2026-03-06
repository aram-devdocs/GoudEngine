//! Null render provider -- silent no-op for headless testing.

use crate::core::error::GoudResult;
use crate::core::providers::render::RenderProvider;
use crate::core::providers::types::{
    BufferDesc, BufferHandle, CameraData, DrawCommand, FrameContext, MeshDrawCommand,
    ParticleDrawCommand, PipelineDesc, PipelineHandle, RenderCapabilities, RenderTargetDesc,
    RenderTargetHandle, ShaderDesc, ShaderHandle, TextDrawCommand, TextureDesc, TextureHandle,
};
use crate::core::providers::{Provider, ProviderLifecycle};

/// A render provider that does nothing. Used for headless testing and as
/// a default before a real renderer is configured.
pub struct NullRenderProvider {
    capabilities: RenderCapabilities,
}

impl NullRenderProvider {
    /// Create a new null render provider.
    pub fn new() -> Self {
        Self {
            capabilities: RenderCapabilities {
                max_texture_units: 0,
                max_texture_size: 0,
                supports_instancing: false,
                supports_compute: false,
                supports_msaa: false,
            },
        }
    }
}

impl Default for NullRenderProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NullRenderProvider {
    fn name(&self) -> &str {
        "null"
    }

    fn version(&self) -> &str {
        "0.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for NullRenderProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl RenderProvider for NullRenderProvider {
    fn render_capabilities(&self) -> &RenderCapabilities {
        &self.capabilities
    }

    fn begin_frame(&mut self) -> GoudResult<FrameContext> {
        Ok(FrameContext { _id: 0 })
    }

    fn end_frame(&mut self, _frame: FrameContext) -> GoudResult<()> {
        Ok(())
    }

    fn resize(&mut self, _width: u32, _height: u32) -> GoudResult<()> {
        Ok(())
    }

    fn create_texture(&mut self, _desc: &TextureDesc) -> GoudResult<TextureHandle> {
        Ok(TextureHandle(0))
    }

    fn destroy_texture(&mut self, _handle: TextureHandle) {}

    fn create_buffer(&mut self, _desc: &BufferDesc) -> GoudResult<BufferHandle> {
        Ok(BufferHandle(0))
    }

    fn destroy_buffer(&mut self, _handle: BufferHandle) {}

    fn create_shader(&mut self, _desc: &ShaderDesc) -> GoudResult<ShaderHandle> {
        Ok(ShaderHandle(0))
    }

    fn destroy_shader(&mut self, _handle: ShaderHandle) {}

    fn create_pipeline(&mut self, _desc: &PipelineDesc) -> GoudResult<PipelineHandle> {
        Ok(PipelineHandle(0))
    }

    fn destroy_pipeline(&mut self, _handle: PipelineHandle) {}

    fn create_render_target(&mut self, _desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        Ok(RenderTargetHandle(0))
    }

    fn destroy_render_target(&mut self, _handle: RenderTargetHandle) {}

    fn draw(&mut self, _cmd: &DrawCommand) -> GoudResult<()> {
        Ok(())
    }

    fn draw_batch(&mut self, _cmds: &[DrawCommand]) -> GoudResult<()> {
        Ok(())
    }

    fn draw_mesh(&mut self, _cmd: &MeshDrawCommand) -> GoudResult<()> {
        Ok(())
    }

    fn draw_text(&mut self, _cmd: &TextDrawCommand) -> GoudResult<()> {
        Ok(())
    }

    fn draw_particles(&mut self, _cmd: &ParticleDrawCommand) -> GoudResult<()> {
        Ok(())
    }

    fn set_viewport(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) {}

    fn set_camera(&mut self, _camera: &CameraData) {}

    fn set_render_target(&mut self, _handle: Option<RenderTargetHandle>) {}

    fn clear(&mut self, _color: [f32; 4]) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_render_construction() {
        let provider = NullRenderProvider::new();
        assert_eq!(provider.name(), "null");
        assert_eq!(provider.version(), "0.0.0");
    }

    #[test]
    fn test_null_render_default() {
        let provider = NullRenderProvider::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_render_init_shutdown() {
        let mut provider = NullRenderProvider::new();
        assert!(provider.init().is_ok());
        assert!(provider.update(0.016).is_ok());
        provider.shutdown();
    }

    #[test]
    fn test_null_render_capabilities() {
        let provider = NullRenderProvider::new();
        let caps = provider.render_capabilities();
        assert_eq!(caps.max_texture_size, 0);
        assert_eq!(caps.max_texture_units, 0);
        assert!(!caps.supports_instancing);
        assert!(!caps.supports_compute);
        assert!(!caps.supports_msaa);
    }

    #[test]
    fn test_null_render_generic_capabilities() {
        let provider = NullRenderProvider::new();
        let caps = provider.capabilities();
        let render_caps = caps.downcast_ref::<RenderCapabilities>().unwrap();
        assert_eq!(render_caps.max_texture_size, 0);
    }

    #[test]
    fn test_null_render_frame_lifecycle() {
        let mut provider = NullRenderProvider::new();
        let frame = provider.begin_frame().unwrap();
        assert!(provider.end_frame(frame).is_ok());
    }

    #[test]
    fn test_null_render_resource_creation() {
        let mut provider = NullRenderProvider::new();
        let tex = provider.create_texture(&TextureDesc::default()).unwrap();
        assert_eq!(tex, TextureHandle(0));
        provider.destroy_texture(tex);

        let buf = provider.create_buffer(&BufferDesc::default()).unwrap();
        assert_eq!(buf, BufferHandle(0));
        provider.destroy_buffer(buf);

        let shader = provider.create_shader(&ShaderDesc::default()).unwrap();
        assert_eq!(shader, ShaderHandle(0));
        provider.destroy_shader(shader);

        let pipeline = provider.create_pipeline(&PipelineDesc::default()).unwrap();
        assert_eq!(pipeline, PipelineHandle(0));
        provider.destroy_pipeline(pipeline);

        let rt = provider
            .create_render_target(&RenderTargetDesc::default())
            .unwrap();
        assert_eq!(rt, RenderTargetHandle(0));
        provider.destroy_render_target(rt);
    }

    #[test]
    fn test_null_render_resize() {
        let mut provider = NullRenderProvider::new();
        assert!(provider.resize(1920, 1080).is_ok());
    }

    #[test]
    fn test_null_render_state_operations() {
        let mut provider = NullRenderProvider::new();
        provider.set_viewport(0, 0, 800, 600);
        provider.set_camera(&CameraData::default());
        provider.set_render_target(None);
        provider.clear([0.0, 0.0, 0.0, 1.0]);
    }
}
