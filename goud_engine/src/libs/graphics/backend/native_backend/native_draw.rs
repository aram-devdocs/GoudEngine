use crate::libs::error::GoudResult;

use super::NativeRenderBackend;
use crate::libs::graphics::backend::render_backend::DrawOps;
use crate::libs::graphics::backend::types::{PrimitiveTopology, VertexLayout};

impl DrawOps for NativeRenderBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_vertex_attributes(layout),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.set_vertex_attributes(layout),
        }
    }

    fn draw_arrays(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.draw_arrays(topology, first, count),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.draw_arrays(topology, first, count),
        }
    }

    fn draw_indexed(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.draw_indexed(topology, count, offset),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.draw_indexed(topology, count, offset),
        }
    }

    fn draw_indexed_u16(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.draw_indexed_u16(topology, count, offset),
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => backend.draw_indexed_u16(topology, count, offset),
        }
    }

    fn draw_arrays_instanced(
        &mut self,
        topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.draw_arrays_instanced(topology, first, count, instance_count)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => {
                backend.draw_arrays_instanced(topology, first, count, instance_count)
            }
        }
    }

    fn draw_indexed_instanced(
        &mut self,
        topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => {
                backend.draw_indexed_instanced(topology, count, offset, instance_count)
            }
            #[cfg(all(feature = "native", feature = "wgpu-backend"))]
            Self::Wgpu(backend) => {
                backend.draw_indexed_instanced(topology, count, offset, instance_count)
            }
        }
    }
}
