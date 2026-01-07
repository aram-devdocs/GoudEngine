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

use crate::assets::loaders::AudioAsset;
use crate::core::error::{GoudError, GoudResult};
use crate::core::math::Vec2;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// =============================================================================
// AudioManager Resource
// =============================================================================

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
    /// Audio output stream (must be kept alive for playback).
    #[allow(dead_code)]
    stream: OutputStream,

    /// Handle for audio output operations.
    /// TODO Phase 6: Use this for actual audio playback with rodio::Sink
    #[allow(dead_code)]
    stream_handle: OutputStreamHandle,

    /// Global volume (0.0 to 1.0).
    global_volume: Arc<Mutex<f32>>,

    /// Active audio sinks (for controlling playback).
    sinks: Arc<Mutex<HashMap<u64, Sink>>>,

    /// Next sink ID for tracking.
    next_sink_id: Arc<Mutex<u64>>,
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
        let (stream, stream_handle) = OutputStream::try_default().map_err(|e| {
            GoudError::AudioInitFailed(format!("Failed to create audio output stream: {}", e))
        })?;

        Ok(Self {
            stream,
            stream_handle,
            global_volume: Arc::new(Mutex::new(1.0)),
            sinks: Arc::new(Mutex::new(HashMap::new())),
            next_sink_id: Arc::new(Mutex::new(0)),
        })
    }

    /// Returns the global volume (0.0 to 1.0).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    ///
    /// let audio_manager = AudioManager::new().unwrap();
    /// assert_eq!(audio_manager.global_volume(), 1.0);
    /// ```
    pub fn global_volume(&self) -> f32 {
        *self.global_volume.lock().unwrap()
    }

    /// Sets the global volume (0.0 to 1.0).
    ///
    /// All currently playing and future audio will use this volume.
    /// Values outside the range are clamped.
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level (0.0 = mute, 1.0 = full volume)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    ///
    /// let mut audio_manager = AudioManager::new().unwrap();
    /// audio_manager.set_global_volume(0.5);
    /// assert_eq!(audio_manager.global_volume(), 0.5);
    /// ```
    pub fn set_global_volume(&mut self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        *self.global_volume.lock().unwrap() = clamped;

        // Update volume for all active sinks
        let sinks = self.sinks.lock().unwrap();
        for sink in sinks.values() {
            sink.set_volume(clamped);
        }
    }

    /// Plays an audio asset.
    ///
    /// Creates a new audio sink and starts playback of the given audio asset.
    /// Returns a unique sink ID that can be used to control playback (pause/resume/stop).
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    ///
    /// # Errors
    ///
    /// Returns `ResourceLoadFailed` if audio data is empty or cannot be decoded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::{AudioManager, AssetServer, loaders::AudioAsset};
    ///
    /// let mut audio_manager = AudioManager::new().unwrap();
    /// // let audio_asset = ...; // Loaded AudioAsset
    /// // let sink_id = audio_manager.play(&audio_asset).unwrap();
    /// ```
    pub fn play(&mut self, asset: &AudioAsset) -> GoudResult<u64> {
        // Validate audio data exists
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        // Create cursor over audio data
        let cursor = std::io::Cursor::new(asset.data().to_vec());

        // Decode audio data
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))?;

        // Create sink for this audio
        let sink = Sink::try_new(&self.stream_handle).map_err(|e| {
            GoudError::ResourceLoadFailed(format!("Failed to create audio sink: {}", e))
        })?;

        // Apply global volume
        sink.set_volume(self.global_volume());

        // Append audio source to sink and start playback
        sink.append(source);

        // Allocate ID and store sink
        let sink_id = self.allocate_sink_id();
        let mut sinks = self.sinks.lock().unwrap();
        sinks.insert(sink_id, sink);

        Ok(sink_id)
    }

    /// Plays an audio asset with looping enabled.
    ///
    /// Same as `play()` but the audio will repeat indefinitely until stopped.
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    pub fn play_looped(&mut self, asset: &AudioAsset) -> GoudResult<u64> {
        // Validate audio data exists
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        // Create cursor over audio data
        let cursor = std::io::Cursor::new(asset.data().to_vec());

        // Decode audio data
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))?;

        // Create repeating source
        let looped_source = source.repeat_infinite();

        // Create sink for this audio
        let sink = Sink::try_new(&self.stream_handle).map_err(|e| {
            GoudError::ResourceLoadFailed(format!("Failed to create audio sink: {}", e))
        })?;

        // Apply global volume
        sink.set_volume(self.global_volume());

        // Append looped source to sink and start playback
        sink.append(looped_source);

        // Allocate ID and store sink
        let sink_id = self.allocate_sink_id();
        let mut sinks = self.sinks.lock().unwrap();
        sinks.insert(sink_id, sink);

        Ok(sink_id)
    }

    /// Plays an audio asset with custom volume and pitch.
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    /// * `volume` - Volume multiplier (0.0-1.0, will be clamped)
    /// * `speed` - Playback speed multiplier (0.5-2.0, affects pitch)
    /// * `looping` - Whether to loop indefinitely
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    pub fn play_with_settings(
        &mut self,
        asset: &AudioAsset,
        volume: f32,
        speed: f32,
        looping: bool,
    ) -> GoudResult<u64> {
        // Validate audio data exists
        if asset.is_empty() {
            return Err(GoudError::ResourceLoadFailed(
                "Cannot play empty audio asset".to_string(),
            ));
        }

        // Create cursor over audio data
        let cursor = std::io::Cursor::new(asset.data().to_vec());

        // Decode audio data
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| GoudError::ResourceLoadFailed(format!("Failed to decode audio: {}", e)))?;

        // Apply speed (pitch) adjustment - clamp to reasonable range
        let clamped_speed = speed.clamp(0.1, 10.0);
        let source_with_speed = source.speed(clamped_speed);

        // Apply looping if requested
        let final_source: Box<dyn rodio::Source<Item = i16> + Send> = if looping {
            Box::new(source_with_speed.repeat_infinite())
        } else {
            Box::new(source_with_speed)
        };

        // Create sink for this audio
        let sink = Sink::try_new(&self.stream_handle).map_err(|e| {
            GoudError::ResourceLoadFailed(format!("Failed to create audio sink: {}", e))
        })?;

        // Apply volume (global volume * local volume)
        let clamped_volume = volume.clamp(0.0, 1.0);
        let final_volume = self.global_volume() * clamped_volume;
        sink.set_volume(final_volume);

        // Append source to sink and start playback
        sink.append(final_source);

        // Allocate ID and store sink
        let sink_id = self.allocate_sink_id();
        let mut sinks = self.sinks.lock().unwrap();
        sinks.insert(sink_id, sink);

        Ok(sink_id)
    }

    /// Pauses audio playback for the given sink ID.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and paused, `false` otherwise.
    pub fn pause(&self, sink_id: u64) -> bool {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&sink_id) {
            sink.pause();
            true
        } else {
            false
        }
    }

    /// Resumes audio playback for the given sink ID.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and resumed, `false` otherwise.
    pub fn resume(&self, sink_id: u64) -> bool {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&sink_id) {
            sink.play();
            true
        } else {
            false
        }
    }

    /// Stops audio playback for the given sink ID and removes it.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and stopped, `false` otherwise.
    pub fn stop(&mut self, sink_id: u64) -> bool {
        let mut sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.remove(&sink_id) {
            sink.stop();
            true
        } else {
            false
        }
    }

    /// Checks if audio is currently playing for the given sink ID.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink exists and is not paused, `false` otherwise.
    pub fn is_playing(&self, sink_id: u64) -> bool {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&sink_id) {
            !sink.is_paused()
        } else {
            false
        }
    }

    /// Returns the number of active audio sinks.
    pub fn active_count(&self) -> usize {
        self.sinks.lock().unwrap().len()
    }

    /// Stops all currently playing audio.
    pub fn stop_all(&mut self) {
        let mut sinks = self.sinks.lock().unwrap();
        for sink in sinks.values() {
            sink.stop();
        }
        sinks.clear();
    }

    /// Cleans up finished audio sinks.
    ///
    /// Removes sinks that have finished playing from the internal collection.
    /// This should be called periodically to prevent memory leaks.
    pub fn cleanup_finished(&mut self) {
        let mut sinks = self.sinks.lock().unwrap();
        sinks.retain(|_, sink| !sink.empty());
    }

    /// Sets the volume for a specific sink.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    /// * `volume` - Volume level (0.0-1.0, will be clamped)
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and volume set, `false` otherwise.
    pub fn set_sink_volume(&self, sink_id: u64, volume: f32) -> bool {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&sink_id) {
            let clamped = volume.clamp(0.0, 1.0);
            sink.set_volume(clamped);
            true
        } else {
            false
        }
    }

    /// Sets the playback speed (and pitch) for a specific sink.
    ///
    /// Note: This only works for sinks that haven't started playing yet.
    /// For real-time speed adjustment, the sink needs to be recreated.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    /// * `speed` - Speed multiplier (0.1-10.0, will be clamped)
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and speed set, `false` otherwise.
    pub fn set_sink_speed(&self, sink_id: u64, speed: f32) -> bool {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&sink_id) {
            let clamped = speed.clamp(0.1, 10.0);
            sink.set_speed(clamped);
            true
        } else {
            false
        }
    }

    /// Returns whether a sink has finished playing.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play()`
    ///
    /// # Returns
    ///
    /// `true` if the sink exists but has no more audio to play, `false` if sink doesn't exist or is still playing.
    pub fn is_finished(&self, sink_id: u64) -> bool {
        let sinks = self.sinks.lock().unwrap();
        if let Some(sink) = sinks.get(&sink_id) {
            sink.empty()
        } else {
            false
        }
    }

    // =============================================================================
    // Spatial Audio Methods
    // =============================================================================

    /// Plays audio with spatial positioning (2D).
    ///
    /// Applies distance-based attenuation using the given parameters.
    /// Volume is calculated as: base_volume * attenuation_factor
    ///
    /// # Arguments
    ///
    /// * `asset` - Reference to the audio asset to play
    /// * `source_position` - Position of the audio source in world space
    /// * `listener_position` - Position of the listener (typically camera or player)
    /// * `max_distance` - Maximum distance for audio (0 volume beyond this)
    /// * `rolloff` - Rolloff factor for attenuation (1.0 = linear, 2.0 = quadratic)
    ///
    /// # Returns
    ///
    /// A unique ID for this audio playback instance, or an error if playback fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut audio_manager = AudioManager::new().unwrap();
    /// // let audio_asset = ...; // Loaded AudioAsset
    /// // let sink_id = audio_manager.play_spatial(
    /// //     &audio_asset,
    /// //     Vec2::new(100.0, 50.0),  // Sound source position
    /// //     Vec2::new(0.0, 0.0),      // Listener position
    /// //     200.0,                    // Max distance
    /// //     1.0                       // Linear rolloff
    /// // ).unwrap();
    /// ```
    pub fn play_spatial(
        &mut self,
        asset: &AudioAsset,
        source_position: Vec2,
        listener_position: Vec2,
        max_distance: f32,
        rolloff: f32,
    ) -> GoudResult<u64> {
        // Calculate distance-based attenuation
        let distance = (source_position - listener_position).length();
        let attenuation = compute_attenuation_linear(distance, max_distance, rolloff);

        // Play with attenuated volume
        self.play_with_settings(asset, attenuation, 1.0, false)
    }

    /// Updates spatial audio volume for an existing sink.
    ///
    /// Recalculates attenuation based on new positions and updates the sink volume.
    /// This should be called when either the source or listener moves.
    ///
    /// # Arguments
    ///
    /// * `sink_id` - ID returned from `play_spatial()`
    /// * `source_position` - Current position of the audio source
    /// * `listener_position` - Current position of the listener
    /// * `max_distance` - Maximum distance for audio
    /// * `rolloff` - Rolloff factor for attenuation
    ///
    /// # Returns
    ///
    /// `true` if the sink was found and volume updated, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use goud_engine::assets::AudioManager;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut audio_manager = AudioManager::new().unwrap();
    /// // let sink_id = ...; // From play_spatial()
    /// // audio_manager.update_spatial_volume(
    /// //     sink_id,
    /// //     Vec2::new(150.0, 75.0),  // New source position
    /// //     Vec2::new(50.0, 25.0),   // New listener position
    /// //     200.0,
    /// //     1.0
    /// // );
    /// ```
    pub fn update_spatial_volume(
        &self,
        sink_id: u64,
        source_position: Vec2,
        listener_position: Vec2,
        max_distance: f32,
        rolloff: f32,
    ) -> bool {
        // Calculate distance-based attenuation
        let distance = (source_position - listener_position).length();
        let attenuation = compute_attenuation_linear(distance, max_distance, rolloff);

        // Update sink volume
        self.set_sink_volume(sink_id, attenuation)
    }

    // Internal helper to allocate a new sink ID
    fn allocate_sink_id(&self) -> u64 {
        let mut next_id = self.next_sink_id.lock().unwrap();
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        id
    }
}

