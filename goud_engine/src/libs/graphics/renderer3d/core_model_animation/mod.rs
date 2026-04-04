//! Animation-related methods for [`Renderer3D`].

mod cpu_skinning;

use super::animation::{AnimationPlayer, BakedAnimationData, BoneChannelMap};
use super::core::Renderer3D;
use super::model::Model3D;
use crate::core::types::{KeyframeAnimation, SkeletonData};
use crate::libs::graphics::backend::BufferHandle;

pub(super) use cpu_skinning::cpu_skin_submesh;

/// Per-sub-mesh data needed for CPU skinning after bone matrices are computed.
///
/// # Safety
/// The raw pointers (`bind_verts`, `bone_indices`, `bone_weights`) point into
/// `Renderer3D::models` HashMap values. They are safe to dereference as long as
/// the `models` HashMap is not mutated (no insertions or removals) between
/// pointer capture in `gather_skin_uploads` and the dereference in the skinning
/// loop. This is guaranteed because `update_animations` only mutates
/// `animation_players` (a separate field from `models`).
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
    objects: &rustc_hash::FxHashMap<u32, super::types::Object3D>,
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

    /// Sets phase-lock mode for a model or instance's animation player.
    ///
    /// When phase-locked, the player uses a global shared clock instead of
    /// per-instance time, guaranteeing G5 cache hits for all instances of
    /// the same source model and clip.
    pub fn set_animation_phase_lock(&mut self, model_id: u32, enabled: bool) -> bool {
        if let Some(player) = self.animation_players.get_mut(&model_id) {
            player.phase_locked = enabled;
            true
        } else {
            false
        }
    }

    /// Enables or disables the pre-baked animation cache for a model.
    ///
    /// When enabled (the default for models with animations), the animation
    /// update loop uses a simple frame lookup + lerp instead of full
    /// per-frame keyframe evaluation. Disabling this forces the model to
    /// fall back to CPU evaluation (useful for debugging or quality
    /// comparison).
    ///
    /// `model_id` must be a source model ID (not an instance).
    pub fn set_animation_baking_enabled(&mut self, model_id: u32, enabled: bool) -> bool {
        let model = match self.models.get_mut(&model_id) {
            Some(m) => m,
            None => return false,
        };

        if enabled {
            // Re-bake if currently disabled.
            if model.baked_animation.is_none() {
                if let Some(ref skel) = model.skeleton {
                    if !model.animations.is_empty() {
                        model.baked_animation = Some(super::animation::bake_animations(
                            skel,
                            &model.animations,
                            &model.bone_channel_maps,
                            self.config.skinning.baked_animation_sample_rate,
                        ));
                    }
                }
            }
        } else {
            model.baked_animation = None;
        }
        true
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
        let anim_eval_start = std::time::Instant::now();

        // Advance all phase-lock global clocks.
        for clock in self.phase_lock_clocks.values_mut() {
            *clock += dt;
            // Wrap to prevent f32 precision loss after long sessions.
            // 3600s keeps precision above 0.25ms (well within animation needs).
            if *clock > 3600.0 {
                *clock %= 3600.0;
            }
        }

        // Collect model IDs and instance IDs that have animation players,
        // reusing the scratch buffer to avoid per-frame allocation.
        self.scratch_player_ids.clear();
        self.scratch_player_ids
            .extend(self.animation_players.keys().copied());
        let player_ids = std::mem::take(&mut self.scratch_player_ids);

        // Phase 1: advance animation time and compute bone matrices.
        //
        // We collect raw pointers to skeleton/animation/channel-map data to
        // avoid cloning.
        // SAFETY: Raw pointers into `self.models` and `self.animation_players` are safe because:
        // 1. `self.models` is not mutated (no insert/remove) during Phase 1 -- only
        //    `self.animation_players` entries are mutated via `get_mut`.
        // 2. `self.models` and `self.animation_players` are separate HashMap fields,
        //    so mutable access to one does not invalidate references to the other.
        type UpdateItem = (
            u32,                               // player ID
            u32,                               // source model ID (for shared eval grouping)
            *const SkeletonData,               // skeleton data
            *const Vec<KeyframeAnimation>,     // animations
            *const Vec<BoneChannelMap>,        // channel maps (fast path)
            Option<*const BakedAnimationData>, // pre-baked animation cache
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
                let baked_ptr = model.baked_animation.as_ref().map(|b| b as *const _);
                Some((
                    id,
                    source_id,
                    skel as *const SkeletonData,
                    &model.animations as *const _,
                    &model.bone_channel_maps as *const _,
                    baked_ptr,
                ))
            })
            .collect();

        // -- G6: Animation LOD --
        let lod_enabled = self.config.skinning.animation_lod_enabled;
        let lod_dist = self.config.skinning.animation_lod_distance;
        let lod_skip_dist = self.config.skinning.animation_lod_skip_distance;
        let camera_pos = self.camera.position;
        let frame_counter = self.frame_counter;

        // -- G5: Shared Animation Evaluation cache --
        let shared_eval = self.config.skinning.shared_animation_eval;
        let mut bone_cache = std::mem::take(&mut self.bone_eval_cache);
        bone_cache.clear();

        for &(player_id, source_model_id, skel_ptr, anims_ptr, maps_ptr, baked_ptr) in &update_list
        {
            // -- G6: LOD distance check --
            if lod_enabled {
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
                        self.stats.animation_evaluations_saved += 1;
                        continue;
                    }
                    if dist > lod_dist
                        && !frame_counter
                            .wrapping_add(player_id as u64)
                            .is_multiple_of(2)
                    {
                        self.stats.animation_evaluations_saved += 1;
                        continue;
                    }
                }
            }

            if let Some(player) = self.animation_players.get_mut(&player_id) {
                // SAFETY: `self.models` is not mutated (no insert/remove) during Phase 1.
                let skeleton = unsafe { &*skel_ptr };
                let animations = unsafe { &*anims_ptr };
                let channel_maps = unsafe { &*maps_ptr };

                // -- Phase-lock: override per-instance time with global clock --
                if player.phase_locked {
                    if let Some(ref mut state) = player.primary {
                        if state.playing {
                            let clock = self
                                .phase_lock_clocks
                                .entry((source_model_id, state.clip_index))
                                .or_insert(0.0);
                            state.time = *clock;
                        }
                    }
                }

                // -- Baked animation fast path --
                if let Some(bp) = baked_ptr {
                    if player.transition.is_none() && player.blend_factor <= f32::EPSILON {
                        if let Some(ref state) = player.primary {
                            if state.playing {
                                // SAFETY: model is not mutated during this loop.
                                let baked = unsafe { &*bp };
                                let bc = skeleton.bones.len();
                                if player.bone_matrices.len() != bc {
                                    player
                                        .bone_matrices
                                        .resize(bc, super::animation::IDENTITY_MAT4);
                                }
                                if baked.sample(
                                    state.clip_index,
                                    state.time,
                                    &mut player.bone_matrices,
                                ) {
                                    super::animation::advance_state_pub(
                                        &mut player.primary,
                                        dt,
                                        animations,
                                    );
                                    self.stats.animation_evaluations_saved += 1;
                                    continue;
                                }
                            }
                        }
                    }
                }

                // -- G5: Shared evaluation --
                if shared_eval && player.transition.is_none() && player.blend_factor <= f32::EPSILON
                {
                    if let Some(ref state) = player.primary {
                        if state.playing {
                            let quantized_time = (state.time * 30.0).round() / 30.0;
                            let cache_key =
                                (source_model_id, state.clip_index, quantized_time.to_bits());

                            if let Some(cached) = bone_cache.get(&cache_key) {
                                super::animation::advance_state_pub(
                                    &mut player.primary,
                                    dt,
                                    animations,
                                );
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

        // Return the cache to self so it can be reused next frame.
        self.bone_eval_cache = bone_cache;

        let anim_eval_us = anim_eval_start.elapsed().as_micros() as u64;
        crate::libs::graphics::frame_timing::record_phase("anim_eval", anim_eval_us);

        // Phase 2: CPU skinning
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
                    // SAFETY: see SkinUpload doc comment.
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

        // Return the scratch buffer so it can be reused next frame.
        self.scratch_player_ids = player_ids;
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
