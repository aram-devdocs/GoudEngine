//! Render descriptor and command types.

use super::handles::{BufferHandle, PipelineHandle, ShaderHandle, TextureHandle};

/// Describes a texture to be created.
#[derive(Debug, Clone, Default)]
pub struct TextureDesc {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format (opaque integer; format enum can come later).
    pub format: u32,
    /// Optional raw pixel data. `None` creates an empty texture.
    pub data: Option<Vec<u8>>,
}

/// Describes a GPU buffer to be created.
#[derive(Debug, Clone, Default)]
pub struct BufferDesc {
    /// Buffer size in bytes.
    pub size: u64,
    /// Usage flags (opaque integer; usage enum can come later).
    pub usage: u32,
    /// Optional initial data.
    pub data: Option<Vec<u8>>,
}

/// Describes a shader to be compiled.
#[derive(Debug, Clone, Default)]
pub struct ShaderDesc {
    /// Vertex shader source code.
    pub vertex_source: String,
    /// Fragment shader source code.
    pub fragment_source: String,
}

/// Describes a render pipeline to be created.
#[derive(Debug, Clone, Default)]
pub struct PipelineDesc {
    /// Shader to use for this pipeline.
    pub shader: Option<ShaderHandle>,
    /// Blend mode (opaque integer; blend enum can come later).
    pub blend_mode: u32,
    /// Depth testing enabled.
    pub depth_test: bool,
    /// Depth writing enabled.
    pub depth_write: bool,
}

/// Describes a render target (framebuffer) to be created.
#[derive(Debug, Clone, Default)]
pub struct RenderTargetDesc {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Number of color attachments.
    pub color_attachments: u32,
    /// Whether to include a depth attachment.
    pub has_depth: bool,
}

/// A low-level draw command submitted to the render provider.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Pipeline state for this draw call.
    pub pipeline: PipelineHandle,
    /// Vertex buffer to draw from.
    pub vertex_buffer: BufferHandle,
    /// Optional index buffer for indexed drawing.
    pub index_buffer: Option<BufferHandle>,
    /// Optional texture binding.
    pub texture: Option<TextureHandle>,
    /// Number of instances to draw (1 for non-instanced).
    pub instance_count: u32,
    /// Number of vertices (or indices if index buffer is present).
    pub vertex_count: u32,
}

/// A 3D mesh draw command.
#[derive(Debug, Clone)]
pub struct MeshDrawCommand {
    /// Pipeline state for this draw call.
    pub pipeline: PipelineHandle,
    /// Vertex buffer containing mesh vertex data.
    pub vertex_buffer: BufferHandle,
    /// Index buffer for indexed drawing.
    pub index_buffer: BufferHandle,
    /// Optional texture binding.
    pub texture: Option<TextureHandle>,
    /// Model transform matrix (column-major 4x4).
    pub transform: [f32; 16],
    /// Number of indices to draw.
    pub index_count: u32,
}

/// A text draw command.
#[derive(Debug, Clone)]
pub struct TextDrawCommand {
    /// The text string to render.
    pub text: String,
    /// Position as [x, y].
    pub position: [f32; 2],
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color as [r, g, b, a].
    pub color: [f32; 4],
}

/// A particle system draw command.
#[derive(Debug, Clone)]
pub struct ParticleDrawCommand {
    /// Pipeline state for particle rendering.
    pub pipeline: PipelineHandle,
    /// Vertex buffer containing particle data.
    pub vertex_buffer: BufferHandle,
    /// Optional texture for particle sprites.
    pub texture: Option<TextureHandle>,
    /// Number of particles to draw.
    pub particle_count: u32,
}

/// Camera data passed to the render provider.
#[derive(Debug, Clone, Default)]
pub struct CameraData {
    /// View matrix (column-major 4x4).
    pub view: [f32; 16],
    /// Projection matrix (column-major 4x4).
    pub projection: [f32; 16],
    /// Camera position in world space.
    pub position: [f32; 3],
}
