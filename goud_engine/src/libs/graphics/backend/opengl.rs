//! OpenGL Backend Implementation
//!
//! This module provides an OpenGL 3.3 Core implementation of the RenderBackend trait.
//! It manages OpenGL state, resources (buffers, textures, shaders), and rendering operations.

use super::{
    types::{BufferHandle, BufferType, BufferUsage, ShaderHandle, TextureFilter, TextureFormat, TextureHandle, TextureWrap},
    BackendCapabilities, BackendInfo, BlendFactor, CullFace, RenderBackend,
};
use crate::core::{
    error::{GoudError, GoudResult},
    handle::HandleAllocator,
};
use std::collections::HashMap;

/// Internal buffer metadata stored alongside the OpenGL buffer ID.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in OpenGL context tests
struct BufferMetadata {
    /// OpenGL buffer object ID
    gl_id: u32,
    /// Type of buffer (Vertex, Index, Uniform)
    buffer_type: BufferType,
    /// Usage hint (Static, Dynamic, Stream)
    usage: BufferUsage,
    /// Size of buffer in bytes
    size: usize,
}

/// Internal texture metadata stored alongside the OpenGL texture ID.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in OpenGL context tests
struct TextureMetadata {
    /// OpenGL texture object ID
    gl_id: u32,
    /// Texture width in pixels
    width: u32,
    /// Texture height in pixels
    height: u32,
    /// Pixel format
    format: TextureFormat,
    /// Filtering mode
    filter: TextureFilter,
    /// Wrapping mode
    wrap: TextureWrap,
}

/// Internal shader metadata stored alongside the OpenGL shader program ID.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in OpenGL context tests
struct ShaderMetadata {
    /// OpenGL shader program ID
    gl_id: u32,
    /// Cached uniform locations by name
    uniform_locations: HashMap<String, i32>,
}

/// OpenGL 3.3 Core backend implementation.
///
/// This backend uses the `gl` crate bindings and requires an active OpenGL context
/// to be created before use (via GLFW, SDL, glutin, etc.).
///
/// # Thread Safety
///
/// OpenGL contexts are thread-local, so this backend is Send but operations
/// must be called from the thread that owns the OpenGL context.
#[allow(dead_code)] // Used in OpenGL context tests
pub struct OpenGLBackend {
    info: BackendInfo,
    clear_color: [f32; 4],

    // Buffer management
    buffer_allocator: HandleAllocator<super::types::BufferMarker>,
    buffers: HashMap<BufferHandle, BufferMetadata>,

    // Currently bound buffers for each type
    bound_vertex_buffer: Option<u32>,
    bound_index_buffer: Option<u32>,
    bound_uniform_buffer: Option<u32>,

    // Texture management
    texture_allocator: HandleAllocator<super::types::TextureMarker>,
    textures: HashMap<TextureHandle, TextureMetadata>,

    // Currently bound textures for each unit (typically 16 units)
    bound_textures: Vec<Option<u32>>,

    // Shader management
    shader_allocator: HandleAllocator<super::types::ShaderMarker>,
    shaders: HashMap<ShaderHandle, ShaderMetadata>,

    // Currently bound shader program
    bound_shader: Option<u32>,
}

#[allow(dead_code)] // Methods used in OpenGL context tests
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
        let version_str = unsafe {
            let ptr = gl::GetString(gl::VERSION);
            if ptr.is_null() {
                return Err(GoudError::BackendNotSupported(
                    "No OpenGL context active".to_string()
                ));
            }
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };

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
            supports_instancing: true,  // GL 3.3 supports instancing
            supports_compute_shaders: false,  // Requires GL 4.3+
            supports_geometry_shaders: true,  // GL 3.2+
            supports_tessellation: false,  // Requires GL 4.0+
            supports_multisampling: true,
            supports_anisotropic_filtering: true,  // Common extension
        };

        let info = BackendInfo {
            name: "OpenGL",
            version: version_str,
            vendor: vendor_str,
            renderer: renderer_str,
            capabilities,
        };

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
        })
    }

    /// Converts BufferType to OpenGL buffer target constant.
    fn buffer_type_to_gl_target(buffer_type: BufferType) -> u32 {
        match buffer_type {
            BufferType::Vertex => gl::ARRAY_BUFFER,
            BufferType::Index => gl::ELEMENT_ARRAY_BUFFER,
            BufferType::Uniform => gl::UNIFORM_BUFFER,
        }
    }

    /// Converts BufferUsage to OpenGL usage hint constant.
    fn buffer_usage_to_gl_usage(usage: BufferUsage) -> u32 {
        match usage {
            BufferUsage::Static => gl::STATIC_DRAW,
            BufferUsage::Dynamic => gl::DYNAMIC_DRAW,
            BufferUsage::Stream => gl::STREAM_DRAW,
        }
    }

    /// Gets the currently bound buffer ID for a buffer type.
    fn get_bound_buffer(&self, buffer_type: BufferType) -> Option<u32> {
        match buffer_type {
            BufferType::Vertex => self.bound_vertex_buffer,
            BufferType::Index => self.bound_index_buffer,
            BufferType::Uniform => self.bound_uniform_buffer,
        }
    }

    /// Sets the currently bound buffer ID for a buffer type.
    fn set_bound_buffer(&mut self, buffer_type: BufferType, gl_id: Option<u32>) {
        match buffer_type {
            BufferType::Vertex => self.bound_vertex_buffer = gl_id,
            BufferType::Index => self.bound_index_buffer = gl_id,
            BufferType::Uniform => self.bound_uniform_buffer = gl_id,
        }
    }
}

