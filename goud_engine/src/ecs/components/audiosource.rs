//! ## Play a Sound Effect
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::components::{AudioSource, AudioChannel};
//! use goud_engine::assets::AssetHandle;
//! use goud_engine::assets::loaders::audio::AudioAsset;
//!
//! let mut world = World::new();
//!
//! // Assume we have a loaded audio asset
//! let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
//!
//! // Spawn entity with audio source
//! let entity = world.spawn_empty();
//! world.insert(entity, AudioSource::new(audio_handle)
//!     .with_volume(0.8)
//!     .with_looping(false)
//!     .with_auto_play(true)
//!     .with_channel(AudioChannel::SFX));
//! ```
//!
//! ## Background Music with Looping
//!
//! ```
//! use goud_engine::ecs::components::{AudioSource, AudioChannel};
//! use goud_engine::assets::AssetHandle;
//! use goud_engine::assets::loaders::audio::AudioAsset;
//!
//! let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
//!
//! let music = AudioSource::new(audio_handle)
//!     .with_volume(0.5)
//!     .with_looping(true)
//!     .with_auto_play(true)
//!     .with_channel(AudioChannel::Music);
//! ```
//!
//! ## Spatial Audio with Attenuation
//!
//! ```
//! use goud_engine::ecs::components::{AudioSource, AudioChannel, AttenuationModel};
//! use goud_engine::assets::AssetHandle;
//! use goud_engine::assets::loaders::audio::AudioAsset;
//!
//! let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
//!
//! let spatial_audio = AudioSource::new(audio_handle)
//!     .with_volume(1.0)
//!     .with_spatial(true)
//!     .with_max_distance(100.0)
//!     .with_attenuation(AttenuationModel::InverseDistance)
//!     .with_channel(AudioChannel::Ambience);
//! ```

use crate::assets::loaders::audio::AudioAsset;
use crate::assets::AssetHandle;
use crate::ecs::Component;

/// Audio channel enumeration for audio mixing and grouping.
///
/// Channels allow you to group audio sources together for volume control,
/// filtering, and organization. Each audio source belongs to one channel.
///
/// # Built-in Channels
///
/// - **Music**: Background music tracks (typically looped)
/// - **SFX**: Sound effects (footsteps, impacts, UI clicks)
/// - **Voice**: Voice-overs, dialogue, speech
/// - **Ambience**: Ambient environment sounds (wind, rain, room tone)
/// - **UI**: User interface sounds (button clicks, menu navigation)
/// - **Custom**: User-defined channels (bits 5-31)
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::AudioChannel;
///
/// let music = AudioChannel::Music;
/// let sfx = AudioChannel::SFX;
/// let custom = AudioChannel::Custom(8); // Custom channel ID 8
///
/// assert_eq!(music.id(), 0);
/// assert_eq!(sfx.id(), 1);
/// assert_eq!(custom.id(), 8);
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioChannel {
    /// Background music tracks (channel ID: 0)
    Music = 0,
    /// Sound effects (channel ID: 1)
    SFX = 1,
    /// Voice-overs and dialogue (channel ID: 2)
    Voice = 2,
    /// Ambient environment sounds (channel ID: 3)
    Ambience = 3,
    /// User interface sounds (channel ID: 4)
    UI = 4,
    /// Custom channel (ID 5-31)
    Custom(u8),
}

impl AudioChannel {
    /// Returns the numeric channel ID (0-31).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioChannel;
    ///
    /// assert_eq!(AudioChannel::Music.id(), 0);
    /// assert_eq!(AudioChannel::SFX.id(), 1);
    /// assert_eq!(AudioChannel::Custom(10).id(), 10);
    /// ```
    pub fn id(&self) -> u8 {
        match self {
            AudioChannel::Music => 0,
            AudioChannel::SFX => 1,
            AudioChannel::Voice => 2,
            AudioChannel::Ambience => 3,
            AudioChannel::UI => 4,
            AudioChannel::Custom(id) => *id,
        }
    }

