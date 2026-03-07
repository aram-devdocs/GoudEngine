//! Null audio provider -- silent no-op for headless testing.

use crate::core::error::GoudResult;
use crate::core::providers::audio::AudioProvider;
use crate::core::providers::types::{
    AudioCapabilities, AudioChannel, PlayConfig, PlaybackId, SoundHandle,
};
use crate::core::providers::{Provider, ProviderLifecycle};

/// An audio provider that does nothing. Used for headless testing and as
/// a default when no audio backend is available.
pub struct NullAudioProvider {
    capabilities: AudioCapabilities,
}

impl NullAudioProvider {
    /// Create a new null audio provider.
    pub fn new() -> Self {
        Self {
            capabilities: AudioCapabilities {
                supports_spatial: false,
                max_channels: 0,
            },
        }
    }
}

impl Default for NullAudioProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NullAudioProvider {
    fn name(&self) -> &str {
        "null"
    }

    fn version(&self) -> &str {
        "0.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for NullAudioProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl AudioProvider for NullAudioProvider {
    fn audio_capabilities(&self) -> &AudioCapabilities {
        &self.capabilities
    }

    fn audio_update(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn play(&mut self, _handle: SoundHandle, _config: &PlayConfig) -> GoudResult<PlaybackId> {
        Ok(PlaybackId(0))
    }

    fn stop(&mut self, _id: PlaybackId) -> GoudResult<()> {
        Ok(())
    }

    fn pause(&mut self, _id: PlaybackId) -> GoudResult<()> {
        Ok(())
    }

    fn resume(&mut self, _id: PlaybackId) -> GoudResult<()> {
        Ok(())
    }

    fn is_playing(&self, _id: PlaybackId) -> bool {
        false
    }

    fn set_volume(&mut self, _id: PlaybackId, _volume: f32) -> GoudResult<()> {
        Ok(())
    }

    fn set_master_volume(&mut self, _volume: f32) {}

    fn set_channel_volume(&mut self, _channel: AudioChannel, _volume: f32) {}

    fn set_listener_position(&mut self, _pos: [f32; 3]) {}

    fn set_source_position(&mut self, _id: PlaybackId, _pos: [f32; 3]) -> GoudResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_audio_construction() {
        let provider = NullAudioProvider::new();
        assert_eq!(provider.name(), "null");
        assert_eq!(provider.version(), "0.0.0");
    }

    #[test]
    fn test_null_audio_default() {
        let provider = NullAudioProvider::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_audio_init_shutdown() {
        let mut provider = NullAudioProvider::new();
        assert!(provider.init().is_ok());
        assert!(provider.update(0.016).is_ok());
        provider.shutdown();
    }

    #[test]
    fn test_null_audio_capabilities() {
        let provider = NullAudioProvider::new();
        let caps = provider.audio_capabilities();
        assert!(!caps.supports_spatial);
        assert_eq!(caps.max_channels, 0);
    }

    #[test]
    fn test_null_audio_playback() {
        let mut provider = NullAudioProvider::new();
        let id = provider
            .play(SoundHandle(1), &PlayConfig::default())
            .unwrap();
        assert_eq!(id, PlaybackId(0));
        assert!(!provider.is_playing(id));
        assert!(provider.pause(id).is_ok());
        assert!(provider.resume(id).is_ok());
        assert!(provider.stop(id).is_ok());
    }

    #[test]
    fn test_null_audio_volume() {
        let mut provider = NullAudioProvider::new();
        assert!(provider.set_volume(PlaybackId(0), 0.5).is_ok());
        provider.set_master_volume(0.8);
        provider.set_channel_volume(AudioChannel::Music, 0.6);
    }

    #[test]
    fn test_null_audio_spatial() {
        let mut provider = NullAudioProvider::new();
        provider.set_listener_position([1.0, 2.0, 3.0]);
        assert!(provider
            .set_source_position(PlaybackId(0), [4.0, 5.0, 6.0])
            .is_ok());
    }

    #[test]
    fn test_null_audio_update() {
        let mut provider = NullAudioProvider::new();
        assert!(provider.audio_update().is_ok());
    }
}
