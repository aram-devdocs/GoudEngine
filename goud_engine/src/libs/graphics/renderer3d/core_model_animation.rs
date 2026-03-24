//! Animation-related methods for [`Renderer3D`].

use super::animation::AnimationPlayer;
use super::core::Renderer3D;
use super::model::Model3D;
use crate::core::types::{KeyframeAnimation, SkeletonData};
use crate::libs::graphics::backend::BufferHandle;

/// Per-sub-mesh data needed for CPU skinning after bone matrices are computed.
struct SkinUpload {
    buffer_handle: BufferHandle,
    bind_verts: *const Vec<f32>,
    bone_indices: *const Vec<[u32; 4]>,
    bone_weights: *const Vec<[f32; 4]>,
}

/// Gather skinning metadata for a model/instance.
///
/// Returns the list of sub-mesh skin uploads, using the *instance's own*
/// Object3D buffer handles (which are dynamic for skinned models).
fn gather_skin_uploads(
    model: &Model3D,
    obj_ids: &[u32],
    objects: &std::collections::HashMap<u32, super::types::Object3D>,
) -> Vec<SkinUpload> {
    let sub_count = model.bind_pose_vertices.len().min(obj_ids.len());
    let mut uploads = Vec::with_capacity(sub_count);
    for i in 0..sub_count {
        if model.bind_pose_bone_indices[i].is_empty() {
            continue;
        }
        if let Some(obj) = objects.get(&obj_ids[i]) {
            uploads.push(SkinUpload {
                buffer_handle: obj.buffer,
                bind_verts: &model.bind_pose_vertices[i] as *const _,
                bone_indices: &model.bind_pose_bone_indices[i] as *const _,
                bone_weights: &model.bind_pose_bone_weights[i] as *const _,
            });
        }
    }
    uploads
}

impl Renderer3D {
    /// Advance all animation players by `dt` seconds, compute bone matrices,
    /// and apply CPU skinning to deform vertex buffers.
    pub fn update_animations(&mut self, dt: f32) {
        // Collect model IDs and instance IDs that have animation players.
        let player_ids: Vec<u32> = self.animation_players.keys().copied().collect();

        // Phase 1: advance animation time and compute bone matrices.
        //
        // We collect raw pointers to skeleton/animation data to avoid cloning.
        // SAFETY: the models HashMap is not mutated during this loop -- only
        // animation_players is mutated via get_mut.
        let update_list: Vec<(u32, *const SkeletonData, *const Vec<KeyframeAnimation>)> =
            player_ids
                .iter()
                .filter_map(|&id| {
                    let model = if self.models.contains_key(&id) {
                        self.models.get(&id)
                    } else {
                        self.model_instances
                            .get(&id)
                            .and_then(|inst| self.models.get(&inst.source_model_id))
                    }?;
                    let skel = model.skeleton.as_ref()?;
                    Some((id, skel as *const SkeletonData, &model.animations as *const _))
                })
                .collect();

        for &(player_id, skel_ptr, anims_ptr) in &update_list {
            if let Some(player) = self.animation_players.get_mut(&player_id) {
                // SAFETY: models HashMap is not mutated during this loop.
                let skeleton = unsafe { &*skel_ptr };
                let animations = unsafe { &*anims_ptr };
                player.update(dt, skeleton, animations);
            }
        }

        // Phase 2: CPU skinning -- deform bind-pose vertices using bone
        // matrices and re-upload each sub-mesh buffer.
        //
        // We collect the work items first to avoid holding multiple borrows
        // of `self` simultaneously.
        struct SkinWork {
            player_id: u32,
            uploads: Vec<SkinUpload>,
        }

        let mut work_items: Vec<SkinWork> = Vec::new();

        for &id in &player_ids {
            // Resolve the model that owns the bind-pose data.
            let model = if self.models.contains_key(&id) {
                self.models.get(&id)
            } else {
                self.model_instances
                    .get(&id)
                    .and_then(|inst| self.models.get(&inst.source_model_id))
            };
            let model = match model {
                Some(m) if !m.bind_pose_bone_indices.is_empty() => m,
                _ => continue,
            };

            // Resolve the object IDs that belong to this player (model or instance).
            let obj_ids: &[u32] = if let Some(m) = self.models.get(&id) {
                &m.mesh_object_ids
            } else if let Some(inst) = self.model_instances.get(&id) {
                &inst.mesh_object_ids
            } else {
                continue;
            };

            let uploads = gather_skin_uploads(model, obj_ids, &self.objects);
            if !uploads.is_empty() {
                work_items.push(SkinWork {
                    player_id: id,
                    uploads,
                });
            }
        }

        for item in work_items {
            let bone_matrices = match self.animation_players.get(&item.player_id) {
                Some(p) => &p.bone_matrices as *const Vec<[f32; 16]>,
                None => continue,
            };

            for upload in &item.uploads {
                // SAFETY: all pointers reference data in self.models / self.animation_players
                // which are not mutated during this skinning pass (only the backend is).
                let bind_verts = unsafe { &*upload.bind_verts };
                let bi = unsafe { &*upload.bone_indices };
                let bw = unsafe { &*upload.bone_weights };
                let bone_mats = unsafe { &*bone_matrices };

                let deformed = cpu_skin_submesh(bind_verts, bi, bw, bone_mats);
                let data: &[u8] = bytemuck::cast_slice(&deformed);
                if let Err(e) = self.backend.update_buffer(upload.buffer_handle, 0, data) {
                    log::error!("CPU skinning buffer upload failed: {e}");
                }
            }
        }
    }

