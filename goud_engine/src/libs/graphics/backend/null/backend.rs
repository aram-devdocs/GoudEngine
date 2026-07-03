//! NullBackend struct and constructor.

use std::collections::HashMap;

use crate::core::handle::HandleAllocator;
use crate::libs::graphics::backend::capabilities::{
    BackendCapabilities, BackendInfo, ShaderLanguage,
};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferMarker, BufferType, RenderTargetHandle, RenderTargetMarker, ShaderMarker,
    TextureHandle, TextureMarker,
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

/// Metadata stored for each null render target.
#[derive(Debug, Clone)]
pub(super) struct NullRenderTargetMeta {
    pub width: u32,
    pub height: u32,
    pub color_texture: TextureHandle,
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
    pub(super) multisampling_enabled: bool,
    pub(super) viewport: (i32, i32, u32, u32),
    pub(super) default_viewport: (i32, i32, u32, u32),
    pub(super) line_width: f32,

    // Buffer management
    pub(super) buffer_allocator: HandleAllocator<BufferMarker>,
    pub(super) buffers: HashMap<BufferHandle, NullBufferMeta>,

    // Texture management
    pub(super) texture_allocator: HandleAllocator<TextureMarker>,
    pub(super) textures: HashMap<TextureHandle, NullTextureMeta>,

    // Render-target management
    pub(super) render_target_allocator: HandleAllocator<RenderTargetMarker>,
    pub(super) render_targets: HashMap<RenderTargetHandle, NullRenderTargetMeta>,
    pub(super) active_render_target: Option<RenderTargetHandle>,

    // Shader management
    pub(super) shader_allocator: HandleAllocator<ShaderMarker>,
    pub(super) shader_create_calls: usize,
    pub(super) draw_arrays_calls: usize,
    pub(super) draw_indexed_calls: usize,
    pub(super) draw_arrays_instanced_calls: usize,
    pub(super) draw_indexed_instanced_calls: usize,

    // Aggregate draw-command counting (used by benches/tests to pin draw counts).
    /// `true` while the backend is inside a shadow pre-pass recording block.
    pub(super) shadow_recording: bool,
    /// Total draw commands of every kind recorded since construction/reset.
    pub(super) total_draw_commands: usize,
    /// Draw commands recorded while `shadow_recording` was active.
    pub(super) shadow_draw_commands: usize,
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
            supports_bc_compression: false,
        };

        let info = BackendInfo {
            name: "Null",
            version: "1.0".to_string(),
            vendor: "Software".to_string(),
            renderer: "NullBackend".to_string(),
            capabilities,
            shader_language: ShaderLanguage::Glsl,
        };

        Self {
            info,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            depth_test_enabled: false,
            blending_enabled: false,
            culling_enabled: false,
            depth_mask_enabled: true,
            multisampling_enabled: false,
            viewport: (0, 0, 800, 600),
            default_viewport: (0, 0, 800, 600),
            line_width: 1.0,
            buffer_allocator: HandleAllocator::new(),
            buffers: HashMap::new(),
            texture_allocator: HandleAllocator::new(),
            textures: HashMap::new(),
            render_target_allocator: HandleAllocator::new(),
            render_targets: HashMap::new(),
            active_render_target: None,
            shader_allocator: HandleAllocator::new(),
            shader_create_calls: 0,
            draw_arrays_calls: 0,
            draw_indexed_calls: 0,
            draw_arrays_instanced_calls: 0,
            draw_indexed_instanced_calls: 0,
            shadow_recording: false,
            total_draw_commands: 0,
            shadow_draw_commands: 0,
        }
    }

    /// Creates a headless null backend that reports the given shader language.
    ///
    /// The default [`NullBackend::new`] reports [`ShaderLanguage::Glsl`], which
    /// routes [`Renderer3D`](crate::libs::graphics::renderer3d::Renderer3D)
    /// down its legacy CPU paths. Constructing the backend with
    /// [`ShaderLanguage::Wgsl`] instead exercises the wgpu draw-recording and
    /// GPU shadow pre-pass code paths without touching a real GPU, which is what
    /// the renderer benchmarks measure.
    pub fn with_shader_language(lang: ShaderLanguage) -> Self {
        let mut backend = Self::new();
        backend.info.shader_language = lang;
        backend
    }

    /// Returns how many shader creation calls have occurred.
    pub fn shader_create_calls(&self) -> usize {
        self.shader_create_calls
    }

    /// Returns how many non-indexed draw calls have occurred.
    pub fn draw_arrays_calls(&self) -> usize {
        self.draw_arrays_calls
    }

    /// Returns how many indexed draw calls have occurred.
    pub fn draw_indexed_calls(&self) -> usize {
        self.draw_indexed_calls
    }

    /// Returns how many non-indexed instanced draw calls have occurred.
    pub fn draw_arrays_instanced_calls(&self) -> usize {
        self.draw_arrays_instanced_calls
    }

    /// Returns how many indexed instanced draw calls have occurred.
    pub fn draw_indexed_instanced_calls(&self) -> usize {
        self.draw_indexed_instanced_calls
    }

    /// Returns the total number of draw commands (of every kind) recorded.
    ///
    /// This includes main-pass, instanced, and shadow-pass draw commands.
    pub fn total_draw_commands(&self) -> usize {
        self.total_draw_commands
    }

    /// Returns the number of draw commands recorded during shadow pre-pass
    /// recording (between `begin_shadow_recording` and `end_shadow_recording`).
    pub fn shadow_draw_commands(&self) -> usize {
        self.shadow_draw_commands
    }

    /// Returns `true` while the backend is inside a shadow pre-pass recording block.
    pub fn is_shadow_recording(&self) -> bool {
        self.shadow_recording
    }

    /// Resets all draw-command counters to zero. Handy for measuring a single
    /// frame in isolation without reconstructing the backend.
    pub fn reset_draw_counters(&mut self) {
        self.draw_arrays_calls = 0;
        self.draw_indexed_calls = 0;
        self.draw_arrays_instanced_calls = 0;
        self.draw_indexed_instanced_calls = 0;
        self.total_draw_commands = 0;
        self.shadow_draw_commands = 0;
    }

    /// Records a single draw command against the aggregate counters. Called by
    /// every `DrawOps` method so that `total_draw_commands` and (when a shadow
    /// pass is being recorded) `shadow_draw_commands` stay in sync.
    pub(super) fn record_draw_command(&mut self) {
        self.total_draw_commands += 1;
        if self.shadow_recording {
            self.shadow_draw_commands += 1;
        }
    }
}

impl Default for NullBackend {
    fn default() -> Self {
        Self::new()
    }
}
