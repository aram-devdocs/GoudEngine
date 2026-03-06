//! Uniform buffer writing, draw command recording, and texture snapshot helpers.

use super::{
    init::UNIFORM_BUFFER_SIZE, DrawCommand, DrawType, PipelineKey, PrimitiveTopology,
    TextureHandle, WgpuBackend,
};
use crate::core::error::{GoudError, GoudResult};

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
            vertex_stride: cmd.vertex_layout.stride,
            vertex_attrs: cmd
                .vertex_layout
                .attributes
                .iter()
                .map(|a| (a.location, a.attribute_type as u8, a.offset, a.normalized))
                .collect(),
        }
    }

    /// Records a draw command using the current render state.
    pub(super) fn record_draw(&mut self, draw_type: DrawType) -> GoudResult<()> {
        let shader = self
            .bound_shader
            .ok_or(GoudError::InvalidState("No shader bound".into()))?;
        let vb = self
            .bound_vertex_buffer
            .ok_or(GoudError::InvalidState("No vertex buffer bound".into()))?;
        let layout = self
            .current_layout
            .clone()
            .ok_or(GoudError::InvalidState("No vertex layout set".into()))?;

        let uniform_snapshot = self
            .shaders
            .get(&shader)
            .map(|s| s.uniform_staging.clone())
            .unwrap_or_default();

        self.draw_commands.push(DrawCommand {
            shader,
            vertex_buffer: vb,
            index_buffer: self.bound_index_buffer,
            vertex_layout: layout,
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

    /// Writes bytes into the staging buffer of the currently bound shader.
    pub(super) fn write_uniform(&mut self, location: i32, data: &[u8]) {
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
