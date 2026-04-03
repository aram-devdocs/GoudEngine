//! Drop implementation for [`Renderer3D`] -- cleans up all GPU resources.

use super::Renderer3D;

impl Drop for Renderer3D {
    fn drop(&mut self) {
        for obj in self.objects.values() {
            self.backend.destroy_buffer(obj.buffer);
        }
        for mesh in self.instanced_meshes.values() {
            self.backend.destroy_buffer(mesh.mesh_buffer);
            self.backend.destroy_buffer(mesh.instance_buffer);
        }
        for e in self.particle_emitters.values() {
            self.backend.destroy_buffer(e.instance_buffer);
        }
        for m in self.skinned_meshes.values() {
            self.backend.destroy_buffer(m.buffer);
        }
        self.backend.destroy_buffer(self.grid_buffer);
        self.backend.destroy_buffer(self.axis_buffer);
        self.backend.destroy_buffer(self.particle_quad_buffer);
        self.backend.destroy_buffer(self.postprocess_quad_buffer);
        for tex in [self.postprocess_texture.take(), self.shadow_texture.take()]
            .into_iter()
            .flatten()
        {
            self.backend.destroy_texture(tex);
        }
        if self.backend.is_buffer_valid(self.debug_draw_buffer) {
            self.backend.destroy_buffer(self.debug_draw_buffer);
        }
        if let Some(buf) = self.bone_storage_buffer {
            self.backend.destroy_buffer(buf);
        }
        if let Some(buf) = self.instanced_bone_storage_buffer {
            self.backend.destroy_buffer(buf);
        }
        for (buf, _) in &self.instanced_skinned_instance_buffers {
            self.backend.destroy_buffer(*buf);
        }
        self.instanced_skinned_instance_buffers.clear();
        if let Some(buf) = self.static_batch_buffer {
            self.backend.destroy_buffer(buf);
        }
        for &sh in &[
            self.shader_handle,
            self.instanced_shader_handle,
            self.grid_shader_handle,
            self.postprocess_shader_handle,
            self.skinned_shader_handle,
            self.instanced_skinned_shader_handle,
            self.depth_only_shader_handle,
        ] {
            self.backend.destroy_shader(sh);
        }
    }
}
