//! [`AudioAsset`] definition and implementation.

use crate::assets::{Asset, AssetType};

use super::format::AudioFormat;

/// Audio asset containing decoded audio data.
///
/// This is a stub implementation. In Phase 6, this will be expanded to include:
/// - Decoded PCM audio data
/// - Sample rate, channel count, bit depth
/// - Audio format information
/// - Integration with rodio audio backend
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
    /// Raw audio data (currently empty stub).
    data: Vec<u8>,
    /// Sample rate in Hz (e.g., 44100, 48000).
    sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo).
    channel_count: u16,
    /// Original audio file format.
    format: AudioFormat,
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
        }
    }

    /// Creates a new audio asset with the given parameters (stub).
    ///
    /// # Arguments
    /// * `data` - Raw audio data bytes
    /// * `sample_rate` - Sample rate in Hz
    /// * `channel_count` - Number of channels (1 or 2)
    /// * `format` - Original file format
    pub fn new(data: Vec<u8>, sample_rate: u32, channel_count: u16, format: AudioFormat) -> Self {
        Self {
            data,
            sample_rate,
            channel_count,
            format,
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

    /// Returns the duration of the audio in seconds (stub - returns 0.0).
    ///
    /// TODO: Calculate actual duration based on sample_rate, channel_count, and bit depth.
    pub fn duration_secs(&self) -> f32 {
        0.0
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
