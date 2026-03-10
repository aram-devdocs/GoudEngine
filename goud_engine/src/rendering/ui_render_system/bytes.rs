use crate::rendering::sprite_batch::types::SpriteVertex;

pub(super) fn vertex_slice_as_bytes(vertices: &[SpriteVertex]) -> &[u8] {
    // SAFETY: SpriteVertex is #[repr(C)] POD data uploaded directly to GPU buffers.
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast(), std::mem::size_of_val(vertices)) }
}

pub(super) fn index_slice_as_bytes(indices: &[u32]) -> &[u8] {
    // SAFETY: u32 index buffers can be safely reinterpreted as raw bytes for upload.
    unsafe { std::slice::from_raw_parts(indices.as_ptr().cast(), std::mem::size_of_val(indices)) }
}
