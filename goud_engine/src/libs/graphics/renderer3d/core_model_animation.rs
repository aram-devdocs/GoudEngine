//! Animation-related methods for [`Renderer3D`].

use super::animation::AnimationPlayer;
use super::core::Renderer3D;

impl Renderer3D {
    /// Advance all animation players by `dt` seconds and update bone matrices.
    pub fn update_animations(&mut self, dt: f32) {
        // Collect model IDs and instance IDs that have animation players.
        let player_ids: Vec<u32> = self.animation_players.keys().copied().collect();

        for id in player_ids {
            // Resolve skeleton and animations from the source model.
            let (skeleton, animations) = if let Some(model) = self.models.get(&id) {
                (model.skeleton.as_ref(), &model.animations)
            } else if let Some(inst) = self.model_instances.get(&id) {
                if let Some(model) = self.models.get(&inst.source_model_id) {
                    (model.skeleton.as_ref(), &model.animations)
                } else {
                    continue;
                }
            } else {
                continue;
            };

            if let Some(skeleton) = skeleton {
                // Clone animations to avoid borrow conflicts with animation_players.
                let anims_cloned = animations.clone();
                if let Some(player) = self.animation_players.get_mut(&id) {
                    player.update(dt, skeleton, &anims_cloned);
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
