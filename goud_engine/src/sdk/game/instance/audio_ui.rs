use crate::core::error::{GoudError, GoudResult};
use crate::ui::UiManager;

use super::GoudGame;

impl GoudGame {
    /// Returns a reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager(&self) -> Option<&crate::assets::AudioManager> {
        self.audio_manager.as_ref()
    }

    /// Returns a mutable reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager_mut(&mut self) -> Option<&mut crate::assets::AudioManager> {
        self.audio_manager.as_mut()
    }

    /// Plays an audio asset on the default SFX channel.
    #[cfg(feature = "native")]
    pub fn play_audio(&mut self, asset: &crate::assets::loaders::AudioAsset) -> GoudResult<u64> {
        self.require_audio_manager_mut()?.play(asset)
    }

    /// Plays an audio asset on a specific channel.
    #[cfg(feature = "native")]
    pub fn play_audio_on_channel(
        &mut self,
        asset: &crate::assets::loaders::AudioAsset,
        channel: crate::ecs::components::AudioChannel,
    ) -> GoudResult<u64> {
        self.require_audio_manager_mut()?
            .play_on_channel(asset, channel)
    }

    /// Plays a 2D spatial audio asset and tracks it for listener/source updates.
    #[cfg(feature = "native")]
    pub fn play_audio_spatial_2d(
        &mut self,
        asset: &crate::assets::loaders::AudioAsset,
        source_position: crate::core::math::Vec2,
        listener_position: crate::core::math::Vec2,
        max_distance: f32,
        rolloff: f32,
    ) -> GoudResult<u64> {
        self.require_audio_manager_mut()?.play_spatial(
            asset,
            source_position,
            listener_position,
            max_distance,
            rolloff,
        )
    }

    /// Sets the global spatial listener position.
    #[cfg(feature = "native")]
    pub fn set_audio_listener_position(&mut self, position: [f32; 3]) -> GoudResult<()> {
        self.require_audio_manager_mut()?
            .set_listener_position(position);
        Ok(())
    }

    /// Updates a tracked spatial source position.
    #[cfg(feature = "native")]
    pub fn set_audio_source_position(
        &mut self,
        sink_id: u64,
        position: [f32; 3],
    ) -> GoudResult<()> {
        if self
            .require_audio_manager_mut()?
            .set_source_position(sink_id, position)
        {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    /// Starts a crossfade from `from_id` to a newly played asset.
    #[cfg(feature = "native")]
    pub fn crossfade_audio_to(
        &mut self,
        from_id: u64,
        to_asset: &crate::assets::loaders::AudioAsset,
        duration_sec: f32,
        channel: crate::ecs::components::AudioChannel,
    ) -> GoudResult<u64> {
        self.require_audio_manager_mut()?
            .crossfade_to(from_id, to_asset, duration_sec, channel)
    }

    /// Starts additive mixing by playing a secondary asset while the primary remains active.
    #[cfg(feature = "native")]
    pub fn mix_audio_with(
        &mut self,
        primary_id: u64,
        secondary_asset: &crate::assets::loaders::AudioAsset,
        secondary_volume: f32,
        secondary_channel: crate::ecs::components::AudioChannel,
    ) -> GoudResult<u64> {
        self.require_audio_manager_mut()?.mix_with(
            primary_id,
            secondary_asset,
            secondary_volume,
            secondary_channel,
        )
    }

    /// Sets global audio volume.
    #[cfg(feature = "native")]
    pub fn set_audio_global_volume(&mut self, volume: f32) -> GoudResult<()> {
        self.require_audio_manager_mut()?.set_global_volume(volume);
        Ok(())
    }

    /// Returns global audio volume.
    #[cfg(feature = "native")]
    pub fn audio_global_volume(&self) -> GoudResult<f32> {
        Ok(self.require_audio_manager()?.global_volume())
    }

    /// Sets per-channel audio volume.
    #[cfg(feature = "native")]
    pub fn set_audio_channel_volume(
        &mut self,
        channel: crate::ecs::components::AudioChannel,
        volume: f32,
    ) -> GoudResult<()> {
        self.require_audio_manager_mut()?
            .set_channel_volume(channel, volume);
        Ok(())
    }

    /// Returns per-channel audio volume.
    #[cfg(feature = "native")]
    pub fn audio_channel_volume(
        &self,
        channel: crate::ecs::components::AudioChannel,
    ) -> GoudResult<f32> {
        Ok(self.require_audio_manager()?.get_channel_volume(channel))
    }

    /// Sets a sink's volume.
    #[cfg(feature = "native")]
    pub fn set_audio_sink_volume(&mut self, sink_id: u64, volume: f32) -> GoudResult<()> {
        if self
            .require_audio_manager()?
            .set_sink_volume(sink_id, volume)
        {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    /// Sets a sink's speed.
    #[cfg(feature = "native")]
    pub fn set_audio_sink_speed(&mut self, sink_id: u64, speed: f32) -> GoudResult<()> {
        if self.require_audio_manager()?.set_sink_speed(sink_id, speed) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    /// Stops a sink.
    #[cfg(feature = "native")]
    pub fn stop_audio(&mut self, sink_id: u64) -> GoudResult<()> {
        if self.require_audio_manager_mut()?.stop(sink_id) {
            Ok(())
        } else {
            Err(GoudError::InvalidHandle)
        }
    }

    /// Stops all sinks.
    #[cfg(feature = "native")]
    pub fn stop_all_audio(&mut self) -> GoudResult<()> {
        self.require_audio_manager_mut()?.stop_all();
        Ok(())
    }

    #[cfg(feature = "native")]
    fn require_audio_manager(&self) -> GoudResult<&crate::assets::AudioManager> {
        self.audio_manager.as_ref().ok_or_else(|| {
            GoudError::AudioInitFailed(
                "Audio manager is unavailable on this game instance".to_string(),
            )
        })
    }

    #[cfg(feature = "native")]
    fn require_audio_manager_mut(&mut self) -> GoudResult<&mut crate::assets::AudioManager> {
        self.audio_manager.as_mut().ok_or_else(|| {
            GoudError::AudioInitFailed(
                "Audio manager is unavailable on this game instance".to_string(),
            )
        })
    }

    /// Returns a reference to the UI manager.
    #[inline]
    pub fn ui_manager(&self) -> &UiManager {
        &self.ui_manager
    }

    /// Returns a mutable reference to the UI manager.
    #[inline]
    pub fn ui_manager_mut(&mut self) -> &mut UiManager {
        &mut self.ui_manager
    }
}
