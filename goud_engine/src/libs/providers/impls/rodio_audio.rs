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

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, SpatialPlayer};

/// Underlying rodio handle type for a playback instance.
enum BackendPlayer {
    Standard(Player),
    Spatial(SpatialPlayer),
}

impl BackendPlayer {
    fn set_volume(&self, volume: f32) {
        match self {
            Self::Standard(player) => player.set_volume(volume),
            Self::Spatial(player) => player.set_volume(volume),
        }
    }

    fn pause(&self) {
        match self {
            Self::Standard(player) => player.pause(),
            Self::Spatial(player) => player.pause(),
        }
    }

    fn play(&self) {
        match self {
            Self::Standard(player) => player.play(),
            Self::Spatial(player) => player.play(),
        }
    }

    fn stop(&self) {
        match self {
            Self::Standard(player) => player.stop(),
            Self::Spatial(player) => player.stop(),
        }
    }

    fn is_paused(&self) -> bool {
        match self {
            Self::Standard(player) => player.is_paused(),
            Self::Spatial(player) => player.is_paused(),
        }
    }

    fn empty(&self) -> bool {
        match self {
            Self::Standard(player) => player.empty(),
            Self::Spatial(player) => player.empty(),
        }
    }

    fn set_source_position(&self, position: [f32; 3]) {
        if let Self::Spatial(player) = self {
            player.set_emitter_position(position);
        }
    }

    fn set_listener_ears(&self, left_ear: [f32; 3], right_ear: [f32; 3]) {
        if let Self::Spatial(player) = self {
            player.set_left_ear_position(left_ear);
            player.set_right_ear_position(right_ear);
        }
    }
}

/// Tracks both a player and its individual volume for composition with master volume.
struct PlayerState {
    player: BackendPlayer,
    /// Individual volume (0.0 to 1.0) before master/channel composition.
    volume: f32,
    /// Channel for channel-volume composition.
    channel: AudioChannel,
}

impl PlayerState {
    fn new(player: BackendPlayer, volume: f32, channel: AudioChannel) -> Self {
        Self {
            player,
            volume: volume.clamp(0.0, 1.0),
            channel,
        }
    }

    fn apply_composed_volume(&self, master: f32, channel_volume: f32) {
        let composed = (master * channel_volume * self.volume).clamp(0.0, 1.0);
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
    listener_position: [f32; 3],
    ear_half_distance: f32,
}

// SAFETY: RodioAudioProvider is Send + Sync because:
// - MixerDeviceSink wraps cpal::Stream (Send but not Sync by default).
//   We only access the mixer through &mut self methods, which provides
//   exclusive access and is safe across threads.
// - Player/SpatialPlayer are Send. HashMap/f32/u64 are Send + Sync.
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
                supports_spatial: true,
                max_channels: 32,
            },
            players: HashMap::new(),
            next_id: 0,
            master_volume: 1.0,
            channel_volumes: HashMap::new(),
            listener_position: [0.0, 0.0, 0.0],
            ear_half_distance: 0.15,
        })
    }

    /// Allocates a new unique playback ID.
    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    fn channel_volume(&self, channel: AudioChannel) -> f32 {
        self.channel_volumes.get(&channel).copied().unwrap_or(1.0)
    }

    fn listener_ear_positions(
        listener_position: [f32; 3],
        ear_half_distance: f32,
    ) -> ([f32; 3], [f32; 3]) {
        let half = ear_half_distance.max(0.001);
        (
            [
                listener_position[0] - half,
                listener_position[1],
                listener_position[2],
            ],
            [
                listener_position[0] + half,
                listener_position[1],
                listener_position[2],
            ],
        )
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
        // player with the correct volume/speed/spatial settings.

        let clamped_speed = config.speed.clamp(0.1, 10.0);
        let clamped_volume = config.volume.clamp(0.0, 1.0);

        let backend_player = if let Some(source_position) = config.position {
            let (left_ear, right_ear) =
                Self::listener_ear_positions(self.listener_position, self.ear_half_distance);
            let spatial = SpatialPlayer::connect_new(
                self.device_sink.mixer(),
                source_position,
                left_ear,
                right_ear,
            );
            spatial.set_speed(clamped_speed);
            BackendPlayer::Spatial(spatial)
        } else {
            let player = Player::connect_new(self.device_sink.mixer());
            player.set_speed(clamped_speed);
            BackendPlayer::Standard(player)
        };

        let state = PlayerState::new(backend_player, clamped_volume, config.channel);
        state.apply_composed_volume(self.master_volume, self.channel_volume(config.channel));

        let id = self.allocate_id();
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
            let channel_volume = self
                .channel_volumes
                .get(&state.channel)
                .copied()
                .unwrap_or(1.0);
            state.apply_composed_volume(self.master_volume, channel_volume);
        }
        Ok(())
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        for state in self.players.values() {
            state.apply_composed_volume(self.master_volume, self.channel_volume(state.channel));
        }
    }

    fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.channel_volumes.insert(channel, clamped);

        for state in self.players.values() {
            if state.channel == channel {
                state.apply_composed_volume(self.master_volume, clamped);
            }
        }
    }

    fn set_listener_position(&mut self, pos: [f32; 3]) {
        self.listener_position = pos;

        let (left_ear, right_ear) = Self::listener_ear_positions(pos, self.ear_half_distance);
        for state in self.players.values() {
            state.player.set_listener_ears(left_ear, right_ear);
        }
    }

    fn set_source_position(&mut self, id: PlaybackId, pos: [f32; 3]) -> GoudResult<()> {
        if let Some(state) = self.players.get(&id.0) {
            state.player.set_source_position(pos);
        }
        Ok(())
    }
}

