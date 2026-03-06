//! Spatial audio playback: distance-based attenuation for positioned audio sources.

use crate::assets::loaders::AudioAsset;
use crate::core::error::GoudResult;
use crate::core::math::Vec2;

use super::{spatial::spatial_attenuation, AudioManager};

impl AudioManager {
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
        let attenuation =
            spatial_attenuation(source_position, listener_position, max_distance, rolloff);
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
        let attenuation =
            spatial_attenuation(source_position, listener_position, max_distance, rolloff);
        self.set_sink_volume(sink_id, attenuation)
    }
}
