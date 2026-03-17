//! wgpu implementation of the [`RenderBackend`] trait.
//!
//! Provides a cross-platform GPU backend using wgpu (WebGPU API). This backend
//! works on desktop (Vulkan/Metal/DX12) and web (WebGPU/WebGL2).
//!
//! # Architecture
//!
//! Unlike OpenGL's immediate-mode API, wgpu uses command buffers and render
//! pipelines. This backend bridges the gap by:
//! - Tracking GPU state changes (depth, blend, cull) and caching render pipelines
//! - Deferring draw calls into a command list replayed at [`end_frame`]
//! - Managing per-shader uniform buffers with CPU staging

use super::{
    types::{
        BufferHandle, BufferMarker, BufferType, DepthFunc, FrontFace, PrimitiveTopology,
        ShaderHandle, ShaderMarker, TextureHandle, TextureMarker, VertexLayout,
    },
    BackendCapabilities, BackendInfo, BlendFactor, BufferOps, ClearOps, CullFace, DrawOps,
    FrameOps, RenderBackend, ShaderOps, StateOps, TextureOps,
};
use crate::core::handle::HandleAllocator;
use std::collections::HashMap;

mod buffer;
mod convert;
mod frame;
mod init;
mod pipeline;
mod readback;
mod resources;
mod shader;
mod texture;
mod uniforms;

// Pull internal types into this module's namespace so submodules can `use super::TypeName`.
use resources::{
    DrawCommand, DrawType, FrameState, PipelineKey, WgpuBufferMeta, WgpuShaderMeta, WgpuTextureMeta,
};

pub use init::{MAX_TEXTURE_UNITS, UNIFORM_BUFFER_SIZE};

// =============================================================================
// WgpuBackend
// =============================================================================

/// wgpu-based render backend for cross-platform GPU rendering.
///
/// Owns the full wgpu device stack (instance, surface, adapter, device, queue)
/// and manages GPU resources via generational handles identical to OpenGLBackend.
pub struct WgpuBackend {
    info: BackendInfo,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,

    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    last_frame_readback: Option<(u32, u32, Vec<u8>)>,

    clear_color: wgpu::Color,
    needs_clear: bool,

    current_frame: Option<FrameState>,
    draw_commands: Vec<DrawCommand>,

    // Render state
    depth_test_enabled: bool,
    depth_write_enabled: bool,
    depth_func: DepthFunc,
    blend_enabled: bool,
    blend_src: BlendFactor,
    blend_dst: BlendFactor,
    cull_enabled: bool,
    cull_face: CullFace,
    front_face_state: FrontFace,

    // Resource management
    buffer_allocator: HandleAllocator<BufferMarker>,
    buffers: HashMap<BufferHandle, WgpuBufferMeta>,

    texture_allocator: HandleAllocator<TextureMarker>,
    textures: HashMap<TextureHandle, WgpuTextureMeta>,

    shader_allocator: HandleAllocator<ShaderMarker>,
    shaders: HashMap<ShaderHandle, WgpuShaderMeta>,

    // Current bindings
    bound_vertex_buffer: Option<BufferHandle>,
    bound_index_buffer: Option<BufferHandle>,
    bound_shader: Option<ShaderHandle>,
    bound_textures: Vec<Option<TextureHandle>>,
    current_layout: Option<VertexLayout>,

    // Pipeline cache
    pipeline_cache: HashMap<PipelineKey, wgpu::RenderPipeline>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

// SAFETY: wgpu Device and Queue are Send+Sync. Surface is Send.
// All other fields are plain data or standard Rust containers.
unsafe impl Send for WgpuBackend {}
unsafe impl Sync for WgpuBackend {}
