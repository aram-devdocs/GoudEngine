//! DrawOps trait implementation for WgpuBackend.

use super::{DrawOps, DrawType, PrimitiveTopology, VertexBufferBinding, VertexLayout, WgpuBackend};
use crate::libs::error::GoudResult;

impl DrawOps for WgpuBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        self.set_vertex_attributes_impl(layout);
    }

    fn set_vertex_bindings(&mut self, bindings: &[VertexBufferBinding]) -> GoudResult<()> {
        self.current_vertex_bindings = bindings.to_vec();
        Ok(())
    }

    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        self.current_topology = topology;
        self.record_draw(DrawType::Arrays { first, count })
    }

    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.current_topology = topology;
        self.record_draw(DrawType::Indexed { count, offset })
    }

    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.current_topology = topology;
        self.record_draw(DrawType::IndexedU16 { count, offset })
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.current_topology = topology;
        self.record_draw(DrawType::ArraysInstanced {
            first,
            count,
            instances: instance_count,
        })
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.current_topology = topology;
        self.record_draw(DrawType::IndexedInstanced {
            count,
            offset,
            instances: instance_count,
        })
    }
}