impl RenderBackend for OpenGLBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    fn begin_frame(&mut self) -> GoudResult<()> {
        // OpenGL doesn't need explicit frame begin
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        // OpenGL doesn't need explicit frame end
        // Swap buffers is handled by windowing system
        Ok(())
    }

    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }

    fn clear_color(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn clear_depth(&mut self) {
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(x, y, width as i32, height as i32);
        }
    }

    fn enable_depth_test(&mut self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    fn disable_depth_test(&mut self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }
    }

    fn enable_blending(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
        }
    }

    fn disable_blending(&mut self) {
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor) {
        let src_gl = Self::blend_factor_to_gl(src);
        let dst_gl = Self::blend_factor_to_gl(dst);
        unsafe {
            gl::BlendFunc(src_gl, dst_gl);
        }
    }

    fn enable_culling(&mut self) {
        unsafe {
            gl::Enable(gl::CULL_FACE);
        }
    }

    fn disable_culling(&mut self) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
    }

    fn set_cull_face(&mut self, face: CullFace) {
        let gl_face = match face {
            CullFace::Front => gl::FRONT,
            CullFace::Back => gl::BACK,
            CullFace::FrontAndBack => gl::FRONT_AND_BACK,
        };
        unsafe {
            gl::CullFace(gl_face);
        }
    }

    // ============================================================================
    // Buffer Operations
    // ============================================================================

    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        let mut gl_id: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut gl_id);
            if gl_id == 0 {
                return Err(GoudError::BufferCreationFailed(
                    "Failed to generate OpenGL buffer".to_string()
                ));
            }
        }

        let target = Self::buffer_type_to_gl_target(buffer_type);
        let gl_usage = Self::buffer_usage_to_gl_usage(usage);

        // Bind and upload data
        unsafe {
            gl::BindBuffer(target, gl_id);
            gl::BufferData(
                target,
                data.len() as isize,
                data.as_ptr() as *const std::ffi::c_void,
                gl_usage,
            );

            // Check for errors
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                gl::DeleteBuffers(1, &gl_id);
                return Err(GoudError::BufferCreationFailed(
                    format!("OpenGL error during buffer creation: 0x{:X}", error)
                ));
            }

            // Unbind
            gl::BindBuffer(target, 0);
        }

        // Allocate handle and store metadata
        let handle = self.buffer_allocator.allocate();
        let metadata = BufferMetadata {
            gl_id,
            buffer_type,
            usage,
            size: data.len(),
        };
        self.buffers.insert(handle, metadata);

        // Update bound buffer tracking
        self.set_bound_buffer(buffer_type, None);

        Ok(handle)
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        let metadata = self.buffers.get(&handle).ok_or(GoudError::InvalidHandle)?;

        if offset + data.len() > metadata.size {
            return Err(GoudError::InvalidState(
                format!(
                    "Buffer update out of bounds: offset {} + size {} > buffer size {}",
                    offset, data.len(), metadata.size
                )
            ));
        }

        let target = Self::buffer_type_to_gl_target(metadata.buffer_type);

        unsafe {
            gl::BindBuffer(target, metadata.gl_id);
            gl::BufferSubData(
                target,
                offset as isize,
                data.len() as isize,
                data.as_ptr() as *const std::ffi::c_void,
            );

            let error = gl::GetError();
            if error != gl::NO_ERROR {
                return Err(GoudError::InternalError(
                    format!("OpenGL error during buffer update: 0x{:X}", error)
                ));
            }

            gl::BindBuffer(target, 0);
        }

        self.set_bound_buffer(metadata.buffer_type, None);

        Ok(())
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        if let Some(metadata) = self.buffers.remove(&handle) {
            unsafe {
                gl::DeleteBuffers(1, &metadata.gl_id);
            }

            // Clear bound buffer tracking if this was bound
            if self.get_bound_buffer(metadata.buffer_type) == Some(metadata.gl_id) {
                self.set_bound_buffer(metadata.buffer_type, None);
            }

            self.buffer_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        self.buffer_allocator.is_alive(handle) && self.buffers.contains_key(&handle)
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        self.buffers.get(&handle).map(|m| m.size)
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        let metadata = self.buffers.get(&handle).ok_or(GoudError::InvalidHandle)?;

        let target = Self::buffer_type_to_gl_target(metadata.buffer_type);

        unsafe {
            gl::BindBuffer(target, metadata.gl_id);
        }

        self.set_bound_buffer(metadata.buffer_type, Some(metadata.gl_id));

        Ok(())
    }

    fn unbind_buffer(&mut self, buffer_type: BufferType) {
        let target = Self::buffer_type_to_gl_target(buffer_type);

        unsafe {
            gl::BindBuffer(target, 0);
        }

        self.set_bound_buffer(buffer_type, None);
    }

    // ============================================================================
    // Texture Operations
    // ============================================================================

    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        if width == 0 || height == 0 {
            return Err(GoudError::TextureCreationFailed(
                "Texture dimensions must be greater than 0".to_string()
            ));
        }

        let mut gl_id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut gl_id);
            if gl_id == 0 {
                return Err(GoudError::TextureCreationFailed(
                    "Failed to generate OpenGL texture".to_string()
                ));
            }
        }

        let (internal_format, pixel_format, pixel_type) = Self::texture_format_to_gl(format);
        let filter_gl = Self::texture_filter_to_gl(filter);
        let wrap_gl = Self::texture_wrap_to_gl(wrap);

        // Bind and upload data
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_id);

            // Upload texture data
            let data_ptr = if data.is_empty() {
                std::ptr::null()
            } else {
                data.as_ptr() as *const std::ffi::c_void
            };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0, // mip level
                internal_format as i32,
                width as i32,
                height as i32,
                0, // border (must be 0)
                pixel_format,
                pixel_type,
                data_ptr,
            );

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter_gl as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter_gl as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_gl as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_gl as i32);

            // Check for errors
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                gl::DeleteTextures(1, &gl_id);
                return Err(GoudError::TextureCreationFailed(
                    format!("OpenGL error during texture creation: 0x{:X}", error)
                ));
            }

            // Unbind
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        // Allocate handle and store metadata
        let handle = self.texture_allocator.allocate();
        let metadata = TextureMetadata {
            gl_id,
            width,
            height,
            format,
            filter,
            wrap,
        };
        self.textures.insert(handle, metadata);

        // Clear all bound texture tracking (texture is now unbound)
        for unit in 0..self.bound_textures.len() {
            if self.bound_textures[unit] == Some(gl_id) {
                self.bound_textures[unit] = None;
            }
        }

        Ok(handle)
    }

    fn update_texture(
        &mut self,
        handle: TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()> {
        let metadata = self.textures.get(&handle).ok_or(GoudError::InvalidHandle)?;

        // Validate region bounds
        if x + width > metadata.width || y + height > metadata.height {
            return Err(GoudError::TextureCreationFailed(
                format!(
                    "Update region ({}x{} at {},{}) exceeds texture bounds ({}x{})",
                    width, height, x, y, metadata.width, metadata.height
                )
            ));
        }

        // Validate data size
        let expected_size = (width * height) as usize * Self::bytes_per_pixel(metadata.format);
        if data.len() != expected_size {
            return Err(GoudError::TextureCreationFailed(
                format!(
                    "Data size mismatch: expected {} bytes, got {}",
                    expected_size,
                    data.len()
                )
            ));
        }

        let (_, pixel_format, pixel_type) = Self::texture_format_to_gl(metadata.format);

        unsafe {
            // Bind texture
            gl::BindTexture(gl::TEXTURE_2D, metadata.gl_id);

            // Upload sub-image
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0, // mip level
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                pixel_format,
                pixel_type,
                data.as_ptr() as *const std::ffi::c_void,
            );

            // Check for errors
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                return Err(GoudError::TextureCreationFailed(
                    format!("OpenGL error during texture update: 0x{:X}", error)
                ));
            }

            // Unbind
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(())
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        if let Some(metadata) = self.textures.remove(&handle) {
            unsafe {
                gl::DeleteTextures(1, &metadata.gl_id);
            }

            // Clear from bound texture tracking
            for unit in 0..self.bound_textures.len() {
                if self.bound_textures[unit] == Some(metadata.gl_id) {
                    self.bound_textures[unit] = None;
                }
            }

            self.texture_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        self.texture_allocator.is_alive(handle) && self.textures.contains_key(&handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.textures.get(&handle).map(|meta| (meta.width, meta.height))
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        let metadata = self.textures.get(&handle).ok_or(GoudError::InvalidHandle)?;

        // Validate texture unit
        if unit >= self.bound_textures.len() as u32 {
            return Err(GoudError::TextureCreationFailed(
                format!(
                    "Texture unit {} exceeds maximum supported units ({})",
                    unit,
                    self.bound_textures.len()
                )
            ));
        }

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
            gl::BindTexture(gl::TEXTURE_2D, metadata.gl_id);
        }

        self.set_bound_texture(unit, Some(metadata.gl_id));

        Ok(())
    }

    fn unbind_texture(&mut self, unit: u32) {
        if unit < self.bound_textures.len() as u32 {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            self.set_bound_texture(unit, None);
        }
    }

    // ============================================================================
    // Shader Operations
    // ============================================================================

    fn create_shader(
        &mut self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> GoudResult<ShaderHandle> {
        if vertex_src.is_empty() {
            return Err(GoudError::ShaderCompilationFailed(
                "Vertex shader source is empty".to_string()
            ));
        }
        if fragment_src.is_empty() {
            return Err(GoudError::ShaderCompilationFailed(
                "Fragment shader source is empty".to_string()
            ));
        }

        // Compile vertex shader
        let vertex_shader = Self::compile_shader(gl::VERTEX_SHADER, vertex_src)?;

        // Compile fragment shader
        let fragment_shader = match Self::compile_shader(gl::FRAGMENT_SHADER, fragment_src) {
            Ok(shader) => shader,
            Err(e) => {
                // Clean up vertex shader on fragment compilation failure
                unsafe {
                    gl::DeleteShader(vertex_shader);
                }
                return Err(e);
            }
        };

        // Link shader program
        let program_id = unsafe { gl::CreateProgram() };
        if program_id == 0 {
            unsafe {
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);
            }
            return Err(GoudError::ShaderLinkFailed(
                "Failed to create shader program".to_string()
            ));
        }

        unsafe {
            gl::AttachShader(program_id, vertex_shader);
            gl::AttachShader(program_id, fragment_shader);
            gl::LinkProgram(program_id);

            // Check link status
            let mut success: i32 = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

            if success == 0 {
                // Get error message
                let mut log_length: i32 = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_length);

                let mut log = vec![0u8; log_length as usize];
                gl::GetProgramInfoLog(
                    program_id,
                    log_length,
                    std::ptr::null_mut(),
                    log.as_mut_ptr() as *mut i8,
                );

                let error_msg = String::from_utf8_lossy(&log).to_string();

                // Clean up
                gl::DeleteProgram(program_id);
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);

                return Err(GoudError::ShaderLinkFailed(error_msg));
            }

            // Clean up shader objects (they're now linked into the program)
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        // Allocate handle and store metadata
        let handle = self.shader_allocator.allocate();
        let metadata = ShaderMetadata {
            gl_id: program_id,
            uniform_locations: HashMap::new(),
        };
        self.shaders.insert(handle, metadata);

        Ok(handle)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        if let Some(metadata) = self.shaders.remove(&handle) {
            unsafe {
                gl::DeleteProgram(metadata.gl_id);
            }

            // Clear bound shader if this was it
            if self.bound_shader == Some(metadata.gl_id) {
                self.bound_shader = None;
            }

            self.shader_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        self.shader_allocator.is_alive(handle) && self.shaders.contains_key(&handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        let metadata = self.shaders.get(&handle).ok_or(GoudError::InvalidHandle)?;

        unsafe {
            gl::UseProgram(metadata.gl_id);
        }

        self.bound_shader = Some(metadata.gl_id);

        Ok(())
    }

    fn unbind_shader(&mut self) {
        unsafe {
            gl::UseProgram(0);
        }

        self.bound_shader = None;
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        let metadata = self.shaders.get(&handle)?;

        // Check if we already cached this location
        if let Some(&location) = metadata.uniform_locations.get(name) {
            return if location >= 0 { Some(location) } else { None };
        }

        // Query OpenGL for the location
        let c_name = std::ffi::CString::new(name).ok()?;
        let location = unsafe {
            gl::GetUniformLocation(metadata.gl_id, c_name.as_ptr())
        };

        // Cache the result (even if negative, to avoid repeated queries)
        // Note: We need mutable access to cache, but this method takes &self
        // In practice, caching would be done externally or with interior mutability
        // For now, just return the location without caching

        if location >= 0 {
            Some(location)
        } else {
            None
        }
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        unsafe {
            gl::Uniform1i(location, value);
        }
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        unsafe {
            gl::Uniform1f(location, value);
        }
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        unsafe {
            gl::Uniform2f(location, x, y);
        }
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        unsafe {
            gl::Uniform3f(location, x, y, z);
        }
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            gl::Uniform4f(location, x, y, z, w);
        }
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
        }
    }

    // ============================================================================
    // Vertex Array Setup
    // ============================================================================

    fn set_vertex_attributes(&mut self, layout: &super::types::VertexLayout) {
        unsafe {
            for attr in &layout.attributes {
                gl::EnableVertexAttribArray(attr.location);

                let gl_type = Self::attribute_type_to_gl_type(attr.attribute_type);
                let component_count = attr.attribute_type.component_count() as i32;

                gl::VertexAttribPointer(
                    attr.location,
                    component_count,
                    gl_type,
                    if attr.normalized { gl::TRUE } else { gl::FALSE },
                    layout.stride as i32,
                    attr.offset as *const _,
                );
            }
        }
    }

    // ============================================================================
    // Draw Calls
    // ============================================================================

    fn draw_arrays(
        &mut self,
        topology: super::types::PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        // Validate state
        if self.bound_shader.is_none() {
            return Err(GoudError::InvalidState(
                "No shader bound for draw call".to_string(),
            ));
        }
        if self.bound_vertex_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No vertex buffer bound for draw call".to_string(),
            ));
        }

        let gl_topology = Self::topology_to_gl(topology);

        unsafe {
            gl::DrawArrays(gl_topology, first as i32, count as i32);
        }

        Ok(())
    }

    fn draw_indexed(
        &mut self,
        topology: super::types::PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        // Validate state
        if self.bound_shader.is_none() {
            return Err(GoudError::InvalidState(
                "No shader bound for draw call".to_string(),
            ));
        }
        if self.bound_vertex_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No vertex buffer bound for draw call".to_string(),
            ));
        }
        if self.bound_index_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No index buffer bound for draw call".to_string(),
            ));
        }

        let gl_topology = Self::topology_to_gl(topology);

        unsafe {
            gl::DrawElements(
                gl_topology,
                count as i32,
                gl::UNSIGNED_INT,
                offset as *const _,
            );
        }

        Ok(())
    }

    fn draw_indexed_u16(
        &mut self,
        topology: super::types::PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        // Validate state
        if self.bound_shader.is_none() {
            return Err(GoudError::InvalidState(
                "No shader bound for draw call".to_string(),
            ));
        }
        if self.bound_vertex_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No vertex buffer bound for draw call".to_string(),
            ));
        }
        if self.bound_index_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No index buffer bound for draw call".to_string(),
            ));
        }

        let gl_topology = Self::topology_to_gl(topology);

        unsafe {
            gl::DrawElements(
                gl_topology,
                count as i32,
                gl::UNSIGNED_SHORT,
                offset as *const _,
            );
        }

        Ok(())
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: super::types::PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        // Check capability
        if !self.info.capabilities.supports_instancing {
            return Err(GoudError::BackendNotSupported(
                "Instanced rendering not supported".to_string(),
            ));
        }

        // Validate state
        if self.bound_shader.is_none() {
            return Err(GoudError::InvalidState(
                "No shader bound for draw call".to_string(),
            ));
        }
        if self.bound_vertex_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No vertex buffer bound for draw call".to_string(),
            ));
        }

        let gl_topology = Self::topology_to_gl(topology);

        unsafe {
            gl::DrawArraysInstanced(
                gl_topology,
                first as i32,
                count as i32,
                instance_count as i32,
            );
        }

        Ok(())
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: super::types::PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        // Check capability
        if !self.info.capabilities.supports_instancing {
            return Err(GoudError::BackendNotSupported(
                "Instanced rendering not supported".to_string(),
            ));
        }

        // Validate state
        if self.bound_shader.is_none() {
            return Err(GoudError::InvalidState(
                "No shader bound for draw call".to_string(),
            ));
        }
        if self.bound_vertex_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No vertex buffer bound for draw call".to_string(),
            ));
        }
        if self.bound_index_buffer.is_none() {
            return Err(GoudError::InvalidState(
                "No index buffer bound for draw call".to_string(),
            ));
        }

        let gl_topology = Self::topology_to_gl(topology);

        unsafe {
            gl::DrawElementsInstanced(
                gl_topology,
                count as i32,
                gl::UNSIGNED_INT,
                offset as *const _,
                instance_count as i32,
            );
        }

        Ok(())
    }
}