    /// Returns the channel name for debugging.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioChannel;
    ///
    /// assert_eq!(AudioChannel::Music.name(), "Music");
    /// assert_eq!(AudioChannel::SFX.name(), "SFX");
    /// assert_eq!(AudioChannel::Custom(10).name(), "Custom(10)");
    /// ```
    pub fn name(&self) -> String {
        match self {
            AudioChannel::Music => "Music".to_string(),
            AudioChannel::SFX => "SFX".to_string(),
            AudioChannel::Voice => "Voice".to_string(),
            AudioChannel::Ambience => "Ambience".to_string(),
            AudioChannel::UI => "UI".to_string(),
            AudioChannel::Custom(id) => format!("Custom({id})"),
        }
    }
}

impl Default for AudioChannel {
    /// Returns `AudioChannel::SFX` as the default.
    fn default() -> Self {
        AudioChannel::SFX
    }
}

impl std::fmt::Display for AudioChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Audio attenuation model for distance-based volume falloff.
///
/// Controls how audio volume decreases with distance from the listener.
/// Different models provide different falloff curves for realistic or
/// stylized audio behavior.
///
/// # Models
///
/// - **Linear**: Linear falloff (volume = 1 - distance/max_distance)
/// - **InverseDistance**: Realistic inverse distance falloff (volume = 1 / (1 + distance))
/// - **Exponential**: Exponential falloff (volume = (1 - distance/max_distance)^rolloff)
/// - **None**: No attenuation (constant volume regardless of distance)
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::AttenuationModel;
///
/// let linear = AttenuationModel::Linear;
/// let inverse = AttenuationModel::InverseDistance;
/// let exponential = AttenuationModel::Exponential { rolloff: 2.0 };
/// let none = AttenuationModel::None;
///
/// assert_eq!(linear.name(), "Linear");
/// assert_eq!(inverse.name(), "InverseDistance");
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttenuationModel {
    /// Linear falloff: volume = max(0, 1 - distance/max_distance)
    Linear,
    /// Inverse distance falloff: volume = 1 / (1 + distance)
    InverseDistance,
    /// Exponential falloff: volume = max(0, (1 - distance/max_distance)^rolloff)
    Exponential {
        /// The exponent for the falloff curve.
        rolloff: f32,
    },
    /// No attenuation (constant volume)
    None,
}

impl AttenuationModel {
    /// Returns the model name for debugging.
    pub fn name(&self) -> &str {
        match self {
            AttenuationModel::Linear => "Linear",
            AttenuationModel::InverseDistance => "InverseDistance",
            AttenuationModel::Exponential { .. } => "Exponential",
            AttenuationModel::None => "None",
        }
    }

    /// Computes the attenuation factor (0.0-1.0) based on distance.
    ///
    /// # Arguments
    ///
    /// - `distance`: Distance from listener (must be >= 0)
    /// - `max_distance`: Maximum distance for attenuation (must be > 0)
    ///
    /// # Returns
    ///
    /// Volume multiplier in range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AttenuationModel;
    ///
    /// let model = AttenuationModel::Linear;
    /// assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
    /// assert_eq!(model.compute_attenuation(50.0, 100.0), 0.5);
    /// assert_eq!(model.compute_attenuation(100.0, 100.0), 0.0);
    /// assert_eq!(model.compute_attenuation(150.0, 100.0), 0.0); // Beyond max
    /// ```
    pub fn compute_attenuation(&self, distance: f32, max_distance: f32) -> f32 {
        match self {
            AttenuationModel::Linear => {
                if distance >= max_distance {
                    0.0
                } else {
                    (1.0 - distance / max_distance).max(0.0)
                }
            }
            AttenuationModel::InverseDistance => 1.0 / (1.0 + distance),
            AttenuationModel::Exponential { rolloff } => {
                if distance >= max_distance {
                    0.0
                } else {
                    ((1.0 - distance / max_distance).powf(*rolloff)).max(0.0)
                }
            }
            AttenuationModel::None => 1.0,
        }
    }
}

impl Default for AttenuationModel {
    /// Returns `AttenuationModel::InverseDistance` as the default (most realistic).
    fn default() -> Self {
        AttenuationModel::InverseDistance
    }
}

impl std::fmt::Display for AttenuationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttenuationModel::Exponential { rolloff } => {
                write!(f, "Exponential(rolloff={rolloff})")
            }
            other => write!(f, "{}", other.name()),
        }
    }
}

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
#[derive(Clone, Debug)]
pub struct AudioSource {
    /// Reference to the audio asset to play
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
    #[allow(dead_code)]
    pub(crate) fn set_sink_id(&mut self, id: Option<u64>) {
        self.sink_id = id;
    }

