//! OpenGL render provider -- wraps `OpenGLBackend` for the provider API.

use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::libs::graphics::backend::RenderBackend;
use crate::libs::providers::render::RenderProvider;
use crate::libs::providers::types::{
    BufferDesc, BufferHandle, CameraData, DrawCommand, FrameContext, MeshDrawCommand,
    ParticleDrawCommand, PipelineDesc, PipelineHandle, RenderCapabilities, RenderTargetDesc,
    RenderTargetHandle, ShaderDesc, ShaderHandle, TextDrawCommand, TextureDesc, TextureHandle,
};
use crate::libs::providers::{Provider, ProviderLifecycle};

/// OpenGL render provider that wraps an existing [`OpenGLBackend`].
///
/// Delegates to the backend for operations that map directly (viewport,
/// clear, begin/end frame). Operations that require the new pipeline
/// abstraction (create_pipeline, draw_mesh, draw_text, draw_particles)
/// return `NotImplemented` until the backend is extended.
pub struct OpenGLRenderProvider {
    backend: OpenGLBackend,
    capabilities: RenderCapabilities,
    frame_counter: u64,
}

impl OpenGLRenderProvider {
    /// Creates a new OpenGL render provider wrapping the given backend.
    pub fn new(backend: OpenGLBackend) -> Self {
        let caps = {
            let info = backend.info();
            let bc = &info.capabilities;
            RenderCapabilities {
                max_texture_units: bc.max_texture_units,
                max_texture_size: bc.max_texture_size,
                supports_instancing: bc.supports_instancing,
                supports_compute: bc.supports_compute_shaders,
                supports_msaa: bc.supports_multisampling,
            }
        };

        Self {
            backend,
            capabilities: caps,
            frame_counter: 0,
        }
    }

    /// Returns a mutable reference to the underlying OpenGL backend.
    ///
    /// Used by higher-layer code that needs direct backend access for
    /// operations not yet exposed through the provider trait (e.g.,
    /// sprite batch rendering, tiled map drawing).
    #[allow(dead_code)]
    pub(crate) fn backend(&mut self) -> &mut OpenGLBackend {
        &mut self.backend
    }
}

impl Provider for OpenGLRenderProvider {
    fn name(&self) -> &str {
        self.backend.info().name
    }

    fn version(&self) -> &str {
        "3.3"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for OpenGLRenderProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {
        // OpenGLBackend cleans up in its Drop impl.
    }
}

impl RenderProvider for OpenGLRenderProvider {
    fn render_capabilities(&self) -> &RenderCapabilities {
        &self.capabilities
    }

    fn begin_frame(&mut self) -> GoudResult<FrameContext> {
        self.backend.begin_frame()?;
        let id = self.frame_counter;
        self.frame_counter = self.frame_counter.wrapping_add(1);
        Ok(FrameContext { id })
    }

    fn end_frame(&mut self, _frame: FrameContext) -> GoudResult<()> {
        self.backend.end_frame()
    }

    fn resize(&mut self, width: u32, height: u32) -> GoudResult<()> {
        self.backend.set_viewport(0, 0, width, height);
        Ok(())
    }

    fn create_texture(&mut self, desc: &TextureDesc) -> GoudResult<TextureHandle> {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};

        let format = match desc.format {
            1 => TextureFormat::R8,
            2 => TextureFormat::RG8,
            _ => TextureFormat::RGBA8,
        };

        let data = desc.data.as_deref().unwrap_or(&[]);
        let handle = self.backend.create_texture(
            desc.width,
            desc.height,
            format,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            data,
        )?;

        Ok(TextureHandle(handle.to_u64()))
    }

    fn destroy_texture(&mut self, handle: TextureHandle) {
        use crate::libs::graphics::backend::types::TextureHandle as BackendTexHandle;

        let backend_handle = BackendTexHandle::from_u64(handle.0);
        self.backend.destroy_texture(backend_handle);
    }

