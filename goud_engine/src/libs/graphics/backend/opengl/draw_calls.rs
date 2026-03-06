//! OpenGL draw call dispatch and vertex attribute setup.

use super::{backend::OpenGLBackend, conversions};
use crate::core::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::{PrimitiveTopology, VertexLayout};

/// Sets up vertex attribute pointers for the currently bound vertex buffer.
pub(super) fn set_vertex_attributes(layout: &VertexLayout) {
    // SAFETY: GL context is active; a vertex buffer is expected to be bound; attribute offsets are valid within the stride.
    unsafe {
        for attr in &layout.attributes {
            gl::EnableVertexAttribArray(attr.location);

            let gl_type = conversions::attribute_type_to_gl_type(attr.attribute_type);
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

/// Draws primitives using array-based vertex data.
pub(super) fn draw_arrays(
    backend: &OpenGLBackend,
    topology: PrimitiveTopology,
    first: u32,
    count: u32,
) -> GoudResult<()> {
    // Validate state
    if backend.bound_shader.is_none() {
        return Err(GoudError::InvalidState(
            "No shader bound for draw call".to_string(),
        ));
    }
    if backend.bound_vertex_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No vertex buffer bound for draw call".to_string(),
        ));
    }

    let gl_topology = conversions::topology_to_gl(topology);

    // SAFETY: GL context is active; shader and vertex buffer are bound (validated above); parameters are in-range.
    unsafe {
        gl::DrawArrays(gl_topology, first as i32, count as i32);
    }

    Ok(())
}

/// Draws primitives using indexed vertex data (u32 indices).
pub(super) fn draw_indexed(
    backend: &OpenGLBackend,
    topology: PrimitiveTopology,
    count: u32,
    offset: usize,
) -> GoudResult<()> {
    // Validate state
    if backend.bound_shader.is_none() {
        return Err(GoudError::InvalidState(
            "No shader bound for draw call".to_string(),
        ));
    }
    if backend.bound_vertex_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No vertex buffer bound for draw call".to_string(),
        ));
    }
    if backend.bound_index_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No index buffer bound for draw call".to_string(),
        ));
    }

    let gl_topology = conversions::topology_to_gl(topology);

    // SAFETY: GL context is active; shader, vertex buffer, and index buffer are bound (validated above); offset is a byte offset into the bound index buffer.
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

/// Draws primitives using indexed vertex data (u16 indices).
pub(super) fn draw_indexed_u16(
    backend: &OpenGLBackend,
    topology: PrimitiveTopology,
    count: u32,
    offset: usize,
) -> GoudResult<()> {
    // Validate state
    if backend.bound_shader.is_none() {
        return Err(GoudError::InvalidState(
            "No shader bound for draw call".to_string(),
        ));
    }
    if backend.bound_vertex_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No vertex buffer bound for draw call".to_string(),
        ));
    }
    if backend.bound_index_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No index buffer bound for draw call".to_string(),
        ));
    }

    let gl_topology = conversions::topology_to_gl(topology);

    // SAFETY: GL context is active; shader, vertex buffer, and index buffer are bound (validated above); offset is a byte offset into the bound u16 index buffer.
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

/// Draws multiple instances of primitives using array-based vertex data.
pub(super) fn draw_arrays_instanced(
    backend: &OpenGLBackend,
    topology: PrimitiveTopology,
    first: u32,
    count: u32,
    instance_count: u32,
) -> GoudResult<()> {
    // Check capability
    if !backend.info.capabilities.supports_instancing {
        return Err(GoudError::BackendNotSupported(
            "Instanced rendering not supported".to_string(),
        ));
    }

    // Validate state
    if backend.bound_shader.is_none() {
        return Err(GoudError::InvalidState(
            "No shader bound for draw call".to_string(),
        ));
    }
    if backend.bound_vertex_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No vertex buffer bound for draw call".to_string(),
        ));
    }

    let gl_topology = conversions::topology_to_gl(topology);

    // SAFETY: GL context is active; shader and vertex buffer are bound (validated above); instancing is confirmed supported.
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

/// Draws multiple instances of primitives using indexed vertex data.
pub(super) fn draw_indexed_instanced(
    backend: &OpenGLBackend,
    topology: PrimitiveTopology,
    count: u32,
    offset: usize,
    instance_count: u32,
) -> GoudResult<()> {
    // Check capability
    if !backend.info.capabilities.supports_instancing {
        return Err(GoudError::BackendNotSupported(
            "Instanced rendering not supported".to_string(),
        ));
    }

    // Validate state
    if backend.bound_shader.is_none() {
        return Err(GoudError::InvalidState(
            "No shader bound for draw call".to_string(),
        ));
    }
    if backend.bound_vertex_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No vertex buffer bound for draw call".to_string(),
        ));
    }
    if backend.bound_index_buffer.is_none() {
        return Err(GoudError::InvalidState(
            "No index buffer bound for draw call".to_string(),
        ));
    }

    let gl_topology = conversions::topology_to_gl(topology);

    // SAFETY: GL context is active; shader, vertex buffer, and index buffer are bound (validated above); instancing is confirmed supported.
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
