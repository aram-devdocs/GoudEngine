//! `RenderBackend` trait definition.
//!
//! This file is a **documented exception** to the 500-line limit. The trait must
//! be contiguous so that readers can see the full API contract in one place.
//! Do NOT split this file further.

use crate::core::error::GoudResult;

use super::blend::{BlendFactor, CullFace};
use super::capabilities::{BackendCapabilities, BackendInfo};
use super::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureFilter,
    TextureFormat, TextureHandle, TextureWrap, VertexLayout,
};

/// Main render backend trait abstracting graphics operations.
///
/// This trait provides a platform-agnostic interface for rendering operations,
/// allowing the engine to support multiple graphics APIs without changing
/// higher-level rendering code.
///
/// # Safety
///
/// Implementations must ensure:
/// - All GPU handles remain valid for their lifetime
/// - Operations on destroyed handles return errors gracefully
/// - Thread safety is maintained per API requirements
///
/// # Object Safety
///
/// This trait is intentionally NOT object-safe to allow for:
/// - Associated types for handle wrappers
/// - Generic methods for efficient implementations
/// - Zero-cost abstractions where possible
pub trait RenderBackend: Send + Sync {
    // ============================================================================
    // Lifecycle & Information
    // ============================================================================

    /// Returns information about this backend implementation.
    fn info(&self) -> &BackendInfo;

    /// Returns the capabilities of this backend.
    fn capabilities(&self) -> &BackendCapabilities {
        &self.info().capabilities
    }

    // ============================================================================
    // Frame Management
    // ============================================================================

    /// Begins a new frame. Called once per frame before any rendering.
    ///
    /// This may perform backend-specific setup like resetting state or
    /// beginning command recording (Vulkan, Metal).
    fn begin_frame(&mut self) -> GoudResult<()>;

    /// Ends the current frame. Called once per frame after all rendering.
    ///
    /// This may submit command buffers or perform cleanup.
    fn end_frame(&mut self) -> GoudResult<()>;

    // ============================================================================
    // Clear Operations
    // ============================================================================