impl std::fmt::Debug for RodioAudioProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RodioAudioProvider")
            .field("master_volume", &self.master_volume)
            .field("active_players", &self.players.len())
            .field("listener_position", &self.listener_position)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_ear_positions_math() {
        let (left, right) = RodioAudioProvider::listener_ear_positions([2.0, 3.0, 4.0], 0.2);
        assert_eq!(left, [1.8, 3.0, 4.0]);
        assert_eq!(right, [2.2, 3.0, 4.0]);
    }

    /// Single integration test to minimize audio device init/teardown cycles.
    /// Multiple RodioAudioProvider instances can cause STATUS_ACCESS_VIOLATION
    /// on Windows CI runners during concurrent Drop of cpal streams.
    #[test]
    fn test_rodio_audio_provider() {
        // May fail in CI without an audio device. That is expected.
        let Ok(mut provider) = RodioAudioProvider::new() else {
            return;
        };

        // Construction
        assert_eq!(provider.name(), "rodio");
        assert_eq!(provider.version(), "1.0.0");

        // Capabilities
        let caps = provider.audio_capabilities();
        assert!(caps.supports_spatial);
        assert_eq!(caps.max_channels, 32);

        // Lifecycle
        assert!(provider.init().is_ok());
        assert!(provider.update(0.016).is_ok());

        // Master volume with clamping
        provider.set_master_volume(0.5);
        assert_eq!(provider.master_volume, 0.5);
        provider.set_master_volume(2.0);
        assert_eq!(provider.master_volume, 1.0);
        provider.set_master_volume(-1.0);
        assert_eq!(provider.master_volume, 0.0);
        provider.set_master_volume(1.0); // Reset for further tests

        // Channel volume
        provider.set_channel_volume(AudioChannel::Music, 0.7);
        assert_eq!(
            provider.channel_volumes.get(&AudioChannel::Music),
            Some(&0.7)
        );

        // Spatial operations before and after creating a spatial player
        provider.set_listener_position([1.0, 2.0, 3.0]);
        assert!(provider
            .set_source_position(PlaybackId(0), [4.0, 5.0, 6.0])
            .is_ok());

        let id = provider
            .play(SoundHandle(1), &PlayConfig::default())
            .unwrap();
        assert_eq!(id, PlaybackId(0));

        let spatial_config = PlayConfig {
            position: Some([0.0, 0.0, 0.0]),
            ..PlayConfig::default()
        };
        let spatial_id = provider.play(SoundHandle(2), &spatial_config).unwrap();
        assert_eq!(spatial_id, PlaybackId(1));
        assert!(provider
            .set_source_position(spatial_id, [2.0, 0.0, 0.0])
            .is_ok());

        assert!(provider.stop(id).is_ok());
        assert!(provider.stop(spatial_id).is_ok());

        // Debug format
        let debug = format!("{:?}", provider);
        assert!(debug.contains("RodioAudioProvider"));
        assert!(debug.contains("master_volume"));

        // Explicit shutdown before drop
        provider.shutdown();
    }
}
