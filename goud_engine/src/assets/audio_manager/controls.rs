//! Per-sink and global volume/speed controls.

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
        *self.global_volume.lock().unwrap()
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
        *self.global_volume.lock().unwrap() = clamped;

        // Update volume for all active players
        let players = self.players.lock().unwrap();
        for player in players.values() {
            player.set_volume(clamped);
        }
    }

    /// Sets the volume for a specific sink.
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
        let players = self.players.lock().unwrap();
        if let Some(player) = players.get(&sink_id) {
            let clamped = volume.clamp(0.0, 1.0);
            player.set_volume(clamped);
            true
        } else {
            false
        }
    }

    /// Sets the playback speed (and pitch) for a specific sink.
    ///
    /// Note: This only works for sinks that haven't started playing yet.
    /// For real-time speed adjustment, the sink needs to be recreated.
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
        let players = self.players.lock().unwrap();
        if let Some(player) = players.get(&sink_id) {
            let clamped = speed.clamp(0.1, 10.0);
            player.set_speed(clamped);
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
        let players = self.players.lock().unwrap();
        if let Some(player) = players.get(&sink_id) {
            player.empty()
        } else {
            false
        }
    }

    /// Allocates a new unique player ID.
    pub(super) fn allocate_player_id(&self) -> u64 {
        let mut next_id = self.next_player_id.lock().unwrap();
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        id
    }
}
