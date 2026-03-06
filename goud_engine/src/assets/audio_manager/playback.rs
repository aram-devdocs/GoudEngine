//! Audio playback methods: play, pause, resume, stop, and cleanup.

use crate::assets::loaders::AudioAsset;
use crate::core::error::{GoudError, GoudResult};
use rodio::{Player, Source};

use super::AudioManager;

impl AudioManager {
    /// Plays an audio asset.
    ///
    /// Creates a new audio player and starts playback of the given audio asset.
    /// Returns a unique player ID that can be used to control playback (pause/resume/stop).
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    ///
    /// # Errors
    ///
    /// Returns `ResourceLoadFailed` if audio data is empty or cannot be decoded.
    pub fn play(&mut self, asset: &AudioAsset) -> GoudResult<u64> {
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        let cursor = std::io::Cursor::new(asset.data().to_vec());
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))?;

        let player = Player::connect_new(self.device_sink.mixer());
        player.set_volume(self.global_volume());
        player.append(source);

        let player_id = self.allocate_player_id();
        self.players.lock().unwrap().insert(player_id, player);

        Ok(player_id)
    }

    /// Plays an audio asset with looping enabled.
    ///
    /// Same as `play()` but the audio will repeat indefinitely until stopped.
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    pub fn play_looped(&mut self, asset: &AudioAsset) -> GoudResult<u64> {
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        let cursor = std::io::Cursor::new(asset.data().to_vec());
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))?;

        let looped_source = source.repeat_infinite();
        let player = Player::connect_new(self.device_sink.mixer());
        player.set_volume(self.global_volume());
        player.append(looped_source);

        let player_id = self.allocate_player_id();
        self.players.lock().unwrap().insert(player_id, player);

        Ok(player_id)
    }

    /// Plays an audio asset with custom volume and pitch.
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    /// * `volume` - Volume multiplier (0.0-1.0, will be clamped)
    /// * `speed` - Playback speed multiplier (0.1-10.0, affects pitch)
    /// * `looping` - Whether to loop indefinitely
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    pub fn play_with_settings(
        &mut self,
        asset: &AudioAsset,
        volume: f32,
        speed: f32,
        looping: bool,
    ) -> GoudResult<u64> {
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        let cursor = std::io::Cursor::new(asset.data().to_vec());
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))?;

        let clamped_speed = speed.clamp(0.1, 10.0);
        let source_with_speed = source.speed(clamped_speed);

        let player = Player::connect_new(self.device_sink.mixer());
        let clamped_volume = volume.clamp(0.0, 1.0);
        let final_volume = self.global_volume() * clamped_volume;
        player.set_volume(final_volume);

        if looping {
            player.append(source_with_speed.repeat_infinite());
        } else {
            player.append(source_with_speed);
        }

        let player_id = self.allocate_player_id();
        self.players.lock().unwrap().insert(player_id, player);

        Ok(player_id)
    }

    /// Pauses audio playback for the given sink ID.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and paused, `false` otherwise.
    pub fn pause(&self, sink_id: u64) -> bool {
        let players = self.players.lock().unwrap();
        if let Some(player) = players.get(&sink_id) {
            player.pause();
            true
        } else {
            false
        }
    }

    /// Resumes audio playback for the given sink ID.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and resumed, `false` otherwise.
    pub fn resume(&self, sink_id: u64) -> bool {
        let players = self.players.lock().unwrap();
        if let Some(player) = players.get(&sink_id) {
            player.play();
            true
        } else {
            false
        }
    }

    /// Stops audio playback for the given sink ID and removes it.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and stopped, `false` otherwise.
    pub fn stop(&mut self, sink_id: u64) -> bool {
        let mut players = self.players.lock().unwrap();
        if let Some(player) = players.remove(&sink_id) {
            player.stop();
            true
        } else {
            false
        }
    }

    /// Checks if audio is currently playing for the given sink ID.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink exists and is not paused, `false` otherwise.
    pub fn is_playing(&self, sink_id: u64) -> bool {
        let players = self.players.lock().unwrap();
        if let Some(player) = players.get(&sink_id) {
            !player.is_paused()
        } else {
            false
        }
    }

    /// Returns the number of active audio players.
    pub fn active_count(&self) -> usize {
        self.players.lock().unwrap().len()
    }

    /// Stops all currently playing audio.
    pub fn stop_all(&mut self) {
        let mut players = self.players.lock().unwrap();
        for player in players.values() {
            player.stop();
        }
        players.clear();
    }

    /// Cleans up finished audio players.
    ///
    /// Removes players that have finished playing from the internal collection.
    /// This should be called periodically to prevent memory leaks.
    pub fn cleanup_finished(&mut self) {
        let mut players = self.players.lock().unwrap();
        players.retain(|_, player| !player.empty());
    }
}
