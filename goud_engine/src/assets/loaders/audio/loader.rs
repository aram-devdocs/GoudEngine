//! [`AudioLoader`] implementing the [`AssetLoader`] trait for audio files.

use crate::assets::{asset::Asset, AssetLoadError, AssetLoader, LoadContext};

use super::{asset::AudioAsset, format::AudioFormat, settings::AudioSettings};

#[cfg(feature = "native")]
use rodio::{Decoder, Source};
#[cfg(feature = "native")]
use std::io::Cursor;

/// Audio asset loader with rodio-based validation and metadata extraction.
///
/// Under the `native` feature, this loader validates that bytes are decodable,
/// extracts sample rate, channel count, and duration, then stores the original
/// encoded bytes (decoding happens at playback time in `AudioManager::play()`).
///
/// Without the `native` feature, falls back to stub behavior.
///
/// # Supported Formats
/// - WAV (.wav)
/// - MP3 (.mp3)
/// - OGG Vorbis (.ogg)
/// - FLAC (.flac)
///
/// # Example
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::audio::{AudioLoader, AudioAsset}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(AudioLoader::default());
///
/// let audio_handle = server.load::<AudioAsset>("sounds/jump.wav");
/// ```
#[derive(Clone, Debug)]
pub struct AudioLoader {
    pub(crate) settings: AudioSettings,
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

    /// Returns a reference to this loader's settings.
    pub fn settings(&self) -> &AudioSettings {
        &self.settings
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
        settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        // Determine format from file extension
        let format = context
            .extension()
            .map(AudioFormat::from_extension)
            .unwrap_or(AudioFormat::Unknown);

        #[cfg(feature = "native")]
        {
            self.load_native(bytes, settings, format)
        }

        #[cfg(not(feature = "native"))]
        {
            let _ = settings;
            Ok(AudioAsset::new(bytes.to_vec(), 44100, 2, format, 0.0))
        }
    }
}

#[cfg(feature = "native")]
impl AudioLoader {
    fn load_native(
        &self,
        bytes: &[u8],
        _settings: &AudioSettings,
        format: AudioFormat,
    ) -> Result<AudioAsset, AssetLoadError> {
        // Validate by attempting to decode
        let cursor = Cursor::new(bytes.to_vec());
        let decoder = Decoder::new(cursor).map_err(|e| {
            AssetLoadError::decode_failed(format!("{} decode error: {}", format.name(), e))
        })?;

        // Extract metadata from the Source trait
        let detected_sample_rate: u32 = decoder.sample_rate().get();
        let detected_channels: u16 = decoder.channels().get();

        // Compute duration: prefer total_duration(), fall back to counting samples
        let duration_secs = if let Some(dur) = decoder.total_duration() {
            dur.as_secs_f32()
        } else {
            // Need a fresh decoder since the first one is partially consumed
            drop(decoder);
            let cursor2 = Cursor::new(bytes.to_vec());
            if let Ok(decoder2) = Decoder::new(cursor2) {
                let sr = decoder2.sample_rate().get();
                let ch = decoder2.channels().get();
                let sample_count = decoder2.count() as f32;
                sample_count / (sr as f32 * ch as f32)
            } else {
                0.0
            }
        };

        Ok(AudioAsset::new(
            bytes.to_vec(),
            detected_sample_rate,
            detected_channels,
            format,
            duration_secs,
        ))
    }
}
