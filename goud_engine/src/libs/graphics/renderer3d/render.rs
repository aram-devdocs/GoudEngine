//! Frame rendering logic for [`Renderer3D`].
//!
//! This module contains the `render` method and the model-matrix helper.
//! It is a separate file to keep `core.rs` under 500 lines.

use cgmath::{perspective, Deg, Matrix, Matrix4};

use crate::libs::graphics::backend::{
    BlendFactor, CullFace, DepthFunc, FrontFace, PrimitiveTopology,
};

use super::core::Renderer3D;
use super::texture::TextureManagerTrait;
use super::types::MAX_LIGHTS;

impl Renderer3D {
    /// Render the scene — grid, objects, and overlays — using the current camera and light state.
    pub fn render(&mut self, texture_manager: Option<&dyn TextureManagerTrait>) {
        self.backend.enable_depth_test();
        self.backend.set_depth_func(DepthFunc::Less);
        self.backend.enable_culling();
        self.backend.set_cull_face(CullFace::Back);
        self.backend.set_front_face(FrontFace::Ccw);

        if self.skybox_config.enabled {
            self.backend.set_clear_color(
                self.skybox_config.color.x,
                self.skybox_config.color.y,
                self.skybox_config.color.z,
                self.skybox_config.color.w,
            );
        }
        self.backend.clear_depth();

        let aspect = self.window_width as f32 / self.window_height as f32;
        let projection: Matrix4<f32> = perspective(Deg(45.0), aspect, 0.1, 1000.0);
        let view = self.camera.view_matrix();
        let view_arr = mat4_to_array(&view);
        let proj_arr = mat4_to_array(&projection);

        // ----------------------------------------------------------------
        // Grid pass
        // ----------------------------------------------------------------
        if self.grid_config.enabled {
            let _ = self.backend.bind_shader(self.grid_shader_handle);

            self.backend
                .set_uniform_mat4(self.grid_uniforms.view, &view_arr);
            self.backend
                .set_uniform_mat4(self.grid_uniforms.projection, &proj_arr);
            self.backend.set_uniform_vec3(
                self.grid_uniforms.view_pos,
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            );
            self.backend
                .set_uniform_float(self.grid_uniforms.alpha, 0.4);
            self.backend.set_uniform_int(
                self.grid_uniforms.fog_enabled,
                i32::from(self.fog_config.enabled),
            );
            self.backend.set_uniform_vec3(
                self.grid_uniforms.fog_color,
                self.fog_config.color.x,
                self.fog_config.color.y,
                self.fog_config.color.z,
            );
            self.backend
                .set_uniform_float(self.grid_uniforms.fog_density, self.fog_config.density);

            self.backend.enable_blending();
            self.backend
                .set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
            self.backend.set_depth_mask(false);

            let _ = self.backend.bind_buffer(self.grid_buffer);
            self.backend.set_vertex_attributes(&self.grid_layout);
            let _ = self.backend.draw_arrays(
                PrimitiveTopology::Lines,
                0,
                self.grid_vertex_count as u32,
            );

            self.backend.set_line_width(3.0);
            let _ = self.backend.bind_buffer(self.axis_buffer);
            self.backend.set_vertex_attributes(&self.grid_layout);
            self.backend
                .set_uniform_float(self.grid_uniforms.alpha, 1.0);
            let _ = self.backend.draw_arrays(
                PrimitiveTopology::Lines,
                0,
                self.axis_vertex_count as u32,
            );
            self.backend.set_line_width(1.0);

            self.backend.set_depth_mask(true);
            self.backend.disable_blending();
            self.backend.unbind_shader();
        }

        // ----------------------------------------------------------------
        // Main objects pass
        // ----------------------------------------------------------------
        let _ = self.backend.bind_shader(self.shader_handle);

        self.backend.set_uniform_mat4(self.uniforms.view, &view_arr);
        self.backend
            .set_uniform_mat4(self.uniforms.projection, &proj_arr);
        self.backend.set_uniform_vec3(
            self.uniforms.view_pos,
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        );

        self.backend.set_uniform_int(
            self.uniforms.fog_enabled,
            i32::from(self.fog_config.enabled),
        );
        self.backend.set_uniform_vec3(
            self.uniforms.fog_color,
            self.fog_config.color.x,
            self.fog_config.color.y,
            self.fog_config.color.z,
        );
        self.backend
            .set_uniform_float(self.uniforms.fog_density, self.fog_config.density);

        let light_count = self.lights.len().min(MAX_LIGHTS) as i32;
        self.backend
            .set_uniform_int(self.uniforms.num_lights, light_count);

        for (i, (_, light)) in self.lights.iter().enumerate().take(MAX_LIGHTS) {
            let lu = &self.uniforms.lights[i];
            self.backend
                .set_uniform_int(lu.light_type, light.light_type as i32);
            self.backend.set_uniform_vec3(
                lu.position,
                light.position.x,
                light.position.y,
                light.position.z,
            );
            self.backend.set_uniform_vec3(
                lu.direction,
                light.direction.x,
                light.direction.y,
                light.direction.z,
            );
            self.backend
                .set_uniform_vec3(lu.color, light.color.x, light.color.y, light.color.z);
            self.backend
                .set_uniform_float(lu.intensity, light.intensity);
            self.backend.set_uniform_float(lu.range, light.range);
            self.backend
                .set_uniform_float(lu.spot_angle, light.spot_angle);
            self.backend
                .set_uniform_int(lu.enabled, i32::from(light.enabled));
        }

        self.backend.set_uniform_int(self.uniforms.texture1, 0);

        for obj in self.objects.values() {
            let model = Self::create_model_matrix(obj.position, obj.rotation, obj.scale);
            let model_arr = mat4_to_array(&model);
            self.backend
                .set_uniform_mat4(self.uniforms.model, &model_arr);

            if obj.texture_id > 0 {
                if let Some(tm) = texture_manager {
                    tm.bind_texture(obj.texture_id, 0);
                }
                self.backend.set_uniform_int(self.uniforms.use_texture, 1);
                self.backend
                    .set_uniform_vec4(self.uniforms.object_color, 1.0, 1.0, 1.0, 1.0);
            } else {
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
                self.backend
                    .set_uniform_vec4(self.uniforms.object_color, 0.8, 0.8, 0.8, 1.0);
            }

            let _ = self.backend.bind_buffer(obj.buffer);
            self.backend.set_vertex_attributes(&self.object_layout);
            let _ =
                self.backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, obj.vertex_count as u32);
        }

