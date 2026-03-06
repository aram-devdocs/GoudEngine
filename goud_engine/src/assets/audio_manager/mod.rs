//! Audio manager resource for centralized audio playback control.
//!
//! The `AudioManager` is an ECS resource that provides a high-level interface
//! for playing, pausing, stopping, and controlling audio in the game engine.
//! It wraps the rodio audio backend and provides a clean API for audio operations.
//!
//! # Features
//!
//! - Play audio from loaded assets
//! - Control global volume
//! - Play, pause, stop, resume audio
//! - Check playback state
//! - Thread-safe for use in parallel systems
//!
//! # Usage
//!
//! ```no_run
//! use goud_engine::ecs::{World, Resource};
//! use goud_engine::assets::AudioManager;
//!
//! let mut world = World::new();
//! let audio_manager = AudioManager::new().expect("Failed to initialize audio");
//! world.insert_resource(audio_manager);
//!
//! // In a system:
//! // fn play_sound_system(audio: ResMut<AudioManager>, assets: Res<AssetServer>) {
//! //     let sound_handle = assets.load("sounds/jump.wav");
//! //     audio.play(sound_handle);
//! // }
//! ```

pub(super) mod spatial;

mod controls;
mod playback;
mod spatial_playback;
#[cfg(test)]
mod tests;

use crate::core::error::{GoudError, GoudResult};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Central audio playback manager resource.
///
/// `AudioManager` is an ECS resource that manages audio playback using the
/// rodio audio library. It maintains the audio output stream and provides
/// methods for playing, pausing, stopping, and controlling audio.
///
/// # Thread Safety
///
/// The AudioManager is thread-safe (`Send + Sync`) and can be used in parallel
/// systems. Internal state is protected by a Mutex.
///
/// # Example
///
/// ```no_run
/// use goud_engine::assets::AudioManager;
///
/// // Create audio manager
/// let mut audio_manager = AudioManager::new().expect("Failed to initialize audio");
///
/// // Set global volume (0.0 to 1.0)
/// audio_manager.set_global_volume(0.5);
///
/// // Get global volume
/// assert_eq!(audio_manager.global_volume(), 0.5);
/// ```
pub struct AudioManager {
    /// Audio device sink (must be kept alive for playback).
    pub(super) device_sink: MixerDeviceSink,

    /// Global volume (0.0 to 1.0).
    pub(super) global_volume: Arc<Mutex<f32>>,

    /// Active audio players (for controlling playback).
    pub(super) players: Arc<Mutex<HashMap<u64, Player>>>,

    /// Next player ID for tracking.
    pub(super) next_player_id: Arc<Mutex<u64>>,
}

impl AudioManager {
    /// Creates a new AudioManager instance.
    ///
    /// Initializes the audio output stream. Returns an error if audio
    /// output is not available (e.g., no audio device found).
    ///
    /// # Errors
    ///
    /// Returns `AudioInitFailed` if the audio system cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    ///
    /// let audio_manager = AudioManager::new().expect("Failed to initialize audio");
    /// ```
    pub fn new() -> GoudResult<Self> {
        let mut device_sink = DeviceSinkBuilder::open_default_sink().map_err(|e| {
            GoudError::AudioInitFailed(format!("Failed to create audio output stream: {}", e))
        })?;
        // Suppress the log message rodio prints when the device sink is dropped.
        device_sink.log_on_drop(false);

        Ok(Self {
            device_sink,
            global_volume: Arc::new(Mutex::new(1.0)),
            players: Arc::new(Mutex::new(HashMap::new())),
            next_player_id: Arc::new(Mutex::new(0)),
        })
    }
}

// SAFETY: AudioManager is Send + Sync because all internal state is protected by Mutex.
// MixerDeviceSink contains a cpal::Stream which is Send but not Sync by default.
// We ensure thread safety by only accessing it through our Mutex-protected methods.
unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("global_volume", &self.global_volume())
            .field("active_players", &self.active_count())
            .finish()
    }
}
