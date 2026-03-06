//! OpenGL buffer create/update/destroy/bind operations.

use super::{
    backend::OpenGLBackend,
    conversions::{buffer_type_to_gl_target, buffer_usage_to_gl_usage},
    BufferMetadata,
};
use crate::core::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::{BufferHandle, BufferType, BufferUsage};

/// Creates a GPU buffer with the specified type, usage, and initial data.
pub(super) fn create_buffer(
    backend: &mut OpenGLBackend,
    buffer_type: BufferType,
    usage: BufferUsage,
    data: &[u8],
) -> GoudResult<BufferHandle> {
    let mut gl_id: u32 = 0;
    unsafe {
        gl::GenBuffers(1, &mut gl_id);
        if gl_id == 0 {
            return Err(GoudError::BufferCreationFailed(
                "Failed to generate OpenGL buffer".to_string(),
            ));
        }
    }

    let target = buffer_type_to_gl_target(buffer_type);
    let gl_usage = buffer_usage_to_gl_usage(usage);

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
            return Err(GoudError::BufferCreationFailed(format!(
                "OpenGL error during buffer creation: 0x{:X}",
                error
            )));
        }

        // Unbind
        gl::BindBuffer(target, 0);
    }

    // Allocate handle and store metadata
    let handle = backend.buffer_allocator.allocate();
    let metadata = BufferMetadata {
        gl_id,
        buffer_type,
        usage,
        size: data.len(),
    };
    backend.buffers.insert(handle, metadata);

    // Update bound buffer tracking
    set_bound_buffer(backend, buffer_type, None);

    Ok(handle)
}

/// Updates the contents of an existing buffer.
pub(super) fn update_buffer(
    backend: &mut OpenGLBackend,
    handle: BufferHandle,
    offset: usize,
    data: &[u8],
) -> GoudResult<()> {
    let metadata = backend
        .buffers
        .get(&handle)
        .ok_or(GoudError::InvalidHandle)?;

    if offset + data.len() > metadata.size {
        return Err(GoudError::InvalidState(format!(
            "Buffer update out of bounds: offset {} + size {} > buffer size {}",
            offset,
            data.len(),
            metadata.size
        )));
    }

    let target = buffer_type_to_gl_target(metadata.buffer_type);
    let gl_id = metadata.gl_id;
    let buf_type = metadata.buffer_type;

    unsafe {
        gl::BindBuffer(target, gl_id);
        gl::BufferSubData(
            target,
            offset as isize,
            data.len() as isize,
            data.as_ptr() as *const std::ffi::c_void,
        );

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            return Err(GoudError::InternalError(format!(
                "OpenGL error during buffer update: 0x{:X}",
                error
            )));
        }

        gl::BindBuffer(target, 0);
    }

    set_bound_buffer(backend, buf_type, None);

    Ok(())
}

/// Destroys a buffer and frees GPU memory.
pub(super) fn destroy_buffer(backend: &mut OpenGLBackend, handle: BufferHandle) -> bool {
    if let Some(metadata) = backend.buffers.remove(&handle) {
        unsafe {
            gl::DeleteBuffers(1, &metadata.gl_id);
        }

        // Clear bound buffer tracking if this was bound
        if get_bound_buffer(backend, metadata.buffer_type) == Some(metadata.gl_id) {
            set_bound_buffer(backend, metadata.buffer_type, None);
        }

        backend.buffer_allocator.deallocate(handle);
        true
    } else {
        false
    }
}

/// Checks if a buffer handle is valid.
pub(super) fn is_buffer_valid(backend: &OpenGLBackend, handle: BufferHandle) -> bool {
    backend.buffer_allocator.is_alive(handle) && backend.buffers.contains_key(&handle)
}

/// Returns the size in bytes of a buffer.
pub(super) fn buffer_size(backend: &OpenGLBackend, handle: BufferHandle) -> Option<usize> {
    backend.buffers.get(&handle).map(|m| m.size)
}

/// Binds a buffer for use in subsequent draw calls.
pub(super) fn bind_buffer(backend: &mut OpenGLBackend, handle: BufferHandle) -> GoudResult<()> {
    let metadata = backend
        .buffers
        .get(&handle)
        .ok_or(GoudError::InvalidHandle)?;

    let target = buffer_type_to_gl_target(metadata.buffer_type);
    let gl_id = metadata.gl_id;
    let buf_type = metadata.buffer_type;

    unsafe {
        gl::BindBuffer(target, gl_id);
    }

    set_bound_buffer(backend, buf_type, Some(gl_id));

    Ok(())
}

/// Unbinds the currently bound buffer of the specified type.
pub(super) fn unbind_buffer(backend: &mut OpenGLBackend, buffer_type: BufferType) {
    let target = buffer_type_to_gl_target(buffer_type);

    unsafe {
        gl::BindBuffer(target, 0);
    }

    set_bound_buffer(backend, buffer_type, None);
}

// ============================================================================
// Internal bound-buffer tracking helpers
// ============================================================================

pub(super) fn get_bound_buffer(backend: &OpenGLBackend, buffer_type: BufferType) -> Option<u32> {
    match buffer_type {
        BufferType::Vertex => backend.bound_vertex_buffer,
        BufferType::Index => backend.bound_index_buffer,
        BufferType::Uniform => backend.bound_uniform_buffer,
    }
}

pub(super) fn set_bound_buffer(
    backend: &mut OpenGLBackend,
    buffer_type: BufferType,
    gl_id: Option<u32>,
) {
    match buffer_type {
        BufferType::Vertex => backend.bound_vertex_buffer = gl_id,
        BufferType::Index => backend.bound_index_buffer = gl_id,
        BufferType::Uniform => backend.bound_uniform_buffer = gl_id,
    }
}