        self.backend.unbind_shader();
    }

    /// Build a TRS (translate-rotate-scale) model matrix from object components.
    pub(super) fn create_model_matrix(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Vector3<f32>,
        scale: cgmath::Vector3<f32>,
    ) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(position);
        let rot_x = Matrix4::from_angle_x(Deg(rotation.x));
        let rot_y = Matrix4::from_angle_y(Deg(rotation.y));
        let rot_z = Matrix4::from_angle_z(Deg(rotation.z));
        let rotation_matrix = rot_z * rot_y * rot_x;
        let scale_matrix = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        translation * rotation_matrix * scale_matrix
    }
}

// ============================================================================
// Helper
// ============================================================================

/// Convert a cgmath [`Matrix4`] to a column-major `[f32; 16]` array.
///
/// cgmath matrices are already column-major, which matches the backend expectation.
pub(super) fn mat4_to_array(m: &Matrix4<f32>) -> [f32; 16] {
    let ptr = m.as_ptr();
    let mut arr = [0.0f32; 16];
    // SAFETY: Matrix4<f32> is 16 contiguous f32 values in column-major order.
    // The source and destination are non-overlapping and properly sized.
    unsafe {
        std::ptr::copy_nonoverlapping(ptr, arr.as_mut_ptr(), 16);
    }
    arr
}
