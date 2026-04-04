//! GPU shadow pre-pass recording for [`Renderer3D`].
//!
//! Extracted from `render/mod.rs` to keep each file under 500 lines.

use super::mat4_to_array;
use crate::libs::graphics::backend::PrimitiveTopology;
use crate::libs::graphics::renderer3d::core::Renderer3D;
use crate::libs::graphics::renderer3d::shadow::compute_light_space_matrix_with_skinned;
use cgmath::{Matrix4, Vector3};

impl Renderer3D {
    /// Records the GPU shadow pre-pass draw commands into the backend's
    /// shadow command list, which is replayed as a depth-only pass.
    ///
    /// Returns `(light_space_matrix_array, shadow_active)`.
    pub(in crate::libs::graphics::renderer3d) fn record_gpu_shadow_pre_pass(
        &mut self,
    ) -> ([f32; 16], bool) {
        let Some((lsm, _dir)) = compute_light_space_matrix_with_skinned(
            &self.objects,
            &self.lights,
            Some(&self.skinned_meshes),
        ) else {
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
            // Shadow frustum culling: project the bounding sphere center into
            // light clip space and reject if it is entirely outside the ortho
            // frustum (NDC range [-1,1] on each axis), accounting for the radius.
            let world_center = obj.position + obj.bounds.center;
            let max_scale = obj.scale.x.max(obj.scale.y).max(obj.scale.z);
            let world_radius = obj.bounds.radius * max_scale;
            if !sphere_in_light_frustum(&lsm, world_center, world_radius) {
                continue;
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

        // Also render skinned meshes to the shadow map using cached model matrices.
        for sm in self.skinned_meshes.values() {
            // Use the cached model matrix (recomputed at the top of render() when dirty).
            let a = &sm.cached_model_matrix;
            let cols: [[f32; 4]; 4] = [
                [a[0], a[1], a[2], a[3]],
                [a[4], a[5], a[6], a[7]],
                [a[8], a[9], a[10], a[11]],
                [a[12], a[13], a[14], a[15]],
            ];
            let model: Matrix4<f32> = cols.into();
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

/// Returns `true` if a world-space bounding sphere is at least partially
/// inside the orthographic light frustum encoded by `lsm`.
///
/// Projects the sphere center into clip space and checks whether the sphere
/// (expanded by its radius projected into clip space) overlaps the NDC cube
/// `[-1, 1]` on each axis.
fn sphere_in_light_frustum(lsm: &Matrix4<f32>, center: Vector3<f32>, radius: f32) -> bool {
    let clip = *lsm * center.extend(1.0);
    // For orthographic projection w is always 1, but guard against edge cases.
    let w = clip.w.abs().max(f32::EPSILON);
    let ndc_x = clip.x / w;
    let ndc_y = clip.y / w;
    let ndc_z = clip.z / w;

    // Approximate the clip-space radius by scaling by the largest axis scale
    // of the light-space-matrix (conservative upper bound).
    let sx = (lsm.x.x * lsm.x.x + lsm.x.y * lsm.x.y + lsm.x.z * lsm.x.z).sqrt();
    let sy = (lsm.y.x * lsm.y.x + lsm.y.y * lsm.y.y + lsm.y.z * lsm.y.z).sqrt();
    let sz = (lsm.z.x * lsm.z.x + lsm.z.y * lsm.z.y + lsm.z.z * lsm.z.z).sqrt();
    let max_scale = sx.max(sy).max(sz);
    let clip_radius = radius * max_scale / w;

    // Reject if the sphere is entirely outside any face of the NDC cube.
    if ndc_x - clip_radius > 1.0 || ndc_x + clip_radius < -1.0 {
        return false;
    }
    if ndc_y - clip_radius > 1.0 || ndc_y + clip_radius < -1.0 {
        return false;
    }
    if ndc_z - clip_radius > 1.0 || ndc_z + clip_radius < -1.0 {
        return false;
    }
    true
}
