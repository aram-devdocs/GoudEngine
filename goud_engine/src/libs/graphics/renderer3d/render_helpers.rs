//! Render helper methods: post-processing, FXAA, shadow textures, uniform upload,
//! and fullscreen blitting.

use super::core::Renderer3D;
use super::postprocess::apply_fxaa_like_filter;
use super::types::MAX_LIGHTS;
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};
use crate::libs::graphics::backend::PrimitiveTopology;

impl Renderer3D {
    pub(super) fn update_shadow_texture(&mut self, rgba8: &[u8], width: u32, height: u32) {
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

    pub(super) fn apply_fxaa_pass(&mut self) {
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

    pub(super) fn apply_postprocess_pipeline(&mut self) {
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

    pub(super) fn blit_fullscreen_texture(&mut self, texture: TextureHandle) {
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

    /// Collect object IDs belonging to skinned models and their instances.
    pub(super) fn collect_skinned_object_ids(&self) -> std::collections::HashSet<u32> {
        let mut ids = std::collections::HashSet::new();
        for model in self.models.values() {
            if model.is_skinned {
                ids.extend(model.mesh_object_ids.iter().copied());
            }
        }
        for inst in self.model_instances.values() {
            if let Some(src) = self.models.get(&inst.source_model_id) {
                if src.is_skinned {
                    ids.extend(inst.mesh_object_ids.iter().copied());
                }
            }
        }
        ids
    }

    /// Render skinned Model3D and ModelInstance3D entries using the skinned shader.
    ///
    /// Iterates models and instances that have `is_skinned == true`, binds the
    /// skinned shader, uploads bone matrices from the matching `AnimationPlayer`,
    /// and draws each sub-mesh with the skinned vertex layout.
    pub(super) fn render_skinned_models(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadows_enabled: bool,
        fog: &super::types::FogConfig,
        lights: &[super::types::Light],
        texture_manager: Option<&dyn super::texture::TextureManagerTrait>,
    ) {
        let scene_model_filter = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.models);

        // Collect draw data: (obj_ids, mat_ids, bone_matrices).
        let mut draws: Vec<(Vec<u32>, Vec<u32>, Vec<[f32; 16]>)> = Vec::new();

        for (&model_id, model) in &self.models {
            if !model.is_skinned {
                continue;
            }
            if let Some(filter) = scene_model_filter {
                if !filter.contains(&model_id) {
                    continue;
                }
            }
            if let Some(player) = self.animation_players.get(&model_id) {
                draws.push((
                    model.mesh_object_ids.clone(),
                    model.mesh_material_ids.clone(),
                    player.bone_matrices.clone(),
                ));
            }
        }

        for (&inst_id, inst) in &self.model_instances {
            let source = match self.models.get(&inst.source_model_id) {
                Some(m) => m,
                None => continue,
            };
            if !source.is_skinned {
                continue;
            }
            if let Some(filter) = scene_model_filter {
                if !filter.contains(&inst_id) {
                    continue;
                }
            }
            if let Some(player) = self.animation_players.get(&inst_id) {
                draws.push((
                    inst.mesh_object_ids.clone(),
                    inst.mesh_material_ids.clone(),
                    player.bone_matrices.clone(),
                ));
            }
        }

        if draws.is_empty() {
            return;
        }

        let _ = self.backend.bind_shader(self.skinned_shader_handle);
        let skinned_unis = self.skinned_uniforms.clone();
        self.apply_main_uniforms(
            view_arr,
            proj_arr,
            shadow_matrix,
            shadows_enabled,
            &skinned_unis.main,
            fog,
            lights,
        );

        for (obj_ids, mat_ids, bone_mats) in &draws {
            for (i, mat) in bone_mats.iter().enumerate() {
                if i < skinned_unis.bone_matrices.len() {
                    self.backend
                        .set_uniform_mat4(skinned_unis.bone_matrices[i], mat);
                }
            }

            for (sub_idx, &obj_id) in obj_ids.iter().enumerate() {
                let (buffer, vc, pos, rot, scl, color, tid) = match self.objects.get(&obj_id) {
                    Some(obj) => {
                        let c = mat_ids
                            .get(sub_idx)
                            .and_then(|mid| self.materials.get(mid))
                            .map(|m| [m.color.x, m.color.y, m.color.z, m.color.w])
                            .unwrap_or([0.8, 0.8, 0.8, 1.0]);
                        (
                            obj.buffer,
                            obj.vertex_count,
                            obj.position,
                            obj.rotation,
                            obj.scale,
                            c,
                            obj.texture_id,
                        )
                    }
                    None => continue,
                };

                let model_mat = Self::create_model_matrix(pos, rot, scl);
                let model_arr = super::render::mat4_to_array(&model_mat);
                self.backend
                    .set_uniform_mat4(skinned_unis.main.model, &model_arr);

                if tid > 0 {
                    if let Some(tm) = texture_manager {
                        tm.bind_texture(tid, 0);
                    } else {
                        let texture_handle = TextureHandle::new(tid, 1);
                        let _ = self.backend.bind_texture(texture_handle, 0);
                    }
                    self.backend
                        .set_uniform_int(skinned_unis.main.use_texture, 1);
                } else {
                    self.backend
                        .set_uniform_int(skinned_unis.main.use_texture, 0);
                }

                self.backend.set_uniform_vec4(
                    skinned_unis.main.object_color,
                    color[0],
                    color[1],
                    color[2],
                    color[3],
                );

                let _ = self.backend.bind_buffer(buffer);
                self.backend.set_vertex_attributes(&self.skinned_layout);
                let _ = self
                    .backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, vc as u32);
                self.stats.draw_calls += 1;
            }
        }

        self.backend.unbind_shader();
    }

    /// Upload view/projection/fog/light uniforms to the currently-bound shader.
    pub(super) fn apply_main_uniforms(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadows_enabled: bool,
        uniforms: &super::shaders::MainUniforms,
        fog: &super::types::FogConfig,
        lights: &[super::types::Light],
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
            .set_uniform_int(uniforms.fog_enabled, i32::from(fog.enabled));
        self.backend
            .set_uniform_vec3(uniforms.fog_color, fog.color.x, fog.color.y, fog.color.z);
        self.backend
            .set_uniform_float(uniforms.fog_density, fog.density);
        let light_count = lights.len().min(MAX_LIGHTS) as i32;
        self.backend
            .set_uniform_int(uniforms.num_lights, light_count);
        for (i, light) in lights.iter().enumerate().take(MAX_LIGHTS) {
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

        if let Some(l) = lights.iter().find(|l| l.enabled) {
            let d = if l.light_type == super::types::LightType::Directional {
                (-l.direction.x, -l.direction.y, -l.direction.z)
            } else {
                (l.position.x, l.position.y, l.position.z)
            };
            self.backend
                .set_uniform_vec3(uniforms.primary_light_dir, d.0, d.1, d.2);
            self.backend.set_uniform_vec3(
                uniforms.primary_light_color,
                l.color.x,
                l.color.y,
                l.color.z,
            );
            self.backend
                .set_uniform_float(uniforms.primary_light_intensity, l.intensity);
        }
    }
}
