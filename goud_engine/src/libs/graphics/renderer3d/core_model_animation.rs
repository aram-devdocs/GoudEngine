//! Animation-related methods for [`Renderer3D`].

use super::animation::AnimationPlayer;
use super::core::Renderer3D;
use crate::core::types::{KeyframeAnimation, SkeletonData};

impl Renderer3D {
    /// Advance all animation players by `dt` seconds and update bone matrices.
    pub fn update_animations(&mut self, dt: f32) {
        // Collect model IDs and instance IDs that have animation players.
        let player_ids: Vec<u32> = self.animation_players.keys().copied().collect();

        // Collect (player_id, skeleton_ptr, animations_ptr) to avoid
        // cloning animation data every frame. This is safe because we only
        // mutate animation_players, never models, during the update loop.
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

        for (player_id, skel_ptr, anims_ptr) in update_list {
            if let Some(player) = self.animation_players.get_mut(&player_id) {
                // SAFETY: models HashMap is not mutated during this loop.
                let skeleton = unsafe { &*skel_ptr };
                let animations = unsafe { &*anims_ptr };
                player.update(dt, skeleton, animations);
            }
        }

        // CPU skinning: apply bone matrices to bind-pose vertices and re-upload.
        let skinned_ids: Vec<u32> = self
            .animation_players
            .keys()
            .copied()
            .collect();

        for id in skinned_ids {
            let bone_mats = match self.animation_players.get(&id) {
                Some(p) => p.bone_matrices.clone(),
                None => continue,
            };
            // Get source model's bind-pose data and object IDs.
            type BindPose = Vec<Vec<([f32; 3], [f32; 3], [f32; 2], [u32; 4], [f32; 4])>>;
            let (obj_ids, bind_pose): (Vec<u32>, *const BindPose) = if let Some(m) = self.models.get(&id) {
                (m.mesh_object_ids.clone(), &m.bind_pose_vertices)
            } else if let Some(inst) = self.model_instances.get(&id) {
                let src = match self.models.get(&inst.source_model_id) {
                    Some(m) => m,
                    None => continue,
                };
                (inst.mesh_object_ids.clone(), &src.bind_pose_vertices)
            } else {
                continue;
            };
            // SAFETY: models not mutated; only objects are updated.
            let bind_data: &BindPose = unsafe { &*bind_pose };
            if bind_data.is_empty() {
                continue;
            }
            for (sub_idx, &obj_id) in obj_ids.iter().enumerate() {
                let sub_bind = match bind_data.get(sub_idx) {
                    Some(b) if !b.is_empty() => b,
                    _ => continue,
                };
                let mut verts = Vec::with_capacity(sub_bind.len() * 8);
                for &(pos, norm, uv, bi, bw) in sub_bind {
                    let skinned_pos = skin_vertex(&bone_mats, pos, bi, bw);
                    let skinned_norm = skin_normal(&bone_mats, norm, bi, bw);
                    verts.extend_from_slice(&skinned_pos);
                    verts.extend_from_slice(&skinned_norm);
                    verts.extend_from_slice(&uv);
                }
                if let Some(obj) = self.objects.get_mut(&obj_id) {
                    obj.vertices = verts.clone();
                    // Re-upload to GPU.
                    let _ = self.backend.update_buffer(obj.buffer, 0, bytemuck::cast_slice(&verts));
                }
            }
        }
    }

}

/// Apply bone weights to transform a vertex position.
fn skin_vertex(bone_mats: &[[f32; 16]], pos: [f32; 3], bi: [u32; 4], bw: [f32; 4]) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for i in 0..4 {
        if bw[i] <= 0.0 { continue; }
        let m = bone_mats.get(bi[i] as usize).copied().unwrap_or(IDENTITY_MAT4);
        // Column-major: m[0..4] is col0, m[4..8] is col1, etc.
        out[0] += bw[i] * (m[0] * pos[0] + m[4] * pos[1] + m[8]  * pos[2] + m[12]);
        out[1] += bw[i] * (m[1] * pos[0] + m[5] * pos[1] + m[9]  * pos[2] + m[13]);
        out[2] += bw[i] * (m[2] * pos[0] + m[6] * pos[1] + m[10] * pos[2] + m[14]);
    }
    out
}

/// Apply bone weights to transform a normal (no translation).
fn skin_normal(bone_mats: &[[f32; 16]], norm: [f32; 3], bi: [u32; 4], bw: [f32; 4]) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for i in 0..4 {
        if bw[i] <= 0.0 { continue; }
        let m = bone_mats.get(bi[i] as usize).copied().unwrap_or(IDENTITY_MAT4);
        out[0] += bw[i] * (m[0] * norm[0] + m[4] * norm[1] + m[8]  * norm[2]);
        out[1] += bw[i] * (m[1] * norm[0] + m[5] * norm[1] + m[9]  * norm[2]);
        out[2] += bw[i] * (m[2] * norm[0] + m[6] * norm[1] + m[10] * norm[2]);
    }
    // Normalize
    let len = (out[0]*out[0] + out[1]*out[1] + out[2]*out[2]).sqrt();
    if len > 1e-8 { out[0] /= len; out[1] /= len; out[2] /= len; }
    out
}

const IDENTITY_MAT4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

impl Renderer3D {
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
