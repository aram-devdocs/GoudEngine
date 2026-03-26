//! Animation-related methods for [`Renderer3D`].

use std::collections::HashMap;

use super::animation::{AnimationPlayer, BoneChannelMap};
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
    if model.bind_pose_vertices.len() != obj_ids.len() {
        log::warn!(
            "gather_skin_uploads: bind_pose_vertices count ({}) != obj_ids count ({})",
            model.bind_pose_vertices.len(),
            obj_ids.len()
        );
    }
    let mut uploads = Vec::with_capacity(sub_count);
    for (i, &obj_id) in obj_ids.iter().enumerate().take(sub_count) {
        if model.bind_pose_bone_indices[i].is_empty() {
            continue;
        }
        if let Some(obj) = objects.get(&obj_id) {
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
    /// Resolve the source [`Model3D`] for a model or instance ID.
    ///
    /// For a model ID this returns the model itself. For an instance ID it
    /// follows `source_model_id` to return the owning model.
    fn resolve_source_model(&self, id: u32) -> Option<&Model3D> {
        self.models.get(&id).or_else(|| {
            self.model_instances
                .get(&id)
                .and_then(|inst| self.models.get(&inst.source_model_id))
        })
    }

    /// Advance all animation players by `dt` seconds, compute bone matrices,
    /// and apply CPU skinning to deform vertex buffers.
    ///
    /// Includes three performance optimizations:
    /// - **G3 (BoneChannelMap)**: Uses pre-computed channel index maps to
    ///   eliminate per-frame string HashMap lookups during animation sampling.
    /// - **G5 (Shared evaluation)**: Groups players with identical animation
    ///   state (same source model, clip, quantized time) and computes bone
    ///   matrices once per group.
    /// - **G6 (Animation LOD)**: Skips or half-rates animation updates for
    ///   models that are far from the camera.
    pub fn update_animations(&mut self, dt: f32) {
        // Collect model IDs and instance IDs that have animation players.
        let player_ids: Vec<u32> = self.animation_players.keys().copied().collect();

        // Phase 1: advance animation time and compute bone matrices.
        //
        // We collect raw pointers to skeleton/animation/channel-map data to
        // avoid cloning.
        // SAFETY: the models HashMap is not mutated during this loop -- only
        // animation_players is mutated via get_mut.
        type UpdateItem = (
            u32,                           // player ID
            u32,                           // source model ID (for shared eval grouping)
            *const SkeletonData,           // skeleton data
            *const Vec<KeyframeAnimation>, // animations
            *const Vec<BoneChannelMap>,    // channel maps (fast path)
        );
        let update_list: Vec<UpdateItem> = player_ids
            .iter()
            .filter_map(|&id| {
                let source_id = if self.models.contains_key(&id) {
                    id
                } else {
                    self.model_instances.get(&id)?.source_model_id
                };
                let model = self.models.get(&source_id)?;
                let skel = model.skeleton.as_ref()?;
                Some((
                    id,
                    source_id,
                    skel as *const SkeletonData,
                    &model.animations as *const _,
                    &model.bone_channel_maps as *const _,
                ))
            })
            .collect();

        // -- G6: Animation LOD --
        // Determine which players to skip or half-rate based on camera distance.
        let lod_enabled = self.config.skinning.animation_lod_enabled;
        let lod_dist = self.config.skinning.animation_lod_distance;
        let lod_skip_dist = self.config.skinning.animation_lod_skip_distance;
        let camera_pos = self.camera.position;
        let frame_counter = self.frame_counter;

        // -- G5: Shared Animation Evaluation cache --
        // Key: (source_model_id, clip_index, quantized_time_bits)
        // Value: computed bone matrices
        let shared_eval = self.config.skinning.shared_animation_eval;
        let mut bone_cache: HashMap<(u32, usize, u32), Vec<[f32; 16]>> = HashMap::new();

        for &(player_id, source_model_id, skel_ptr, anims_ptr, maps_ptr) in &update_list {
            // -- G6: LOD distance check --
            if lod_enabled {
                // Get the position of this model/instance's first mesh object.
                let obj_pos = self
                    .models
                    .get(&player_id)
                    .map(|m| &m.mesh_object_ids)
                    .or_else(|| {
                        self.model_instances
                            .get(&player_id)
                            .map(|i| &i.mesh_object_ids)
                    })
                    .and_then(|ids| ids.first())
                    .and_then(|&oid| self.objects.get(&oid))
                    .map(|obj| obj.position);

                if let Some(pos) = obj_pos {
                    let dx = camera_pos.x - pos.x;
                    let dy = camera_pos.y - pos.y;
                    let dz = camera_pos.z - pos.z;
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                    if dist > lod_skip_dist {
                        // Freeze: keep last pose, don't update at all.
                        self.stats.animation_evaluations_saved += 1;
                        continue;
                    }
                    if dist > lod_dist {
                        // Half rate: skip every other frame using player_id parity.
                        if !frame_counter
                            .wrapping_add(player_id as u64)
                            .is_multiple_of(2)
                        {
                            self.stats.animation_evaluations_saved += 1;
                            continue;
                        }
                    }
                }
            }

            if let Some(player) = self.animation_players.get_mut(&player_id) {
                // SAFETY: models HashMap is not mutated during this loop.
                let skeleton = unsafe { &*skel_ptr };
                let animations = unsafe { &*anims_ptr };
                let channel_maps = unsafe { &*maps_ptr };

                // -- G5: Shared evaluation --
                // Check if we can reuse a previously computed result.
                if shared_eval && player.transition.is_none() && player.blend_factor <= f32::EPSILON
                {
                    if let Some(ref state) = player.primary {
                        if state.playing {
                            let quantized_time = (state.time * 30.0).round() / 30.0;
                            let cache_key =
                                (source_model_id, state.clip_index, quantized_time.to_bits());

                            if let Some(cached) = bone_cache.get(&cache_key) {
                                // Advance time without recomputing matrices.
                                super::animation::advance_state_pub(
                                    &mut player.primary,
                                    dt,
                                    animations,
                                );
                                // Copy cached bone matrices.
                                let copy_len = cached.len().min(player.bone_matrices.len());
                                player.bone_matrices[..copy_len]
                                    .copy_from_slice(&cached[..copy_len]);
                                self.stats.animation_evaluations_saved += 1;
                                continue;
                            }
                        }
                    }
                }

                // Use the fast channel-map path (G3).
                if !channel_maps.is_empty() {
                    player.update_with_channel_maps(dt, skeleton, animations, channel_maps);
                } else {
                    // Fallback to string-based path (should not happen if model
                    // was loaded through load_model, but handle gracefully).
                    player.update(dt, skeleton, animations);
                }
                self.stats.animation_evaluations += 1;

                // -- G5: Cache the result for shared eval --
                if shared_eval && player.transition.is_none() && player.blend_factor <= f32::EPSILON
                {
                    if let Some(ref state) = player.primary {
                        let quantized_time = (state.time * 30.0).round() / 30.0;
                        let cache_key =
                            (source_model_id, state.clip_index, quantized_time.to_bits());
                        bone_cache
                            .entry(cache_key)
                            .or_insert_with(|| player.bone_matrices.clone());
                    }
                }
            }
        }

        // Phase 2: CPU skinning -- deform bind-pose vertices using bone
        // matrices and re-upload each sub-mesh buffer.
        //
        // Skipped when GPU skinning is active (bone matrices are uploaded to a
        // storage buffer instead, and the vertex shader performs the deformation).
        let gpu_skinning = matches!(self.config.skinning.mode, super::config::SkinningMode::Gpu)
            && self.backend.supports_storage_buffers();

        if !gpu_skinning {
            struct SkinWork {
                player_id: u32,
                uploads: Vec<SkinUpload>,
            }

            let mut work_items: Vec<SkinWork> = Vec::new();

            for &id in &player_ids {
                let model = match self.resolve_source_model(id) {
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

                    cpu_skin_submesh(bind_verts, bi, bw, bone_mats, &mut self.skin_scratch_buffer);
                    let data: &[u8] = bytemuck::cast_slice(&self.skin_scratch_buffer);
                    if let Err(e) = self.backend.update_buffer(upload.buffer_handle, 0, data) {
                        log::error!("CPU skinning buffer upload failed: {e}");
                    }
                }
            }
        }
    }

    /// Returns the number of animations in a model.
    pub fn get_animation_count(&self, model_id: u32) -> Option<usize> {
        self.resolve_source_model(model_id)
            .map(|m| m.animations.len())
    }

    /// Returns the name of an animation by index.
    pub fn get_animation_name(&self, model_id: u32, anim_index: usize) -> Option<String> {
        self.resolve_source_model(model_id)
            .and_then(|m| m.animations.get(anim_index).map(|a| a.name.clone()))
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
        self.resolve_source_model(id)
            .map(|m| m.animations.as_slice())
    }
}

// ============================================================================
// CPU skinning
// ============================================================================

/// Minimum individual bone weight to contribute to skinning.
///
/// Bones with weight below this threshold are skipped, saving unnecessary
/// matrix multiplications for near-zero influences.
const WEIGHT_EPSILON: f32 = 0.001;

/// Minimum accumulated bone weight for a vertex to be skinned.
///
/// Vertices with total weight below this threshold retain their original
/// bind-pose position and normal instead of collapsing to the origin.
const SKIN_WEIGHT_EPSILON: f32 = 1e-6;

/// Apply skeletal deformation to a bind-pose sub-mesh on the CPU.
///
/// The bind-pose buffer uses the standard 8-float layout per vertex:
/// `[pos.x, pos.y, pos.z, norm.x, norm.y, norm.z, uv.u, uv.v]`.
///
/// Writes deformed positions and normals into `out`, which is resized and
/// populated from `bind_verts` as needed. Reusing a caller-owned scratch
/// buffer across frames avoids per-call allocation.
fn cpu_skin_submesh(
    bind_verts: &[f32],
    bone_indices: &[[u32; 4]],
    bone_weights: &[[f32; 4]],
    bone_matrices: &[[f32; 16]],
    out: &mut Vec<f32>,
) {
    const FPV: usize = 8; // floats per vertex
    let vert_count = bind_verts.len() / FPV;
    out.resize(bind_verts.len(), 0.0);
    out.copy_from_slice(bind_verts);

    for v in 0..vert_count {
        let base = v * FPV;
        let pos = [bind_verts[base], bind_verts[base + 1], bind_verts[base + 2]];
        let nrm = [
            bind_verts[base + 3],
            bind_verts[base + 4],
            bind_verts[base + 5],
        ];

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
        let mut total_weight = 0.0f32;

        for i in 0..4 {
            let w = bw[i];
            if w < WEIGHT_EPSILON {
                continue;
            }
            let idx = bi[i] as usize;
            if idx >= bone_matrices.len() {
                continue;
            }
            total_weight += w;
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

        // When total weight is zero the vertex has no bone influence -- keep the
        // original bind-pose position and normal so it does not collapse to the origin.
        if total_weight < SKIN_WEIGHT_EPSILON {
            continue;
        }

        // Normalize the accumulated result when total weight is not 1.0.
        if (total_weight - 1.0).abs() > SKIN_WEIGHT_EPSILON {
            let inv_w = 1.0 / total_weight;
            sp[0] *= inv_w;
            sp[1] *= inv_w;
            sp[2] *= inv_w;
            sn[0] *= inv_w;
            sn[1] *= inv_w;
            sn[2] *= inv_w;
        }

        // Normalize the skinned normal.
        let len = (sn[0] * sn[0] + sn[1] * sn[1] + sn[2] * sn[2]).sqrt();
        if len > SKIN_WEIGHT_EPSILON {
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
}
