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
        ShaderHandle, ShaderMarker, TextureHandle, TextureMarker, VertexBufferBinding,
        VertexLayout,
    },
    BackendCapabilities, BackendInfo, BlendFactor, BufferOps, ClearOps, CullFace, DrawOps,
    FrameOps, RenderBackend, ShaderLanguage, ShaderOps, StateOps, TextureOps,
};
use crate::core::handle::HandleAllocator;
use std::collections::HashMap;

mod buffer;
mod convert;
mod frame;
mod frame_draw_ops;
mod frame_trait_impls;
mod init;
mod pipeline;
mod readback;
mod resources;
#[cfg(feature = "sdl-window")]
mod sdl_init;
#[cfg(feature = "sdl-window")]
pub(crate) mod sdl_surface;
mod shader;
mod shadow_pass;
#[cfg(test)]
mod shadow_pass_tests;
#[cfg(feature = "switch-vulkan")]
mod switch_init;
#[cfg(feature = "switch-vulkan")]
pub(crate) mod switch_surface;
mod texture;
mod uniforms;
#[cfg(feature = "xbox-gdk")]
mod xbox_init;
#[cfg(feature = "xbox-gdk")]
pub(crate) mod xbox_surface;

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
    surface: Option<wgpu::Surface<'static>>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
    surface_supports_copy_src: bool,

    /// Persisted for surface recreation on mobile resume.
    wgpu_instance: wgpu::Instance,
    /// Persisted for surface recreation on mobile resume.
    #[allow(dead_code)]
    wgpu_adapter: wgpu::Adapter,
    /// Persisted for surface recreation on mobile resume.
    /// `None` on platforms without winit (e.g. Xbox GDK).
    window: Option<std::sync::Arc<winit::window::Window>>,

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
    pending_destroy_buffers: Vec<BufferHandle>,

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
    current_vertex_bindings: Vec<VertexBufferBinding>,
    current_topology: PrimitiveTopology,

    // Pipeline cache
    pipeline_cache: HashMap<PipelineKey, wgpu::RenderPipeline>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group layout for storage buffers (used for GPU skinning bone matrices).
    storage_bind_group_layout: wgpu::BindGroupLayout,

    // Cached 1x1 white fallback texture bind group for draws without a bound texture.
    fallback_tex_bind_group: wgpu::BindGroup,
    /// Fallback empty storage buffer bind group when no storage buffer is bound.
    fallback_storage_bind_group: wgpu::BindGroup,

    /// Currently bound storage buffer for the next draw command.
    bound_storage_buffer: Option<BufferHandle>,

    /// Cached storage buffer bind groups, keyed by buffer handle.
    /// Avoids recreating bind groups every frame for the same buffer.
    storage_bind_group_cache: HashMap<BufferHandle, wgpu::BindGroup>,

    /// Per-frame ring buffer for uniform snapshots.  Draw commands store an
    /// `(offset, size)` into this buffer instead of cloning the full 4KB
    /// staging buffer per draw.  Cleared at `begin_frame`.
    uniform_ring: Vec<u8>,

    // Shadow pass resources
    /// Bind group layout for the shadow depth texture + comparison sampler (group 3).
    shadow_bind_group_layout: wgpu::BindGroupLayout,
    /// Shadow depth texture (Depth32Float) used as a render target for the shadow pass.
    shadow_depth_texture: Option<wgpu::Texture>,
    /// Depth attachment view for rendering into the shadow map.
    shadow_depth_view: Option<wgpu::TextureView>,
    /// Sampling view used in the main pass to read the shadow map.
    shadow_sample_view: Option<wgpu::TextureView>,
    /// Comparison sampler for hardware PCF shadow lookup.
    shadow_sampler: Option<wgpu::Sampler>,
    /// Bind group binding shadow_sample_view + shadow_sampler at group 3.
    shadow_bind_group: Option<wgpu::BindGroup>,
    /// Fallback 1x1 depth texture bind group for when no shadow map is active.
    fallback_shadow_bind_group: wgpu::BindGroup,
    /// Draw commands recorded during the shadow pre-pass.
    shadow_draw_commands: Vec<DrawCommand>,
    /// When true, `record_draw()` appends to `shadow_draw_commands`.
    recording_shadow: bool,
    /// Current shadow map resolution (0 = not yet created).
    shadow_map_size: u32,
    /// Pipeline cache for depth-only shadow pipelines (different target format).
    shadow_pipeline_cache: HashMap<PipelineKey, wgpu::RenderPipeline>,
    /// Whether a readback has been requested for the current frame.
    readback_requested: bool,

    /// Scratch buffer reused each frame to collect main-pass pipeline keys,
    /// avoiding a per-frame `Vec<PipelineKey>` allocation.
    scratch_pipeline_keys: Vec<PipelineKey>,
    /// Scratch buffer reused each frame to collect shadow-pass pipeline keys.
    scratch_shadow_pipeline_keys: Vec<PipelineKey>,
    /// Scratch buffer reused each frame for shadow uniform offsets.
    /// With ~1.4k shadow casters per frame this avoids a 5KB+ Vec<u32> alloc.
    scratch_shadow_offsets: Vec<u32>,
    /// Scratch set reused each frame to dedupe shadow uniform-buffer growth
    /// per shader, avoiding a per-frame FxHashSet allocation.
    scratch_shadow_grown_shaders: rustc_hash::FxHashSet<ShaderHandle>,
}

// SAFETY: wgpu Device and Queue are Send+Sync. Surface is Send.
// All other fields are plain data or standard Rust containers.
// Sync is sound because WgpuBackend is always accessed behind a Mutex
// via SharedNativeRenderBackend — no unsynchronized shared access occurs.
unsafe impl Send for WgpuBackend {}
unsafe impl Sync for WgpuBackend {}
