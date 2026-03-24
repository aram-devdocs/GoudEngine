//! Internal GPU resource metadata types for the wgpu backend.
//!
//! These types represent metadata for GPU buffers, textures, shaders, draw commands,
//! pipeline cache keys, and per-frame state. They are private to the wgpu backend
//! module tree.

use super::{
    BlendFactor, BufferHandle, BufferType, CullFace, DepthFunc, FrontFace, PrimitiveTopology,
    ShaderHandle, TextureHandle, VertexBufferBinding,
};
use smallvec::SmallVec;
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
    /// Kept alive so the bind group remains valid.
    pub(super) _view: wgpu::TextureView,
    /// Kept alive so the bind group remains valid.
    pub(super) _sampler: wgpu::Sampler,
    pub(super) width: u32,
    pub(super) height: u32,
    /// Cached bind group for this texture (view + sampler). Created once at
    /// texture creation time and reused every frame instead of being recreated
    /// per draw command.
    pub(super) bind_group: wgpu::BindGroup,
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
}

// =============================================================================
// Draw command recording
// =============================================================================

/// A draw command recorded during the frame, replayed in `end_frame`.
///
/// Uses `SmallVec` for vertex bindings and textures to avoid heap allocation
/// in the common case (1-2 vertex bindings, 0-2 bound textures).
pub(super) struct DrawCommand {
    pub(super) shader: ShaderHandle,
    pub(super) index_buffer: Option<BufferHandle>,
    pub(super) vertex_bindings: SmallVec<[VertexBufferBinding; 2]>,
    pub(super) bound_textures: SmallVec<[(u32, TextureHandle); 2]>,
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
    /// Byte offset into `WgpuBackend::uniform_ring` for this command's uniform data.
    pub(super) uniform_ring_offset: u32,
    /// Number of bytes of uniform data stored in the ring buffer.
    pub(super) uniform_ring_size: u32,
    pub(super) draw_type: DrawType,
}

pub(super) enum DrawType {
    Arrays {
        first: u32,
        count: u32,
    },
    Indexed {
        count: u32,
        offset: usize,
    },
    IndexedU16 {
        count: u32,
        offset: usize,
    },
    ArraysInstanced {
        first: u32,
        count: u32,
        instances: u32,
    },
    IndexedInstanced {
        count: u32,
        offset: usize,
        instances: u32,
    },
}

impl DrawType {
    pub(super) fn first_index(&self) -> u32 {
        match self {
            Self::Indexed { offset, .. } | Self::IndexedInstanced { offset, .. } => {
                (offset / std::mem::size_of::<u32>()) as u32
            }
            Self::IndexedU16 { offset, .. } => (offset / std::mem::size_of::<u16>()) as u32,
            _ => 0,
        }
    }
}

// =============================================================================
// Pipeline cache key
// =============================================================================

/// Pipeline cache key combining all state that affects pipeline creation.
///
/// The `vertex_layout_hash` field is a precomputed `u64` hash of the vertex
/// buffer layouts (stride, step mode, attribute locations/types/offsets).
/// This avoids allocating nested `Vec`s for every draw command just to build
/// the cache key.
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
    pub(super) vertex_layout_hash: u64,
}

// =============================================================================
// Per-frame state
// =============================================================================

pub(super) struct FrameState {
    pub(super) surface_texture: wgpu::SurfaceTexture,
    pub(super) surface_view: wgpu::TextureView,
}
