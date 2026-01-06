//! Audio asset loading (stub implementation).
//!
//! This module provides basic types for audio assets. Full audio decoding
//! and playback will be implemented in Phase 6 with the rodio integration.

use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetType, LoadContext};
use std::fmt;

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

/// Audio file format.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum AudioFormat {
    /// WAV (Waveform Audio File Format).
    #[default]
    Wav = 0,
    /// MP3 (MPEG-1 Audio Layer III).
    Mp3 = 1,
    /// OGG Vorbis.
    Ogg = 2,
    /// FLAC (Free Lossless Audio Codec).
    Flac = 3,
    /// Unknown format.
    Unknown = 255,
}

impl AudioFormat {
    /// Returns the file extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Flac => "flac",
            AudioFormat::Unknown => "",
        }
    }

    /// Returns the human-readable name of this format.
    pub fn name(self) -> &'static str {
        match self {
            AudioFormat::Wav => "WAV",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Ogg => "OGG Vorbis",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Unknown => "Unknown",
        }
    }

    /// Returns the format corresponding to the given file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "wav" => AudioFormat::Wav,
            "mp3" => AudioFormat::Mp3,
            "ogg" => AudioFormat::Ogg,
            "flac" => AudioFormat::Flac,
            _ => AudioFormat::Unknown,
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Audio loading settings.
#[derive(Clone, Debug)]
pub struct AudioSettings {
    /// Whether to load the entire audio file into memory.
    /// If false, audio will be streamed from disk (not yet implemented).
    pub preload: bool,
    /// Target sample rate (0 = use original).
    pub target_sample_rate: u32,
    /// Target channel count (0 = use original, 1 = mono, 2 = stereo).
    pub target_channel_count: u16,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            preload: true,
            target_sample_rate: 0,   // Use original
            target_channel_count: 0, // Use original
        }
    }
}

/// Audio asset loader (stub implementation).
///
/// This loader recognizes audio file extensions but returns empty AudioAsset stubs.
/// Full audio decoding will be implemented in Phase 6 with rodio integration.
///
/// # Supported Formats
/// - WAV (.wav)
/// - MP3 (.mp3)
/// - OGG Vorbis (.ogg)
/// - FLAC (.flac)
///
/// # Example
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::audio::AudioLoader};
///
/// let mut server = AssetServer::new();
/// server.register_loader(AudioLoader::default());
///
/// // This will return an empty stub until Phase 6
/// let audio_handle = server.load::<AudioAsset>("sounds/jump.wav");
/// ```
#[derive(Clone, Debug)]
pub struct AudioLoader {
    // Settings are intentionally unused in the stub implementation.
    // They will be used in Phase 6 when actual audio decoding is implemented.
    #[allow(dead_code)]
    settings: AudioSettings,
}

impl AudioLoader {
    /// Creates a new audio loader with default settings.
    pub fn new() -> Self {
        Self {
            settings: AudioSettings::default(),
        }
    }

    /// Creates a new audio loader with custom settings.
    pub fn with_settings(settings: AudioSettings) -> Self {
        Self { settings }
    }
}

impl Default for AudioLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader for AudioLoader {
    type Asset = AudioAsset;
    type Settings = AudioSettings;

