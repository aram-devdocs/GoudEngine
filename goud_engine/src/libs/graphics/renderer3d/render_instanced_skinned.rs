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
            let source = match self.models.get(&inst.source_model_id) {
                Some(m) if m.is_skinned => m,
                _ => continue,
            };
            let _ = source; // just checking is_skinned
            if self.animation_players.contains_key(&inst_id) {
                groups
                    .entry(inst.source_model_id)
                    .or_default()
                    .push(inst_id);
            }
        }

        // Also include the source model itself if it has an animation player.
        for (&model_id, model) in &self.models {
            if !model.is_skinned {
                continue;
            }
            if self.animation_players.contains_key(&model_id) {
                groups.entry(model_id).or_default().insert(0, model_id);
            }
        }

        // Filter to groups with at least 2 entries (instanced rendering benefit).
        let groups: Vec<(u32, Vec<u32>)> = groups
            .into_iter()
            .filter(|(_, ids)| ids.len() >= 2)
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
                    instance_ids: instance_ids.clone(),
                })
            })
            .collect();

        for gd in &group_data {
            // Pack all instances' bone matrices into one storage buffer.
            let total_bones = gd.instance_ids.len() * gd.bone_count;
            let mut packed_bones: Vec<f32> = Vec::with_capacity(total_bones * 16);
            let mut instance_data: Vec<f32> = Vec::new();

            for (inst_idx, &inst_id) in gd.instance_ids.iter().enumerate() {
                let bone_offset = (inst_idx * gd.bone_count) as f32;

                if let Some(player) = self.animation_players.get(&inst_id) {
                    for mat in &player.bone_matrices {
                        packed_bones.extend_from_slice(mat);
                    }
                    for _ in player.bone_matrices.len()..gd.bone_count {
                        packed_bones.extend_from_slice(&IDENTITY_MAT4);
                    }
                } else {
                    for _ in 0..gd.bone_count {
                        packed_bones.extend_from_slice(&IDENTITY_MAT4);
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

            // Upload bone matrices to storage buffer.
            let bone_data: &[u8] = bytemuck::cast_slice(&packed_bones);
            self.ensure_bone_storage_buffer(bone_data.len());
            if let Some(storage_handle) = self.bone_storage_buffer {
                if let Err(e) = self
                    .backend
                    .update_storage_buffer(storage_handle, 0, bone_data)
                {
                    log::error!("Instanced skinning storage buffer upload failed: {e}");
                    continue;
                }
                let _ = self.backend.bind_storage_buffer(storage_handle, 0);
            }

            let instance_bytes: &[u8] = bytemuck::cast_slice(&instance_data);
            let instance_count = gd.instance_ids.len() as u32;

            use crate::libs::graphics::backend::{BufferType, BufferUsage, VertexBufferBinding};

            // Reuse persistent instance buffer; grow with next-power-of-two sizing.
            let required_size = instance_bytes.len();
            if self.instanced_skinned_instance_buffer.is_none()
                || self.instanced_skinned_instance_buffer_size < required_size
            {
                if let Some(old) = self.instanced_skinned_instance_buffer.take() {
                    self.backend.destroy_buffer(old);
                }
                let alloc_size = required_size.next_power_of_two().max(64);
                let initial = vec![0u8; alloc_size];
                match self
                    .backend
                    .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &initial)
                {
                    Ok(h) => {
                        self.instanced_skinned_instance_buffer = Some(h);
                        self.instanced_skinned_instance_buffer_size = alloc_size;
                    }
                    Err(e) => {
                        log::error!("Failed to create instanced skinned instance buffer: {e}");
                        self.backend.unbind_storage_buffer();
                        continue;
                    }
                }
            }

            let instance_buffer = match self.instanced_skinned_instance_buffer {
                Some(h) => {
                    if let Err(e) = self.backend.update_buffer(h, 0, instance_bytes) {
                        log::error!("Failed to update instanced skinned instance buffer: {e}");
                        self.backend.unbind_storage_buffer();
                        continue;
                    }
                    h
                }
                None => {
                    self.backend.unbind_storage_buffer();
                    continue;
                }
            };

            for &obj_id in &gd.mesh_object_ids {
                let (buffer, vc) = match self.objects.get(&obj_id) {
                    Some(obj) => (obj.buffer, obj.vertex_count),
                    None => continue,
                };

                self.backend
                    .set_uniform_int(inst_skinned_unis.main.use_texture, 0);
                self.backend.set_uniform_vec4(
                    inst_skinned_unis.main.object_color,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                );

                let bindings = [
                    VertexBufferBinding::per_vertex(buffer, self.skinned_layout.clone()),
                    VertexBufferBinding::per_instance(
                        instance_buffer,
                        self.instanced_skinned_instance_layout.clone(),
                    ),
                ];
                let _ = self.backend.set_vertex_bindings(&bindings);
                if let Err(e) = self.backend.draw_arrays_instanced(
                    PrimitiveTopology::Triangles,
                    0,
                    vc as u32,
                    instance_count,
                ) {
                    eprintln!("[INST] draw_arrays_instanced failed: {e}");
                }
                self.stats.draw_calls += 1;
                self.stats.instanced_draw_calls += 1;
                self.stats.active_instances += instance_count;
                self.stats.skinned_instances += instance_count;
            }

            // Buffer is kept alive for reuse -- do not destroy.
            self.backend.unbind_storage_buffer();
        }

        self.backend.unbind_shader();

        handled_ids
    }
}