    fn create_buffer(&mut self, desc: &BufferDesc) -> GoudResult<BufferHandle> {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage};

        let data = desc.data.as_deref().unwrap_or(&[]);
        let usage = match desc.usage {
            1 => BufferUsage::Dynamic,
            2 => BufferUsage::Stream,
            _ => BufferUsage::Static,
        };

        let handle = self
            .backend
            .create_buffer(BufferType::Vertex, usage, data)?;
        Ok(BufferHandle(handle.to_u64()))
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) {
        use crate::libs::graphics::backend::types::BufferHandle as BackendBufHandle;

        let backend_handle = BackendBufHandle::from_u64(handle.0);
        self.backend.destroy_buffer(backend_handle);
    }

    fn create_shader(&mut self, desc: &ShaderDesc) -> GoudResult<ShaderHandle> {
        let handle = self
            .backend
            .create_shader(&desc.vertex_source, &desc.fragment_source)?;
        Ok(ShaderHandle(handle.to_u64()))
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) {
        use crate::libs::graphics::backend::types::ShaderHandle as BackendShHandle;

        let backend_handle = BackendShHandle::from_u64(handle.0);
        self.backend.destroy_shader(backend_handle);
    }

    fn create_pipeline(&mut self, _desc: &PipelineDesc) -> GoudResult<PipelineHandle> {
        Err(GoudError::NotImplemented(
            "OpenGL pipeline abstraction not yet available".to_string(),
        ))
    }

    fn destroy_pipeline(&mut self, _handle: PipelineHandle) {}

    fn create_render_target(
        &mut self,
        _desc: &RenderTargetDesc,
    ) -> GoudResult<RenderTargetHandle> {
        Err(GoudError::NotImplemented(
            "OpenGL render target creation via provider not yet available".to_string(),
        ))
    }

    fn destroy_render_target(&mut self, _handle: RenderTargetHandle) {}

    fn draw(&mut self, _cmd: &DrawCommand) -> GoudResult<()> {
        Err(GoudError::NotImplemented(
            "Generic draw command requires pipeline mapping".to_string(),
        ))
    }

    fn draw_batch(&mut self, _cmds: &[DrawCommand]) -> GoudResult<()> {
        Err(GoudError::NotImplemented(
            "Batched draw requires pipeline mapping".to_string(),
        ))
    }

    fn draw_mesh(&mut self, _cmd: &MeshDrawCommand) -> GoudResult<()> {
        Err(GoudError::NotImplemented(
            "Mesh draw not yet bridged to OpenGL backend".to_string(),
        ))
    }

    fn draw_text(&mut self, _cmd: &TextDrawCommand) -> GoudResult<()> {
        Err(GoudError::NotImplemented(
            "Text rendering not yet available via provider API".to_string(),
        ))
    }

    fn draw_particles(&mut self, _cmd: &ParticleDrawCommand) -> GoudResult<()> {
        Err(GoudError::NotImplemented(
            "Particle rendering not yet available via provider API".to_string(),
        ))
    }

    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.backend.set_viewport(x, y, width, height);
    }

    fn set_camera(&mut self, _camera: &CameraData) {
        // Camera uniforms are set through the shader system, not the backend
        // directly. This will be wired when pipeline state management is added.
    }

    fn set_render_target(&mut self, _handle: Option<RenderTargetHandle>) {
        // Render target binding requires the FBO abstraction not yet in the
        // provider layer.
    }

    fn clear(&mut self, color: [f32; 4]) {
        self.backend
            .set_clear_color(color[0], color[1], color[2], color[3]);
        self.backend.clear_color();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // OpenGLRenderProvider requires an active GL context for construction.
    // All tests below are guarded: they skip gracefully when no GL context
    // is available (the typical case in CI).

    #[test]
    fn test_opengl_render_provider_type_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OpenGLRenderProvider>();
    }
}
