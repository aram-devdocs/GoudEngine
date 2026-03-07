//! AudioSource component for spatial audio playback.

use crate::assets::loaders::audio::AudioAsset;
use crate::assets::AssetHandle;
use crate::ecs::Component;

use super::attenuation::AttenuationModel;
use super::channel::AudioChannel;

/// AudioSource component for spatial audio playback.
///
/// Attach this component to an entity to enable audio playback. The audio system
/// will automatically handle playback, looping, volume, pitch, and spatial audio
/// based on the component's configuration.
///
/// # Fields
///
/// - `audio`: Reference to the audio asset to play
/// - `playing`: Whether the audio is currently playing
/// - `looping`: Whether the audio should loop when it finishes
/// - `volume`: Volume multiplier (0.0 = silent, 1.0 = full volume)
/// - `pitch`: Pitch multiplier (0.5 = half speed, 2.0 = double speed)
/// - `channel`: Audio channel for grouping and mixing
/// - `auto_play`: Whether to start playing automatically when spawned
/// - `spatial`: Whether to apply spatial audio (requires Transform)
/// - `max_distance`: Maximum distance for spatial audio attenuation
/// - `attenuation`: Distance-based volume falloff model
/// - `sink_id`: Internal audio sink ID (managed by audio system)
///
/// # Examples
///
/// See module-level documentation for usage examples.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioSource {
    /// Reference to the audio asset to play
    // TODO(#219): Serialize as asset path string for full scene round-trip
    #[serde(skip)]
    pub audio: AssetHandle<AudioAsset>,
    /// Whether the audio is currently playing
    pub playing: bool,
    /// Whether the audio should loop when it finishes
    pub looping: bool,
    /// Volume multiplier (0.0 = silent, 1.0 = full volume)
    pub volume: f32,
    /// Pitch multiplier (0.5 = half speed, 2.0 = double speed)
    pub pitch: f32,
    /// Audio channel for grouping and mixing
    pub channel: AudioChannel,
    /// Whether to start playing automatically when spawned
    pub auto_play: bool,
    /// Whether to apply spatial audio (requires Transform)
    pub spatial: bool,
    /// Maximum distance for spatial audio attenuation
    pub max_distance: f32,
    /// Distance-based volume falloff model
    pub attenuation: AttenuationModel,
    /// Internal audio sink ID (managed by audio system)
    #[serde(skip)]
    pub(crate) sink_id: Option<u64>,
}

impl AudioSource {
    /// Creates a new AudioSource with default settings.
    ///
    /// # Arguments
    ///
    /// - `audio`: Reference to the audio asset to play
    ///
    /// # Default Values
    ///
    /// - playing: false (stopped)
    /// - looping: false (one-shot)
    /// - volume: 1.0 (full volume)
    /// - pitch: 1.0 (normal speed)
    /// - channel: SFX
    /// - auto_play: false
    /// - spatial: false (non-spatial)
    /// - max_distance: 100.0
    /// - attenuation: InverseDistance
    /// - sink_id: None
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioSource;
    /// use goud_engine::assets::AssetHandle;
    /// use goud_engine::assets::loaders::audio::AudioAsset;
    ///
    /// let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
    /// let source = AudioSource::new(audio_handle);
    ///
    /// assert_eq!(source.playing, false);
    /// assert_eq!(source.volume, 1.0);
    /// assert_eq!(source.pitch, 1.0);
    /// ```
    pub fn new(audio: AssetHandle<AudioAsset>) -> Self {
        Self {
            audio,
            playing: false,
            looping: false,
            volume: 1.0,
            pitch: 1.0,
            channel: AudioChannel::default(),
            auto_play: false,
            spatial: false,
            max_distance: 100.0,
            attenuation: AttenuationModel::default(),
            sink_id: None,
        }
    }

    /// Sets the volume (0.0-1.0, clamped).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioSource;
    /// use goud_engine::assets::AssetHandle;
    /// use goud_engine::assets::loaders::audio::AudioAsset;
    ///
    /// let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
    /// let source = AudioSource::new(audio_handle).with_volume(0.5);
    /// assert_eq!(source.volume, 0.5);
    /// ```
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets the pitch (0.5-2.0, clamped).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioSource;
    /// use goud_engine::assets::AssetHandle;
    /// use goud_engine::assets::loaders::audio::AudioAsset;
    ///
    /// let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
    /// let source = AudioSource::new(audio_handle).with_pitch(1.5);
    /// assert_eq!(source.pitch, 1.5);
    /// ```
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.5, 2.0);
        self
    }

    /// Sets whether the audio should loop.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Sets the audio channel.
    pub fn with_channel(mut self, channel: AudioChannel) -> Self {
        self.channel = channel;
        self
    }

    /// Sets whether to start playing automatically when spawned.
    pub fn with_auto_play(mut self, auto_play: bool) -> Self {
        self.auto_play = auto_play;
        self
    }

    /// Sets whether to apply spatial audio (requires Transform component).
    pub fn with_spatial(mut self, spatial: bool) -> Self {
        self.spatial = spatial;
        self
    }

    /// Sets the maximum distance for spatial audio attenuation.
    pub fn with_max_distance(mut self, max_distance: f32) -> Self {
        self.max_distance = max_distance.max(0.1);
        self
    }

    /// Sets the attenuation model for spatial audio.
    pub fn with_attenuation(mut self, attenuation: AttenuationModel) -> Self {
        self.attenuation = attenuation;
        self
    }

    /// Starts playing the audio.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pauses the audio (retains playback position).
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stops the audio (resets playback position).
    pub fn stop(&mut self) {
        self.playing = false;
        self.sink_id = None;
    }

    /// Returns whether the audio is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Returns whether the audio is spatial.
    pub fn is_spatial(&self) -> bool {
        self.spatial
    }

    /// Returns whether the audio has an active sink.
    pub fn has_sink(&self) -> bool {
        self.sink_id.is_some()
    }

    /// Sets the internal sink ID (managed by audio system).
    #[cfg(test)]
    pub(crate) fn set_sink_id(&mut self, id: Option<u64>) {
        self.sink_id = id;
    }

    /// Returns the internal sink ID.
    #[cfg(test)]
    pub(crate) fn sink_id(&self) -> Option<u64> {
        self.sink_id
    }
}

impl Component for AudioSource {}

impl Default for AudioSource {
    fn default() -> Self {
        Self::new(AssetHandle::default())
    }
}

impl std::fmt::Display for AudioSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AudioSource(playing={}, looping={}, volume={:.2}, pitch={:.2}, channel={}, spatial={})",
            self.playing, self.looping, self.volume, self.pitch, self.channel, self.spatial
        )
    }
}
