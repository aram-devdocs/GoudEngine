//! Render provider trait definition.
//!
//! The `RenderProvider` trait abstracts the rendering backend, enabling
//! runtime selection between OpenGL, wgpu, or null (headless) renderers.

use super::diagnostics::RenderDiagnosticsV1;
use super::types::{
    BufferDesc, BufferHandle, CameraData, DrawCommand, FrameContext, MeshDrawCommand,
    ParticleDrawCommand, PipelineDesc, PipelineHandle, RenderCapabilities, RenderTargetDesc,
    RenderTargetHandle, ShaderDesc, ShaderHandle, TextDrawCommand, TextureDesc, TextureHandle,
};
use super::{Provider, ProviderLifecycle};
use crate::core::error::GoudResult;

/// Trait for rendering backends.
///
/// All methods use opaque handle types and descriptor structs to remain
/// backend-agnostic. The trait is object-safe and stored as
/// `Box<dyn RenderProvider>`.
///
/// # Frame Protocol
///
/// Each frame must follow the begin/end pattern:
/// 1. Call `begin_frame()` to get a `FrameContext` token
/// 2. Issue draw commands
/// 3. Call `end_frame(frame)` to finalize and present
///
/// The `FrameContext` token is consumed by `end_frame`, preventing
/// the caller from skipping frame finalization.
pub trait RenderProvider: Provider + ProviderLifecycle {
    /// Returns the typed render capabilities for this provider.
    ///
    /// Prefer this over `Provider::capabilities()` to avoid downcasting.
    fn render_capabilities(&self) -> &RenderCapabilities;

    // -------------------------------------------------------------------------
    // Frame Lifecycle
    // -------------------------------------------------------------------------

    /// Begin a new frame. Returns an opaque token consumed by `end_frame`.
    fn begin_frame(&mut self) -> GoudResult<FrameContext>;

    /// End the current frame and present. Consumes the frame token.
    fn end_frame(&mut self, frame: FrameContext) -> GoudResult<()>;

    /// Handle window/surface resize.
    fn resize(&mut self, width: u32, height: u32) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Resource Management
    // -------------------------------------------------------------------------

    /// Create a texture from a descriptor.
    fn create_texture(&mut self, desc: &TextureDesc) -> GoudResult<TextureHandle>;

    /// Destroy a previously created texture.
    fn destroy_texture(&mut self, handle: TextureHandle);

    /// Create a GPU buffer from a descriptor.
    fn create_buffer(&mut self, desc: &BufferDesc) -> GoudResult<BufferHandle>;

    /// Destroy a previously created buffer.
    fn destroy_buffer(&mut self, handle: BufferHandle);

    /// Compile and create a shader from a descriptor.
    fn create_shader(&mut self, desc: &ShaderDesc) -> GoudResult<ShaderHandle>;

    /// Destroy a previously created shader.
    fn destroy_shader(&mut self, handle: ShaderHandle);

    /// Create a render pipeline from a descriptor.
    fn create_pipeline(&mut self, desc: &PipelineDesc) -> GoudResult<PipelineHandle>;

    /// Destroy a previously created pipeline.
    fn destroy_pipeline(&mut self, handle: PipelineHandle);

    /// Create a render target (framebuffer) from a descriptor.
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle>;

    /// Destroy a previously created render target.
    fn destroy_render_target(&mut self, handle: RenderTargetHandle);

    // -------------------------------------------------------------------------
    // Drawing
    // -------------------------------------------------------------------------

    /// Submit a single draw command.
    fn draw(&mut self, cmd: &DrawCommand) -> GoudResult<()>;

    /// Submit a batch of draw commands.
    fn draw_batch(&mut self, cmds: &[DrawCommand]) -> GoudResult<()>;

    /// Submit a 3D mesh draw command.
    fn draw_mesh(&mut self, cmd: &MeshDrawCommand) -> GoudResult<()>;

    /// Submit a text draw command.
    fn draw_text(&mut self, cmd: &TextDrawCommand) -> GoudResult<()>;

    /// Submit a particle system draw command.
    fn draw_particles(&mut self, cmd: &ParticleDrawCommand) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Render State
    // -------------------------------------------------------------------------

    /// Set the viewport rectangle.
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32);

    /// Set the active camera data (view + projection matrices).
    fn set_camera(&mut self, camera: &CameraData);

    /// Set the active render target. `None` renders to the default framebuffer.
    fn set_render_target(&mut self, handle: Option<RenderTargetHandle>);

    /// Clear the current render target with the given color [r, g, b, a].
    fn clear(&mut self, color: [f32; 4]);

    // -------------------------------------------------------------------------
    // Diagnostics
    // -------------------------------------------------------------------------

    /// Returns a snapshot of render diagnostics for the current frame.
    fn render_diagnostics(&self) -> RenderDiagnosticsV1;
}
