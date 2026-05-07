//! Frame rendering logic for [`Renderer3D`].

mod shadow_render;
mod skinned_render;
mod util;

use super::core::Renderer3D;
use super::frustum::Frustum;
use super::shadow::build_directional_shadow_map;
use super::texture::TextureManagerTrait;
use crate::libs::graphics::backend::{
    types::TextureHandle, BlendFactor, CullFace, DepthFunc, FrontFace, PrimitiveTopology,
};
use cgmath::{perspective, Deg, Matrix4};

pub(super) use util::mat4_to_array;

impl Renderer3D {
    ///
    /// When a current scene is set, only objects/models/lights belonging to that
    /// scene are rendered and the scene's fog/skybox/grid configs are used.
    /// When no scene is active all entities are rendered (backward-compatible).
    pub fn render(&mut self, texture_manager: Option<&dyn TextureManagerTrait>) {
        let render3d_start = std::time::Instant::now();

        // Rebuild the static batch VBO if any static flags changed.
        if self.static_batch_dirty && self.config.batching.static_batching_enabled {
            self.rebuild_static_batch();
        }

        // Recompute cached model matrices for any dirty skinned meshes, before
        // the shadow pass and main render pass that both read them.
        for sm in self.skinned_meshes.values_mut() {
            if sm.transform_dirty {
                let model = Self::create_model_matrix(sm.position, sm.rotation, sm.scale);
                sm.cached_model_matrix = mat4_to_array(&model);
                sm.transform_dirty = false;
            }
        }

        self.frame_counter = self.frame_counter.wrapping_add(1);
        let anim_evals = self.stats.animation_evaluations;
        let anim_saved = self.stats.animation_evaluations_saved;
        self.stats = Default::default();
        self.stats.animation_evaluations = anim_evals;
        self.stats.animation_evaluations_saved = anim_saved;
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

        let (eff_fog, eff_skybox, eff_grid) = if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get(&scene_id) {
                (scene.fog, scene.skybox, scene.grid)
            } else {
                (self.fog_config, self.skybox_config, self.grid_config)
            }
        } else {
            (self.fog_config, self.skybox_config, self.grid_config)
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

        let aspect = self.window_width as f32 / self.window_height.max(1) as f32;
        let projection: Matrix4<f32> = perspective(
            Deg(self.config.frustum_culling.fov_degrees),
            aspect,
            self.config.frustum_culling.near_plane,
            self.config.frustum_culling.far_plane,
        );
        let view = self.camera.view_matrix();
        let view_arr = mat4_to_array(&view);
        let proj_arr = mat4_to_array(&projection);
        let shadow_start = std::time::Instant::now();

        // Determine whether we use the GPU shadow pre-pass (wgpu backend) or
        // the legacy CPU software rasterizer (OpenGL backend).
        let gpu_shadow = matches!(
            self.backend.info().shader_language,
            crate::libs::graphics::backend::ShaderLanguage::Wgsl
        );

        let (shadow_matrix, shadow_active) = if self.config.shadows.enabled {
            if gpu_shadow {
                self.record_gpu_shadow_pre_pass()
            } else {
                // Legacy CPU shadow rasterizer for the OpenGL backend.
                let shadow_map = build_directional_shadow_map(
                    &self.objects,
                    &self.lights,
                    self.config.shadows.map_size,
                );
                let matrix = shadow_map
                    .as_ref()
                    .map(|m| mat4_to_array(&m.light_space_matrix))
                    .unwrap_or_else(|| mat4_to_array(&Matrix4::from_scale(1.0)));
                if let Some(map) = shadow_map.as_ref() {
                    self.update_shadow_texture(&map.rgba8, map.size, map.size);
                }
                (matrix, shadow_map.is_some())
            }
        } else {
            (mat4_to_array(&Matrix4::from_scale(1.0)), false)
        };

        let shadow_us = shadow_start.elapsed().as_micros() as u64;
        crate::libs::graphics::frame_timing::record_phase("shadow_build", shadow_us);

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
                .set_uniform_float(self.grid_uniforms.alpha, eff_grid.alpha);
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

        let scene_obj_filter = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.objects);
        let scene_light_filter = self
            .current_scene
            .and_then(|sid| self.scenes.get(&sid))
            .map(|s| &s.lights);

        let frustum = if self.config.frustum_culling.enabled {
            Some(Frustum::from_view_projection(&(projection * view)))
        } else {
            None
        };

        let skinned_obj_ids = &self.skinned_object_ids;

        self.stats.total_objects = self.objects.len() as u32;

        let has_static_batch =
            self.static_batch_buffer.is_some() && self.config.batching.static_batching_enabled;

        // Pre-resolved draw data for visible objects, avoiding repeated HashMap
        // lookups in the draw loop.
        #[derive(Clone)]
        struct VisibleDrawData {
            mat_id: u32,
            texture_id: u32,
            buffer: crate::libs::graphics::backend::BufferHandle,
            vertex_count: i32,
            position: cgmath::Vector3<f32>,
            rotation: cgmath::Vector3<f32>,
            scale: cgmath::Vector3<f32>,
            color: [f32; 4],
        }

