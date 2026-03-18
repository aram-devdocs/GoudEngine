//! Internal GPU resource metadata types for the wgpu backend.
//!
//! These types represent metadata for GPU buffers, textures, shaders, draw commands,
//! pipeline cache keys, and per-frame state. They are private to the wgpu backend
//! module tree.

use super::{
    BlendFactor, BufferHandle, BufferType, CullFace, DepthFunc, FrontFace, PrimitiveTopology,
    ShaderHandle, TextureHandle, VertexBufferBinding,
};
use std::collections::HashMap;

// =============================================================================
// GPU resource metadata
// =============================================================================

pub(super) struct WgpuBufferMeta {
    pub(super) buffer: wgpu::Buffer,
    pub(super) buffer_type: BufferType,
    pub(super) size: usize,
}

pub(super) struct WgpuTextureMeta {
    pub(super) _texture: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
    pub(super) sampler: wgpu::Sampler,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) struct UniformSlot {
    pub(super) offset: usize,
    pub(super) _size: usize,
}

pub(super) struct WgpuShaderMeta {
    pub(super) vertex_module: wgpu::ShaderModule,
    pub(super) fragment_module: wgpu::ShaderModule,
    pub(super) uniform_slots: HashMap<String, UniformSlot>,
    pub(super) uniform_staging: Vec<u8>,
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) uniform_bind_group: wgpu::BindGroup,
    pub(super) _next_uniform_offset: usize,
}

// =============================================================================
// Draw command recording
// =============================================================================

/// A draw command recorded during the frame, replayed in `end_frame`.
pub(super) struct DrawCommand {
    pub(super) shader: ShaderHandle,
    pub(super) index_buffer: Option<BufferHandle>,
    pub(super) vertex_bindings: Vec<VertexBufferBinding>,
    pub(super) bound_textures: Vec<(u32, TextureHandle)>,
    pub(super) topology: PrimitiveTopology,
    pub(super) depth_test: bool,
    pub(super) depth_write: bool,
    pub(super) depth_func: DepthFunc,
    pub(super) blend_enabled: bool,
    pub(super) blend_src: BlendFactor,
    pub(super) blend_dst: BlendFactor,
    pub(super) cull_enabled: bool,
    pub(super) cull_face: CullFace,
    pub(super) front_face: FrontFace,
    pub(super) uniform_snapshot: Vec<u8>,
    pub(super) draw_type: DrawType,
}

pub(super) enum DrawType {
    Arrays {
        first: u32,
        count: u32,
    },
    Indexed {
        count: u32,
        _offset: usize,
    },
    IndexedU16 {
        count: u32,
        _offset: usize,
    },
    ArraysInstanced {
        first: u32,
        count: u32,
        instances: u32,
    },
    IndexedInstanced {
        count: u32,
        _offset: usize,
        instances: u32,
    },
}

// =============================================================================
// Pipeline cache key
// =============================================================================

/// Pipeline cache key combining all state that affects pipeline creation.
#[derive(Hash, Eq, PartialEq, Clone)]
pub(super) struct PipelineKey {
    pub(super) shader: ShaderHandle,
    pub(super) topology: u8,
    pub(super) depth_test: bool,
    pub(super) depth_write: bool,
    pub(super) depth_func: u8,
    pub(super) blend_enabled: bool,
    pub(super) blend_src: u8,
    pub(super) blend_dst: u8,
    pub(super) cull_enabled: bool,
    pub(super) cull_face: u8,
    pub(super) front_face: u8,
    pub(super) vertex_buffers: Vec<(u32, u8, Vec<(u32, u8, u32, bool)>)>,
}

// =============================================================================
// Per-frame state
// =============================================================================

pub(super) struct FrameState {
    pub(super) surface_texture: wgpu::SurfaceTexture,
    pub(super) surface_view: wgpu::TextureView,
}
