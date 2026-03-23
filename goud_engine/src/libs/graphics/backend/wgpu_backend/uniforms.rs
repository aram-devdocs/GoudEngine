//! Uniform buffer writing, draw command recording, and texture snapshot helpers.

use super::{
    init::UNIFORM_BUFFER_SIZE, DrawCommand, DrawType, PipelineKey, PrimitiveTopology,
    TextureHandle, WgpuBackend,
};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::{VertexBufferBinding, VertexStepMode};

impl WgpuBackend {
    /// Snapshots currently bound textures as `(unit, handle)` pairs.
    pub(super) fn snapshot_textures(&self) -> Vec<(u32, TextureHandle)> {
        self.bound_textures
            .iter()
            .enumerate()
            .filter_map(|(i, t)| t.map(|h| (i as u32, h)))
            .collect()
    }

    /// Builds the pipeline cache key for a given draw command.
    pub(super) fn make_pipeline_key(&self, cmd: &DrawCommand) -> PipelineKey {
        PipelineKey {
            shader: cmd.shader,
            topology: cmd.topology as u8,
            depth_test: cmd.depth_test,
            depth_write: cmd.depth_write,
            depth_func: cmd.depth_func as u8,
            blend_enabled: cmd.blend_enabled,
            blend_src: cmd.blend_src as u8,
            blend_dst: cmd.blend_dst as u8,
            cull_enabled: cmd.cull_enabled,
            cull_face: cmd.cull_face as u8,
            front_face: cmd.front_face as u8,
            vertex_buffers: cmd
                .vertex_bindings
                .iter()
                .map(|binding| {
                    (
                        binding.layout.stride,
                        binding.step_mode as u8,
                        binding
                            .layout
                            .attributes
                            .iter()
                            .map(|a| (a.location, a.attribute_type as u8, a.offset, a.normalized))
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    /// Records a draw command using the current render state.
    pub(super) fn record_draw(&mut self, draw_type: DrawType) -> GoudResult<()> {
        let shader = self
            .bound_shader
            .ok_or(GoudError::InvalidState("No shader bound".into()))?;
        let vertex_bindings = if !self.current_vertex_bindings.is_empty() {
            self.current_vertex_bindings.clone()
        } else {
            vec![VertexBufferBinding {
                buffer: self
                    .bound_vertex_buffer
                    .ok_or(GoudError::InvalidState("No vertex buffer bound".into()))?,
                layout: self
                    .current_layout
                    .clone()
                    .ok_or(GoudError::InvalidState("No vertex layout set".into()))?,
                step_mode: VertexStepMode::Vertex,
            }]
        };

        let uniform_snapshot = self
            .shaders
            .get(&shader)
            .map(|s| s.uniform_staging.clone())
            .unwrap_or_default();

        self.draw_commands.push(DrawCommand {
            shader,
            index_buffer: self.bound_index_buffer,
            vertex_bindings,
            bound_textures: self.snapshot_textures(),
            topology: PrimitiveTopology::Triangles,
            depth_test: self.depth_test_enabled,
            depth_write: self.depth_write_enabled,
            depth_func: self.depth_func,
            blend_enabled: self.blend_enabled,
            blend_src: self.blend_src,
            blend_dst: self.blend_dst,
            cull_enabled: self.cull_enabled,
            cull_face: self.cull_face,
            front_face: self.front_face_state,
            uniform_snapshot,
            draw_type,
        });
        Ok(())
    }

    /// Uploads per-draw-command uniform data into aligned slots and returns
    /// the byte offset for each command.  Grows the GPU buffer if needed.
    pub(super) fn upload_per_draw_uniforms(&mut self) -> Vec<u32> {
        let align = self.device.limits().min_uniform_buffer_offset_alignment as usize;
        let slot_size = {
            let snap = self
                .draw_commands
                .iter()
                .map(|c| c.uniform_snapshot.len())
                .max()
                .unwrap_or(256);
            (snap + align - 1) & !(align - 1)
        };

        let total_needed = self.draw_commands.len() * slot_size;
        let cmd_offsets: Vec<u32> = (0..self.draw_commands.len())
            .map(|i| (i * slot_size) as u32)
            .collect();

        // Grow uniform buffers up-front before any writes.
        for cmd in &self.draw_commands {
            if let Some(meta) = self.shaders.get_mut(&cmd.shader) {
                if total_needed > meta.uniform_buffer.size() as usize {
                    let new_size = total_needed.next_power_of_two().max(slot_size);
                    meta.uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("uniforms"),
                        size: new_size as u64,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    meta.uniform_bind_group =
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.uniform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &meta.uniform_buffer,
                                    offset: 0,
                                    size: std::num::NonZeroU64::new(slot_size as u64),
                                }),
                            }],
                        });
                }
            }
        }

        // Write all uniform snapshots into the (now correctly-sized) buffer.
        for (i, cmd) in self.draw_commands.iter().enumerate() {
            let offset = cmd_offsets[i] as u64;
            if let Some(meta) = self.shaders.get(&cmd.shader) {
                self.queue
                    .write_buffer(&meta.uniform_buffer, offset, &cmd.uniform_snapshot);
            }
        }

        cmd_offsets
    }

    /// Writes bytes into the staging buffer of the currently bound shader.
    pub(super) fn write_uniform(&mut self, location: i32, data: &[u8]) {
        if location < 0 {
            return;
        }

        let offset = (location as usize) * 4;
        if let Some(shader_handle) = self.bound_shader {
            if let Some(meta) = self.shaders.get_mut(&shader_handle) {
                let end = (offset + data.len()).min(UNIFORM_BUFFER_SIZE);
                if offset < end {
                    meta.uniform_staging[offset..end].copy_from_slice(&data[..end - offset]);
                }
            }
        }
    }
}
