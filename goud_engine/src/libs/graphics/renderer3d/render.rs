//! Frame rendering logic for [`Renderer3D`].
use super::core::Renderer3D;
use super::postprocess::apply_fxaa_like_filter;
use super::shadow::build_directional_shadow_map;
use super::texture::TextureManagerTrait;
use super::types::MAX_LIGHTS;
use crate::libs::graphics::backend::{
    types::{TextureFilter, TextureFormat, TextureHandle, TextureWrap},
    BlendFactor, CullFace, DepthFunc, FrontFace, PrimitiveTopology, VertexBufferBinding,
};
use cgmath::{perspective, Deg, Matrix4};

impl Renderer3D {
    /// Render the scene -- grid, objects, skinned meshes, and overlays --
    /// using the current camera and light state.
    pub fn render(&mut self, texture_manager: Option<&dyn TextureManagerTrait>) {
        self.stats = Default::default();
        self.backend.set_viewport(
            self.viewport.0,
            self.viewport.1,
            self.viewport.2,
            self.viewport.3,
        );
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
        let shadow_map =
            build_directional_shadow_map(&self.objects, &self.lights, self.shadow_map_size);
        let shadow_matrix = shadow_map
            .as_ref()
            .map(|map| mat4_to_array(&map.light_space_matrix))
            .unwrap_or_else(|| mat4_to_array(&Matrix4::from_scale(1.0)));
        if let Some(map) = shadow_map.as_ref() {
            self.update_shadow_texture(&map.rgba8, map.size, map.size);
        }

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

        // Material-aware object rendering: snapshot to avoid borrow conflicts.
        let obj_snapshots: Vec<(
            crate::libs::graphics::backend::BufferHandle,
            i32,
            cgmath::Vector3<f32>,
            cgmath::Vector3<f32>,
            cgmath::Vector3<f32>,
            [f32; 4],
            u32,
        )> = self
            .objects
            .iter()
            .map(|(&id, obj)| {
                if let Some(&mat_id) = self.object_materials.get(&id) {
                    if let Some(mat) = self.materials.get(&mat_id) {
                        let c = &mat.color;
                        return (
                            obj.buffer,
                            obj.vertex_count,
                            obj.position,
                            obj.rotation,
                            obj.scale,
                            [c.x, c.y, c.z, c.w],
                            obj.texture_id,
                        );
                    }
                }
                let bc = if obj.texture_id > 0 {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.8, 0.8, 0.8, 1.0]
                };
                (
                    obj.buffer,
                    obj.vertex_count,
                    obj.position,
                    obj.rotation,
                    obj.scale,
                    bc,
                    obj.texture_id,
                )
            })
            .collect();

        let _ = self.backend.bind_shader(self.shader_handle);
        let uniforms = self.uniforms.clone();
        self.apply_main_uniforms(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_map.is_some(),
            &uniforms,
        );

        for (buffer, vc, pos, rot, scl, bc, tid) in &obj_snapshots {
            let model = Self::create_model_matrix(*pos, *rot, *scl);
            let model_arr = mat4_to_array(&model);
            self.backend
                .set_uniform_mat4(self.uniforms.model, &model_arr);
            if *tid > 0 {
                if let Some(tm) = texture_manager {
                    tm.bind_texture(*tid, 0);
                } else {
                    let texture_handle = TextureHandle::new(*tid, 1);
                    let _ = self.backend.bind_texture(texture_handle, 0);
                }
                self.backend.set_uniform_int(self.uniforms.use_texture, 1);
            } else {
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
            }
            self.backend
                .set_uniform_vec4(self.uniforms.object_color, bc[0], bc[1], bc[2], bc[3]);
            let _ = self.backend.bind_buffer(*buffer);
            self.backend.set_vertex_attributes(&self.object_layout);
            let _ = self
                .backend
                .draw_arrays(PrimitiveTopology::Triangles, 0, *vc as u32);
            self.stats.draw_calls += 1;
        }

