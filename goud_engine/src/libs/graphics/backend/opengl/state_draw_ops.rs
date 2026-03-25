//! DrawOps trait implementation for OpenGLBackend — forwarded to draw_calls module.

use super::{
    super::{DrawOps, PrimitiveTopology, VertexLayout},
    backend::OpenGLBackend,
};
use crate::libs::error::GoudResult;
use crate::libs::graphics::backend::types::VertexBufferBinding;

impl DrawOps for OpenGLBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        super::draw_calls::set_vertex_attributes_cached(self, layout)
    }

    fn set_vertex_bindings(&mut self, bindings: &[VertexBufferBinding]) -> GoudResult<()> {
        super::draw_calls::set_vertex_bindings_cached(self, bindings)
    }

    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        super::draw_calls::draw_arrays(self, topology, first, count)
    }

    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        super::draw_calls::draw_indexed(self, topology, count, offset)
    }

    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        super::draw_calls::draw_indexed_u16(self, topology, count, offset)
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        super::draw_calls::draw_arrays_instanced(self, topology, first, count, instance_count)
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        super::draw_calls::draw_indexed_instanced(self, topology, count, offset, instance_count)
    }
}
