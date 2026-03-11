use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::{BufferType, BufferUsage};
use crate::rendering::sprite_batch::types::SpriteVertex;

use super::TextBatch;

impl TextBatch {
    /// Uploads vertex and index data to the GPU.
    pub(super) fn upload_buffers(&mut self, backend: &mut dyn RenderBackend) -> Result<(), String> {
        let vert_bytes = vertex_slice_as_bytes(&self.vertices);

        match self.vertex_buffer {
            Some(buf) => {
                backend
                    .update_buffer(buf, 0, vert_bytes)
                    .map_err(|e| format!("text VBO update failed: {e}"))?;
            }
            None => {
                let buf = backend
                    .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, vert_bytes)
                    .map_err(|e| format!("text VBO create failed: {e}"))?;
                self.vertex_buffer = Some(buf);
            }
        }

        let idx_bytes = index_slice_as_bytes(&self.indices);

        match self.index_buffer {
            Some(buf) => {
                backend
                    .update_buffer(buf, 0, idx_bytes)
                    .map_err(|e| format!("text IBO update failed: {e}"))?;
            }
            None => {
                let buf = backend
                    .create_buffer(BufferType::Index, BufferUsage::Dynamic, idx_bytes)
                    .map_err(|e| format!("text IBO create failed: {e}"))?;
                self.index_buffer = Some(buf);
            }
        }

        Ok(())
    }
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
