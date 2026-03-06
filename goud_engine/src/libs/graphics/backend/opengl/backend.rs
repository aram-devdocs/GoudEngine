//! OpenGL backend struct definition, constructor, and cleanup.

use super::{
    super::BackendCapabilities, super::BackendInfo, BufferMetadata, ShaderMetadata, TextureMetadata,
};
use crate::core::{error::GoudError, error::GoudResult, handle::HandleAllocator};
use crate::libs::graphics::backend::types::{
    BufferHandle, BufferMarker, ShaderHandle, ShaderMarker, TextureHandle, TextureMarker,
};
use std::collections::HashMap;

/// OpenGL 3.3 Core backend implementation.
///
/// This backend uses the `gl` crate bindings and requires an active OpenGL context
/// to be created before use (via GLFW, SDL, glutin, etc.).
///
/// # Thread Safety
///
/// OpenGL contexts are thread-local, so this backend is Send but operations
/// must be called from the thread that owns the OpenGL context.
pub struct OpenGLBackend {
    pub(super) info: BackendInfo,
    pub(super) clear_color: [f32; 4],

    // Buffer management
    pub(super) buffer_allocator: HandleAllocator<BufferMarker>,
    pub(super) buffers: HashMap<BufferHandle, BufferMetadata>,

    // Currently bound buffers for each type
    pub(super) bound_vertex_buffer: Option<u32>,
    pub(super) bound_index_buffer: Option<u32>,
    pub(super) bound_uniform_buffer: Option<u32>,

    // Texture management
    pub(super) texture_allocator: HandleAllocator<TextureMarker>,
    pub(super) textures: HashMap<TextureHandle, TextureMetadata>,

    // Currently bound textures for each unit (typically 16 units)
    pub(super) bound_textures: Vec<Option<u32>>,

    // Shader management
    pub(super) shader_allocator: HandleAllocator<ShaderMarker>,
    pub(super) shaders: HashMap<ShaderHandle, ShaderMetadata>,

    // Currently bound shader program
    pub(super) bound_shader: Option<u32>,

    // Default VAO kept bound for backends that require vertex array state (OpenGL 3.3 core)
    pub(super) default_vao: u32,
}

impl OpenGLBackend {
    /// Creates a new OpenGL backend.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No OpenGL context is active
    /// - OpenGL version is insufficient (< 3.3)
    /// - Failed to query OpenGL information
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // After creating an OpenGL context with GLFW/SDL/etc:
    /// let backend = OpenGLBackend::new()?;
    /// ```
    pub fn new() -> GoudResult<Self> {
        // Query OpenGL information
        // Note: These queries require an active OpenGL context
        // SAFETY: GL context is active; GetString returns a pointer valid for the context lifetime, checked for null before use.
        let version_str = unsafe {
            let ptr = gl::GetString(gl::VERSION);
            if ptr.is_null() {
                return Err(GoudError::BackendNotSupported(
                    "No OpenGL context active".to_string(),
                ));
            }
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };

        // SAFETY: GL context is active; GetString(VENDOR) returns a static C string valid for the context lifetime, null-checked before use.
        let vendor_str = unsafe {
            let ptr = gl::GetString(gl::VENDOR);
            if ptr.is_null() {
                "Unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(ptr as *const i8)
                    .to_string_lossy()
                    .into_owned()
            }
        };

        // SAFETY: GL context is active; GetString(RENDERER) returns a static C string valid for the context lifetime, null-checked before use.
        let renderer_str = unsafe {
            let ptr = gl::GetString(gl::RENDERER);
            if ptr.is_null() {
                "Unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(ptr as *const i8)
                    .to_string_lossy()
                    .into_owned()
            }
        };

        // Query capabilities
        let mut max_texture_units: i32 = 0;
        let mut max_texture_size: i32 = 0;
        let mut max_vertex_attribs: i32 = 0;
        let mut max_uniform_buffer_size: i32 = 0;

        // SAFETY: GL context is active; output variables are stack-allocated integers with correct size for GetIntegerv.
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut max_texture_units);
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_texture_size);
            gl::GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_vertex_attribs);
            gl::GetIntegerv(gl::MAX_UNIFORM_BLOCK_SIZE, &mut max_uniform_buffer_size);
        }

        let capabilities = BackendCapabilities {
            max_texture_units: max_texture_units.max(8) as u32,
            max_texture_size: max_texture_size.max(2048) as u32,
            max_vertex_attributes: max_vertex_attribs.max(8) as u32,
            max_uniform_buffer_size: max_uniform_buffer_size.max(16384) as u32,
            supports_instancing: true,       // GL 3.3 supports instancing
            supports_compute_shaders: false, // Requires GL 4.3+
            supports_geometry_shaders: true, // GL 3.2+
            supports_tessellation: false,    // Requires GL 4.0+
            supports_multisampling: true,
            supports_anisotropic_filtering: true, // Common extension
        };

        let info = BackendInfo {
            name: "OpenGL",
            version: version_str,
            vendor: vendor_str,
            renderer: renderer_str,
            capabilities,
        };

        // Create a default VAO that stays bound for the lifetime of the backend.
        // OpenGL 3.3 core requires a VAO for draw calls and vertex attribute setup.
        let mut default_vao = 0u32;
        // SAFETY: GL context is active; default_vao is a stack-allocated output variable; the VAO is bound immediately after generation and freed in Drop.
        unsafe {
            gl::GenVertexArrays(1, &mut default_vao);
            gl::BindVertexArray(default_vao);
        }

        let max_units = capabilities.max_texture_units as usize;
        Ok(Self {
            info,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            buffer_allocator: HandleAllocator::new(),
            buffers: HashMap::new(),
            bound_vertex_buffer: None,
            bound_index_buffer: None,
            bound_uniform_buffer: None,
            texture_allocator: HandleAllocator::new(),
            textures: HashMap::new(),
            bound_textures: vec![None; max_units],
            shader_allocator: HandleAllocator::new(),
            shaders: HashMap::new(),
            bound_shader: None,
            default_vao,
        })
    }
}

impl Drop for OpenGLBackend {
    fn drop(&mut self) {
        // SAFETY: Cleaning up all GL resources owned by this backend instance
        unsafe {
            for meta in self.buffers.values() {
                gl::DeleteBuffers(1, &meta.gl_id);
            }
            for meta in self.textures.values() {
                gl::DeleteTextures(1, &meta.gl_id);
            }
            for meta in self.shaders.values() {
                gl::DeleteProgram(meta.gl_id);
            }
            gl::DeleteVertexArrays(1, &self.default_vao);
        }
    }
}
