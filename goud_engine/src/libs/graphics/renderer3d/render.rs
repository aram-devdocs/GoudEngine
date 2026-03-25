//! Frame rendering logic for [`Renderer3D`].
use super::core::Renderer3D;
use super::frustum::Frustum;
use super::shadow::build_directional_shadow_map;
use super::texture::TextureManagerTrait;
use crate::libs::graphics::backend::{
    types::TextureHandle, BlendFactor, CullFace, DepthFunc, FrontFace, PrimitiveTopology,
};
use cgmath::{perspective, Deg, Matrix4};

impl Renderer3D {
    ///
    /// When a current scene is set, only objects/models/lights belonging to that
    /// scene are rendered and the scene's fog/skybox/grid configs are used.
    /// When no scene is active all entities are rendered (backward-compatible).
    pub fn render(&mut self, texture_manager: Option<&dyn TextureManagerTrait>) {
        // Rebuild the static batch VBO if any static flags changed.
        if self.static_batch_dirty && self.config.batching.static_batching_enabled {
            self.rebuild_static_batch();
        }

        self.frame_counter = self.frame_counter.wrapping_add(1);
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

        // Resolve effective environment configs from the active scene (or renderer defaults).
        let (eff_fog, eff_skybox, eff_grid) = if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get(&scene_id) {
                (scene.fog.clone(), scene.skybox.clone(), scene.grid.clone())
            } else {
                (
                    self.fog_config.clone(),
                    self.skybox_config.clone(),
                    self.grid_config.clone(),
                )
            }
        } else {
            (
                self.fog_config.clone(),
                self.skybox_config.clone(),
                self.grid_config.clone(),
            )
        };

        if eff_skybox.enabled {
            self.backend.set_clear_color(
                eff_skybox.color.x,
                eff_skybox.color.y,
                eff_skybox.color.z,
                eff_skybox.color.w,
            );
        }
        self.backend.clear_depth();

        let aspect = self.window_width as f32 / self.window_height as f32;
        let projection: Matrix4<f32> = perspective(Deg(45.0), aspect, 0.1, 1000.0);
        let view = self.camera.view_matrix();
        let view_arr = mat4_to_array(&view);
        let proj_arr = mat4_to_array(&projection);
        let shadow_map = if self.config.shadows.enabled && self.shadows_enabled {
            build_directional_shadow_map(&self.objects, &self.lights, self.shadow_map_size)
        } else {
            None
        };
        let shadow_matrix = shadow_map
            .as_ref()
            .map(|map| mat4_to_array(&map.light_space_matrix))
            .unwrap_or_else(|| mat4_to_array(&Matrix4::from_scale(1.0)));
        if let Some(map) = shadow_map.as_ref() {
            self.update_shadow_texture(&map.rgba8, map.size, map.size);
        }

