//! [`AudioLoader`] implementing the [`AssetLoader`] trait for audio files.

use crate::assets::{asset::Asset, AssetLoadError, AssetLoader, LoadContext};

use super::{asset::AudioAsset, format::AudioFormat, settings::AudioSettings};

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
/// use goud_engine::assets::{AssetServer, loaders::audio::{AudioLoader, AudioAsset}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(AudioLoader::default());
///
/// // This will return an empty stub until Phase 6
/// let audio_handle = server.load::<AudioAsset>("sounds/jump.wav");
/// ```
#[derive(Clone, Debug)]
pub struct AudioLoader {
    // Settings are stored for test inspection and will be actively used in
    // Phase 6 when actual audio decoding is implemented with rodio.
    #[cfg(test)]
    pub(crate) settings: AudioSettings,
    #[cfg(not(test))]
    _settings: AudioSettings,
}

impl AudioLoader {
    /// Creates a new audio loader with default settings.
    pub fn new() -> Self {
        Self {
            #[cfg(test)]
            settings: AudioSettings::default(),
            #[cfg(not(test))]
            _settings: AudioSettings::default(),
        }
    }

    /// Creates a new audio loader with custom settings.
    pub fn with_settings(settings: AudioSettings) -> Self {
        Self {
            #[cfg(test)]
            settings,
            #[cfg(not(test))]
            _settings: settings,
        }
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