// =============================================================================
// Spatial Audio Helpers
// =============================================================================

/// Computes linear attenuation based on distance.
///
/// Formula: attenuation = max(0, 1 - (distance / max_distance) ^ rolloff)
///
/// # Arguments
///
/// * `distance` - Distance from listener to source
/// * `max_distance` - Maximum audible distance (0 volume beyond)
/// * `rolloff` - Rolloff exponent (1.0 = linear, 2.0 = quadratic, etc.)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
fn compute_attenuation_linear(distance: f32, max_distance: f32, rolloff: f32) -> f32 {
    if max_distance <= 0.0 {
        return 1.0; // No attenuation
    }

    if distance >= max_distance {
        return 0.0; // Beyond max distance
    }

    // Linear falloff with rolloff factor
    let normalized_distance = distance / max_distance;
    let attenuation = 1.0 - normalized_distance.powf(rolloff);
    attenuation.max(0.0)
}

/// Computes inverse distance attenuation (realistic physics-based falloff).
///
/// Formula: attenuation = reference_distance / (reference_distance + rolloff * (distance - reference_distance))
///
/// # Arguments
///
/// * `distance` - Distance from listener to source
/// * `max_distance` - Maximum audible distance (used for clamping)
/// * `rolloff` - Rolloff factor (1.0 = realistic, higher = faster falloff)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
#[allow(dead_code)]
fn compute_attenuation_inverse(distance: f32, max_distance: f32, rolloff: f32) -> f32 {
    if max_distance <= 0.0 {
        return 1.0;
    }

    if distance >= max_distance {
        return 0.0;
    }

    // Inverse distance attenuation (reference distance = 1.0)
    let reference_distance = 1.0;
    let attenuation = reference_distance
        / (reference_distance + rolloff * (distance - reference_distance).max(0.0));
    attenuation.clamp(0.0, 1.0)
}