    /// Returns the number of animations in a model.
    pub fn get_animation_count(&self, model_id: u32) -> Option<usize> {
        if let Some(m) = self.models.get(&model_id) {
            Some(m.animations.len())
        } else if let Some(inst) = self.model_instances.get(&model_id) {
            self.models
                .get(&inst.source_model_id)
                .map(|m| m.animations.len())
        } else {
            None
        }
    }

    /// Returns the name of an animation by index.
    pub fn get_animation_name(&self, model_id: u32, anim_index: usize) -> Option<String> {
        let animations = if let Some(m) = self.models.get(&model_id) {
            &m.animations
        } else if let Some(inst) = self.model_instances.get(&model_id) {
            if let Some(m) = self.models.get(&inst.source_model_id) {
                &m.animations
            } else {
                return None;
            }
        } else {
            return None;
        };

        animations.get(anim_index).map(|a| a.name.clone())
    }

    /// Returns a reference to the animation player for a model/instance, if any.
    pub fn animation_player(&self, id: u32) -> Option<&AnimationPlayer> {
        self.animation_players.get(&id)
    }

    /// Returns a mutable reference to the animation player for a model/instance, if any.
    pub fn animation_player_mut(&mut self, id: u32) -> Option<&mut AnimationPlayer> {
        self.animation_players.get_mut(&id)
    }

    /// Returns animations for a model or instance's source model.
    pub fn get_model_animations(
        &self,
        id: u32,
    ) -> Option<&[crate::assets::loaders::animation::KeyframeAnimation]> {
        if let Some(m) = self.models.get(&id) {
            Some(&m.animations)
        } else if let Some(inst) = self.model_instances.get(&id) {
            self.models
                .get(&inst.source_model_id)
                .map(|m| m.animations.as_slice())
        } else {
            None
        }
    }
}

// ============================================================================
// CPU skinning
// ============================================================================

/// Apply skeletal deformation to a bind-pose sub-mesh on the CPU.
///
/// The bind-pose buffer uses the standard 8-float layout per vertex:
/// `[pos.x, pos.y, pos.z, norm.x, norm.y, norm.z, uv.u, uv.v]`.
///
/// Returns a new buffer with deformed positions and normals, ready for
/// GPU upload via `update_buffer`.
fn cpu_skin_submesh(
    bind_verts: &[f32],
    bone_indices: &[[u32; 4]],
    bone_weights: &[[f32; 4]],
    bone_matrices: &[[f32; 16]],
) -> Vec<f32> {
    const FPV: usize = 8; // floats per vertex
    let vert_count = bind_verts.len() / FPV;
    let mut out = bind_verts.to_vec();

    for v in 0..vert_count {
        let base = v * FPV;
        let pos = [bind_verts[base], bind_verts[base + 1], bind_verts[base + 2]];
        let nrm = [bind_verts[base + 3], bind_verts[base + 4], bind_verts[base + 5]];

        let bi = if v < bone_indices.len() {
            bone_indices[v]
        } else {
            [0; 4]
        };
        let bw = if v < bone_weights.len() {
            bone_weights[v]
        } else {
            [0.0; 4]
        };

        let mut sp = [0.0f32; 3];
        let mut sn = [0.0f32; 3];

        for i in 0..4 {
            let w = bw[i];
            if w <= 0.0 {
                continue;
            }
            let idx = bi[i] as usize;
            if idx >= bone_matrices.len() {
                continue;
            }
            let m = &bone_matrices[idx]; // column-major [f32; 16]
            // Transform position: M * [pos, 1]
            sp[0] += w * (m[0] * pos[0] + m[4] * pos[1] + m[8] * pos[2] + m[12]);
            sp[1] += w * (m[1] * pos[0] + m[5] * pos[1] + m[9] * pos[2] + m[13]);
            sp[2] += w * (m[2] * pos[0] + m[6] * pos[1] + m[10] * pos[2] + m[14]);
            // Transform normal: upper-left 3x3 of M (no translation)
            sn[0] += w * (m[0] * nrm[0] + m[4] * nrm[1] + m[8] * nrm[2]);
            sn[1] += w * (m[1] * nrm[0] + m[5] * nrm[1] + m[9] * nrm[2]);
            sn[2] += w * (m[2] * nrm[0] + m[6] * nrm[1] + m[10] * nrm[2]);
        }

        // Normalize the skinned normal.
        let len = (sn[0] * sn[0] + sn[1] * sn[1] + sn[2] * sn[2]).sqrt();
        if len > 1e-8 {
            sn[0] /= len;
            sn[1] /= len;
            sn[2] /= len;
        }

        out[base] = sp[0];
        out[base + 1] = sp[1];
        out[base + 2] = sp[2];
        out[base + 3] = sn[0];
        out[base + 4] = sn[1];
        out[base + 5] = sn[2];
        // UV (base+6, base+7) unchanged.
    }

    out
}