        if eff_grid.enabled {
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
            self.backend
                .set_uniform_int(self.grid_uniforms.fog_enabled, i32::from(eff_fog.enabled));
            self.backend.set_uniform_vec3(
                self.grid_uniforms.fog_color,
                eff_fog.color.x,
                eff_fog.color.y,
                eff_fog.color.z,
            );
            self.backend
                .set_uniform_float(self.grid_uniforms.fog_density, eff_fog.density);
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

        // Resolve the scene filter set for objects and lights.
        let scene_obj_filter = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.objects);
        let scene_light_filter = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.lights);

        // Build frustum for culling if enabled.
        let frustum = if self.config.frustum_culling.enabled {
            Some(Frustum::from_view_projection(&(projection * view)))
        } else {
            None
        };

        // Object IDs belonging to skinned models — excluded from the standard pass.
        // Uses the persistent set maintained incrementally by load_model/instantiate/destroy.
        let skinned_obj_ids = &self.skinned_object_ids;

        // Track total object count before culling.
        self.stats.total_objects = self.objects.len() as u32;

        // Whether static objects are covered by the batch and should be skipped
        // in the per-object draw loop.
        let has_static_batch =
            self.static_batch_buffer.is_some() && self.config.batching.static_batching_enabled;

        // Collect visible object IDs into the pre-allocated buffer (B3: avoids
        // per-frame Vec<DrawCmd> allocation for large scenes).
        self.visible_object_ids.clear();
        for (&id, obj) in &self.objects {
            if skinned_obj_ids.contains(&id) {
                continue;
            }
            // Static objects are drawn via the batch buffer, skip them here.
            if has_static_batch && obj.is_static {
                continue;
            }
            if let Some(filter) = scene_obj_filter {
                if !filter.contains(&id) {
                    continue;
                }
            }
            if let Some(ref f) = frustum {
                let world_center = obj.position + obj.bounds.center;
                let max_scale = obj.scale.x.max(obj.scale.y).max(obj.scale.z);
                let world_radius = obj.bounds.radius * max_scale;
                if !f.intersects_sphere(world_center, world_radius) {
                    continue;
                }
            }
            self.visible_object_ids.push(id);
        }

        // Sort by material and texture to minimize GPU state changes.
        let material_sorting = self.config.batching.material_sorting_enabled;
        if material_sorting {
            let objects = &self.objects;
            let object_materials = &self.object_materials;
            self.visible_object_ids.sort_by(|&a, &b| {
                let mat_a = object_materials.get(&a).copied().unwrap_or(0);
                let mat_b = object_materials.get(&b).copied().unwrap_or(0);
                let tex_a = objects.get(&a).map_or(0, |o| o.texture_id);
                let tex_b = objects.get(&b).map_or(0, |o| o.texture_id);
                (mat_a, tex_a).cmp(&(mat_b, tex_b))
            });
        }

        self.stats.visible_objects = self.visible_object_ids.len() as u32;
        self.stats.culled_objects = self
            .stats
            .total_objects
            .saturating_sub(self.stats.visible_objects);

        // Snapshot filtered lights for uniform upload.
        let filtered_lights: Vec<super::types::Light> = self
            .lights
            .iter()
            .filter(|(&id, _)| scene_light_filter.is_none_or(|set| set.contains(&id)))
            .map(|(_, l)| l.clone())
            .collect();

        let _ = self.backend.bind_shader(self.shader_handle);
        let uniforms = self.uniforms.clone();
        self.apply_main_uniforms(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_map.is_some(),
            &uniforms,
            &eff_fog,
            &filtered_lights,
        );

        // Draw static batch groups before the per-object loop.
        if has_static_batch {
            self.render_static_batch(texture_manager);
        }

        let mut last_texture_id = u32::MAX;
        // Draw loop: look up object data inline from the pre-collected IDs.
        for i in 0..self.visible_object_ids.len() {
            let obj_id = self.visible_object_ids[i];
            let obj = match self.objects.get(&obj_id) {
                Some(o) => o,
                None => continue,
            };
            let buffer = obj.buffer;
            let vertex_count = obj.vertex_count;
            let position = obj.position;
            let rotation = obj.rotation;
            let scale = obj.scale;
            let texture_id = obj.texture_id;
            let mat_id = self.object_materials.get(&obj_id).copied().unwrap_or(0);
            let color = if let Some(mat) = self.materials.get(&mat_id) {
                let c = &mat.color;
                [c.x, c.y, c.z, c.w]
            } else if texture_id > 0 {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.8, 0.8, 0.8, 1.0]
            };

            let model = Self::create_model_matrix(position, rotation, scale);
            let model_arr = mat4_to_array(&model);
            self.backend
                .set_uniform_mat4(self.uniforms.model, &model_arr);
            if texture_id > 0 {
                if texture_id != last_texture_id {
                    if let Some(tm) = texture_manager {
                        tm.bind_texture(texture_id, 0);
                    } else {
                        let texture_handle = TextureHandle::new(texture_id, 1);
                        let _ = self.backend.bind_texture(texture_handle, 0);
                    }
                    last_texture_id = texture_id;
                    self.stats.texture_binds += 1;
                }
                self.backend.set_uniform_int(self.uniforms.use_texture, 1);
            } else {
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
            }
            self.backend.set_uniform_vec4(
                self.uniforms.object_color,
                color[0],
                color[1],
                color[2],
                color[3],
            );
            let _ = self.backend.bind_buffer(buffer);
            self.backend.set_vertex_attributes(&self.object_layout);
            let _ = self
                .backend
                .draw_arrays(PrimitiveTopology::Triangles, 0, vertex_count as u32);
            self.stats.draw_calls += 1;
        }

        // Skinned mesh rendering pass — uses dedicated skinned shader with bone uniforms.
        if !self.skinned_meshes.is_empty() {
            let gpu_skinning =
                matches!(self.config.skinning.mode, super::config::SkinningMode::Gpu)
                    && self.backend.supports_storage_buffers();

            let _ = self.backend.bind_shader(self.skinned_shader_handle);
            let skinned_unis = self.skinned_uniforms.clone();
            self.apply_main_uniforms(
                &view_arr,
                &proj_arr,
                &shadow_matrix,
                shadow_map.is_some(),
                &skinned_unis.main,
                &eff_fog,
                &filtered_lights,
            );

            let skinned_snaps: Vec<(
                crate::libs::graphics::backend::BufferHandle,
                i32,
                cgmath::Vector3<f32>,
                cgmath::Vector3<f32>,
                cgmath::Vector3<f32>,
                Vec<[f32; 16]>,
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
                        sm.bone_matrices.clone(),
                    )
                })
                .collect();

            for (buffer, vc, pos, rot, scl, bone_mats) in &skinned_snaps {
                let model = Self::create_model_matrix(*pos, *rot, *scl);
                let model_arr = mat4_to_array(&model);
                self.backend
                    .set_uniform_mat4(skinned_unis.main.model, &model_arr);
                self.backend
                    .set_uniform_int(skinned_unis.main.use_texture, 0);
                self.backend
                    .set_uniform_vec4(skinned_unis.main.object_color, 0.8, 0.8, 0.8, 1.0);

                self.stats.skinned_instances += 1;

                if gpu_skinning {
                    let bone_data: &[u8] = bytemuck::cast_slice(bone_mats);
                    self.ensure_bone_storage_buffer(bone_data.len());
                    if let Some(storage_handle) = self.bone_storage_buffer {
                        if let Err(e) =
                            self.backend
                                .update_storage_buffer(storage_handle, 0, bone_data)
                        {
                            log::error!("Failed to upload bone matrices: {e}");
                        }
                        let _ = self.backend.bind_storage_buffer(storage_handle, 0);
                    }
                    self.stats.bone_matrix_uploads += 1;
                } else {
                    for (i, mat) in bone_mats.iter().enumerate() {
                        if i < skinned_unis.bone_matrices.len() {
                            self.backend
                                .set_uniform_mat4(skinned_unis.bone_matrices[i], mat);
                        }
                    }
                    self.stats.bone_matrix_uploads += 1;
                }

                let _ = self.backend.bind_buffer(*buffer);
                self.backend.set_vertex_attributes(&self.skinned_layout);
                let _ = self
                    .backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, *vc as u32);
                self.stats.draw_calls += 1;

                if gpu_skinning {
                    self.backend.unbind_storage_buffer();
                }
            }

            self.backend.unbind_shader();
            // Re-bind main shader for subsequent rendering.
            let _ = self.backend.bind_shader(self.shader_handle);
        }

        // Single-instance skinned model rendering pass.
        self.render_skinned_models(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_map.is_some(),
            &eff_fog,
            &filtered_lights,
            texture_manager,
        );

        // Instanced skinned rendering: groups skinned instances by source model
        // and draws with one instanced draw call per unique model.
        self.render_instanced_skinned_models(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_map.is_some(),
            &eff_fog,
            &filtered_lights,
            texture_manager,
        );

        self.backend.unbind_shader();

        // Instanced mesh and particle rendering.
        self.render_instanced_and_particles(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_map.is_some(),
            &eff_fog,
            &filtered_lights,
            texture_manager,
        );

        // Post-processing pipeline.
        if self.postprocess_pipeline.pass_count() > 0 {
            self.apply_postprocess_pipeline();
        }

        if self.anti_aliasing_mode.uses_fxaa() {
            self.apply_fxaa_pass();
        }

        // Debug draw pass.
        self.render_debug_draw(&view_arr, &proj_arr, &eff_fog);
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
}

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
