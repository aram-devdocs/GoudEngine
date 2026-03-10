//! Buffer operations: create, update, destroy, bind, unbind.

use super::{super::types::BufferUsage, BufferHandle, BufferType, WgpuBackend, WgpuBufferMeta};
use crate::libs::error::{GoudError, GoudResult};

impl WgpuBackend {
    pub(super) fn create_buffer_impl(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        let wgpu_usage = match buffer_type {
            BufferType::Vertex => wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            BufferType::Index => wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            BufferType::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };

        let buffer = if data.is_empty() {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 64,
                usage: wgpu_usage,
                mapped_at_creation: false,
            })
        } else {
            wgpu::util::DeviceExt::create_buffer_init(
                &self.device,
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: data,
                    usage: wgpu_usage,
                },
            )
        };

        let handle = self.buffer_allocator.allocate();
        self.buffers.insert(
            handle,
            WgpuBufferMeta {
                buffer,
                buffer_type,
                size: if data.is_empty() { 64 } else { data.len() },
            },
        );

        let _ = usage; // Usage hints don't apply to wgpu (managed automatically)
        Ok(handle)
    }

    pub(super) fn update_buffer_impl(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        let meta = self.buffers.get(&handle).ok_or(GoudError::InvalidHandle)?;
        if offset + data.len() > meta.size {
            return Err(GoudError::InvalidState(format!(
                "Buffer update out of bounds: {} + {} > {}",
                offset,
                data.len(),
                meta.size
            )));
        }
        self.queue.write_buffer(&meta.buffer, offset as u64, data);
        Ok(())
    }

    pub(super) fn destroy_buffer_impl(&mut self, handle: BufferHandle) -> bool {
        if self.buffers.remove(&handle).is_some() {
            self.buffer_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    pub(super) fn is_buffer_valid_impl(&self, handle: BufferHandle) -> bool {
        self.buffers.contains_key(&handle)
    }

    pub(super) fn buffer_size_impl(&self, handle: BufferHandle) -> Option<usize> {
        self.buffers.get(&handle).map(|m| m.size)
    }

    pub(super) fn bind_buffer_impl(&mut self, handle: BufferHandle) -> GoudResult<()> {
        let meta = self.buffers.get(&handle).ok_or(GoudError::InvalidHandle)?;
        match meta.buffer_type {
            BufferType::Vertex => self.bound_vertex_buffer = Some(handle),
            BufferType::Index => self.bound_index_buffer = Some(handle),
            BufferType::Uniform => {}
        }
        Ok(())
    }

    pub(super) fn unbind_buffer_impl(&mut self, buffer_type: BufferType) {
        match buffer_type {
            BufferType::Vertex => self.bound_vertex_buffer = None,
            BufferType::Index => self.bound_index_buffer = None,
            BufferType::Uniform => {}
        }
    }
}
