//! Skinned mesh rendering pass extracted from the main render loop.

use super::super::core::Renderer3D;
use super::super::types::FogConfig;
use super::super::types::Light;
use crate::libs::graphics::backend::PrimitiveTopology;

impl Renderer3D {
    /// Renders all standalone skinned meshes (not model-instanced).
    ///
    /// This is called from the main `render()` method after the static/dynamic
    /// object pass. It handles both GPU and CPU skinning paths.
    pub(super) fn render_skinned_mesh_pass(
        &mut self,
        view_arr: &[f32; 16],
        proj_arr: &[f32; 16],
        shadow_matrix: &[f32; 16],
        shadow_active: bool,
        eff_fog: &FogConfig,
        filtered_lights: &[Light],
    ) {
        if self.skinned_meshes.is_empty() {
            return;
        }

        let gpu_skinning = matches!(
            self.config.skinning.mode,
            super::super::config::SkinningMode::Gpu
        ) && self.backend.supports_storage_buffers();

        let _ = self.backend.bind_shader(self.skinned_shader_handle);
        let skinned_unis = self.skinned_uniforms.clone();
        self.apply_main_uniforms(
            view_arr,
            proj_arr,
            shadow_matrix,
            shadow_active,
            &skinned_unis.main,
            eff_fog,
            filtered_lights,
        );

        // Lightweight per-mesh metadata (no bone data cloned).
        struct SkinnedSnap {
            buffer: crate::libs::graphics::backend::BufferHandle,
            vertex_count: i32,
            model_matrix: [f32; 16],
            color: [f32; 4],
        }

        // Pass 1: pack bones into scratch buffer and collect metadata.
        let mut scratch_packed = std::mem::take(&mut self.scratch_packed_bones);
        scratch_packed.clear();
        let mut scratch_offsets = std::mem::take(&mut self.scratch_bone_offsets);
        scratch_offsets.clear();

        let mut skinned_snaps: Vec<SkinnedSnap> = Vec::with_capacity(self.skinned_meshes.len());

        if gpu_skinning {
            let bone_pack_start = std::time::Instant::now();
            for sm in self.skinned_meshes.values() {
                scratch_offsets.push((scratch_packed.len() / 16) as i32);
                for mat in &sm.bone_matrices {
                    scratch_packed.extend_from_slice(mat);
                }
                skinned_snaps.push(SkinnedSnap {
                    buffer: sm.buffer,
                    vertex_count: sm.vertex_count,
                    model_matrix: sm.cached_model_matrix,
                    color: sm.color,
                });
            }
            let bone_pack_us = bone_pack_start.elapsed().as_micros() as u64;
            crate::libs::graphics::frame_timing::record_phase("bone_pack", bone_pack_us);

            if !scratch_packed.is_empty() {
                let bone_upload_start = std::time::Instant::now();
                let bone_data: &[u8] = bytemuck::cast_slice(&scratch_packed);
                self.ensure_bone_storage_buffer(bone_data.len());
                if let Some(storage_handle) = self.bone_storage_buffer {
                    if let Err(e) = self
                        .backend
                        .update_storage_buffer(storage_handle, 0, bone_data)
                    {
                        log::error!("Failed to upload bone matrices: {e}");
                    }
                    let _ = self.backend.bind_storage_buffer(storage_handle, 0);
                }
                let bone_upload_us = bone_upload_start.elapsed().as_micros() as u64;
                crate::libs::graphics::frame_timing::record_phase("bone_upload", bone_upload_us);
                self.stats.bone_matrix_uploads += 1;
            }
        } else {
            // CPU skinning path: collect metadata only (bones uploaded per-mesh below).
            for sm in self.skinned_meshes.values() {
                skinned_snaps.push(SkinnedSnap {
                    buffer: sm.buffer,
                    vertex_count: sm.vertex_count,
                    model_matrix: sm.cached_model_matrix,
                    color: sm.color,
                });
            }
        }

        // Pass 2: draw each skinned mesh using metadata.
        // For CPU skinning we need to read bone_matrices from skinned_meshes
        // again, but only to upload uniforms (no clone).
        let skinned_mesh_keys: Vec<u32> = self.skinned_meshes.keys().copied().collect();
        for (snap_idx, snap) in skinned_snaps.iter().enumerate() {
            self.backend
                .set_uniform_mat4(skinned_unis.main.model, &snap.model_matrix);
            self.backend
                .set_uniform_int(skinned_unis.main.use_texture, 0);
            self.backend.set_uniform_vec4(
                skinned_unis.main.object_color,
                snap.color[0],
                snap.color[1],
                snap.color[2],
                snap.color[3],
            );
            self.stats.skinned_instances += 1;

            if gpu_skinning {
                self.backend
                    .set_uniform_int(skinned_unis.bone_offset, scratch_offsets[snap_idx]);
            } else if let Some(sm) = skinned_mesh_keys
                .get(snap_idx)
                .and_then(|k| self.skinned_meshes.get(k))
            {
                for (i, mat) in sm.bone_matrices.iter().enumerate() {
                    if i < skinned_unis.bone_matrices.len() {
                        self.backend
                            .set_uniform_mat4(skinned_unis.bone_matrices[i], mat);
                    }
                }
                self.stats.bone_matrix_uploads += 1;
            }

            let _ = self.backend.bind_buffer(snap.buffer);
            self.backend.set_vertex_attributes(&self.skinned_layout);
            let _ =
                self.backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, snap.vertex_count as u32);
            self.stats.draw_calls += 1;
        }

        // Return scratch buffers.
        self.scratch_packed_bones = scratch_packed;
        self.scratch_bone_offsets = scratch_offsets;

        if gpu_skinning {
            self.backend.unbind_storage_buffer();
        }
        self.backend.unbind_shader();
        let _ = self.backend.bind_shader(self.shader_handle);
    }
}
