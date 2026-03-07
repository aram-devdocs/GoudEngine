//! Buffer operations sub-trait for `RenderBackend`.

use crate::libs::error::GoudResult;
use crate::libs::graphics::backend::types::{BufferHandle, BufferType, BufferUsage};

/// GPU buffer management operations.
///
/// Handles creation, updating, binding, and destruction of vertex,
/// index, and uniform buffers.
pub trait BufferOps {
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
    fn update_buffer(&mut self, handle: BufferHandle, offset: usize, data: &[u8])
        -> GoudResult<()>;

    /// Destroys a buffer and frees GPU memory.
    ///
    /// # Returns
    /// `true` if the buffer was destroyed, `false` if the handle was invalid.
    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool;

    /// Checks if a buffer handle is valid and refers to an existing buffer.
    fn is_buffer_valid(&self, handle: BufferHandle) -> bool;

    /// Returns the size in bytes of a buffer, or `None` if the handle is invalid.
    fn buffer_size(&self, handle: BufferHandle) -> Option<usize>;

    /// Binds a buffer for use in subsequent draw calls.
    ///
    /// # Note
    /// The buffer type determines which binding point is used:
    /// - Vertex buffers bind to GL_ARRAY_BUFFER
    /// - Index buffers bind to GL_ELEMENT_ARRAY_BUFFER
    /// - Uniform buffers bind to GL_UNIFORM_BUFFER
    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()>;

    /// Unbinds the currently bound buffer of the specified type.
    fn unbind_buffer(&mut self, buffer_type: BufferType);
}
