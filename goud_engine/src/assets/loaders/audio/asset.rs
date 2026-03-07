//! [`AudioAsset`] definition and implementation.

use crate::assets::{Asset, AssetType};

use super::format::AudioFormat;

/// Audio asset containing encoded audio data and pre-computed metadata.
///
/// The `data` field stores the original encoded file bytes (WAV, OGG, etc.),
/// not decoded PCM. Decoding happens at playback time in `AudioManager::play()`.
/// Metadata (sample rate, channels, duration) is extracted during loading so
/// callers can query properties without decoding.
///
/// # Example
/// ```
/// use goud_engine::assets::loaders::AudioAsset;
///
/// let audio = AudioAsset::empty();
/// assert_eq!(audio.sample_rate(), 44100);
/// assert_eq!(audio.channel_count(), 2);
/// assert!(audio.is_empty());
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct AudioAsset {
    /// Encoded audio file bytes (not decoded PCM).
    data: Vec<u8>,
    /// Sample rate in Hz (e.g., 44100, 48000).
    sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo).
    channel_count: u16,
    /// Original audio file format.
    format: AudioFormat,
    /// Pre-computed duration in seconds.
    duration_secs: f32,
}

impl AudioAsset {
    /// Creates an empty audio asset (stub).
    ///
    /// # Example
    /// ```
    /// use goud_engine::assets::loaders::AudioAsset;
    ///
    /// let audio = AudioAsset::empty();
    /// assert!(audio.is_empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            sample_rate: 44100,
            channel_count: 2,
            format: AudioFormat::Wav,
            duration_secs: 0.0,
        }
    }

    /// Creates a new audio asset with the given parameters.
    ///
    /// # Arguments
    /// * `data` - Encoded audio file bytes (not decoded PCM)
    /// * `sample_rate` - Sample rate in Hz
    /// * `channel_count` - Number of channels (1 or 2)
    /// * `format` - Original file format
    /// * `duration_secs` - Pre-computed duration in seconds
    pub fn new(
        data: Vec<u8>,
        sample_rate: u32,
        channel_count: u16,
        format: AudioFormat,
        duration_secs: f32,
    ) -> Self {
        Self {
            data,
            sample_rate,
            channel_count,
            format,
            duration_secs,
        }
    }

    /// Returns the raw audio data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of audio channels.
    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }

    /// Returns the original audio format.
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Returns true if the audio data is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the size of the audio data in bytes.
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }

    /// Returns the pre-computed duration of the audio in seconds.
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    /// Returns the bits per sample (rodio decodes to i16, so always 16).
    pub fn bits_per_sample(&self) -> u16 {
        16
    }

    /// Returns true if this is mono audio.
    pub fn is_mono(&self) -> bool {
        self.channel_count == 1
    }

    /// Returns true if this is stereo audio.
    pub fn is_stereo(&self) -> bool {
        self.channel_count == 2
    }
}

impl Asset for AudioAsset {
    fn asset_type_name() -> &'static str {
        "AudioAsset"
    }

    fn asset_type() -> AssetType {
        AssetType::Audio
    }

    fn extensions() -> &'static [&'static str] {
        &["wav", "mp3", "ogg", "flac"]
    }
}