    fn extensions(&self) -> &[&str] {
        AudioAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        // Determine format from file extension
        let format = context
            .extension()
            .map(AudioFormat::from_extension)
            .unwrap_or(AudioFormat::Unknown);

        // TODO Phase 6: Implement actual audio decoding with rodio
        // For now, return a stub with the raw bytes and default settings
        Ok(AudioAsset::new(bytes.to_vec(), 44100, 2, format))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // AudioAsset Tests
    // ============================================================================

    #[test]
    fn test_audio_asset_empty() {
        let audio = AudioAsset::empty();
        assert!(audio.is_empty());
        assert_eq!(audio.sample_rate(), 44100);
        assert_eq!(audio.channel_count(), 2);
        assert_eq!(audio.format(), AudioFormat::Wav);
        assert_eq!(audio.size_bytes(), 0);
    }

    #[test]
    fn test_audio_asset_new() {
        let data = vec![1, 2, 3, 4];
        let audio = AudioAsset::new(data.clone(), 48000, 1, AudioFormat::Mp3);

        assert_eq!(audio.data(), &data);
        assert_eq!(audio.sample_rate(), 48000);
        assert_eq!(audio.channel_count(), 1);
        assert_eq!(audio.format(), AudioFormat::Mp3);
        assert_eq!(audio.size_bytes(), 4);
        assert!(!audio.is_empty());
    }

    #[test]
    fn test_audio_asset_is_mono() {
        let mono = AudioAsset::new(vec![], 44100, 1, AudioFormat::Wav);
        let stereo = AudioAsset::new(vec![], 44100, 2, AudioFormat::Wav);

        assert!(mono.is_mono());
        assert!(!mono.is_stereo());
        assert!(!stereo.is_mono());
        assert!(stereo.is_stereo());
    }

    #[test]
    fn test_audio_asset_duration_secs() {
        // Stub always returns 0.0
        let audio = AudioAsset::new(vec![1, 2, 3, 4], 44100, 2, AudioFormat::Wav);
        assert_eq!(audio.duration_secs(), 0.0);
    }

    #[test]
    fn test_audio_asset_clone() {
        let audio1 = AudioAsset::new(vec![1, 2, 3], 48000, 1, AudioFormat::Ogg);
        let audio2 = audio1.clone();

        assert_eq!(audio1, audio2);
    }

    #[test]
    fn test_audio_asset_debug() {
        let audio = AudioAsset::empty();
        let debug_str = format!("{:?}", audio);
        assert!(debug_str.contains("AudioAsset"));
    }

    #[test]
    fn test_audio_asset_trait() {
        assert_eq!(AudioAsset::asset_type_name(), "AudioAsset");
        assert_eq!(AudioAsset::asset_type(), AssetType::Audio);
        assert_eq!(AudioAsset::extensions(), &["wav", "mp3", "ogg", "flac"]);
    }

    // ============================================================================
    // AudioFormat Tests
    // ============================================================================

    #[test]
    fn test_audio_format_extension() {
        assert_eq!(AudioFormat::Wav.extension(), "wav");
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::Ogg.extension(), "ogg");
        assert_eq!(AudioFormat::Flac.extension(), "flac");
        assert_eq!(AudioFormat::Unknown.extension(), "");
    }

    #[test]
    fn test_audio_format_name() {
        assert_eq!(AudioFormat::Wav.name(), "WAV");
        assert_eq!(AudioFormat::Mp3.name(), "MP3");
        assert_eq!(AudioFormat::Ogg.name(), "OGG Vorbis");
        assert_eq!(AudioFormat::Flac.name(), "FLAC");
        assert_eq!(AudioFormat::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_audio_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("WAV"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("ogg"), AudioFormat::Ogg);
        assert_eq!(AudioFormat::from_extension("flac"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_extension("xyz"), AudioFormat::Unknown);
    }

    #[test]
    fn test_audio_format_default() {
        assert_eq!(AudioFormat::default(), AudioFormat::Wav);
    }

    #[test]
    fn test_audio_format_display() {
        assert_eq!(format!("{}", AudioFormat::Wav), "WAV");
        assert_eq!(format!("{}", AudioFormat::Mp3), "MP3");
        assert_eq!(format!("{}", AudioFormat::Ogg), "OGG Vorbis");
    }

    #[test]
    fn test_audio_format_clone() {
        let format1 = AudioFormat::Mp3;
        let format2 = format1;
        assert_eq!(format1, format2);
    }

    #[test]
    fn test_audio_format_eq() {
        assert_eq!(AudioFormat::Wav, AudioFormat::Wav);
        assert_ne!(AudioFormat::Wav, AudioFormat::Mp3);
    }

    #[test]
    fn test_audio_format_debug() {
        let format = AudioFormat::Ogg;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Ogg"));
    }

    // ============================================================================
    // AudioSettings Tests
    // ============================================================================

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert!(settings.preload);
        assert_eq!(settings.target_sample_rate, 0);
        assert_eq!(settings.target_channel_count, 0);
    }

    #[test]
    fn test_audio_settings_custom() {
        let settings = AudioSettings {
            preload: false,
            target_sample_rate: 22050,
            target_channel_count: 1,
        };

        assert!(!settings.preload);
        assert_eq!(settings.target_sample_rate, 22050);
        assert_eq!(settings.target_channel_count, 1);
    }

    #[test]
    fn test_audio_settings_clone() {
        let settings1 = AudioSettings::default();
        let settings2 = settings1.clone();

        assert_eq!(settings1.preload, settings2.preload);
        assert_eq!(settings1.target_sample_rate, settings2.target_sample_rate);
    }

    #[test]
    fn test_audio_settings_debug() {
        let settings = AudioSettings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("AudioSettings"));
    }

