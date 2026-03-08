//! Per-sink, per-channel, and global volume/speed controls.

use crate::ecs::components::AudioChannel;

use super::AudioManager;

impl AudioManager {
    /// Returns the global volume (0.0 to 1.0).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    ///
    /// let audio_manager = AudioManager::new().unwrap();
    /// assert_eq!(audio_manager.global_volume(), 1.0);
    /// ```
    pub fn global_volume(&self) -> f32 {
        *self
            .global_volume
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Sets the global volume (0.0 to 1.0).
    ///
    /// All currently playing and future audio will use this volume.
    /// Values outside the range are clamped.
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level (0.0 = mute, 1.0 = full volume)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    ///
    /// let mut audio_manager = AudioManager::new().unwrap();
    /// audio_manager.set_global_volume(0.5);
    /// assert_eq!(audio_manager.global_volume(), 0.5);
    /// ```
    pub fn set_global_volume(&mut self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        *self
            .global_volume
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = clamped;

        // Reapply composed volume to all active players
        let channel_volumes = self
            .channel_volumes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for entry in players.values() {
            let ch_vol = channel_volumes.get(&entry.channel).copied().unwrap_or(1.0);
            entry
                .player
                .set_volume(clamped * ch_vol * entry.individual_volume);
        }
    }

    /// Sets the volume for a specific sink.
    ///
    /// Updates both the stored individual volume and the effective playback
    /// volume (global * channel * individual).
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    /// * `volume` - Volume level (0.0-1.0, will be clamped)
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and volume set, `false` otherwise.
    pub fn set_sink_volume(&self, sink_id: u64, volume: f32) -> bool {
        let mut players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.get_mut(&sink_id) {
            let clamped = volume.clamp(0.0, 1.0);
            entry.individual_volume = clamped;
            let effective = self.effective_volume(entry.channel, clamped);
            entry.player.set_volume(effective);
            true
        } else {
            false
        }
    }

    /// Sets the playback speed (and pitch) for a specific sink.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    /// * `speed` - Speed multiplier (0.1-10.0, will be clamped)
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and speed set, `false` otherwise.
    pub fn set_sink_speed(&self, sink_id: u64, speed: f32) -> bool {
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.get(&sink_id) {
            let clamped = speed.clamp(0.1, 10.0);
            entry.player.set_speed(clamped);
            true
        } else {
            false
        }
    }

    /// Returns whether a sink has finished playing.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink exists but has no more audio to play,
    /// `false` if the sink does not exist or is still playing.
    pub fn is_finished(&self, sink_id: u64) -> bool {
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.get(&sink_id) {
            entry.player.empty()
        } else {
            false
        }
    }

    /// Allocates a new unique player ID.
    pub(super) fn allocate_player_id(&self) -> u64 {
        let mut next_id = self
            .next_player_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        id
    }

    // =========================================================================
    // Per-channel volume
    // =========================================================================

    /// Sets the volume multiplier for an entire audio channel.
    ///
    /// All currently-playing sounds on this channel have their effective
    /// volume recomputed immediately.
    ///
    /// # Arguments
    ///
    /// * `channel` - The audio channel to adjust
    /// * `volume` - Volume level (0.0-1.0, will be clamped)
    pub fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.channel_volumes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(channel, clamped);

        // Reapply volume to all players on this channel
        let global = self.global_volume();
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for entry in players.values() {
            if entry.channel == channel {
                entry
                    .player
                    .set_volume(global * clamped * entry.individual_volume);
            }
        }
    }

    /// Returns the current volume multiplier for a channel.
    ///
    /// Returns `1.0` for channels that have not been explicitly set.
    pub fn get_channel_volume(&self, channel: AudioChannel) -> f32 {
        self.channel_volumes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&channel)
            .copied()
            .unwrap_or(1.0)
    }

    /// Computes the effective playback volume from global, channel, and
    /// individual multipliers.
    pub(crate) fn effective_volume(&self, channel: AudioChannel, individual: f32) -> f32 {
        let global = self.global_volume();
        let ch_vol = self.get_channel_volume(channel);
        global * ch_vol * individual
    }
}
