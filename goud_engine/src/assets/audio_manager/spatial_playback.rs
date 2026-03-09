//! Spatial audio playback: distance-based attenuation for positioned audio sources.

use std::collections::HashSet;

use crate::assets::loaders::AudioAsset;
use crate::core::error::GoudResult;
use crate::core::math::Vec2;
use crate::ecs::components::AudioChannel;

use super::spatial::spatial_attenuation_3d;
use super::{AudioManager, SpatialSourceState};

impl AudioManager {
    /// Returns the current 3D listener position.
    pub fn listener_position(&self) -> [f32; 3] {
        *self
            .listener_position
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Sets the current 3D listener position.
    pub fn set_listener_position(&self, position: [f32; 3]) {
        *self
            .listener_position
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = position;
        self.refresh_spatial_sources();
    }

    /// Registers or updates a spatial source for an active sink.
    ///
    /// Returns `true` if the sink exists and the source was tracked.
    pub fn register_spatial_source(
        &self,
        sink_id: u64,
        source_position: [f32; 3],
        max_distance: f32,
        rolloff: f32,
        base_volume: f32,
    ) -> bool {
        if !self.has_sink(sink_id) {
            return false;
        }

        let state = SpatialSourceState {
            source_position,
            max_distance: max_distance.max(0.1),
            rolloff: rolloff.max(0.01),
            base_volume: base_volume.clamp(0.0, 1.0),
        };

        self.spatial_sources
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(sink_id, state);

        self.refresh_spatial_sources();
        true
    }

    /// Updates the position of a tracked spatial source.
    ///
    /// Returns `true` if the source exists and was updated.
    pub fn set_source_position(&self, sink_id: u64, source_position: [f32; 3]) -> bool {
        let mut spatial = self
            .spatial_sources
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(state) = spatial.get_mut(&sink_id) {
            state.source_position = source_position;
            drop(spatial);
            self.refresh_spatial_sources();
            true
        } else {
            false
        }
    }

    /// Retains only spatial sources whose sink IDs are listed in `active_sink_ids`.
    pub fn retain_spatial_sources(&self, active_sink_ids: &[u64]) {
        let active: HashSet<u64> = active_sink_ids.iter().copied().collect();
        self.spatial_sources
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|sink_id, _| active.contains(sink_id));
        self.refresh_spatial_sources();
    }

    /// Re-applies attenuation to all tracked spatial sinks.
    pub fn refresh_spatial_sources(&self) {
        let listener = self.listener_position();
        let global = self.global_volume();
        let channel_volumes = self
            .channel_volumes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        let mut players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        self.spatial_sources
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|sink_id, state| {
                let Some(entry) = players.get_mut(sink_id) else {
                    return false;
                };

                let attenuation = spatial_attenuation_3d(
                    state.source_position,
                    listener,
                    state.max_distance,
                    state.rolloff,
                );
                let individual = (state.base_volume * attenuation).clamp(0.0, 1.0);
                entry.individual_volume = individual;

                let channel_volume = channel_volumes.get(&entry.channel).copied().unwrap_or(1.0);
                entry
                    .player
                    .set_volume(global * channel_volume * entry.individual_volume);
                true
            });
    }

    /// Plays audio with spatial positioning (2D).
    ///
    /// The sink is tracked so future listener/source movement can update its
    /// attenuation without replaying the sound.
    pub fn play_spatial(
        &mut self,
        asset: &AudioAsset,
        source_position: Vec2,
        listener_position: Vec2,
        max_distance: f32,
        rolloff: f32,
    ) -> GoudResult<u64> {
        let sink_id = self.play_with_settings(asset, 1.0, 1.0, false, AudioChannel::SFX)?;
        let listener = [listener_position.x, listener_position.y, 0.0];
        let source = [source_position.x, source_position.y, 0.0];
        self.set_listener_position(listener);
        let _ = self.register_spatial_source(sink_id, source, max_distance, rolloff, 1.0);
        Ok(sink_id)
    }

    /// Updates spatial audio volume for an existing sink (2D convenience API).
    pub fn update_spatial_volume(
        &self,
        sink_id: u64,
        source_position: Vec2,
        listener_position: Vec2,
        max_distance: f32,
        rolloff: f32,
    ) -> bool {
        if !self.has_sink(sink_id) {
            return false;
        }

        self.set_listener_position([listener_position.x, listener_position.y, 0.0]);

        if !self.set_source_position(sink_id, [source_position.x, source_position.y, 0.0]) {
            return self.register_spatial_source(
                sink_id,
                [source_position.x, source_position.y, 0.0],
                max_distance,
                rolloff,
                1.0,
            );
        }

        true
    }
}
