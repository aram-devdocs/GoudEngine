//! [`AudioAsset`] definition and implementation.

use crate::assets::{Asset, AssetType};
use std::path::{Path, PathBuf};

use super::format::AudioFormat;

/// Storage strategy for audio data.
///
/// Small files are kept fully in memory for low-latency playback.
/// Large files store only a path and stream from disk during playback,
/// avoiding high memory usage for music tracks.
#[derive(Clone, PartialEq, Debug)]
pub enum AudioData {
    /// Encoded audio bytes held entirely in memory.
    InMemory(Vec<u8>),
    /// Reference to an on-disk file for streaming playback.
    Streaming {
        /// Filesystem path to the audio file.
        path: PathBuf,
        /// Size of the file in bytes (cached at load time).
        size_bytes: u64,
    },
}

/// Audio asset containing encoded audio data and pre-computed metadata.
///
/// The `data` field stores either the original encoded file bytes or a path
/// for streaming from disk. Decoding happens at playback time in
/// `AudioManager::play()`. Metadata (sample rate, channels, duration) is
/// extracted during loading so callers can query properties without decoding.
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
    /// Audio data storage (in-memory or streaming reference).
    data: AudioData,
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
            data: AudioData::InMemory(Vec::new()),
            sample_rate: 44100,
            channel_count: 2,
            format: AudioFormat::Wav,
            duration_secs: 0.0,
        }
    }

    /// Creates a new audio asset with the given parameters.
    ///
    /// # Arguments
    /// * `data` - Audio data storage (in-memory bytes or streaming path)
    /// * `sample_rate` - Sample rate in Hz
    /// * `channel_count` - Number of channels (1 or 2)
    /// * `format` - Original file format
    /// * `duration_secs` - Pre-computed duration in seconds
    pub fn new(
        data: AudioData,
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

    /// Returns the raw audio data bytes, or `None` for streaming assets.
    pub fn data(&self) -> Option<&[u8]> {
        match &self.data {
            AudioData::InMemory(bytes) => Some(bytes),
            AudioData::Streaming { .. } => None,
        }
    }

    /// Returns a reference to the underlying [`AudioData`] storage.
    pub(crate) fn audio_data(&self) -> &AudioData {
        &self.data
    }

    /// Returns `true` if this asset streams from disk instead of memory.
    pub fn is_streaming(&self) -> bool {
        matches!(&self.data, AudioData::Streaming { .. })
    }

    /// Returns the file path for streaming assets, or `None` for in-memory.
    pub fn file_path(&self) -> Option<&Path> {
        match &self.data {
            AudioData::Streaming { path, .. } => Some(path),
            AudioData::InMemory(_) => None,
        }
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
        match &self.data {
            AudioData::InMemory(bytes) => bytes.is_empty(),
            AudioData::Streaming { size_bytes, .. } => *size_bytes == 0,
        }
    }

    /// Returns the size of the audio data in bytes.
    pub fn size_bytes(&self) -> usize {
        match &self.data {
            AudioData::InMemory(bytes) => bytes.len(),
            AudioData::Streaming { size_bytes, .. } => *size_bytes as usize,
        }
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