        let default_color = self.config.default_material_color;
        let mut visible_draw_data: Vec<VisibleDrawData> = Vec::new();
        self.visible_object_ids.clear();
        for (&id, obj) in &self.objects {
            if skinned_obj_ids.contains(&id) {
                continue;
            }
            if has_static_batch && self.static_batched_ids.contains(&id) {
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
            // Resolve material and color at visibility-check time.
            let mat_id = self.object_materials.get(&id).copied().unwrap_or(0);
            let color = if let Some(mat) = self.materials.get(&mat_id) {
                let c = &mat.color;
                [c.x, c.y, c.z, c.w]
            } else if obj.texture_id > 0 {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                default_color
            };
            self.visible_object_ids.push(id);
            visible_draw_data.push(VisibleDrawData {
                mat_id,
                texture_id: obj.texture_id,
                buffer: obj.buffer,
                vertex_count: obj.vertex_count,
                position: obj.position,
                rotation: obj.rotation,
                scale: obj.scale,
                color,
            });
        }

        let material_sorting = self.config.batching.material_sorting_enabled;
        if material_sorting {
            // Build index-based sort keys to avoid moving the full VisibleDrawData during sort.
            let mut indices: Vec<usize> = (0..visible_draw_data.len()).collect();
            indices.sort_unstable_by_key(|&i| {
                (visible_draw_data[i].mat_id, visible_draw_data[i].texture_id)
            });
            // Reorder visible_draw_data and visible_object_ids in-place via the
            // sorted indices.
            let mut sorted_data: Vec<VisibleDrawData> = Vec::with_capacity(visible_draw_data.len());
            let mut sorted_ids: Vec<u32> = Vec::with_capacity(self.visible_object_ids.len());
            for &i in &indices {
                // SAFETY: each index in `indices` is unique and in-bounds.
                sorted_ids.push(self.visible_object_ids[i]);
            }
            for &i in &indices {
                sorted_data.push(visible_draw_data[i].clone());
            }
            visible_draw_data = sorted_data;
            self.visible_object_ids = sorted_ids;
        }

        self.stats.visible_objects = self.visible_object_ids.len() as u32;
        self.stats.culled_objects = self
            .stats
            .total_objects
            .saturating_sub(self.stats.visible_objects);

        self.scratch_filtered_lights.clear();
        for (&id, l) in &self.lights {
            if scene_light_filter.is_none_or(|set| set.contains(&id)) {
                self.scratch_filtered_lights.push(*l);
            }
        }
        // Take ownership of the scratch buffer for the duration of the frame,
        // returning it at the end to avoid borrow conflicts with `&mut self`.
        let filtered_lights = std::mem::take(&mut self.scratch_filtered_lights);

        let _ = self.backend.bind_shader(self.shader_handle);
        let uniforms = self.uniforms.clone();
        self.apply_main_uniforms(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_active,
            &uniforms,
            &eff_fog,
            &filtered_lights,
        );

        if has_static_batch {
            self.render_static_batch(texture_manager);
        }

        let mut last_texture_id = u32::MAX;
        for draw in &visible_draw_data {
            let model = Self::create_model_matrix(draw.position, draw.rotation, draw.scale);
            let model_arr = mat4_to_array(&model);
            self.backend
                .set_uniform_mat4(self.uniforms.model, &model_arr);
            if draw.texture_id > 0 {
                if draw.texture_id != last_texture_id {
                    if let Some(tm) = texture_manager {
                        tm.bind_texture(draw.texture_id, 0);
                    } else {
                        let texture_handle = TextureHandle::new(draw.texture_id, 1);
                        let _ = self.backend.bind_texture(texture_handle, 0);
                    }
                    last_texture_id = draw.texture_id;
                    self.stats.texture_binds += 1;
                }
                self.backend.set_uniform_int(self.uniforms.use_texture, 1);
            } else {
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
            }
            self.backend.set_uniform_vec4(
                self.uniforms.object_color,
                draw.color[0],
                draw.color[1],
                draw.color[2],
                draw.color[3],
            );
            let _ = self.backend.bind_buffer(draw.buffer);
            self.backend.set_vertex_attributes(&self.object_layout);
            let _ =
                self.backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, draw.vertex_count as u32);
            self.stats.draw_calls += 1;
        }

        // Skinned mesh rendering pass (extracted to skinned_render.rs).
        self.render_skinned_mesh_pass(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_active,
            &eff_fog,
            &filtered_lights,
        );

        let instanced_skinned_ids = self.render_instanced_skinned_models(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_active,
            &eff_fog,
            &filtered_lights,
            texture_manager,
        );

        self.render_skinned_models(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_active,
            &eff_fog,
            &filtered_lights,
            texture_manager,
            &instanced_skinned_ids,
        );

        self.backend.unbind_shader();

        self.flush_dirty_plane_instance_pools();

        self.render_instanced_and_particles(
            &view_arr,
            &proj_arr,
            &shadow_matrix,
            shadow_active,
            &eff_fog,
            &filtered_lights,
            texture_manager,
        );

        if self.postprocess_pipeline.pass_count() > 0 {
            self.apply_postprocess_pipeline();
        }

        if self.anti_aliasing_mode.uses_fxaa() {
            self.apply_fxaa_pass();
        }

        self.render_debug_draw(&view_arr, &proj_arr, &eff_fog);
        self.backend.disable_culling();

        // Return the scratch buffer so it can be reused next frame.
        self.scratch_filtered_lights = filtered_lights;

        let render3d_us = render3d_start.elapsed().as_micros() as u64;
        crate::libs::graphics::frame_timing::record_phase("render3d_scene", render3d_us);
    }
}