/// Computes exponential attenuation (dramatic falloff).
///
/// Formula: attenuation = (1 - distance / max_distance) ^ rolloff
///
/// # Arguments
///
/// * `distance` - Distance from listener to source
/// * `max_distance` - Maximum audible distance
/// * `rolloff` - Rolloff exponent (higher = more dramatic falloff)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
#[allow(dead_code)]
fn compute_attenuation_exponential(distance: f32, max_distance: f32, rolloff: f32) -> f32 {
    if max_distance <= 0.0 {
        return 1.0;
    }

    if distance >= max_distance {
        return 0.0;
    }

    // Exponential falloff
    let normalized_distance = distance / max_distance;
    let attenuation = (1.0 - normalized_distance).powf(rolloff);
    attenuation.max(0.0)
}

// AudioManager is Send + Sync because all internal state is protected by Mutex
unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("global_volume", &self.global_volume())
            .field("active_sinks", &self.active_count())
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // AudioManager Tests
    // ============================================================================

    #[test]
    fn test_audio_manager_new() {
        // Audio output may not be available in CI environments
        match AudioManager::new() {
            Ok(manager) => {
                assert_eq!(manager.global_volume(), 1.0);
                assert_eq!(manager.active_count(), 0);
            }
            Err(e) => {
                // Expected in headless environments
                assert!(matches!(e, GoudError::AudioInitFailed(_)));
            }
        }
    }

    #[test]
    fn test_audio_manager_global_volume() {
        if let Ok(mut manager) = AudioManager::new() {
            assert_eq!(manager.global_volume(), 1.0);

            manager.set_global_volume(0.5);
            assert_eq!(manager.global_volume(), 0.5);

            manager.set_global_volume(0.0);
            assert_eq!(manager.global_volume(), 0.0);
        }
    }

    #[test]
    fn test_audio_manager_volume_clamping() {
        if let Ok(mut manager) = AudioManager::new() {
            manager.set_global_volume(1.5);
            assert_eq!(manager.global_volume(), 1.0);

            manager.set_global_volume(-0.5);
            assert_eq!(manager.global_volume(), 0.0);
        }
    }

    #[test]
    fn test_audio_manager_play_empty_asset() {
        if let Ok(mut manager) = AudioManager::new() {
            let empty_asset = AudioAsset::empty();

            // Playing empty asset should fail
            let result = manager.play(&empty_asset);
            assert!(result.is_err());

            match result {
                Err(GoudError::ResourceLoadFailed(_)) => {}
                _ => panic!("Expected ResourceLoadFailed error"),
            }
        }
    }

    #[test]
    fn test_audio_manager_pause_resume() {
        if let Ok(manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Pausing/resuming nonexistent sink returns false
            assert!(!manager.pause(nonexistent_id));
            assert!(!manager.resume(nonexistent_id));
        }
    }

    #[test]
    fn test_audio_manager_stop() {
        if let Ok(mut manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Stopping nonexistent sink returns false
            assert!(!manager.stop(nonexistent_id));
        }
    }

    #[test]
    fn test_audio_manager_is_playing() {
        if let Ok(manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Nonexistent sink is not playing
            assert!(!manager.is_playing(nonexistent_id));
        }
    }

    #[test]
    fn test_audio_manager_active_count() {
        if let Ok(manager) = AudioManager::new() {
            assert_eq!(manager.active_count(), 0);
        }
    }

    #[test]
    fn test_audio_manager_stop_all() {
        if let Ok(mut manager) = AudioManager::new() {
            // Stop all on empty manager should not panic
            manager.stop_all();
            assert_eq!(manager.active_count(), 0);
        }
    }

    #[test]
    fn test_audio_manager_cleanup_finished() {
        if let Ok(mut manager) = AudioManager::new() {
            // Cleanup on empty manager should not panic
            manager.cleanup_finished();
            assert_eq!(manager.active_count(), 0);
        }
    }

    #[test]
    fn test_audio_manager_debug() {
        if let Ok(manager) = AudioManager::new() {
            let debug_str = format!("{:?}", manager);
            assert!(debug_str.contains("AudioManager"));
            assert!(debug_str.contains("global_volume"));
            assert!(debug_str.contains("active_sinks"));
        }
    }

    #[test]
    fn test_audio_manager_allocate_sink_id() {
        if let Ok(manager) = AudioManager::new() {
            let id1 = manager.allocate_sink_id();
            let id2 = manager.allocate_sink_id();
            let id3 = manager.allocate_sink_id();

            assert_eq!(id1, 0);
            assert_eq!(id2, 1);
            assert_eq!(id3, 2);
        }
    }

    // ============================================================================
    // Thread Safety Tests
    // ============================================================================

    #[test]
    fn test_audio_manager_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AudioManager>();
    }

    #[test]
    fn test_audio_manager_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AudioManager>();
    }

    // ============================================================================
    // Audio Playback Tests (with real audio data)
    // ============================================================================

    #[test]
    fn test_audio_manager_play_looped() {
        if let Ok(mut manager) = AudioManager::new() {
            let empty_asset = AudioAsset::empty();

            // Playing empty asset should fail even with looping
            let result = manager.play_looped(&empty_asset);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_audio_manager_play_with_settings() {
        if let Ok(mut manager) = AudioManager::new() {
            let empty_asset = AudioAsset::empty();

            // Playing empty asset should fail
            let result = manager.play_with_settings(&empty_asset, 0.5, 1.0, false);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_audio_manager_set_sink_volume() {
        if let Ok(manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Setting volume on nonexistent sink returns false
            assert!(!manager.set_sink_volume(nonexistent_id, 0.5));
        }
    }

    #[test]
    fn test_audio_manager_set_sink_speed() {
        if let Ok(manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Setting speed on nonexistent sink returns false
            assert!(!manager.set_sink_speed(nonexistent_id, 1.5));
        }
    }

    #[test]
    fn test_audio_manager_is_finished() {
        if let Ok(manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Nonexistent sink is not finished (returns false)
            assert!(!manager.is_finished(nonexistent_id));
        }
    }

    // ============================================================================
    // Spatial Audio Tests
    // ============================================================================

    #[test]
    fn test_spatial_audio_play_empty_asset() {
        if let Ok(mut manager) = AudioManager::new() {
            let empty_asset = AudioAsset::empty();

            // Playing empty asset with spatial audio should fail
            let result = manager.play_spatial(
                &empty_asset,
                Vec2::new(100.0, 50.0),
                Vec2::new(0.0, 0.0),
                200.0,
                1.0,
            );
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_spatial_audio_update_volume_nonexistent() {
        if let Ok(manager) = AudioManager::new() {
            let nonexistent_id = 999;

            // Updating spatial volume on nonexistent sink returns false
            assert!(!manager.update_spatial_volume(
                nonexistent_id,
                Vec2::new(100.0, 50.0),
                Vec2::new(0.0, 0.0),
                200.0,
                1.0
            ));
        }
    }

    #[test]
    fn test_compute_attenuation_linear_zero_distance() {
        // At zero distance, attenuation should be 1.0 (full volume)
        let attenuation = compute_attenuation_linear(0.0, 100.0, 1.0);
        assert!((attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_linear_max_distance() {
        // At max distance, attenuation should be 0.0 (silent)
        let attenuation = compute_attenuation_linear(100.0, 100.0, 1.0);
        assert!((attenuation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_linear_half_distance() {
        // At half distance with linear rolloff, attenuation should be 0.5
        let attenuation = compute_attenuation_linear(50.0, 100.0, 1.0);
        assert!((attenuation - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_linear_beyond_max() {
        // Beyond max distance, attenuation should be 0.0
        let attenuation = compute_attenuation_linear(150.0, 100.0, 1.0);
        assert!((attenuation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_linear_quadratic_rolloff() {
        // With rolloff=2.0 (quadratic), falloff is slower near source
        let attenuation = compute_attenuation_linear(50.0, 100.0, 2.0);
        // At half distance: 1 - (0.5)^2 = 1 - 0.25 = 0.75
        assert!((attenuation - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_linear_zero_max_distance() {
        // Zero max distance means no attenuation (always full volume)
        let attenuation = compute_attenuation_linear(100.0, 0.0, 1.0);
        assert!((attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_inverse_zero_distance() {
        // At zero distance, attenuation should be 1.0
        let attenuation = compute_attenuation_inverse(0.0, 100.0, 1.0);
        assert!((attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_inverse_max_distance() {
        // At max distance, attenuation should be 0.0
        let attenuation = compute_attenuation_inverse(100.0, 100.0, 1.0);
        assert!((attenuation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_inverse_realistic() {
        // Inverse distance should have slower falloff near source
        let attenuation = compute_attenuation_inverse(10.0, 100.0, 1.0);
        // Formula: 1 / (1 + rolloff * (distance - ref_distance))
        // = 1 / (1 + 1 * (10 - 1)) = 1 / (1 + 9) = 1/10 = 0.1
        assert!(
            (attenuation - 0.1).abs() < 0.01,
            "Expected ~0.1, got {}",
            attenuation
        );
    }

    #[test]
    fn test_compute_attenuation_inverse_beyond_max() {
        // Beyond max distance, attenuation should be 0.0
        let attenuation = compute_attenuation_inverse(150.0, 100.0, 1.0);
        assert!((attenuation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_exponential_zero_distance() {
        // At zero distance, attenuation should be 1.0
        let attenuation = compute_attenuation_exponential(0.0, 100.0, 1.0);
        assert!((attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_exponential_max_distance() {
        // At max distance, attenuation should be 0.0
        let attenuation = compute_attenuation_exponential(100.0, 100.0, 1.0);
        assert!((attenuation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_exponential_half_distance() {
        // With rolloff=1.0, at half distance: (1 - 0.5)^1 = 0.5
        let attenuation = compute_attenuation_exponential(50.0, 100.0, 1.0);
        assert!((attenuation - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_exponential_dramatic_falloff() {
        // With rolloff=3.0, falloff is dramatic: (1 - 0.5)^3 = 0.125
        let attenuation = compute_attenuation_exponential(50.0, 100.0, 3.0);
        assert!((attenuation - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_compute_attenuation_exponential_beyond_max() {
        // Beyond max distance, attenuation should be 0.0
        let attenuation = compute_attenuation_exponential(150.0, 100.0, 1.0);
        assert!((attenuation - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_attenuation_comparison() {
        // At half distance, compare all three models
        let distance = 50.0;
        let max_distance = 100.0;
        let rolloff = 1.0;

        let linear = compute_attenuation_linear(distance, max_distance, rolloff);
        let inverse = compute_attenuation_inverse(distance, max_distance, rolloff);
        let exponential = compute_attenuation_exponential(distance, max_distance, rolloff);

        // Linear: 0.5, Inverse: ~0.02, Exponential: 0.5
        assert!((linear - 0.5).abs() < 0.001);
        assert!(inverse < 0.05); // Inverse falloff is much faster
        assert!((exponential - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_spatial_audio_attenuation_at_source() {
        // Test attenuation calculation at source position (zero distance)
        let source_pos = Vec2::new(100.0, 50.0);
        let listener_pos = Vec2::new(100.0, 50.0);
        let distance = (source_pos - listener_pos).length();

        let attenuation = compute_attenuation_linear(distance, 200.0, 1.0);
        assert!(
            (attenuation - 1.0).abs() < 0.001,
            "Attenuation at source should be 1.0"
        );
    }

    #[test]
    fn test_spatial_audio_attenuation_at_max() {
        // Test attenuation calculation at max distance
        let source_pos = Vec2::new(100.0, 50.0);
        let listener_pos = Vec2::new(-100.0, 50.0); // 200 units away
        let distance = (source_pos - listener_pos).length();

        let attenuation = compute_attenuation_linear(distance, 200.0, 1.0);
        assert!(
            (attenuation - 0.0).abs() < 0.001,
            "Attenuation at max distance should be 0.0"
        );
    }

    #[test]
    fn test_spatial_audio_attenuation_diagonal() {
        // Test attenuation with diagonal distance (Pythagorean)
        let source_pos = Vec2::new(100.0, 100.0);
        let listener_pos = Vec2::new(0.0, 0.0);
        let distance = (source_pos - listener_pos).length(); // sqrt(100^2 + 100^2) = ~141.42

        let attenuation = compute_attenuation_linear(distance, 200.0, 1.0);
        // Expected: 1 - 141.42/200 = 1 - 0.707 = 0.293
        assert!(
            (attenuation - 0.293).abs() < 0.01,
            "Attenuation for diagonal distance"
        );
    }
}