    /// Returns the internal sink ID.
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // AudioChannel tests
    #[test]
    fn test_audio_channel_id() {
        assert_eq!(AudioChannel::Music.id(), 0);
        assert_eq!(AudioChannel::SFX.id(), 1);
        assert_eq!(AudioChannel::Voice.id(), 2);
        assert_eq!(AudioChannel::Ambience.id(), 3);
        assert_eq!(AudioChannel::UI.id(), 4);
        assert_eq!(AudioChannel::Custom(10).id(), 10);
    }

    #[test]
    fn test_audio_channel_name() {
        assert_eq!(AudioChannel::Music.name(), "Music");
        assert_eq!(AudioChannel::SFX.name(), "SFX");
        assert_eq!(AudioChannel::Voice.name(), "Voice");
        assert_eq!(AudioChannel::Ambience.name(), "Ambience");
        assert_eq!(AudioChannel::UI.name(), "UI");
        assert_eq!(AudioChannel::Custom(10).name(), "Custom(10)");
    }

    #[test]
    fn test_audio_channel_default() {
        assert_eq!(AudioChannel::default(), AudioChannel::SFX);
    }

    #[test]
    fn test_audio_channel_display() {
        assert_eq!(format!("{}", AudioChannel::Music), "Music");
        assert_eq!(format!("{}", AudioChannel::Custom(5)), "Custom(5)");
    }

    #[test]
    fn test_audio_channel_clone_copy() {
        let channel = AudioChannel::Music;
        let cloned = channel;
        assert_eq!(channel, cloned);
    }