        // Skinned mesh rendering pass.
        if !self.skinned_meshes.is_empty() {
            let skinned_snaps: Vec<(
                crate::libs::graphics::backend::BufferHandle,
                i32,
                cgmath::Vector3<f32>,
                cgmath::Vector3<f32>,
                cgmath::Vector3<f32>,
            )> = self
                .skinned_meshes
                .values()
                .map(|sm| {
                    (
                        sm.buffer,
                        sm.vertex_count,
                        sm.position,
                        sm.rotation,
                        sm.scale,
                    )
                })
                .collect();

            for (buffer, vc, pos, rot, scl) in &skinned_snaps {
                let model = Self::create_model_matrix(*pos, *rot, *scl);
                let model_arr = mat4_to_array(&model);
                self.backend
                    .set_uniform_mat4(self.uniforms.model, &model_arr);
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
                self.backend
                    .set_uniform_vec4(self.uniforms.object_color, 0.8, 0.8, 0.8, 1.0);
                let _ = self.backend.bind_buffer(*buffer);
                self.backend.set_vertex_attributes(&self.object_layout);
                let _ = self
                    .backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, *vc as u32);
                self.stats.draw_calls += 1;
            }
        }

        self.backend.unbind_shader();

        if !self.instanced_meshes.is_empty() || !self.particle_emitters.is_empty() {
            let _ = self.backend.bind_shader(self.instanced_shader_handle);
            let instanced_uniforms = self.instanced_uniforms.clone();
            self.apply_main_uniforms(
                &view_arr,
                &proj_arr,
                &shadow_matrix,
                shadow_map.is_some(),
                &instanced_uniforms,
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
                self.backend.set_uniform_vec4(
                    self.instanced_uniforms.object_color,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                );
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
                self.backend.set_uniform_vec4(
                    self.instanced_uniforms.object_color,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                );
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

        // Post-processing pipeline.
        if self.postprocess_pipeline.pass_count() > 0 {
            self.apply_postprocess_pipeline();
        }

        if self.anti_aliasing_mode.uses_fxaa() {
            self.apply_fxaa_pass();
        }

        if self.debug_draw_vertex_count > 0 {
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
                .set_uniform_float(self.grid_uniforms.alpha, 1.0);
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

        self.backend.disable_culling();
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

    fn apply_main_uniforms(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadows_enabled: bool,
        uniforms: &super::shaders::MainUniforms,
    ) {
        self.backend.set_uniform_mat4(uniforms.view, view_arr);
        self.backend.set_uniform_mat4(uniforms.projection, proj_arr);
        self.backend
            .set_uniform_mat4(uniforms.light_space_matrix, shadow_matrix);
        self.backend.set_uniform_vec3(
            uniforms.view_pos,
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        );
        self.backend
            .set_uniform_int(uniforms.fog_enabled, i32::from(self.fog_config.enabled));
        self.backend.set_uniform_vec3(
            uniforms.fog_color,
            self.fog_config.color.x,
            self.fog_config.color.y,
            self.fog_config.color.z,
        );
        self.backend
            .set_uniform_float(uniforms.fog_density, self.fog_config.density);
        let light_count = self.lights.len().min(MAX_LIGHTS) as i32;
        self.backend
            .set_uniform_int(uniforms.num_lights, light_count);
        for (i, (_, light)) in self.lights.iter().enumerate().take(MAX_LIGHTS) {
            let lu = &uniforms.lights[i];
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
        self.backend.set_uniform_int(uniforms.texture1, 0);
        if let Some(texture) = self.shadow_texture {
            let _ = self.backend.bind_texture(texture, 1);
        }
        self.backend.set_uniform_int(uniforms.shadow_map, 1);
        self.backend
            .set_uniform_int(uniforms.shadows_enabled, i32::from(shadows_enabled));
        self.backend
            .set_uniform_float(uniforms.shadow_bias, self.shadow_bias);
    }

    fn update_shadow_texture(&mut self, rgba8: &[u8], width: u32, height: u32) {
        let recreate = self.shadow_texture.is_none();
        if recreate {
            self.shadow_texture = self
                .backend
                .create_texture(
                    width,
                    height,
                    TextureFormat::RGBA8,
                    TextureFilter::Linear,
                    TextureWrap::ClampToEdge,
                    rgba8,
                )
                .ok();
            return;
        }
        if let Some(texture) = self.shadow_texture {
            let _ = self
                .backend
                .update_texture(texture, 0, 0, width, height, rgba8);
        }
    }

    fn apply_fxaa_pass(&mut self) {
        let width = self.viewport.2.max(1);
        let height = self.viewport.3.max(1);
        let Ok(frame) = self.backend.read_default_framebuffer_rgba8(width, height) else {
            return;
        };
        let filtered = apply_fxaa_like_filter(width, height, &frame);
        if self.postprocess_texture.is_none() || self.postprocess_texture_size != (width, height) {
            if let Some(texture) = self.postprocess_texture.take() {
                self.backend.destroy_texture(texture);
            }
            self.postprocess_texture = self
                .backend
                .create_texture(
                    width,
                    height,
                    TextureFormat::RGBA8,
                    TextureFilter::Linear,
                    TextureWrap::ClampToEdge,
                    &filtered,
                )
                .ok();
            self.postprocess_texture_size = (width, height);
        } else if let Some(texture) = self.postprocess_texture {
            let _ = self
                .backend
                .update_texture(texture, 0, 0, width, height, &filtered);
        }
        let Some(texture) = self.postprocess_texture else {
            return;
        };
        self.blit_fullscreen_texture(texture);
    }

    fn apply_postprocess_pipeline(&mut self) {
        let width = self.viewport.2.max(1);
        let height = self.viewport.3.max(1);
        let Ok(mut frame) = self.backend.read_default_framebuffer_rgba8(width, height) else {
            return;
        };
        self.postprocess_pipeline.run(width, height, &mut frame);
        if self.postprocess_texture.is_none() || self.postprocess_texture_size != (width, height) {
            if let Some(texture) = self.postprocess_texture.take() {
                self.backend.destroy_texture(texture);
            }
            self.postprocess_texture = self
                .backend
                .create_texture(
                    width,
                    height,
                    TextureFormat::RGBA8,
                    TextureFilter::Linear,
                    TextureWrap::ClampToEdge,
                    &frame,
                )
                .ok();
            self.postprocess_texture_size = (width, height);
        } else if let Some(texture) = self.postprocess_texture {
            let _ = self
                .backend
                .update_texture(texture, 0, 0, width, height, &frame);
        }
        let Some(texture) = self.postprocess_texture else {
            return;
        };
        self.blit_fullscreen_texture(texture);
    }

    fn blit_fullscreen_texture(&mut self, texture: TextureHandle) {
        self.backend.disable_depth_test();
        self.backend.disable_culling();
        self.backend.set_depth_mask(false);
        let _ = self.backend.bind_shader(self.postprocess_shader_handle);
        let _ = self.backend.bind_texture(texture, 0);
        if let Some(location) = self
            .backend
            .get_uniform_location(self.postprocess_shader_handle, "screenTexture")
        {
            self.backend.set_uniform_int(location, 0);
        }
        let _ = self.backend.bind_buffer(self.postprocess_quad_buffer);
        self.backend.set_vertex_attributes(&self.postprocess_layout);
        let _ = self.backend.draw_arrays(PrimitiveTopology::Triangles, 0, 6);
        self.backend.unbind_shader();
        self.backend.set_depth_mask(true);
        self.backend.enable_depth_test();
    }
}

// ============================================================================
// Helper
// ============================================================================

/// Convert a cgmath [`Matrix4`] to a column-major `[f32; 16]` array.
///
/// cgmath matrices are already column-major, which matches the backend expectation.
pub(super) fn mat4_to_array(m: &Matrix4<f32>) -> [f32; 16] {
    let cols: &[[f32; 4]; 4] = m.as_ref();
    [
        cols[0][0], cols[0][1], cols[0][2], cols[0][3], cols[1][0], cols[1][1], cols[1][2],
        cols[1][3], cols[2][0], cols[2][1], cols[2][2], cols[2][3], cols[3][0], cols[3][1],
        cols[3][2], cols[3][3],
    ]
}
