//! Audio playback methods: play, pause, resume, stop, and cleanup.

use crate::assets::loaders::audio::asset::AudioData;
use crate::assets::loaders::AudioAsset;
use crate::core::error::{GoudError, GoudResult};
use crate::ecs::components::AudioChannel;
use rodio::{Decoder, Player, Source};

use super::{AudioManager, PlayerEntry};

/// Opens a streaming audio file and returns a decoder over a buffered reader.
fn open_streaming_decoder(
    path: &std::path::Path,
) -> GoudResult<Decoder<std::io::BufReader<std::fs::File>>> {
    let file = std::fs::File::open(path).map_err(|e| {
        GoudError::ResourceLoadFailed(format!(
            "Failed to open audio file '{}': {}",
            path.display(),
            e
        ))
    })?;
    let reader = std::io::BufReader::new(file);
    Decoder::new(reader)
        .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))
}

/// Creates a decoder from in-memory bytes.
fn open_memory_decoder(bytes: &[u8]) -> GoudResult<Decoder<std::io::Cursor<Vec<u8>>>> {
    let cursor = std::io::Cursor::new(bytes.to_vec());
    Decoder::new(cursor)
        .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))
}

impl AudioManager {
    /// Plays an audio asset on the default SFX channel.
    ///
    /// Creates a new audio player and starts playback of the given audio asset.
    /// Returns a unique player ID that can be used to control playback.
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
        self.play_on_channel(asset, AudioChannel::SFX)
    }

    /// Plays an audio asset on a specific channel.
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    /// * `channel` - The audio channel to play on
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    pub fn play_on_channel(
        &mut self,
        asset: &AudioAsset,
        channel: AudioChannel,
    ) -> GoudResult<u64> {
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        let player = Player::connect_new(self.device_sink.mixer());
        let effective = self.effective_volume(channel, 1.0);
        player.set_volume(effective);

        match asset.audio_data() {
            AudioData::InMemory(bytes) => {
                player.append(open_memory_decoder(bytes)?);
            }
            AudioData::Streaming { path, .. } => {
                player.append(open_streaming_decoder(path)?);
            }
        }

        let player_id = self.allocate_player_id();
        self.players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                player_id,
                PlayerEntry {
                    player,
                    channel,
                    individual_volume: 1.0,
                },
            );

        Ok(player_id)
    }

    /// Plays an audio asset with looping enabled on the default SFX channel.
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

        let channel = AudioChannel::SFX;
        let player = Player::connect_new(self.device_sink.mixer());
        let effective = self.effective_volume(channel, 1.0);
        player.set_volume(effective);

        match asset.audio_data() {
            AudioData::InMemory(bytes) => {
                player.append(open_memory_decoder(bytes)?.repeat_infinite());
            }
            AudioData::Streaming { path, .. } => {
                player.append(open_streaming_decoder(path)?.repeat_infinite());
            }
        }

        let player_id = self.allocate_player_id();
        self.players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                player_id,
                PlayerEntry {
                    player,
                    channel,
                    individual_volume: 1.0,
                },
            );

        Ok(player_id)
    }

    /// Plays an audio asset with custom volume, pitch, and channel.
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    /// * `volume` - Individual volume multiplier (0.0-1.0, will be clamped)
    /// * `speed` - Playback speed multiplier (0.1-10.0, affects pitch)
    /// * `looping` - Whether to loop indefinitely
    /// * `channel` - The audio channel to play on
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
        channel: AudioChannel,
    ) -> GoudResult<u64> {
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        let clamped_speed = speed.clamp(0.1, 10.0);
        let player = Player::connect_new(self.device_sink.mixer());
        let clamped_volume = volume.clamp(0.0, 1.0);
        let effective = self.effective_volume(channel, clamped_volume);
        player.set_volume(effective);

        match asset.audio_data() {
            AudioData::InMemory(bytes) => {
                let source = open_memory_decoder(bytes)?.speed(clamped_speed);
                if looping {
                    player.append(source.repeat_infinite());
                } else {
                    player.append(source);
                }
            }
            AudioData::Streaming { path, .. } => {
                let source = open_streaming_decoder(path)?.speed(clamped_speed);
                if looping {
                    player.append(source.repeat_infinite());
                } else {
                    player.append(source);
                }
            }
        }

        let player_id = self.allocate_player_id();
        self.players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                player_id,
                PlayerEntry {
                    player,
                    channel,
                    individual_volume: clamped_volume,
                },
            );

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
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.get(&sink_id) {
            entry.player.pause();
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
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.get(&sink_id) {
            entry.player.play();
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
        let mut players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.remove(&sink_id) {
            entry.player.stop();
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
        let players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = players.get(&sink_id) {
            !entry.player.is_paused()
        } else {
            false
        }
    }

    /// Returns the number of active audio players.
    pub fn active_count(&self) -> usize {
        self.players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    /// Stops all currently playing audio.
    pub fn stop_all(&mut self) {
        let mut players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for entry in players.values() {
            entry.player.stop();
        }
        players.clear();
    }

    /// Cleans up finished audio players.
    ///
    /// Removes players that have finished playing from the internal collection.
    /// This should be called periodically to prevent memory leaks.
    pub fn cleanup_finished(&mut self) {
        let mut players = self
            .players
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        players.retain(|_, entry| !entry.player.empty());
    }
}
