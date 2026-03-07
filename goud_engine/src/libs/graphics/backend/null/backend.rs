//! NullBackend struct and constructor.

use std::collections::HashMap;

use crate::core::handle::HandleAllocator;
use crate::libs::graphics::backend::capabilities::{BackendCapabilities, BackendInfo};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferMarker, BufferType, ShaderMarker, TextureHandle, TextureMarker,
};

/// Metadata stored for each null buffer.
#[derive(Debug, Clone)]
pub(super) struct NullBufferMeta {
    pub size: usize,
    pub _buffer_type: BufferType,
}

/// Metadata stored for each null texture.
#[derive(Debug, Clone)]
pub(super) struct NullTextureMeta {
    pub width: u32,
    pub height: u32,
}

/// A no-op render backend that tracks resource state without GPU access.
///
/// This backend is designed for headless testing in CI environments where no
/// GPU or display server is available. All operations succeed immediately,
/// and resource handles are tracked via generational allocators so that
/// lifecycle tests (create/destroy/use-after-free) work correctly.
pub struct NullBackend {
    pub(super) info: BackendInfo,
    pub(super) clear_color: [f32; 4],

    // State tracking
    pub(super) depth_test_enabled: bool,
    pub(super) blending_enabled: bool,
    pub(super) culling_enabled: bool,
    pub(super) depth_mask_enabled: bool,
    pub(super) viewport: (i32, i32, u32, u32),
    pub(super) line_width: f32,

    // Buffer management
    pub(super) buffer_allocator: HandleAllocator<BufferMarker>,
    pub(super) buffers: HashMap<BufferHandle, NullBufferMeta>,

    // Texture management
    pub(super) texture_allocator: HandleAllocator<TextureMarker>,
    pub(super) textures: HashMap<TextureHandle, NullTextureMeta>,

    // Shader management
    pub(super) shader_allocator: HandleAllocator<ShaderMarker>,
}

// SAFETY: NullBackend contains only pure Rust data (no raw pointers,
// no thread-local state). Send + Sync is safe.
unsafe impl Send for NullBackend {}
unsafe impl Sync for NullBackend {}

impl NullBackend {
    /// Creates a new headless null backend.
    pub fn new() -> Self {
        let capabilities = BackendCapabilities {
            max_texture_units: 8,
            max_texture_size: 4096,
            max_vertex_attributes: 16,
            max_uniform_buffer_size: 16384,
            supports_instancing: true,
            supports_compute_shaders: false,
            supports_geometry_shaders: false,
            supports_tessellation: false,
            supports_multisampling: false,
            supports_anisotropic_filtering: false,
        };

        let info = BackendInfo {
            name: "Null",
            version: "1.0".to_string(),
            vendor: "Software".to_string(),
            renderer: "NullBackend".to_string(),
            capabilities,
        };

        Self {
            info,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            depth_test_enabled: false,
            blending_enabled: false,
            culling_enabled: false,
            depth_mask_enabled: true,
            viewport: (0, 0, 800, 600),
            line_width: 1.0,
            buffer_allocator: HandleAllocator::new(),
            buffers: HashMap::new(),
            texture_allocator: HandleAllocator::new(),
            textures: HashMap::new(),
            shader_allocator: HandleAllocator::new(),
        }
    }
}

impl Default for NullBackend {
    fn default() -> Self {
        Self::new()
    }
}
