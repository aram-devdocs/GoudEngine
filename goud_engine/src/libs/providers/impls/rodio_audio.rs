//! Rodio audio provider -- wraps the rodio audio library directly.
//!
//! This provider uses the rodio crate (a Layer 1 dependency) rather than
//! the `AudioManager` from `assets/` (Layer 2), preserving the layer
//! hierarchy. Higher-layer code can bridge between this provider and
//! the asset system as needed.

use std::collections::HashMap;

use crate::libs::error::{GoudError, GoudResult};
use crate::libs::providers::audio::AudioProvider;
use crate::libs::providers::types::{
    AudioCapabilities, AudioChannel, PlayConfig, PlaybackId, SoundHandle,
};
use crate::libs::providers::{Provider, ProviderLifecycle};

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};

/// Tracks both a player and its individual volume for composition with master volume.
struct PlayerState {
    player: Player,
    /// Individual volume (0.0 to 1.0) before master composition.
    volume: f32,
}

impl PlayerState {
    /// Creates a new player state with the given volume.
    fn new(player: Player, volume: f32) -> Self {
        Self {
            player,
            volume: volume.clamp(0.0, 1.0),
        }
    }

    /// Applies the effective volume = master_volume * individual_volume.
    fn apply_composed_volume(&self, master: f32) {
        let composed = (master * self.volume).clamp(0.0, 1.0);
        self.player.set_volume(composed);
    }
}

/// Rodio-based audio provider for desktop platforms.
///
/// Manages an audio output device and a collection of active playback
/// instances. Each `play()` call creates a new `Player` connected to
/// the device mixer.
///
/// # Thread Safety
///
/// This provider is `Send + Sync` (required by `Provider`). Internal
/// player state is managed through the `HashMap` which is only accessed
/// through `&mut self` methods.
pub struct RodioAudioProvider {
    device_sink: MixerDeviceSink,
    capabilities: AudioCapabilities,
    players: HashMap<u64, PlayerState>,
    next_id: u64,
    master_volume: f32,
    channel_volumes: HashMap<AudioChannel, f32>,
}

// SAFETY: RodioAudioProvider is Send + Sync because:
// - MixerDeviceSink wraps cpal::Stream (Send but not Sync by default).
//   We only access the mixer through &mut self methods, which provides
//   exclusive access and is safe across threads.
// - Player is Send. HashMap/f32/u64 are Send + Sync.
unsafe impl Send for RodioAudioProvider {}
// SAFETY: All mutable state is accessed through &mut self. The Provider
// trait requires Sync, and since we never share interior mutable state
// without &mut self, this is sound.
unsafe impl Sync for RodioAudioProvider {}

impl RodioAudioProvider {
    /// Creates a new Rodio audio provider.
    ///
    /// Opens the default audio output device. Returns an error if no
    /// audio device is available (e.g., in CI environments).
    pub fn new() -> GoudResult<Self> {
        let mut device_sink = DeviceSinkBuilder::open_default_sink().map_err(|e| {
            GoudError::AudioInitFailed(format!("Failed to open default audio device: {}", e))
        })?;
        device_sink.log_on_drop(false);

        Ok(Self {
            device_sink,
            capabilities: AudioCapabilities {
                supports_spatial: false,
                max_channels: 32,
            },
            players: HashMap::new(),
            next_id: 0,
            master_volume: 1.0,
            channel_volumes: HashMap::new(),
        })
    }

    /// Allocates a new unique playback ID.
    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Computes the effective volume for a playback instance.
    fn effective_volume(&self, config: &PlayConfig) -> f32 {
        let channel_vol = self
            .channel_volumes
            .get(&config.channel)
            .copied()
            .unwrap_or(1.0);
        self.master_volume * channel_vol * config.volume.clamp(0.0, 1.0)
    }
}

impl Provider for RodioAudioProvider {
    fn name(&self) -> &str {
        "rodio"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for RodioAudioProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        // Clean up finished players to prevent unbounded growth.
        self.players.retain(|_, state| !state.player.empty());
        Ok(())
    }

    fn shutdown(&mut self) {
        for state in self.players.values() {
            state.player.stop();
        }
        self.players.clear();
    }
}

impl AudioProvider for RodioAudioProvider {
    fn audio_capabilities(&self) -> &AudioCapabilities {
        &self.capabilities
    }

    fn audio_update(&mut self) -> GoudResult<()> {
        self.players.retain(|_, state| !state.player.empty());
        Ok(())
    }

