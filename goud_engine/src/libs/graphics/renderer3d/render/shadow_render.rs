//! GPU shadow pre-pass recording for [`Renderer3D`].
//!
//! Extracted from `render/mod.rs` to keep each file under 500 lines.

use super::mat4_to_array;
use crate::libs::graphics::backend::PrimitiveTopology;
use crate::libs::graphics::renderer3d::core::Renderer3D;
use crate::libs::graphics::renderer3d::shadow::compute_light_space_matrix;
use cgmath::Matrix4;

impl Renderer3D {
    /// Records the GPU shadow pre-pass draw commands into the backend's
    /// shadow command list, which is replayed as a depth-only pass.
    ///
    /// Returns `(light_space_matrix_array, shadow_active)`.
    pub(in crate::libs::graphics::renderer3d) fn record_gpu_shadow_pre_pass(
        &mut self,
    ) -> ([f32; 16], bool) {
        let Some((lsm, _dir)) = compute_light_space_matrix(&self.objects, &self.lights) else {
            return (mat4_to_array(&Matrix4::from_scale(1.0)), false);
        };
        let lsm_arr = mat4_to_array(&lsm);

        self.backend
            .ensure_shadow_resources(self.config.shadows.map_size);
        self.backend.begin_shadow_recording();

        if self
            .backend
            .bind_shader(self.depth_only_shader_handle)
            .is_err()
        {
            log::warn!("Failed to bind depth-only shader, skipping shadow pass");
            self.backend.end_shadow_recording();
            return (mat4_to_array(&Matrix4::from_scale(1.0)), false);
        }

        let scene_obj_filter_shadow = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.objects);

        for (&id, obj) in &self.objects {
            if obj.vertices.is_empty() {
                continue;
            }
            if let Some(filter) = scene_obj_filter_shadow {
                if !filter.contains(&id) {
                    continue;
                }
            }
            let model = Self::create_model_matrix(obj.position, obj.rotation, obj.scale);
            let mvp = lsm * model;
            let mvp_arr = mat4_to_array(&mvp);
            self.backend
                .set_uniform_mat4(self.depth_only_uniforms.mvp, &mvp_arr);
            let _ = self.backend.bind_buffer(obj.buffer);
            self.backend.set_vertex_attributes(&self.depth_only_layout);
            let _ =
                self.backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, obj.vertex_count as u32);
        }

        // Also render skinned meshes to the shadow map.
        for sm in self.skinned_meshes.values() {
            let model = Self::create_model_matrix(sm.position, sm.rotation, sm.scale);
            let mvp = lsm * model;
            let mvp_arr = mat4_to_array(&mvp);
            self.backend
                .set_uniform_mat4(self.depth_only_uniforms.mvp, &mvp_arr);
            let _ = self.backend.bind_buffer(sm.buffer);
            self.backend.set_vertex_attributes(&self.depth_only_layout);
            let _ =
                self.backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, sm.vertex_count as u32);
        }

        self.backend.end_shadow_recording();
        (lsm_arr, true)
    }
}