#[allow(dead_code)] // Used in OpenGL context tests
impl OpenGLBackend {
    /// Compiles a single shader stage (vertex, fragment, etc.).
    ///
    /// Returns the OpenGL shader ID on success, or an error with compilation message.
    fn compile_shader(shader_type: u32, source: &str) -> GoudResult<u32> {
        let shader_id = unsafe { gl::CreateShader(shader_type) };
        if shader_id == 0 {
            return Err(GoudError::ShaderCompilationFailed(
                "Failed to create shader object".to_string()
            ));
        }

        // Compile the shader
        let c_source = std::ffi::CString::new(source)
            .map_err(|_| GoudError::ShaderCompilationFailed(
                "Shader source contains null byte".to_string()
            ))?;

        unsafe {
            gl::ShaderSource(shader_id, 1, &c_source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader_id);

            // Check compilation status
            let mut success: i32 = 0;
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

            if success == 0 {
                // Get error message
                let mut log_length: i32 = 0;
                gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut log_length);

                let mut log = vec![0u8; log_length as usize];
                gl::GetShaderInfoLog(
                    shader_id,
                    log_length,
                    std::ptr::null_mut(),
                    log.as_mut_ptr() as *mut i8,
                );

                let error_msg = String::from_utf8_lossy(&log).to_string();

                gl::DeleteShader(shader_id);

                let stage_name = match shader_type {
                    gl::VERTEX_SHADER => "Vertex",
                    gl::FRAGMENT_SHADER => "Fragment",
                    gl::GEOMETRY_SHADER => "Geometry",
                    gl::COMPUTE_SHADER => "Compute",
                    _ => "Unknown",
                };

                return Err(GoudError::ShaderCompilationFailed(
                    format!("{} shader compilation failed: {}", stage_name, error_msg)
                ));
            }
        }

