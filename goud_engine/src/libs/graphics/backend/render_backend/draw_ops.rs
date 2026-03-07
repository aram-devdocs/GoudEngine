//! Draw call and vertex array sub-trait for `RenderBackend`.

use crate::libs::error::GoudResult;
use crate::libs::graphics::backend::types::{PrimitiveTopology, VertexLayout};

/// Draw call and vertex attribute operations.
///
/// Configures vertex attribute pointers and issues draw commands
/// for array-based, indexed, and instanced rendering.
pub trait DrawOps {
    /// Sets up vertex attribute pointers for the currently bound vertex buffer.
    ///
    /// # Arguments
    /// * `layout` - Description of vertex attributes in the buffer
    ///
    /// # Note
    /// - The vertex buffer must be bound before calling this
    /// - This configures how the GPU interprets the vertex data
    /// - Enables all attributes in the layout
    fn set_vertex_attributes(&mut self, layout: &VertexLayout);

    /// Draws primitives using array-based vertex data.
    ///
    /// # Arguments
    /// * `topology` - Primitive type to draw (Triangles, Lines, Points, etc.)
    /// * `first` - Index of the first vertex to draw
    /// * `count` - Number of vertices to draw
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
    /// # Note
    /// Assumes indices are u32. For u16 indices, use `draw_indexed_u16`.
    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()>;

    /// Draws primitives using indexed vertex data with u16 indices.
    ///
    /// # Note
    /// Same as `draw_indexed` but for u16 index type (more memory efficient
    /// for small meshes).
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
    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()>;
}
