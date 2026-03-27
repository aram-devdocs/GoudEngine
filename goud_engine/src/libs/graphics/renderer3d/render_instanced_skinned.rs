//! Instanced skinned model rendering for [`Renderer3D`].
//!
//! Groups skinned instances by source model and draws with one instanced
//! draw call per unique model per sub-mesh.
//!
//! Falls back to a no-op when GPU skinning is unavailable; the per-instance
//! draw path in `render_skinned_models` handles CPU-skinned models.

use super::animation::IDENTITY_MAT4;
use super::core::Renderer3D;
use crate::libs::graphics::backend::PrimitiveTopology;

impl Renderer3D {
    /// Render skinned models using instanced drawing, grouped by source model.
    ///
    /// Returns the set of model/instance IDs that were rendered via instancing,
    /// so the per-object skinned pass can skip them.
    pub(super) fn render_instanced_skinned_models(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadows_enabled: bool,
        fog: &super::types::FogConfig,
        lights: &[super::types::Light],
        _texture_manager: Option<&dyn super::texture::TextureManagerTrait>,
    ) -> std::collections::HashSet<u32> {
        let gpu_skinning = matches!(self.config.skinning.mode, super::config::SkinningMode::Gpu)
            && self.backend.supports_storage_buffers();

        if !gpu_skinning {
            return std::collections::HashSet::new();
        }

        // Group instances by source_model_id.
        let mut groups: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
        for (&inst_id, inst) in &self.model_instances {
            match self.models.get(&inst.source_model_id) {
                Some(m) if m.is_skinned => {}
                _ => continue,
            };
            if self.animation_players.contains_key(&inst_id) {
                groups
                    .entry(inst.source_model_id)
                    .or_default()
                    .push(inst_id);
            }
        }

        // Source models are NOT added to instanced groups — they are rendered
        // by the per-object skinned path instead.  Including them here caused
        // template models (sitting at the default position) to appear as
        // T-pose ghosts at the origin.

        // Filter to groups meeting the minimum instance threshold for instanced rendering.
        let min_instances = self.config.batching.min_instances_for_batching;
        let groups: Vec<(u32, Vec<u32>)> = groups
            .into_iter()
            .filter(|(_, ids)| ids.len() >= min_instances)
            .collect();

        if groups.is_empty() {
            return std::collections::HashSet::new();
        }

        // Collect all IDs that will be rendered via instancing.
        let mut handled_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for (_, ids) in &groups {
            for &id in ids {
                handled_ids.insert(id);
            }
        }

        use crate::libs::graphics::backend::DepthFunc;
        self.backend.set_depth_func(DepthFunc::LessEqual);

        let _ = self
            .backend
            .bind_shader(self.instanced_skinned_shader_handle);
        let inst_skinned_unis = self.instanced_skinned_uniforms.clone();
        self.apply_main_uniforms(
            view_arr,
            proj_arr,
            shadow_matrix,
            shadows_enabled,
            &inst_skinned_unis.main,
            fog,
            lights,
        );

        // Pre-collect per-group data to avoid borrow conflicts with self.
        struct GroupData {
            bone_count: usize,
            mesh_object_ids: Vec<u32>,
            mesh_material_ids: Vec<u32>,
            instance_ids: Vec<u32>,
        }

        let group_data: Vec<GroupData> = groups
            .iter()
            .filter_map(|(source_model_id, instance_ids)| {
                let source = self.models.get(source_model_id)?;
                let bone_count = source.skeleton.as_ref().map_or(0, |s| s.bones.len());
                if bone_count == 0 {
                    return None;
                }
                Some(GroupData {
                    bone_count,
                    mesh_object_ids: source.mesh_object_ids.clone(),
                    mesh_material_ids: source.mesh_material_ids.clone(),
                    instance_ids: instance_ids.clone(),
                })
            })
            .collect();

        // Pack ALL groups' bone matrices into one storage buffer with
        // per-group offsets so a single upload covers every group.
        // Each instance's bone_offset in the instance data includes the
        // group's base offset into the packed buffer.
        struct GroupRenderData {
            instance_data: Vec<f32>,
            instance_count: u32,
            mesh_object_ids: Vec<u32>,
            mesh_material_ids: Vec<u32>,
        }

        let mut all_packed_bones: Vec<f32> = Vec::new();
        let mut group_render_data: Vec<GroupRenderData> = Vec::new();

        for gd in &group_data {
            let group_bone_offset = all_packed_bones.len() / 16;
            let mut instance_data: Vec<f32> = Vec::new();

            for (inst_idx, &inst_id) in gd.instance_ids.iter().enumerate() {
                let bone_offset = (group_bone_offset + inst_idx * gd.bone_count) as f32;

                if let Some(player) = self.animation_players.get(&inst_id) {
                    for mat in &player.bone_matrices {
                        all_packed_bones.extend_from_slice(mat);
                    }
                    for _ in player.bone_matrices.len()..gd.bone_count {
                        all_packed_bones.extend_from_slice(&IDENTITY_MAT4);
                    }
                } else {
                    for _ in 0..gd.bone_count {
                        all_packed_bones.extend_from_slice(&IDENTITY_MAT4);
                    }
                }

                let obj_ids = if let Some(m) = self.models.get(&inst_id) {
                    m.mesh_object_ids.clone()
                } else if let Some(inst) = self.model_instances.get(&inst_id) {
                    inst.mesh_object_ids.clone()
                } else {
                    continue;
                };

                let (pos, rot, scl) = if let Some(&first_oid) = obj_ids.first() {
                    if let Some(obj) = self.objects.get(&first_oid) {
                        (obj.position, obj.rotation, obj.scale)
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let model_mat = Self::create_model_matrix(pos, rot, scl);
                let model_arr = super::render::mat4_to_array(&model_mat);
                instance_data.extend_from_slice(&model_arr);
                instance_data.push(bone_offset);
                instance_data.extend_from_slice(&[0.0, 0.0, 0.0]); // padding
                instance_data.extend_from_slice(&[1.0, 1.0, 1.0, 1.0]); // color
            }

            group_render_data.push(GroupRenderData {
                instance_data,
                instance_count: gd.instance_ids.len() as u32,
                mesh_object_ids: gd.mesh_object_ids.clone(),
                mesh_material_ids: gd.mesh_material_ids.clone(),
            });
        }

        // Single upload of all groups' bone matrices to a dedicated instanced buffer.
        if !all_packed_bones.is_empty() {
            let bone_data: &[u8] = bytemuck::cast_slice(&all_packed_bones);
            self.ensure_instanced_bone_storage_buffer(bone_data.len());
            if let Some(storage_handle) = self.instanced_bone_storage_buffer {
                if let Err(e) = self
                    .backend
                    .update_storage_buffer(storage_handle, 0, bone_data)
                {
                    log::error!("Instanced skinning storage buffer upload failed: {e}");
                    return handled_ids;
                }
                let _ = self.backend.bind_storage_buffer(storage_handle, 0);
            }
        }

        use crate::libs::graphics::backend::{BufferType, BufferUsage, VertexBufferBinding};

        // Clone layouts once outside the per-draw loop to avoid per-mesh-per-frame allocations.
        let skinned_layout = self.skinned_layout.clone();
        let instance_layout = self.instanced_skinned_instance_layout.clone();

        // Each group needs its OWN instance buffer because wgpu stages
        // write_buffer calls and only the last write to a given offset
        // survives into the render pass.  Reuse a pool of per-group buffers.
        for (group_idx, grd) in group_render_data.iter().enumerate() {
            let instance_bytes: &[u8] = bytemuck::cast_slice(&grd.instance_data);
            let required_size = instance_bytes.len();

            // Grow the pool if needed.
            while self.instanced_skinned_instance_buffers.len() <= group_idx {
                let alloc_size = required_size.next_power_of_two().max(64);
                let initial = vec![0u8; alloc_size];
                match self
                    .backend
                    .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &initial)
                {
                    Ok(h) => {
                        self.instanced_skinned_instance_buffers
                            .push((h, alloc_size));
                    }
                    Err(e) => {
                        log::error!("Failed to create instanced skinned instance buffer: {e}");
                        break;
                    }
                }
            }

            // Resize this slot if it's too small.
            if let Some((ref mut handle, ref mut cap)) =
                self.instanced_skinned_instance_buffers.get_mut(group_idx)
            {
                if *cap < required_size {
                    self.backend.destroy_buffer(*handle);
                    let alloc_size = required_size.next_power_of_two().max(64);
                    let initial = vec![0u8; alloc_size];
                    match self.backend.create_buffer(
                        BufferType::Vertex,
                        BufferUsage::Dynamic,
                        &initial,
                    ) {
                        Ok(h) => {
                            *handle = h;
                            *cap = alloc_size;
                        }
                        Err(e) => {
                            log::error!("Failed to resize instance buffer: {e}");
                            continue;
                        }
                    }
                }
            }

            let instance_buffer =
                if let Some(&(h, _)) = self.instanced_skinned_instance_buffers.get(group_idx) {
                    if let Err(e) = self.backend.update_buffer(h, 0, instance_bytes) {
                        log::error!("Failed to update instanced skinned instance buffer: {e}");
                        continue;
                    }
                    h
                } else {
                    continue;
                };

            for (sub_idx, &obj_id) in grd.mesh_object_ids.iter().enumerate() {
                let (buffer, vc, texture_id) = match self.objects.get(&obj_id) {
                    Some(obj) => (obj.buffer, obj.vertex_count, obj.texture_id),
                    None => continue,
                };

                // Look up sub-mesh material color (same pattern as regular object path).
                let mat_color = grd
                    .mesh_material_ids
                    .get(sub_idx)
                    .and_then(|&mid| self.materials.get(&mid))
                    .map(|m| [m.color.x, m.color.y, m.color.z, m.color.w])
                    .unwrap_or(self.config.default_material_color);

                self.bind_or_skip_texture(
                    texture_id,
                    _texture_manager,
                    inst_skinned_unis.main.use_texture,
                );
                self.backend.set_uniform_vec4(
                    inst_skinned_unis.main.object_color,
                    mat_color[0],
                    mat_color[1],
                    mat_color[2],
                    mat_color[3],
                );

                let bindings = [
                    VertexBufferBinding::per_vertex(buffer, skinned_layout.clone()),
                    VertexBufferBinding::per_instance(
                        instance_buffer,
                        instance_layout.clone(),
                    ),
                ];
                let _ = self.backend.set_vertex_bindings(&bindings);
                if let Err(e) = self.backend.draw_arrays_instanced(
                    PrimitiveTopology::Triangles,
                    0,
                    vc as u32,
                    grd.instance_count,
                ) {
                    log::error!("draw_arrays_instanced failed: {e}");
                }
                self.stats.draw_calls += 1;
                self.stats.instanced_draw_calls += 1;
                self.stats.active_instances += grd.instance_count;
                self.stats.skinned_instances += grd.instance_count;
            }
        }

        self.backend.unbind_storage_buffer();

        self.backend.unbind_shader();
        self.backend.set_depth_func(DepthFunc::Less);

        handled_ids
    }
}