    // ============================================================================
    // AudioLoader Tests
    // ============================================================================

    #[test]
    fn test_audio_loader_new() {
        let loader = AudioLoader::new();
        assert!(loader.settings.preload);
    }

    #[test]
    fn test_audio_loader_default() {
        let loader = AudioLoader::default();
        assert!(loader.settings.preload);
    }

    #[test]
    fn test_audio_loader_with_settings() {
        let settings = AudioSettings {
            preload: false,
            target_sample_rate: 22050,
            target_channel_count: 1,
        };

        let loader = AudioLoader::with_settings(settings);
        assert!(!loader.settings.preload);
        assert_eq!(loader.settings.target_sample_rate, 22050);
    }

    #[test]
    fn test_audio_loader_extensions() {
        let loader = AudioLoader::new();
        assert_eq!(loader.extensions(), &["wav", "mp3", "ogg", "flac"]);
    }

    #[test]
    fn test_audio_loader_load_wav() {
        let loader = AudioLoader::new();
        let mut context = LoadContext::new("test.wav".into());
        let bytes = vec![1, 2, 3, 4];
        let settings = AudioSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert_eq!(audio.format(), AudioFormat::Wav);
        assert_eq!(audio.data(), &bytes);
    }

    #[test]
    fn test_audio_loader_load_mp3() {
        let loader = AudioLoader::new();
        let mut context = LoadContext::new("sound.mp3".into());
        let bytes = vec![5, 6, 7, 8];
        let settings = AudioSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert_eq!(audio.format(), AudioFormat::Mp3);
    }

    #[test]
    fn test_audio_loader_load_ogg() {
        let loader = AudioLoader::new();
        let mut context = LoadContext::new("music.ogg".into());
        let bytes = vec![9, 10, 11, 12];
        let settings = AudioSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert_eq!(audio.format(), AudioFormat::Ogg);
    }

    #[test]
    fn test_audio_loader_load_flac() {
        let loader = AudioLoader::new();
        let mut context = LoadContext::new("track.flac".into());
        let bytes = vec![13, 14, 15, 16];
        let settings = AudioSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert_eq!(audio.format(), AudioFormat::Flac);
    }

    #[test]
    fn test_audio_loader_load_unknown_extension() {
        let loader = AudioLoader::new();
        let mut context = LoadContext::new("file.xyz".into());
        let bytes = vec![1, 2, 3, 4];
        let settings = AudioSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert_eq!(audio.format(), AudioFormat::Unknown);
    }

    #[test]
    fn test_audio_loader_clone() {
        let loader1 = AudioLoader::new();
        let loader2 = loader1.clone();

        assert_eq!(loader1.settings.preload, loader2.settings.preload);
    }

    #[test]
    fn test_audio_loader_debug() {
        let loader = AudioLoader::new();
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("AudioLoader"));
    }

    // ============================================================================
    // Thread Safety Tests
    // ============================================================================

    #[test]
    fn test_audio_asset_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AudioAsset>();
    }

    #[test]
    fn test_audio_asset_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AudioAsset>();
    }

    #[test]
    fn test_audio_format_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AudioFormat>();
    }

    #[test]
    fn test_audio_format_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AudioFormat>();
    }

    #[test]
    fn test_audio_settings_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AudioSettings>();
    }

    #[test]
    fn test_audio_settings_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AudioSettings>();
    }

    #[test]
    fn test_audio_loader_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AudioLoader>();
    }

    #[test]
    fn test_audio_loader_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AudioLoader>();
    }
}
