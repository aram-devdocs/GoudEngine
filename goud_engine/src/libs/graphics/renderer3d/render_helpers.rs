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
    /// Bind a texture for an object (or signal no-texture) and set the `use_texture` uniform.
    pub(super) fn bind_or_skip_texture(
        &mut self,
        texture_id: u32,
        texture_manager: Option<&dyn super::texture::TextureManagerTrait>,
        use_texture_uniform: i32,
    ) {
        if texture_id > 0 {
            if let Some(tm) = texture_manager {
                tm.bind_texture(texture_id, 0);
            } else {
                let texture_handle = TextureHandle::new(texture_id, 1);
                let _ = self.backend.bind_texture(texture_handle, 0);
            }
            self.backend.set_uniform_int(use_texture_uniform, 1);
        } else {
            self.backend.set_uniform_int(use_texture_uniform, 0);
        }
    }

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
        self.backend.request_readback();
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
        self.backend.request_readback();
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
        instanced_ids: &std::collections::HashSet<u32>,
    ) {
        let scene_model_filter = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.models);

        // Collect draw data using raw pointers to avoid cloning Vec data per frame.
        struct SkinnedDraw {
            obj_ids: *const Vec<u32>,
            mat_ids: *const Vec<u32>,
            bone_mats: *const Vec<[f32; 16]>,
        }

        let mut draws: Vec<SkinnedDraw> = Vec::new();

        for (&model_id, model) in &self.models {
            if !model.is_skinned || instanced_ids.contains(&model_id) {
                continue;
            }
            if let Some(filter) = scene_model_filter {
                if !filter.contains(&model_id) {
                    continue;
                }
            }
            if let Some(player) = self.animation_players.get(&model_id) {
                draws.push(SkinnedDraw {
                    obj_ids: &model.mesh_object_ids as *const _,
                    mat_ids: &model.mesh_material_ids as *const _,
                    bone_mats: &player.bone_matrices as *const _,
                });
            }
        }

        for (&inst_id, inst) in &self.model_instances {
            if instanced_ids.contains(&inst_id) {
                continue;
            }
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
                draws.push(SkinnedDraw {
                    obj_ids: &inst.mesh_object_ids as *const _,
                    mat_ids: &inst.mesh_material_ids as *const _,
                    bone_mats: &player.bone_matrices as *const _,
                });
            }
        }

        if draws.is_empty() {
            return;
        }

        let gpu_skinning = matches!(self.config.skinning.mode, super::config::SkinningMode::Gpu)
            && self.backend.supports_storage_buffers();

        // Use LessEqual depth so sub-meshes at the same depth (same bone
        // transforms) can overwrite each other — required for multi-material
        // models where body + joints share overlapping geometry.
        use crate::libs::graphics::backend::DepthFunc;
        self.backend.set_depth_func(DepthFunc::LessEqual);

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

        // GPU skinning: pack ALL models' bone matrices into one storage buffer
        // with per-draw offsets so a single upload covers every model.
        let mut bone_offsets: Vec<i32> = Vec::new();
        if gpu_skinning {
            let mut packed_bones: Vec<f32> = Vec::new();
            for draw in &draws {
                // SAFETY: animation_players is not mutated during this loop.
                let bone_mats = unsafe { &*draw.bone_mats };
                bone_offsets.push((packed_bones.len() / 16) as i32);
                for mat in bone_mats.iter() {
                    packed_bones.extend_from_slice(mat);
                }
            }

            if !packed_bones.is_empty() {
                let bone_data: &[u8] = bytemuck::cast_slice(&packed_bones);
                self.ensure_bone_storage_buffer(bone_data.len());
                if let Some(storage_handle) = self.bone_storage_buffer {
                    if let Err(e) = self
                        .backend
                        .update_storage_buffer(storage_handle, 0, bone_data)
                    {
                        log::error!("Failed to upload bone matrices to storage buffer: {e}");
                    }
                    let _ = self.backend.bind_storage_buffer(storage_handle, 0);
                }
                self.stats.bone_matrix_uploads += 1;
            }
        }

        for (draw_idx, draw) in draws.iter().enumerate() {
            // SAFETY: self.models, self.model_instances, and self.animation_players
            // are not mutated during this draw loop -- only the backend receives calls.
            let obj_ids = unsafe { &*draw.obj_ids };
            let mat_ids = unsafe { &*draw.mat_ids };
            let bone_mats = unsafe { &*draw.bone_mats };

            self.stats.skinned_instances += 1;

            if gpu_skinning {
                // Set per-draw bone offset into the packed storage buffer.
                self.backend
                    .set_uniform_int(skinned_unis.bone_offset, bone_offsets[draw_idx]);
            } else {
                // Upload bone matrices as individual uniforms (OpenGL/CPU path).
                for (i, mat) in bone_mats.iter().enumerate() {
                    if i < skinned_unis.bone_matrices.len() {
                        self.backend
                            .set_uniform_mat4(skinned_unis.bone_matrices[i], mat);
                    }
                }
                self.stats.bone_matrix_uploads += 1;
            }

            for (sub_idx, &obj_id) in obj_ids.iter().enumerate() {
                let (buffer, vc, pos, rot, scl, color, tid) = match self.objects.get(&obj_id) {
                    Some(obj) => {
                        let c = mat_ids
                            .get(sub_idx)
                            .and_then(|mid| self.materials.get(mid))
                            .map(|m| [m.color.x, m.color.y, m.color.z, m.color.w])
                            .unwrap_or(self.config.default_material_color);
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

                self.bind_or_skip_texture(tid, texture_manager, skinned_unis.main.use_texture);

                self.backend.set_uniform_vec4(
                    skinned_unis.main.object_color,
                    color[0],
                    color[1],
                    color[2],
                    color[3],
                );
                let _ = self.backend.bind_buffer(buffer);
                if gpu_skinning {
                    self.backend.set_vertex_attributes(&self.skinned_layout);
                } else {
                    self.backend.set_vertex_attributes(&self.object_layout);
                }
                let _ = self
                    .backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, vc as u32);
                self.stats.draw_calls += 1;
            }
        }

        if gpu_skinning {
            self.backend.unbind_storage_buffer();
        }

        self.backend.unbind_shader();
        self.backend.set_depth_func(DepthFunc::Less);
    }

    /// Ensure the bone storage buffer exists and is large enough for the given byte size.
    pub(super) fn ensure_bone_storage_buffer(&mut self, required_bytes: usize) {
        let current_size = self.bone_storage_buffer_size;
        if self.bone_storage_buffer.is_some() && current_size >= required_bytes {
            return;
        }

        // Destroy old buffer if it exists.
        if let Some(old) = self.bone_storage_buffer.take() {
            self.backend.destroy_buffer(old);
        }

        // Allocate with some headroom to avoid frequent reallocations.
        let alloc_size = required_bytes.next_power_of_two().max(64);
        let initial_data = vec![0u8; alloc_size];
        match self.backend.create_storage_buffer(&initial_data) {
            Ok(handle) => {
                self.bone_storage_buffer = Some(handle);
                self.bone_storage_buffer_size = alloc_size;
            }
            Err(e) => {
                log::error!("Failed to create bone storage buffer: {e}");
            }
        }
    }

    /// Ensure the instanced bone storage buffer exists and is large enough.
    pub(super) fn ensure_instanced_bone_storage_buffer(&mut self, required_bytes: usize) {
        let current_size = self.instanced_bone_storage_buffer_size;
        if self.instanced_bone_storage_buffer.is_some() && current_size >= required_bytes {
            return;
        }

        if let Some(old) = self.instanced_bone_storage_buffer.take() {
            self.backend.destroy_buffer(old);
        }

        let alloc_size = required_bytes.next_power_of_two().max(64);
        let initial_data = vec![0u8; alloc_size];
        match self.backend.create_storage_buffer(&initial_data) {
            Ok(handle) => {
                self.instanced_bone_storage_buffer = Some(handle);
                self.instanced_bone_storage_buffer_size = alloc_size;
            }
            Err(e) => {
                log::error!("Failed to create instanced bone storage buffer: {e}");
            }
        }
    }

    // render_instanced_skinned_models is in render_instanced_skinned.rs

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
        let shadows = &self.config.shadows;
        self.backend
            .set_uniform_float(uniforms.shadow_bias, shadows.bias);
        self.backend
            .set_uniform_float(uniforms.shadow_strength, shadows.shadow_strength);
        let texel = if self.config.shadows.map_size > 0 {
            1.0 / self.config.shadows.map_size as f32
        } else {
            0.0
        };
        self.backend
            .set_uniform_vec2(uniforms.shadow_texel_size, texel, texel);

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
