use crate::assets::loaders::AudioAsset;
use crate::core::error::{GoudError, GoudResult};
use crate::core::math::Vec2;
use crate::ecs::components::AudioChannel;

const AUDIO_UNAVAILABLE: &str = "Desktop audio is unavailable in this build";

/// Mobile-safe placeholder used when desktop audio support is not compiled in.
#[derive(Debug, Default)]
pub struct AudioManager;

#[allow(missing_docs)]
impl AudioManager {
    /// Creates a placeholder audio manager for builds without desktop audio.
    pub fn new() -> GoudResult<Self> {
        Err(GoudError::AudioInitFailed(AUDIO_UNAVAILABLE.to_string()))
    }

    pub fn play(&mut self, _asset: &AudioAsset) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn play_on_channel(
        &mut self,
        _asset: &AudioAsset,
        _channel: AudioChannel,
    ) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn play_looped(&mut self, _asset: &AudioAsset) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn play_with_settings(
        &mut self,
        _asset: &AudioAsset,
        _volume: f32,
        _speed: f32,
        _looping: bool,
        _channel: AudioChannel,
    ) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn pause(&self, _sink_id: u64) -> bool {
        false
    }

    pub fn resume(&self, _sink_id: u64) -> bool {
        false
    }

    pub fn stop(&mut self, _sink_id: u64) -> bool {
        false
    }

    pub fn is_playing(&self, _sink_id: u64) -> bool {
        false
    }

    pub fn active_count(&self) -> usize {
        0
    }

    pub fn stop_all(&mut self) {}

    pub fn cleanup_finished(&mut self) {}

    pub fn set_crossfade_mix(&self, _from_id: u64, _to_id: u64, _mix: f32) -> bool {
        false
    }

    pub fn crossfade_to(
        &mut self,
        _from_id: u64,
        _to_asset: &AudioAsset,
        _duration_sec: f32,
        _channel: AudioChannel,
    ) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn mix_with(
        &mut self,
        _primary_id: u64,
        _secondary_asset: &AudioAsset,
        _secondary_volume: f32,
        _secondary_channel: AudioChannel,
    ) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn update_crossfades(&mut self, _delta_sec: f32) {}

    pub fn active_crossfade_count(&self) -> usize {
        0
    }

    pub fn listener_position(&self) -> [f32; 3] {
        [0.0, 0.0, 0.0]
    }

    pub fn set_listener_position(&self, _position: [f32; 3]) {}

    pub fn register_spatial_source(
        &self,
        _sink_id: u64,
        _source_position: [f32; 3],
        _max_distance: f32,
        _rolloff: f32,
        _base_volume: f32,
    ) -> bool {
        false
    }

    pub fn set_source_position(&self, _sink_id: u64, _source_position: [f32; 3]) -> bool {
        false
    }

    pub fn retain_spatial_sources(&self, _active_sink_ids: &[u64]) {}

    pub fn refresh_spatial_sources(&self) {}

    pub fn play_spatial(
        &mut self,
        _asset: &AudioAsset,
        _source_position: Vec2,
        _listener_position: Vec2,
        _max_distance: f32,
        _rolloff: f32,
    ) -> GoudResult<u64> {
        Err(audio_unavailable())
    }

    pub fn update_spatial_volume(
        &self,
        _sink_id: u64,
        _source_position: Vec2,
        _listener_position: Vec2,
        _max_distance: f32,
        _rolloff: f32,
    ) -> bool {
        false
    }

    pub fn global_volume(&self) -> f32 {
        1.0
    }

    pub fn set_global_volume(&mut self, _volume: f32) {}

    pub fn set_sink_volume(&self, _sink_id: u64, _volume: f32) -> bool {
        false
    }

    pub fn set_sink_speed(&self, _sink_id: u64, _speed: f32) -> bool {
        false
    }

    pub fn is_finished(&self, _sink_id: u64) -> bool {
        true
    }

    pub fn set_channel_volume(&mut self, _channel: AudioChannel, _volume: f32) {}

    pub fn get_channel_volume(&self, _channel: AudioChannel) -> f32 {
        1.0
    }

    pub(crate) fn has_sink(&self, _sink_id: u64) -> bool {
        false
    }

    pub(crate) fn sink_volume(&self, _sink_id: u64) -> Option<f32> {
        None
    }
}

fn audio_unavailable() -> GoudError {
    GoudError::AudioInitFailed(AUDIO_UNAVAILABLE.to_string())
}
