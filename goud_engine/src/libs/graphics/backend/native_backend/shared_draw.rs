use crate::libs::error::GoudResult;

use super::SharedNativeRenderBackend;
use crate::libs::graphics::backend::render_backend::DrawOps;
use crate::libs::graphics::backend::types::{PrimitiveTopology, VertexLayout};

impl DrawOps for SharedNativeRenderBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        self.lock().set_vertex_attributes(layout);
    }

    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        self.lock().draw_arrays(topology, first, count)
    }

    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.lock().draw_indexed(topology, count, offset)
    }

    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.lock().draw_indexed_u16(topology, count, offset)
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.lock()
            .draw_arrays_instanced(topology, first, count, instance_count)
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.lock()
            .draw_indexed_instanced(topology, count, offset, instance_count)
    }
}