    fn play(&mut self, _handle: SoundHandle, config: &PlayConfig) -> GoudResult<PlaybackId> {
        // The provider receives a SoundHandle but does not own the sound
        // data. The actual audio data must be loaded and appended by
        // higher-layer code through a bridge method. Here we create the
        // player with the correct volume/speed settings.
        let player = Player::connect_new(self.device_sink.mixer());
        let vol = self.effective_volume(config);
        player.set_volume(vol);
        player.set_speed(config.speed.clamp(0.1, 10.0));

        let id = self.allocate_id();
        // Store both the player and its individual volume for composition
        let state = PlayerState::new(player, config.volume);
        self.players.insert(id, state);
        Ok(PlaybackId(id))
    }

    fn stop(&mut self, id: PlaybackId) -> GoudResult<()> {
        if let Some(state) = self.players.remove(&id.0) {
            state.player.stop();
        }
        Ok(())
    }

    fn pause(&mut self, id: PlaybackId) -> GoudResult<()> {
        if let Some(state) = self.players.get(&id.0) {
            state.player.pause();
        }
        Ok(())
    }

    fn resume(&mut self, id: PlaybackId) -> GoudResult<()> {
        if let Some(state) = self.players.get(&id.0) {
            state.player.play();
        }
        Ok(())
    }

    fn is_playing(&self, id: PlaybackId) -> bool {
        self.players
            .get(&id.0)
            .map(|s| !s.player.is_paused() && !s.player.empty())
            .unwrap_or(false)
    }

    fn set_volume(&mut self, id: PlaybackId, volume: f32) -> GoudResult<()> {
        if let Some(state) = self.players.get_mut(&id.0) {
            state.volume = volume.clamp(0.0, 1.0);
            state.apply_composed_volume(self.master_volume);
        }
        Ok(())
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        // Update all active players with composed volume (master * individual)
        for state in self.players.values() {
            state.apply_composed_volume(self.master_volume);
        }
    }

    fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) {
        self.channel_volumes.insert(channel, volume.clamp(0.0, 1.0));
    }

    fn set_listener_position(&mut self, _pos: [f32; 3]) {
        // Spatial audio not yet supported.
    }

    fn set_source_position(&mut self, _id: PlaybackId, _pos: [f32; 3]) -> GoudResult<()> {
        // Spatial audio not yet supported.
        Ok(())
    }
}

impl std::fmt::Debug for RodioAudioProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RodioAudioProvider")
            .field("master_volume", &self.master_volume)
            .field("active_players", &self.players.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rodio_audio_construction() {
        // May fail in CI without an audio device. That is expected.
        if let Ok(provider) = RodioAudioProvider::new() {
            assert_eq!(provider.name(), "rodio");
            assert_eq!(provider.version(), "1.0.0");
        }
    }

    #[test]
    fn test_rodio_audio_capabilities() {
        if let Ok(provider) = RodioAudioProvider::new() {
            let caps = provider.audio_capabilities();
            assert!(!caps.supports_spatial);
            assert_eq!(caps.max_channels, 32);
        }
    }

    #[test]
    fn test_rodio_audio_lifecycle() {
        if let Ok(mut provider) = RodioAudioProvider::new() {
            assert!(provider.init().is_ok());
            assert!(provider.update(0.016).is_ok());
            provider.shutdown();
        }
    }

    #[test]
    fn test_rodio_audio_master_volume() {
        if let Ok(mut provider) = RodioAudioProvider::new() {
            provider.set_master_volume(0.5);
            assert_eq!(provider.master_volume, 0.5);

            // Clamp to [0, 1]
            provider.set_master_volume(2.0);
            assert_eq!(provider.master_volume, 1.0);

            provider.set_master_volume(-1.0);
            assert_eq!(provider.master_volume, 0.0);
        }
    }

    #[test]
    fn test_rodio_audio_channel_volume() {
        if let Ok(mut provider) = RodioAudioProvider::new() {
            provider.set_channel_volume(AudioChannel::Music, 0.7);
            assert_eq!(
                provider.channel_volumes.get(&AudioChannel::Music),
                Some(&0.7)
            );
        }
    }

    #[test]
    fn test_rodio_audio_spatial_stubs() {
        if let Ok(mut provider) = RodioAudioProvider::new() {
            provider.set_listener_position([1.0, 2.0, 3.0]);
            assert!(provider
                .set_source_position(PlaybackId(0), [4.0, 5.0, 6.0])
                .is_ok());
        }
    }

    #[test]
    fn test_rodio_audio_play_creates_player() {
        if let Ok(mut provider) = RodioAudioProvider::new() {
            let id = provider
                .play(SoundHandle(1), &PlayConfig::default())
                .unwrap();
            assert_eq!(id, PlaybackId(0));
            assert!(provider.stop(id).is_ok());
        }
    }

    #[test]
    fn test_rodio_audio_debug_format() {
        if let Ok(provider) = RodioAudioProvider::new() {
            let debug = format!("{:?}", provider);
            assert!(debug.contains("RodioAudioProvider"));
            assert!(debug.contains("master_volume"));
        }
    }
}