    #[test]
    fn test_audio_channel_eq_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AudioChannel::Music);
        set.insert(AudioChannel::SFX);
        assert!(set.contains(&AudioChannel::Music));
        assert!(!set.contains(&AudioChannel::Voice));
    }

    // AttenuationModel tests
    #[test]
    fn test_attenuation_model_name() {
        assert_eq!(AttenuationModel::Linear.name(), "Linear");
        assert_eq!(AttenuationModel::InverseDistance.name(), "InverseDistance");
        assert_eq!(
            AttenuationModel::Exponential { rolloff: 2.0 }.name(),
            "Exponential"
        );
        assert_eq!(AttenuationModel::None.name(), "None");
    }

    #[test]
    fn test_attenuation_linear() {
        let model = AttenuationModel::Linear;
        assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
        assert_eq!(model.compute_attenuation(50.0, 100.0), 0.5);
        assert_eq!(model.compute_attenuation(100.0, 100.0), 0.0);
        assert_eq!(model.compute_attenuation(150.0, 100.0), 0.0);
    }

    #[test]
    fn test_attenuation_inverse_distance() {
        let model = AttenuationModel::InverseDistance;
        assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
        assert!((model.compute_attenuation(1.0, 100.0) - 0.5).abs() < 0.01);
        assert!(model.compute_attenuation(50.0, 100.0) < 1.0);
    }

    #[test]
    fn test_attenuation_exponential() {
        let model = AttenuationModel::Exponential { rolloff: 2.0 };
        assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
        assert_eq!(model.compute_attenuation(50.0, 100.0), 0.25); // (0.5)^2
        assert_eq!(model.compute_attenuation(100.0, 100.0), 0.0);
    }

    #[test]
    fn test_attenuation_none() {
        let model = AttenuationModel::None;
        assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
        assert_eq!(model.compute_attenuation(50.0, 100.0), 1.0);
        assert_eq!(model.compute_attenuation(1000.0, 100.0), 1.0);
    }

    #[test]
    fn test_attenuation_default() {
        let model = AttenuationModel::default();
        assert_eq!(model.name(), "InverseDistance");
    }

    #[test]
    fn test_attenuation_display() {
        assert_eq!(format!("{}", AttenuationModel::Linear), "Linear");
        assert_eq!(
            format!("{}", AttenuationModel::Exponential { rolloff: 3.0 }),
            "Exponential(rolloff=3)"
        );
    }

    // AudioSource tests
    #[test]
    fn test_audio_source_new() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle);

        assert_eq!(source.playing, false);
        assert_eq!(source.looping, false);
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.pitch, 1.0);
        assert_eq!(source.channel, AudioChannel::SFX);
        assert_eq!(source.auto_play, false);
        assert_eq!(source.spatial, false);
        assert_eq!(source.max_distance, 100.0);
        assert_eq!(source.attenuation.name(), "InverseDistance");
        assert_eq!(source.sink_id, None);
    }

    #[test]
    fn test_audio_source_with_volume() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle).with_volume(0.5);
        assert_eq!(source.volume, 0.5);

        // Test clamping
        let source = AudioSource::new(handle).with_volume(-0.1);
        assert_eq!(source.volume, 0.0);

        let source = AudioSource::new(handle).with_volume(1.5);
        assert_eq!(source.volume, 1.0);
    }

    #[test]
    fn test_audio_source_with_pitch() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle).with_pitch(1.5);
        assert_eq!(source.pitch, 1.5);

        // Test clamping
        let source = AudioSource::new(handle).with_pitch(0.1);
        assert_eq!(source.pitch, 0.5);

        let source = AudioSource::new(handle).with_pitch(3.0);
        assert_eq!(source.pitch, 2.0);
    }

    #[test]
    fn test_audio_source_builder_pattern() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle)
            .with_volume(0.8)
            .with_pitch(1.2)
            .with_looping(true)
            .with_channel(AudioChannel::Music)
            .with_auto_play(true)
            .with_spatial(true)
            .with_max_distance(200.0)
            .with_attenuation(AttenuationModel::Linear);

        assert_eq!(source.volume, 0.8);
        assert_eq!(source.pitch, 1.2);
        assert_eq!(source.looping, true);
        assert_eq!(source.channel, AudioChannel::Music);
        assert_eq!(source.auto_play, true);
        assert_eq!(source.spatial, true);
        assert_eq!(source.max_distance, 200.0);
        assert_eq!(source.attenuation.name(), "Linear");
    }

    #[test]
    fn test_audio_source_play_pause_stop() {
        let handle = AssetHandle::default();
        let mut source = AudioSource::new(handle);

        assert_eq!(source.is_playing(), false);

        source.play();
        assert_eq!(source.is_playing(), true);

        source.pause();
        assert_eq!(source.is_playing(), false);

        source.play();
        assert_eq!(source.is_playing(), true);

        source.stop();
        assert_eq!(source.is_playing(), false);
        assert_eq!(source.sink_id, None);
    }

    #[test]
    fn test_audio_source_is_spatial() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle);
        assert_eq!(source.is_spatial(), false);

        let source = source.with_spatial(true);
        assert_eq!(source.is_spatial(), true);
    }

    #[test]
    fn test_audio_source_sink_id() {
        let handle = AssetHandle::default();
        let mut source = AudioSource::new(handle);

        assert_eq!(source.has_sink(), false);
        assert_eq!(source.sink_id(), None);

        source.set_sink_id(Some(42));
        assert_eq!(source.has_sink(), true);
        assert_eq!(source.sink_id(), Some(42));

        source.set_sink_id(None);
        assert_eq!(source.has_sink(), false);
    }

    #[test]
    fn test_audio_source_default() {
        let source = AudioSource::default();
        assert_eq!(source.playing, false);
        assert_eq!(source.volume, 1.0);
    }

    #[test]
    fn test_audio_source_display() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle)
            .with_volume(0.75)
            .with_pitch(1.5)
            .with_channel(AudioChannel::Music);

        let display = format!("{source}");
        assert!(display.contains("playing=false"));
        assert!(display.contains("volume=0.75"));
        assert!(display.contains("pitch=1.50"));
        assert!(display.contains("channel=Music"));
    }

    #[test]
    fn test_audio_source_component() {
        let handle = AssetHandle::default();
        let _source: Box<dyn Component> = Box::new(AudioSource::new(handle));
    }

    #[test]
    fn test_audio_source_clone() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle).with_volume(0.5);
        let cloned = source.clone();
        assert_eq!(cloned.volume, 0.5);
    }

    #[test]
    fn test_audio_source_debug() {
        let handle = AssetHandle::default();
        let source = AudioSource::new(handle);
        let debug = format!("{source:?}");
        assert!(debug.contains("AudioSource"));
    }
}
