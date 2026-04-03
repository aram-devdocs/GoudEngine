//! Instanced mesh and particle rendering pass for [`Renderer3D`].

use super::core::Renderer3D;
use super::render::mat4_to_array;
use super::texture::TextureManagerTrait;
use super::types::FogConfig;
use crate::libs::graphics::backend::{
    types::TextureHandle, BlendFactor, PrimitiveTopology, VertexBufferBinding,
};

impl Renderer3D {
    /// Render instanced meshes and particle emitters.
    pub(super) fn render_instanced_and_particles(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadows_enabled: bool,
        eff_fog: &FogConfig,
        filtered_lights: &[super::types::Light],
        texture_manager: Option<&dyn TextureManagerTrait>,
    ) {
        if self.instanced_meshes.is_empty() && self.particle_emitters.is_empty() {
            return;
        }

        let _ = self.backend.bind_shader(self.instanced_shader_handle);
        let instanced_uniforms = self.instanced_uniforms.clone();
        self.apply_main_uniforms(
            view_arr,
            proj_arr,
            shadow_matrix,
            shadows_enabled,
            &instanced_uniforms,
            eff_fog,
            filtered_lights,
        );
        for mesh in self.instanced_meshes.values() {
            if mesh.texture_id > 0 {
                if let Some(tm) = texture_manager {
                    tm.bind_texture(mesh.texture_id, 0);
                } else {
                    let texture_handle = TextureHandle::new(mesh.texture_id, 1);
                    let _ = self.backend.bind_texture(texture_handle, 0);
                }
                self.backend
                    .set_uniform_int(self.instanced_uniforms.use_texture, 1);
            } else {
                self.backend
                    .set_uniform_int(self.instanced_uniforms.use_texture, 0);
            }
            self.backend
                .set_uniform_vec4(self.instanced_uniforms.object_color, 1.0, 1.0, 1.0, 1.0);
            let bindings = [
                VertexBufferBinding::per_vertex(mesh.mesh_buffer, self.object_layout.clone()),
                VertexBufferBinding::per_instance(
                    mesh.instance_buffer,
                    self.instance_layout.clone(),
                ),
            ];
            let _ = self.backend.set_vertex_bindings(&bindings);
            let _ = self.backend.draw_arrays_instanced(
                PrimitiveTopology::Triangles,
                0,
                mesh.vertex_count,
                mesh.instances.len() as u32,
            );
            self.stats.draw_calls += 1;
            self.stats.instanced_draw_calls += 1;
            self.stats.active_instances += mesh.instances.len() as u32;
        }
        if !self.particle_emitters.is_empty() {
            self.backend.enable_blending();
            self.backend
                .set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        }
        for emitter in self.particle_emitters.values() {
            if emitter.particles.is_empty() {
                continue;
            }
            if emitter.config.texture_id > 0 {
                if let Some(tm) = texture_manager {
                    tm.bind_texture(emitter.config.texture_id, 0);
                } else {
                    let texture_handle = TextureHandle::new(emitter.config.texture_id, 1);
                    let _ = self.backend.bind_texture(texture_handle, 0);
                }
                self.backend
                    .set_uniform_int(self.instanced_uniforms.use_texture, 1);
            } else {
                self.backend
                    .set_uniform_int(self.instanced_uniforms.use_texture, 0);
            }
            self.backend
                .set_uniform_vec4(self.instanced_uniforms.object_color, 1.0, 1.0, 1.0, 1.0);
            let bindings = [
                VertexBufferBinding::per_vertex(
                    self.particle_quad_buffer,
                    self.object_layout.clone(),
                ),
                VertexBufferBinding::per_instance(
                    emitter.instance_buffer,
                    self.instance_layout.clone(),
                ),
            ];
            let _ = self.backend.set_vertex_bindings(&bindings);
            let _ = self.backend.draw_arrays_instanced(
                PrimitiveTopology::Triangles,
                0,
                self.particle_quad_vertex_count,
                emitter.particles.len() as u32,
            );
            self.stats.draw_calls += 1;
            self.stats.instanced_draw_calls += 1;
            self.stats.particle_draw_calls += 1;
            self.stats.active_particles += emitter.particles.len() as u32;
        }
        if !self.particle_emitters.is_empty() {
            self.backend.disable_blending();
        }
        self.backend.unbind_shader();
    }

