use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::{BufferType, BufferUsage};
use crate::rendering::sprite_batch::types::SpriteVertex;

use super::TextBatch;

impl TextBatch {
    /// Uploads vertex and index data to the GPU.
    ///
    /// When the existing buffer is too small for the current frame's data, the
    /// old buffer is destroyed and a new, correctly sized one is created.
    pub(super) fn upload_buffers(&mut self, backend: &mut dyn RenderBackend) -> Result<(), String> {
        let vert_bytes = vertex_slice_as_bytes(&self.vertices);
        self.vertex_buffer =
            Some(upload_or_grow(backend, self.vertex_buffer, BufferType::Vertex, vert_bytes, "VBO")?);

        let idx_bytes = index_slice_as_bytes(&self.indices);
        self.index_buffer =
            Some(upload_or_grow(backend, self.index_buffer, BufferType::Index, idx_bytes, "IBO")?);

        Ok(())
    }
}

/// Uploads `data` into an existing buffer when it fits, otherwise destroys the
/// old buffer and creates a new one.
fn upload_or_grow(
    backend: &mut dyn RenderBackend,
    existing: Option<crate::libs::graphics::backend::types::BufferHandle>,
    buf_type: BufferType,
    data: &[u8],
    label: &str,
) -> Result<crate::libs::graphics::backend::types::BufferHandle, String> {
    if let Some(buf) = existing {
        if backend.buffer_size(buf).unwrap_or(0) >= data.len() {
            backend
                .update_buffer(buf, 0, data)
                .map_err(|e| format!("text {label} update failed: {e}"))?;
            return Ok(buf);
        }
        backend.destroy_buffer(buf);
    }
    backend
        .create_buffer(buf_type, BufferUsage::Dynamic, data)
        .map_err(|e| format!("text {label} create failed: {e}"))
}

/// Reinterprets a `&[SpriteVertex]` as `&[u8]` for GPU upload.
fn vertex_slice_as_bytes(vertices: &[SpriteVertex]) -> &[u8] {
    // SAFETY: SpriteVertex is #[repr(C)] with no padding invariants.
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast(), std::mem::size_of_val(vertices)) }
}

/// Reinterprets a `&[u32]` as `&[u8]` for GPU upload.
fn index_slice_as_bytes(indices: &[u32]) -> &[u8] {
    // SAFETY: u32 has no alignment/validity invariants beyond its size.
    unsafe { std::slice::from_raw_parts(indices.as_ptr().cast(), std::mem::size_of_val(indices)) }
}
