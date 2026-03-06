//! Audio provider trait definition.
//!
//! The `AudioProvider` trait abstracts the audio backend, enabling
//! runtime selection between Rodio, WebAudio, or null (silent).

use super::types::{AudioCapabilities, AudioChannel, PlayConfig, PlaybackId, SoundHandle};
use super::{Provider, ProviderLifecycle};
use crate::libs::error::GoudResult;

/// Trait for audio backends.
///
/// Uses `[f32; 3]` for spatial positions to avoid depending on external math
/// types. Concrete implementations convert to/from their internal vector types.
///
/// The trait is object-safe and stored as `Box<dyn AudioProvider>`.
pub trait AudioProvider: Provider + ProviderLifecycle {
    /// Returns the typed audio capabilities for this provider.
    fn audio_capabilities(&self) -> &AudioCapabilities;

    // -------------------------------------------------------------------------
    // Streaming
    // -------------------------------------------------------------------------

    /// Per-frame audio update (e.g., stream refill, listener sync).
    ///
    /// This is called by `ProviderLifecycle::update()` internally. The
    /// separate method exists so providers can distinguish audio-specific
    /// updates from the generic lifecycle update.
    fn audio_update(&mut self) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Playback Control
    // -------------------------------------------------------------------------

    /// Play a sound with the given configuration. Returns a playback ID
    /// for controlling the active instance.
    fn play(&mut self, handle: SoundHandle, config: &PlayConfig) -> GoudResult<PlaybackId>;

    /// Stop a playing sound instance.
    fn stop(&mut self, id: PlaybackId) -> GoudResult<()>;

    /// Pause a playing sound instance.
    fn pause(&mut self, id: PlaybackId) -> GoudResult<()>;

    /// Resume a paused sound instance.
    fn resume(&mut self, id: PlaybackId) -> GoudResult<()>;

    /// Check if a sound instance is currently playing.
    fn is_playing(&self, id: PlaybackId) -> bool;

    // -------------------------------------------------------------------------
    // Volume Control
    // -------------------------------------------------------------------------

    /// Set the volume of a specific playback instance (0.0 to 1.0).
    fn set_volume(&mut self, id: PlaybackId, volume: f32) -> GoudResult<()>;

    /// Set the master volume (0.0 to 1.0). Affects all audio output.
    fn set_master_volume(&mut self, volume: f32);

    /// Set the volume for a specific audio channel (0.0 to 1.0).
    fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32);

    // -------------------------------------------------------------------------
    // Spatial Audio
    // -------------------------------------------------------------------------

    /// Set the listener position for spatial audio as [x, y, z].
    fn set_listener_position(&mut self, pos: [f32; 3]);

    /// Set the position of a playing sound source as [x, y, z].
    fn set_source_position(&mut self, id: PlaybackId, pos: [f32; 3]) -> GoudResult<()>;
}