    /// Sets the clear color for subsequent clear operations.
    ///
    /// # Arguments
    /// * `r` - Red component (0.0 to 1.0)
    /// * `g` - Green component (0.0 to 1.0)
    /// * `b` - Blue component (0.0 to 1.0)
    /// * `a` - Alpha component (0.0 to 1.0)
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32);

    /// Clears the color buffer using the current clear color.
    fn clear_color(&mut self);

    /// Clears the depth buffer.
    fn clear_depth(&mut self);

    /// Clears both color and depth buffers.
    ///
    /// Default implementation calls both clear methods, but backends
    /// can override for more efficient combined operations.
    fn clear(&mut self) {
        self.clear_color();
        self.clear_depth();
    }

    // ============================================================================
    // State Management
    // ============================================================================

    /// Sets the viewport rectangle.
    ///
    /// # Arguments
    /// * `x` - X coordinate of lower-left corner
    /// * `y` - Y coordinate of lower-left corner
    /// * `width` - Viewport width in pixels
    /// * `height` - Viewport height in pixels
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32);

    /// Enables depth testing.
    fn enable_depth_test(&mut self);

    /// Disables depth testing.
    fn disable_depth_test(&mut self);

    /// Enables alpha blending.
    fn enable_blending(&mut self);

    /// Disables alpha blending.
    fn disable_blending(&mut self);

    /// Sets the blend function.
    ///
    /// # Arguments
    /// * `src` - Source blend factor
    /// * `dst` - Destination blend factor
    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor);

    /// Enables face culling.
    fn enable_culling(&mut self);

    /// Disables face culling.
    fn disable_culling(&mut self);

    /// Sets which faces to cull.
    fn set_cull_face(&mut self, face: CullFace);

    /// Sets the depth comparison function.
    fn set_depth_func(&mut self, func: super::types::DepthFunc);

    /// Sets the front face winding order.
    fn set_front_face(&mut self, face: super::types::FrontFace);

    /// Enables or disables writing to the depth buffer.
    fn set_depth_mask(&mut self, enabled: bool);

    /// Sets the line width for line primitives.
    fn set_line_width(&mut self, width: f32);

    // ============================================================================
    // Buffer Operations
    // ============================================================================

    /// Creates a GPU buffer with the specified type, usage, and initial data.
    ///
    /// # Arguments
    /// * `buffer_type` - The type of buffer (Vertex, Index, Uniform)
    /// * `usage` - Usage hint for optimization (Static, Dynamic, Stream)
    /// * `data` - Initial data to upload (may be empty)
    ///
    /// # Returns
    /// A handle to the created buffer, or an error if creation failed.
    ///
    /// # Example
    /// ```rust,ignore
    /// let vertices: &[f32] = &[/* vertex data */];
    /// let handle = backend.create_buffer(
    ///     BufferType::Vertex,
    ///     BufferUsage::Static,
    ///     bytemuck::cast_slice(vertices)
    /// )?;
    /// ```
    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle>;

    /// Updates the contents of an existing buffer.
    ///
    /// # Arguments
    /// * `handle` - Handle to the buffer to update
    /// * `offset` - Byte offset into the buffer
    /// * `data` - New data to upload
    ///
    /// # Errors
    /// Returns an error if:
    /// - Handle is invalid or buffer was destroyed
    /// - Offset + data size exceeds buffer size
    /// - Buffer usage is Static (use Dynamic for frequent updates)
    ///
    /// # Example
    /// ```rust,ignore
    /// backend.update_buffer(handle, 0, bytemuck::cast_slice(&new_data))?;
    /// ```
    fn update_buffer(&mut self, handle: BufferHandle, offset: usize, data: &[u8])
        -> GoudResult<()>;

    /// Destroys a buffer and frees GPU memory.
    ///
    /// # Arguments
    /// * `handle` - Handle to the buffer to destroy
    ///
    /// # Returns
    /// `true` if the buffer was destroyed, `false` if the handle was invalid.
    ///
    /// # Safety
    /// The handle becomes invalid after this call. Using it in subsequent
    /// operations will return errors.
    ///
    /// # Example
    /// ```rust,ignore
    /// if backend.destroy_buffer(handle) {
    ///     println!("Buffer destroyed successfully");
    /// }
    /// ```
    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool;

    /// Checks if a buffer handle is valid and refers to an existing buffer.
    ///
    /// # Arguments
    /// * `handle` - Handle to check
    ///
    /// # Returns
    /// `true` if the buffer exists, `false` otherwise.
    fn is_buffer_valid(&self, handle: BufferHandle) -> bool;

    /// Returns the size in bytes of a buffer.
    ///
    /// # Arguments
    /// * `handle` - Handle to the buffer
    ///
    /// # Returns
    /// The buffer size in bytes, or `None` if the handle is invalid.
    fn buffer_size(&self, handle: BufferHandle) -> Option<usize>;

    /// Binds a buffer for use in subsequent draw calls.
    ///
    /// # Arguments
    /// * `handle` - Handle to the buffer to bind
    ///
    /// # Errors
    /// Returns an error if the handle is invalid.
    ///
    /// # Note
    /// The buffer type determines which binding point is used:
    /// - Vertex buffers bind to GL_ARRAY_BUFFER
    /// - Index buffers bind to GL_ELEMENT_ARRAY_BUFFER
    /// - Uniform buffers bind to GL_UNIFORM_BUFFER
    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()>;

    /// Unbinds the currently bound buffer of the specified type.
    ///
    /// # Arguments
    /// * `buffer_type` - The type of buffer to unbind
    fn unbind_buffer(&mut self, buffer_type: BufferType);

    // ============================================================================
    // Texture Operations
    // ============================================================================

    /// Creates a GPU texture with the specified dimensions, format, and initial data.
    ///
    /// # Arguments
    /// * `width` - Texture width in pixels (must be > 0)
    /// * `height` - Texture height in pixels (must be > 0)
    /// * `format` - Pixel format (RGBA8, RGB8, etc.)
    /// * `filter` - Minification/magnification filtering mode
    /// * `wrap` - Texture coordinate wrapping mode
    /// * `data` - Initial pixel data (may be empty for render targets)
    ///
    /// # Returns
    /// A handle to the created texture, or an error if creation failed.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pixels: &[u8] = &[/* RGBA data */];
    /// let handle = backend.create_texture(
    ///     256, 256,
    ///     TextureFormat::RGBA8,
    ///     TextureFilter::Linear,
    ///     TextureWrap::Repeat,
    ///     pixels
    /// )?;
    /// ```
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle>;

    /// Updates a region of an existing texture with new pixel data.
    ///
    /// # Arguments
    /// * `handle` - Handle to the texture to update
    /// * `x` - X offset in pixels (0 = left edge)
    /// * `y` - Y offset in pixels (0 = bottom edge)
    /// * `width` - Width of the update region in pixels
    /// * `height` - Height of the update region in pixels
    /// * `data` - New pixel data for the region
    ///
    /// # Errors
    /// Returns an error if:
    /// - Handle is invalid or texture was destroyed
    /// - Region exceeds texture bounds
    /// - Data size doesn't match region size and format
    ///
    /// # Example
    /// ```rust,ignore
    /// backend.update_texture(handle, 0, 0, 128, 128, &new_pixels)?;
    /// ```
    fn update_texture(
        &mut self,
        handle: TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()>;

    /// Destroys a texture and frees GPU memory.
    ///
    /// # Arguments
    /// * `handle` - Handle to the texture to destroy
    ///
    /// # Returns
    /// `true` if the texture was destroyed, `false` if the handle was invalid.
    ///
    /// # Safety
    /// The handle becomes invalid after this call. Using it in subsequent
    /// operations will return errors.
    ///
    /// # Example
    /// ```rust,ignore
    /// if backend.destroy_texture(handle) {
    ///     println!("Texture destroyed successfully");
    /// }
    /// ```
    fn destroy_texture(&mut self, handle: TextureHandle) -> bool;

    /// Checks if a texture handle is valid and refers to an existing texture.
    ///
    /// # Arguments
    /// * `handle` - Handle to check
    ///
    /// # Returns
    /// `true` if the texture exists, `false` otherwise.
    fn is_texture_valid(&self, handle: TextureHandle) -> bool;

    /// Returns the dimensions of a texture.
    ///
    /// # Arguments
    /// * `handle` - Handle to the texture
    ///
    /// # Returns
    /// The texture dimensions (width, height), or `None` if the handle is invalid.
    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)>;

    /// Binds a texture to a texture unit for use in subsequent draw calls.
    ///
    /// # Arguments
    /// * `handle` - Handle to the texture to bind
    /// * `unit` - Texture unit index (0-based, typically 0-15 supported)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Handle is invalid
    /// - Texture unit exceeds maximum supported units
    ///
    /// # Note
    /// Multiple textures can be bound simultaneously to different units
    /// for multi-texturing effects.
    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()>;

    /// Unbinds any texture from the specified texture unit.
    ///
    /// # Arguments
    /// * `unit` - Texture unit index to unbind
    fn unbind_texture(&mut self, unit: u32);

    // ============================================================================
    // Shader Operations
    // ============================================================================

    /// Compiles and links a shader program from vertex and fragment shader sources.
    ///
    /// # Arguments
    /// * `vertex_src` - GLSL vertex shader source code
    /// * `fragment_src` - GLSL fragment shader source code
    ///
    /// # Returns
    /// A handle to the compiled shader program, or an error if compilation/linking failed.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Vertex shader compilation fails
    /// - Fragment shader compilation fails
    /// - Program linking fails
    /// - Either source is empty
    ///
    /// # Example
    /// ```rust,ignore
    /// let vertex = r#"
    ///     #version 330 core
    ///     layout(location = 0) in vec3 position;
    ///     void main() {
    ///         gl_Position = vec4(position, 1.0);
    ///     }
    /// "#;
    /// let fragment = r#"
    ///     #version 330 core
    ///     out vec4 FragColor;
    ///     void main() {
    ///         FragColor = vec4(1.0, 0.0, 0.0, 1.0);
    ///     }
    /// "#;
    /// let handle = backend.create_shader(vertex, fragment)?;
    /// ```
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle>;

    /// Destroys a shader program and frees GPU memory.
    ///
    /// # Arguments
    /// * `handle` - Handle to the shader program to destroy
    ///
    /// # Returns
    /// `true` if the shader was destroyed, `false` if the handle was invalid.
    ///
    /// # Safety
    /// The handle becomes invalid after this call. Using it in subsequent
    /// operations will return errors.
    ///
    /// # Example
    /// ```rust,ignore
    /// if backend.destroy_shader(handle) {
    ///     println!("Shader destroyed successfully");
    /// }
    /// ```
    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool;

    /// Checks if a shader handle is valid and refers to an existing shader program.
    ///
    /// # Arguments
    /// * `handle` - Handle to check
    ///
    /// # Returns
    /// `true` if the shader exists, `false` otherwise.
    fn is_shader_valid(&self, handle: ShaderHandle) -> bool;

    /// Binds a shader program for use in subsequent draw calls.
    ///
    /// # Arguments
    /// * `handle` - Handle to the shader program to bind
    ///
    /// # Errors
    /// Returns an error if the handle is invalid.
    ///
    /// # Note
    /// Only one shader program can be active at a time.
    /// Binding a new shader replaces the previous one.
    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()>;

    /// Unbinds the currently bound shader program.
    fn unbind_shader(&mut self);

    /// Gets the location of a uniform variable in a shader program.
    ///
    /// # Arguments
    /// * `handle` - Handle to the shader program
    /// * `name` - Name of the uniform variable
    ///
    /// # Returns
    /// The uniform location, or `None` if:
    /// - Handle is invalid
    /// - Uniform doesn't exist
    /// - Uniform was optimized out by the compiler
    ///
    /// # Note
    /// The shader must be bound before setting uniform values.
    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32>;

    /// Sets a uniform integer value.
    ///
    /// # Arguments
    /// * `location` - Uniform location from `get_uniform_location`
    /// * `value` - Integer value to set
    ///
    /// # Note
    /// The shader containing this uniform must be currently bound.
    fn set_uniform_int(&mut self, location: i32, value: i32);

    /// Sets a uniform float value.
    ///
    /// # Arguments
    /// * `location` - Uniform location from `get_uniform_location`
    /// * `value` - Float value to set
    ///
    /// # Note
    /// The shader containing this uniform must be currently bound.
    fn set_uniform_float(&mut self, location: i32, value: f32);

    /// Sets a uniform vec2 value.
    ///
    /// # Arguments
    /// * `location` - Uniform location
    /// * `x`, `y` - Vector components
    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32);

    /// Sets a uniform vec3 value.
    ///
    /// # Arguments
    /// * `location` - Uniform location
    /// * `x`, `y`, `z` - Vector components
    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32);

    /// Sets a uniform vec4 value.
    ///
    /// # Arguments
    /// * `location` - Uniform location
    /// * `x`, `y`, `z`, `w` - Vector components
    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32);

    /// Sets a uniform mat4 value.
    ///
    /// # Arguments
    /// * `location` - Uniform location
    /// * `matrix` - 16 floats in column-major order
    ///
    /// # Note
    /// Matrix data must be in column-major order (OpenGL standard).
    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]);

    // ============================================================================
    // Vertex Array Setup
    // ============================================================================

    /// Sets up vertex attribute pointers for the currently bound vertex buffer.
    ///
    /// # Arguments
    /// * `layout` - Description of vertex attributes in the buffer
    ///
    /// # Note
    /// - The vertex buffer must be bound before calling this
    /// - This configures how the GPU interprets the vertex data
    /// - Enables all attributes in the layout
    ///
    /// # Example
    /// ```rust,ignore
    /// let layout = VertexLayout::new(20)
    ///     .with_attribute(VertexAttribute::new(0, VertexAttributeType::Float3, 0, false))
    ///     .with_attribute(VertexAttribute::new(1, VertexAttributeType::Float2, 12, false));
    /// backend.bind_buffer(vertex_buffer)?;
    /// backend.set_vertex_attributes(&layout);
    /// ```
    fn set_vertex_attributes(&mut self, layout: &VertexLayout);

    // ============================================================================
    // Draw Calls
    // ============================================================================

    /// Draws primitives using array-based vertex data.
    ///
    /// # Arguments
    /// * `topology` - Primitive type to draw (Triangles, Lines, Points, etc.)
    /// * `first` - Index of the first vertex to draw
    /// * `count` - Number of vertices to draw
    ///
    /// # Errors
    /// Returns an error if:
    /// - No vertex buffer is bound
    /// - No shader is bound
    /// - Vertex attributes are not configured
    ///
    /// # Example
    /// ```rust,ignore
    /// backend.bind_shader(shader)?;
    /// backend.bind_buffer(vertex_buffer)?;
    /// backend.set_vertex_attributes(&layout);
    /// backend.draw_arrays(PrimitiveTopology::Triangles, 0, 3)?; // Draw one triangle
    /// ```
    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()>;

    /// Draws primitives using indexed vertex data.
    ///
    /// # Arguments
    /// * `topology` - Primitive type to draw
    /// * `count` - Number of indices to draw
    /// * `offset` - Byte offset into the index buffer
    ///
    /// # Errors
    /// Returns an error if:
    /// - No vertex buffer is bound
    /// - No index buffer is bound
    /// - No shader is bound
    /// - Vertex attributes are not configured
    ///
    /// # Note
    /// Assumes indices are u32. For u16 indices, use draw_indexed_u16.
    ///
    /// # Example
    /// ```rust,ignore
    /// backend.bind_shader(shader)?;
    /// backend.bind_buffer(vertex_buffer)?;
    /// backend.set_vertex_attributes(&layout);
    /// backend.bind_buffer(index_buffer)?;
    /// backend.draw_indexed(PrimitiveTopology::Triangles, 6, 0)?; // Draw 2 triangles
    /// ```
    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()>;

    /// Draws primitives using indexed vertex data with u16 indices.
    ///
    /// # Arguments
    /// * `topology` - Primitive type to draw
    /// * `count` - Number of indices to draw
    /// * `offset` - Byte offset into the index buffer
    ///
    /// # Note
    /// Same as draw_indexed but for u16 index type (more memory efficient for small meshes).
    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()>;

    /// Draws multiple instances of primitives using array-based vertex data.
    ///
    /// # Arguments
    /// * `topology` - Primitive type to draw
    /// * `first` - Index of the first vertex
    /// * `count` - Number of vertices per instance
    /// * `instance_count` - Number of instances to draw
    ///
    /// # Errors
    /// Returns an error if:
    /// - Instanced rendering is not supported (check capabilities)
    /// - No vertex buffer is bound
    /// - No shader is bound
    ///
    /// # Example
    /// ```rust,ignore
    /// // Draw 100 copies of a quad using instanced rendering
    /// backend.draw_arrays_instanced(PrimitiveTopology::Triangles, 0, 6, 100)?;
    /// ```
    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()>;

    /// Draws multiple instances of primitives using indexed vertex data.
    ///
    /// # Arguments
    /// * `topology` - Primitive type to draw
    /// * `count` - Number of indices per instance
    /// * `offset` - Byte offset into the index buffer
    /// * `instance_count` - Number of instances to draw
    ///
    /// # Note
    /// Combines indexed drawing with instancing for maximum efficiency.
    /// Requires backend to support instancing (check capabilities).
    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()>;
}