        Ok(shader_id)
    }

    /// Converts BlendFactor to OpenGL blend factor constant.
    fn blend_factor_to_gl(factor: BlendFactor) -> u32 {
        match factor {
            BlendFactor::Zero => gl::ZERO,
            BlendFactor::One => gl::ONE,
            BlendFactor::SrcColor => gl::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => gl::DST_COLOR,
            BlendFactor::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => gl::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => gl::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
            BlendFactor::ConstantColor => gl::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha => gl::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
        }
    }

    /// Converts TextureFormat to OpenGL internal format and pixel format/type.
    ///
    /// Returns (internal_format, format, type) tuple for use with glTexImage2D.
    fn texture_format_to_gl(format: TextureFormat) -> (u32, u32, u32) {
        match format {
            TextureFormat::R8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
            TextureFormat::RG8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
            TextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            TextureFormat::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            TextureFormat::RGBA16F => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT),
            TextureFormat::RGBA32F => (gl::RGBA32F, gl::RGBA, gl::FLOAT),
            TextureFormat::Depth => (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_INT),
            TextureFormat::DepthStencil => (gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
        }
    }

    /// Converts TextureFilter to OpenGL filter constant.
    fn texture_filter_to_gl(filter: TextureFilter) -> u32 {
        match filter {
            TextureFilter::Nearest => gl::NEAREST,
            TextureFilter::Linear => gl::LINEAR,
        }
    }

    /// Converts TextureWrap to OpenGL wrap constant.
    fn texture_wrap_to_gl(wrap: TextureWrap) -> u32 {
        match wrap {
            TextureWrap::Repeat => gl::REPEAT,
            TextureWrap::MirroredRepeat => gl::MIRRORED_REPEAT,
            TextureWrap::ClampToEdge => gl::CLAMP_TO_EDGE,
            TextureWrap::ClampToBorder => gl::CLAMP_TO_BORDER,
        }
    }

    /// Returns the number of bytes per pixel for a given texture format.
    fn bytes_per_pixel(format: TextureFormat) -> usize {
        match format {
            TextureFormat::R8 => 1,
            TextureFormat::RG8 => 2,
            TextureFormat::RGB8 => 3,
            TextureFormat::RGBA8 => 4,
            TextureFormat::RGBA16F => 8,  // 4 channels × 2 bytes
            TextureFormat::RGBA32F => 16, // 4 channels × 4 bytes
            TextureFormat::Depth => 4,    // 24-bit or 32-bit typically
            TextureFormat::DepthStencil => 4, // 24 + 8 bits = 32 bits
        }
    }

    /// Gets the OpenGL ID of the currently bound texture on a given unit.
    fn get_bound_texture(&self, unit: u32) -> Option<u32> {
        self.bound_textures.get(unit as usize).copied().flatten()
    }

    /// Sets the currently bound texture for a given unit.
    fn set_bound_texture(&mut self, unit: u32, gl_id: Option<u32>) {
        if let Some(slot) = self.bound_textures.get_mut(unit as usize) {
            *slot = gl_id;
        }
    }

    /// Converts PrimitiveTopology to OpenGL primitive mode constant.
    fn topology_to_gl(topology: super::types::PrimitiveTopology) -> u32 {
        use crate::libs::graphics::backend::types::PrimitiveTopology;
        match topology {
            PrimitiveTopology::Points => gl::POINTS,
            PrimitiveTopology::Lines => gl::LINES,
            PrimitiveTopology::LineStrip => gl::LINE_STRIP,
            PrimitiveTopology::Triangles => gl::TRIANGLES,
            PrimitiveTopology::TriangleStrip => gl::TRIANGLE_STRIP,
            PrimitiveTopology::TriangleFan => gl::TRIANGLE_FAN,
        }
    }

    /// Converts VertexAttributeType to OpenGL type constant.
    fn attribute_type_to_gl_type(attr_type: super::types::VertexAttributeType) -> u32 {
        use super::types::VertexAttributeType;
        match attr_type {
            VertexAttributeType::Float
            | VertexAttributeType::Float2
            | VertexAttributeType::Float3
            | VertexAttributeType::Float4 => gl::FLOAT,
            VertexAttributeType::Int
            | VertexAttributeType::Int2
            | VertexAttributeType::Int3
            | VertexAttributeType::Int4 => gl::INT,
            VertexAttributeType::UInt
            | VertexAttributeType::UInt2
            | VertexAttributeType::UInt3
            | VertexAttributeType::UInt4 => gl::UNSIGNED_INT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require an OpenGL context to run properly.
    // In CI, they may be skipped or run with a headless context.

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_opengl_backend_creation() {
        let result = OpenGLBackend::new();
        if result.is_ok() {
            let backend = result.unwrap();
            assert_eq!(backend.info().name, "OpenGL");
            assert!(backend.info().version.contains("3.") || backend.info().version.contains("4."));
        }
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_buffer_lifecycle() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create a buffer
        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let handle = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            &data,
        ).unwrap();

        // Verify it's valid
        assert!(backend.is_buffer_valid(handle));
        assert_eq!(backend.buffer_size(handle), Some(8));

        // Destroy it
        assert!(backend.destroy_buffer(handle));
        assert!(!backend.is_buffer_valid(handle));
        assert_eq!(backend.buffer_size(handle), None);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_buffer_update() {
        let mut backend = OpenGLBackend::new().unwrap();

        let data: Vec<u8> = vec![0; 16];
        let handle = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Dynamic,
            &data,
        ).unwrap();

        let new_data: Vec<u8> = vec![1, 2, 3, 4];
        backend.update_buffer(handle, 0, &new_data).unwrap();

        backend.destroy_buffer(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_buffer_bind() {
        let mut backend = OpenGLBackend::new().unwrap();

        let data: Vec<u8> = vec![1, 2, 3, 4];
        let handle = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            &data,
        ).unwrap();

        backend.bind_buffer(handle).unwrap();
        backend.unbind_buffer(BufferType::Vertex);

        backend.destroy_buffer(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_multiple_buffers() {
        let mut backend = OpenGLBackend::new().unwrap();

        let data1: Vec<u8> = vec![1, 2, 3, 4];
        let data2: Vec<u8> = vec![5, 6, 7, 8];

        let handle1 = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            &data1,
        ).unwrap();

        let handle2 = backend.create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            &data2,
        ).unwrap();

        assert!(backend.is_buffer_valid(handle1));
        assert!(backend.is_buffer_valid(handle2));
        assert_ne!(handle1, handle2);

        backend.destroy_buffer(handle1);
        backend.destroy_buffer(handle2);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_buffer_update_out_of_bounds() {
        let mut backend = OpenGLBackend::new().unwrap();

        let data: Vec<u8> = vec![0; 8];
        let handle = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Dynamic,
            &data,
        ).unwrap();

        let new_data: Vec<u8> = vec![1; 16]; // Too large
        let result = backend.update_buffer(handle, 0, &new_data);
        assert!(result.is_err());

        backend.destroy_buffer(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_invalid_handle_operations() {
        let mut backend = OpenGLBackend::new().unwrap();
        let invalid_handle = BufferHandle::INVALID;

        assert!(!backend.is_buffer_valid(invalid_handle));
        assert_eq!(backend.buffer_size(invalid_handle), None);
        assert!(backend.bind_buffer(invalid_handle).is_err());
        assert!(!backend.destroy_buffer(invalid_handle));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_buffer_slot_reuse() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create and destroy a buffer
        let data: Vec<u8> = vec![1, 2, 3, 4];
        let handle1 = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            &data,
        ).unwrap();
        backend.destroy_buffer(handle1);

        // Create another buffer - should reuse the slot
        let handle2 = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            &data,
        ).unwrap();

        // Handles should have same index but different generation
        assert_eq!(handle1.index(), handle2.index());
        assert_ne!(handle1.generation(), handle2.generation());

        // Old handle should not be valid
        assert!(!backend.is_buffer_valid(handle1));
        assert!(backend.is_buffer_valid(handle2));

        backend.destroy_buffer(handle2);
    }

    #[test]
    fn test_opengl_backend_implements_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OpenGLBackend>();
    }

    #[test]
    fn test_buffer_type_to_gl_target() {
        assert_eq!(OpenGLBackend::buffer_type_to_gl_target(BufferType::Vertex), gl::ARRAY_BUFFER);
        assert_eq!(OpenGLBackend::buffer_type_to_gl_target(BufferType::Index), gl::ELEMENT_ARRAY_BUFFER);
        assert_eq!(OpenGLBackend::buffer_type_to_gl_target(BufferType::Uniform), gl::UNIFORM_BUFFER);
    }

    #[test]
    fn test_buffer_usage_to_gl_usage() {
        assert_eq!(OpenGLBackend::buffer_usage_to_gl_usage(BufferUsage::Static), gl::STATIC_DRAW);
        assert_eq!(OpenGLBackend::buffer_usage_to_gl_usage(BufferUsage::Dynamic), gl::DYNAMIC_DRAW);
        assert_eq!(OpenGLBackend::buffer_usage_to_gl_usage(BufferUsage::Stream), gl::STREAM_DRAW);
    }

    #[test]
    fn test_blend_factor_to_gl() {
        assert_eq!(OpenGLBackend::blend_factor_to_gl(BlendFactor::Zero), gl::ZERO);
        assert_eq!(OpenGLBackend::blend_factor_to_gl(BlendFactor::One), gl::ONE);
        assert_eq!(OpenGLBackend::blend_factor_to_gl(BlendFactor::SrcAlpha), gl::SRC_ALPHA);
        assert_eq!(OpenGLBackend::blend_factor_to_gl(BlendFactor::OneMinusSrcAlpha), gl::ONE_MINUS_SRC_ALPHA);
    }

    // ============================================================================
    // Texture Tests
    // ============================================================================

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_lifecycle() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create a texture
        let pixels: Vec<u8> = vec![255; 256 * 256 * 4]; // 256x256 RGBA white texture
        let handle = backend.create_texture(
            256, 256,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        ).unwrap();

        // Verify it's valid
        assert!(backend.is_texture_valid(handle));
        assert_eq!(backend.texture_size(handle), Some((256, 256)));

        // Destroy it
        assert!(backend.destroy_texture(handle));
        assert!(!backend.is_texture_valid(handle));
        assert_eq!(backend.texture_size(handle), None);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_empty_data() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create a texture with no initial data (render target use case)
        let handle = backend.create_texture(
            512, 512,
            TextureFormat::RGBA8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
            &[],
        ).unwrap();

        assert!(backend.is_texture_valid(handle));
        assert_eq!(backend.texture_size(handle), Some((512, 512)));

        backend.destroy_texture(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_update() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create a texture
        let pixels: Vec<u8> = vec![0; 256 * 256 * 4];
        let handle = backend.create_texture(
            256, 256,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        ).unwrap();

        // Update a region
        let new_pixels: Vec<u8> = vec![255; 64 * 64 * 4]; // 64x64 white region
        backend.update_texture(handle, 0, 0, 64, 64, &new_pixels).unwrap();

        backend.destroy_texture(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_update_bounds_check() {
        let mut backend = OpenGLBackend::new().unwrap();

        let pixels: Vec<u8> = vec![0; 128 * 128 * 4];
        let handle = backend.create_texture(
            128, 128,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        ).unwrap();

        // Try to update a region that exceeds bounds
        let new_pixels: Vec<u8> = vec![255; 64 * 64 * 4];
        let result = backend.update_texture(handle, 100, 100, 64, 64, &new_pixels);
        assert!(result.is_err());

        backend.destroy_texture(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_bind() {
        let mut backend = OpenGLBackend::new().unwrap();

        let pixels: Vec<u8> = vec![255; 64 * 64 * 4];
        let handle = backend.create_texture(
            64, 64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        ).unwrap();

        // Bind to texture unit 0
        backend.bind_texture(handle, 0).unwrap();
        backend.unbind_texture(0);

        backend.destroy_texture(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_multiple_units() {
        let mut backend = OpenGLBackend::new().unwrap();

        let pixels1: Vec<u8> = vec![255; 64 * 64 * 4];
        let pixels2: Vec<u8> = vec![128; 64 * 64 * 4];

        let handle1 = backend.create_texture(
            64, 64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels1,
        ).unwrap();

        let handle2 = backend.create_texture(
            64, 64,
            TextureFormat::RGBA8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
            &pixels2,
        ).unwrap();

        // Bind to different units
        backend.bind_texture(handle1, 0).unwrap();
        backend.bind_texture(handle2, 1).unwrap();

        backend.unbind_texture(0);
        backend.unbind_texture(1);

        backend.destroy_texture(handle1);
        backend.destroy_texture(handle2);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_invalid_dimensions() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Zero width
        let result = backend.create_texture(
            0, 256,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &[],
        );
        assert!(result.is_err());

        // Zero height
        let result = backend.create_texture(
            256, 0,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &[],
        );
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_texture_slot_reuse() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create and destroy a texture
        let pixels: Vec<u8> = vec![255; 64 * 64 * 4];
        let handle1 = backend.create_texture(
            64, 64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        ).unwrap();
        backend.destroy_texture(handle1);

        // Create another texture - should reuse the slot
        let handle2 = backend.create_texture(
            64, 64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        ).unwrap();

        // Handles should have same index but different generation
        assert_eq!(handle1.index(), handle2.index());
        assert_ne!(handle1.generation(), handle2.generation());

        // Old handle should not be valid
        assert!(!backend.is_texture_valid(handle1));
        assert!(backend.is_texture_valid(handle2));

        backend.destroy_texture(handle2);
    }

    #[test]
    fn test_texture_format_to_gl() {
        let (internal, format, type_) = OpenGLBackend::texture_format_to_gl(TextureFormat::RGBA8);
        assert_eq!(internal, gl::RGBA8);
        assert_eq!(format, gl::RGBA);
        assert_eq!(type_, gl::UNSIGNED_BYTE);

        let (internal, format, type_) = OpenGLBackend::texture_format_to_gl(TextureFormat::RGB8);
        assert_eq!(internal, gl::RGB8);
        assert_eq!(format, gl::RGB);
        assert_eq!(type_, gl::UNSIGNED_BYTE);
    }

    #[test]
    fn test_texture_filter_to_gl() {
        assert_eq!(OpenGLBackend::texture_filter_to_gl(TextureFilter::Nearest), gl::NEAREST);
        assert_eq!(OpenGLBackend::texture_filter_to_gl(TextureFilter::Linear), gl::LINEAR);
    }

    #[test]
    fn test_texture_wrap_to_gl() {
        assert_eq!(OpenGLBackend::texture_wrap_to_gl(TextureWrap::Repeat), gl::REPEAT);
        assert_eq!(OpenGLBackend::texture_wrap_to_gl(TextureWrap::MirroredRepeat), gl::MIRRORED_REPEAT);
        assert_eq!(OpenGLBackend::texture_wrap_to_gl(TextureWrap::ClampToEdge), gl::CLAMP_TO_EDGE);
        assert_eq!(OpenGLBackend::texture_wrap_to_gl(TextureWrap::ClampToBorder), gl::CLAMP_TO_BORDER);
    }

    #[test]
    fn test_bytes_per_pixel() {
        assert_eq!(OpenGLBackend::bytes_per_pixel(TextureFormat::R8), 1);
        assert_eq!(OpenGLBackend::bytes_per_pixel(TextureFormat::RG8), 2);
        assert_eq!(OpenGLBackend::bytes_per_pixel(TextureFormat::RGB8), 3);
        assert_eq!(OpenGLBackend::bytes_per_pixel(TextureFormat::RGBA8), 4);
        assert_eq!(OpenGLBackend::bytes_per_pixel(TextureFormat::RGBA16F), 8);
        assert_eq!(OpenGLBackend::bytes_per_pixel(TextureFormat::RGBA32F), 16);
    }

    // ============================================================================
    // Shader Tests
    // ============================================================================

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_lifecycle() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Create a simple shader
        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() {
                gl_Position = vec4(position, 1.0);
            }
        "#;

        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() {
                FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let handle = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Verify it's valid
        assert!(backend.is_shader_valid(handle));

        // Destroy it
        assert!(backend.destroy_shader(handle));
        assert!(!backend.is_shader_valid(handle));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_empty_sources() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Empty vertex shader
        let result = backend.create_shader("", "void main() {}");
        assert!(result.is_err());

        // Empty fragment shader
        let result = backend.create_shader("void main() {}", "");
        assert!(result.is_err());

        // Both empty
        let result = backend.create_shader("", "");
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_compilation_error() {
        let mut backend = OpenGLBackend::new().unwrap();

        // Invalid vertex shader
        let vertex_src = "this is not valid GLSL code";
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;

        let result = backend.create_shader(vertex_src, fragment_src);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GoudError::ShaderCompilationFailed(_)));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_bind_unbind() {
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;

        let handle = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind shader
        backend.bind_shader(handle).unwrap();
        assert_eq!(backend.bound_shader, backend.shaders.get(&handle).map(|m| m.gl_id));

        // Unbind shader
        backend.unbind_shader();
        assert_eq!(backend.bound_shader, None);

        backend.destroy_shader(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_invalid_handle() {
        let mut backend = OpenGLBackend::new().unwrap();
        let invalid_handle = ShaderHandle::INVALID;

        assert!(!backend.is_shader_valid(invalid_handle));
        assert!(backend.bind_shader(invalid_handle).is_err());
        assert!(!backend.destroy_shader(invalid_handle));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_uniform_location() {
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            uniform vec4 color;
            out vec4 FragColor;
            void main() { FragColor = color; }
        "#;

        let handle = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Get uniform location
        let location = backend.get_uniform_location(handle, "color");
        assert!(location.is_some());
        assert!(location.unwrap() >= 0);

        // Non-existent uniform
        let location = backend.get_uniform_location(handle, "nonexistent");
        assert!(location.is_none());

        backend.destroy_shader(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_set_uniforms() {
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            uniform int intVal;
            uniform float floatVal;
            uniform vec2 vec2Val;
            uniform vec3 vec3Val;
            uniform vec4 vec4Val;
            uniform mat4 matVal;
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;

        let handle = backend.create_shader(vertex_src, fragment_src).unwrap();
        backend.bind_shader(handle).unwrap();

        // Set various uniform types
        if let Some(loc) = backend.get_uniform_location(handle, "intVal") {
            backend.set_uniform_int(loc, 42);
        }
        if let Some(loc) = backend.get_uniform_location(handle, "floatVal") {
            backend.set_uniform_float(loc, 3.14);
        }
        if let Some(loc) = backend.get_uniform_location(handle, "vec2Val") {
            backend.set_uniform_vec2(loc, 1.0, 2.0);
        }
        if let Some(loc) = backend.get_uniform_location(handle, "vec3Val") {
            backend.set_uniform_vec3(loc, 1.0, 2.0, 3.0);
        }
        if let Some(loc) = backend.get_uniform_location(handle, "vec4Val") {
            backend.set_uniform_vec4(loc, 1.0, 2.0, 3.0, 4.0);
        }
        if let Some(loc) = backend.get_uniform_location(handle, "matVal") {
            let identity: [f32; 16] = [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ];
            backend.set_uniform_mat4(loc, &identity);
        }

        backend.destroy_shader(handle);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_multiple_shaders() {
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;

        let fragment_src1 = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0, 0.0, 0.0, 1.0); }
        "#;

        let fragment_src2 = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(0.0, 1.0, 0.0, 1.0); }
        "#;

        let handle1 = backend.create_shader(vertex_src, fragment_src1).unwrap();
        let handle2 = backend.create_shader(vertex_src, fragment_src2).unwrap();

        assert!(backend.is_shader_valid(handle1));
        assert!(backend.is_shader_valid(handle2));
        assert_ne!(handle1, handle2);

        backend.destroy_shader(handle1);
        backend.destroy_shader(handle2);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_slot_reuse() {
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;

        // Create and destroy a shader
        let handle1 = backend.create_shader(vertex_src, fragment_src).unwrap();
        backend.destroy_shader(handle1);

        // Create another shader - should reuse the slot
        let handle2 = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Handles should have same index but different generation
        assert_eq!(handle1.index(), handle2.index());
        assert_ne!(handle1.generation(), handle2.generation());

        // Old handle should not be valid
        assert!(!backend.is_shader_valid(handle1));
        assert!(backend.is_shader_valid(handle2));

        backend.destroy_shader(handle2);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_shader_destroy_clears_bound_state() {
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;

        let handle = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind the shader
        backend.bind_shader(handle).unwrap();
        assert!(backend.bound_shader.is_some());

        // Destroy it
        backend.destroy_shader(handle);

        // Bound state should be cleared
        assert!(backend.bound_shader.is_none());
    }

    // ============================================================================
    // Draw Call Tests
    // ============================================================================

    #[test]
    fn test_topology_to_gl_conversion() {
        use crate::libs::graphics::backend::types::PrimitiveTopology;
        assert_eq!(OpenGLBackend::topology_to_gl(PrimitiveTopology::Points), gl::POINTS);
        assert_eq!(OpenGLBackend::topology_to_gl(PrimitiveTopology::Lines), gl::LINES);
        assert_eq!(OpenGLBackend::topology_to_gl(PrimitiveTopology::LineStrip), gl::LINE_STRIP);
        assert_eq!(OpenGLBackend::topology_to_gl(PrimitiveTopology::Triangles), gl::TRIANGLES);
        assert_eq!(OpenGLBackend::topology_to_gl(PrimitiveTopology::TriangleStrip), gl::TRIANGLE_STRIP);
        assert_eq!(OpenGLBackend::topology_to_gl(PrimitiveTopology::TriangleFan), gl::TRIANGLE_FAN);
    }

    #[test]
    fn test_attribute_type_to_gl_conversion() {
        use crate::libs::graphics::backend::types::VertexAttributeType;

        // Float types
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Float), gl::FLOAT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Float2), gl::FLOAT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Float3), gl::FLOAT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Float4), gl::FLOAT);

        // Int types
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Int), gl::INT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Int2), gl::INT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Int3), gl::INT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::Int4), gl::INT);

        // UInt types
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::UInt), gl::UNSIGNED_INT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::UInt2), gl::UNSIGNED_INT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::UInt3), gl::UNSIGNED_INT);
        assert_eq!(OpenGLBackend::attribute_type_to_gl_type(VertexAttributeType::UInt4), gl::UNSIGNED_INT);
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_arrays_without_shader_fails() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology};
        let mut backend = OpenGLBackend::new().unwrap();

        // Create and bind a vertex buffer
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
        ];
        let buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();
        backend.bind_buffer(buffer).unwrap();

        // Try to draw without binding shader
        let result = backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GoudError::InvalidState(_)));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_arrays_without_vertex_buffer_fails() {
        use crate::libs::graphics::backend::types::PrimitiveTopology;
        let mut backend = OpenGLBackend::new().unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0, 0.0, 0.0, 1.0); }
        "#;

        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();
        backend.bind_shader(shader).unwrap();

        // Try to draw without binding vertex buffer
        let result = backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GoudError::InvalidState(_)));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_arrays_basic() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Create vertex buffer
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
        ];
        let buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        // Create shader
        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0, 0.0, 0.0, 1.0); }
        "#;
        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind resources
        backend.bind_shader(shader).unwrap();
        backend.bind_buffer(buffer).unwrap();

        // Set up vertex attributes
        let layout = VertexLayout::new(12)
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false));
        backend.set_vertex_attributes(&layout);

        // Draw should succeed
        let result = backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_indexed_without_index_buffer_fails() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Create vertex buffer and shader
        let vertices: [f32; 12] = [
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.5, 0.5, 0.0,
            -0.5, 0.5, 0.0,
        ];
        let buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;
        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind shader and vertex buffer
        backend.bind_shader(shader).unwrap();
        backend.bind_buffer(buffer).unwrap();

        let layout = VertexLayout::new(12)
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false));
        backend.set_vertex_attributes(&layout);

        // Try to draw indexed without index buffer
        let result = backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GoudError::InvalidState(_)));
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_indexed_basic() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Create vertex buffer (quad)
        let vertices: [f32; 12] = [
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.5, 0.5, 0.0,
            -0.5, 0.5, 0.0,
        ];
        let vertex_buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        // Create index buffer (two triangles)
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = backend.create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            bytemuck::cast_slice(&indices)
        ).unwrap();

        // Create shader
        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;
        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind resources
        backend.bind_shader(shader).unwrap();
        backend.bind_buffer(vertex_buffer).unwrap();

        let layout = VertexLayout::new(12)
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false));
        backend.set_vertex_attributes(&layout);

        backend.bind_buffer(index_buffer).unwrap();

        // Draw should succeed
        let result = backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_indexed_u16() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Create vertex buffer
        let vertices: [f32; 12] = [
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.5, 0.5, 0.0,
            -0.5, 0.5, 0.0,
        ];
        let vertex_buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        // Create u16 index buffer
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = backend.create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            bytemuck::cast_slice(&indices)
        ).unwrap();

        // Create shader
        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() { gl_Position = vec4(position, 1.0); }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;
        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind resources
        backend.bind_shader(shader).unwrap();
        backend.bind_buffer(vertex_buffer).unwrap();

        let layout = VertexLayout::new(12)
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false));
        backend.set_vertex_attributes(&layout);

        backend.bind_buffer(index_buffer).unwrap();

        // Draw should succeed with u16 indices
        let result = backend.draw_indexed_u16(PrimitiveTopology::Triangles, 6, 0);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_arrays_instanced() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Skip if instancing not supported
        if !backend.info().capabilities.supports_instancing {
            return;
        }

        // Create vertex buffer
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
        ];
        let buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        // Create shader
        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() {
                vec3 offset = vec3(gl_InstanceID * 0.5, 0.0, 0.0);
                gl_Position = vec4(position + offset, 1.0);
            }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;
        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind resources
        backend.bind_shader(shader).unwrap();
        backend.bind_buffer(buffer).unwrap();

        let layout = VertexLayout::new(12)
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false));
        backend.set_vertex_attributes(&layout);

        // Draw 10 instances should succeed
        let result = backend.draw_arrays_instanced(PrimitiveTopology::Triangles, 0, 3, 10);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_draw_indexed_instanced() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, PrimitiveTopology, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Skip if instancing not supported
        if !backend.info().capabilities.supports_instancing {
            return;
        }

        // Create vertex buffer
        let vertices: [f32; 12] = [
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.5, 0.5, 0.0,
            -0.5, 0.5, 0.0,
        ];
        let vertex_buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        // Create index buffer
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = backend.create_buffer(
            BufferType::Index,
            BufferUsage::Static,
            bytemuck::cast_slice(&indices)
        ).unwrap();

        // Create shader
        let vertex_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            void main() {
                vec3 offset = vec3(gl_InstanceID * 0.3, 0.0, 0.0);
                gl_Position = vec4(position + offset, 1.0);
            }
        "#;
        let fragment_src = r#"
            #version 330 core
            out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
        "#;
        let shader = backend.create_shader(vertex_src, fragment_src).unwrap();

        // Bind resources
        backend.bind_shader(shader).unwrap();
        backend.bind_buffer(vertex_buffer).unwrap();

        let layout = VertexLayout::new(12)
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false));
        backend.set_vertex_attributes(&layout);

        backend.bind_buffer(index_buffer).unwrap();

        // Draw 5 instances should succeed
        let result = backend.draw_indexed_instanced(PrimitiveTopology::Triangles, 6, 0, 5);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires OpenGL context
    fn test_set_vertex_attributes_multiple() {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage, VertexAttribute, VertexAttributeType, VertexLayout};
        let mut backend = OpenGLBackend::new().unwrap();

        // Create vertex buffer with position and color
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 4],
        }

        unsafe impl bytemuck::Pod for Vertex {}
        unsafe impl bytemuck::Zeroable for Vertex {}

        let vertices = [
            Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0, 1.0] },
        ];

        let buffer = backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&vertices)
        ).unwrap();

        backend.bind_buffer(buffer).unwrap();

        // Set up layout with two attributes
        let layout = VertexLayout::new(28) // 3 floats + 4 floats = 7 * 4 bytes = 28 bytes
            .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false))
            .with_attribute(VertexAttribute::new(1, VertexAttributeType::Float4, 12, false));

        // Should not panic
        backend.set_vertex_attributes(&layout);
    }
}