    /// Render debug draw lines.
    pub(super) fn render_debug_draw(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        eff_fog: &FogConfig,
    ) {
        if self.debug_draw_vertex_count <= 0 {
            return;
        }
        let _ = self.backend.bind_shader(self.grid_shader_handle);
        self.backend
            .set_uniform_mat4(self.grid_uniforms.view, view_arr);
        self.backend
            .set_uniform_mat4(self.grid_uniforms.projection, proj_arr);
        self.backend.set_uniform_vec3(
            self.grid_uniforms.view_pos,
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        );
        self.backend
            .set_uniform_float(self.grid_uniforms.alpha, 1.0);
        self.backend
            .set_uniform_int(self.grid_uniforms.fog_enabled, i32::from(eff_fog.enabled));
        self.backend.set_uniform_vec3(
            self.grid_uniforms.fog_color,
            eff_fog.color.x,
            eff_fog.color.y,
            eff_fog.color.z,
        );
        self.backend
            .set_uniform_float(self.grid_uniforms.fog_density, eff_fog.density());
        self.backend
            .set_uniform_int(self.grid_uniforms.fog_mode, eff_fog.mode_int());
        self.backend
            .set_uniform_float(self.grid_uniforms.fog_start, eff_fog.start());
        self.backend
            .set_uniform_float(self.grid_uniforms.fog_end, eff_fog.end());
        self.backend.enable_blending();
        self.backend
            .set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
        self.backend.set_depth_mask(false);
        let _ = self.backend.bind_buffer(self.debug_draw_buffer);
        self.backend.set_vertex_attributes(&self.grid_layout);
        let _ = self.backend.draw_arrays(
            PrimitiveTopology::Lines,
            0,
            self.debug_draw_vertex_count as u32,
        );
        self.backend.set_depth_mask(true);
        self.backend.disable_blending();
        self.backend.unbind_shader();
    }

    /// Render the static batch buffer groups.
    pub(super) fn render_static_batch(
        &mut self,
        texture_manager: Option<&dyn TextureManagerTrait>,
    ) {
        // Identity model matrix: vertices are already world-space.
        let identity = mat4_to_array(&cgmath::Matrix4::from_scale(1.0f32));
        self.backend
            .set_uniform_mat4(self.uniforms.model, &identity);

        // Temporarily take groups to avoid borrow conflict with &mut self.
        let groups = std::mem::take(&mut self.static_batch_groups);
        let batch_buf = match self.static_batch_buffer {
            Some(buf) => buf,
            None => {
                log::error!("static_batch_buffer is None despite has_static_batch being true");
                return;
            }
        };
        let mut batch_last_tex = u32::MAX;

        for group in &groups {
            if group.texture_id > 0 {
                if group.texture_id != batch_last_tex {
                    if let Some(tm) = texture_manager {
                        tm.bind_texture(group.texture_id, 0);
                    } else {
                        let th = TextureHandle::new(group.texture_id, 1);
                        let _ = self.backend.bind_texture(th, 0);
                    }
                    batch_last_tex = group.texture_id;
                    self.stats.texture_binds += 1;
                }
                self.backend.set_uniform_int(self.uniforms.use_texture, 1);
            } else {
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
            }
            self.backend.set_uniform_vec4(
                self.uniforms.object_color,
                group.color[0],
                group.color[1],
                group.color[2],
                group.color[3],
            );
            let _ = self.backend.bind_buffer(batch_buf);
            self.backend.set_vertex_attributes(&self.object_layout);
            let _ = self.backend.draw_arrays(
                PrimitiveTopology::Triangles,
                group.start_vertex,
                group.vertex_count,
            );
            self.stats.draw_calls += 1;
        }
        self.static_batch_groups = groups;
    }
}
