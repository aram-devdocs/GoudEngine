//! Crossfade and mix helpers for [`AudioManager`].

use crate::assets::loaders::AudioAsset;
use crate::core::error::{GoudError, GoudResult};
use crate::ecs::components::AudioChannel;

use super::{AudioManager, CrossfadeState};

#[inline]
pub(super) fn clamp_duration(duration_sec: f32) -> f32 {
    if duration_sec.is_finite() {
        duration_sec.max(0.0)
    } else {
        0.0
    }
}

#[inline]
pub(super) fn crossfade_progress(elapsed_sec: f32, duration_sec: f32) -> f32 {
    if duration_sec <= f32::EPSILON {
        return 1.0;
    }

    (elapsed_sec / duration_sec).clamp(0.0, 1.0)
}

#[inline]
pub(super) fn crossfade_pair(from_start: f32, to_target: f32, progress: f32) -> (f32, f32) {
    let t = progress.clamp(0.0, 1.0);
    (
        from_start.clamp(0.0, 1.0) * (1.0 - t),
        to_target.clamp(0.0, 1.0) * t,
    )
}

impl AudioManager {
    /// Applies an immediate pair mix in `[0, 1]` between two active sinks.
    ///
    /// `mix = 0` keeps `from_id` at full volume and mutes `to_id`.
    /// `mix = 1` mutes `from_id` and sets `to_id` to full volume.
    ///
    /// Returns `false` when either sink does not exist.
    pub fn set_crossfade_mix(&self, from_id: u64, to_id: u64, mix: f32) -> bool {
        if from_id == to_id {
            return self.set_sink_volume(from_id, 1.0);
        }

        if !self.has_sink(from_id) || !self.has_sink(to_id) {
            return false;
        }

        let t = mix.clamp(0.0, 1.0);
        let from_ok = self.set_sink_volume(from_id, 1.0 - t);
        let to_ok = self.set_sink_volume(to_id, t);
        from_ok && to_ok
    }

    /// Starts a crossfade from an active sink to a newly played asset.
    ///
    /// - Invalid durations are clamped to `0.0`
    /// - Duration `<= 0` performs an immediate stop+switch
    /// - Missing `from_id` returns `InvalidHandle`
    pub fn crossfade_to(
        &mut self,
        from_id: u64,
        to_asset: &AudioAsset,
        duration_sec: f32,
        channel: AudioChannel,
    ) -> GoudResult<u64> {
        let from_volume = self.sink_volume(from_id).ok_or(GoudError::InvalidHandle)?;
        let duration_sec = clamp_duration(duration_sec);

        if duration_sec <= f32::EPSILON {
            let to_id = self.play_with_settings(to_asset, from_volume, 1.0, false, channel)?;
            let _ = self.stop(from_id);
            return Ok(to_id);
        }

        let to_id = self.play_with_settings(to_asset, 0.0, 1.0, false, channel)?;

        self.crossfades
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                to_id,
                CrossfadeState {
                    from_id,
                    to_id,
                    elapsed_sec: 0.0,
                    duration_sec,
                    from_start_volume: from_volume.clamp(0.0, 1.0),
                    to_target_volume: from_volume.clamp(0.0, 1.0),
                },
            );

        Ok(to_id)
    }

    /// Starts a secondary layer while keeping the primary sink playing.
    ///
    /// This is additive mixing: the primary sink remains untouched and a new
    /// sink is started with `secondary_volume` clamped to `[0, 1]`.
    pub fn mix_with(
        &mut self,
        primary_id: u64,
        secondary_asset: &AudioAsset,
        secondary_volume: f32,
        secondary_channel: AudioChannel,
    ) -> GoudResult<u64> {
        if !self.has_sink(primary_id) {
            return Err(GoudError::InvalidHandle);
        }

        let clamped_secondary = secondary_volume.clamp(0.0, 1.0);
        self.play_with_settings(
            secondary_asset,
            clamped_secondary,
            1.0,
            false,
            secondary_channel,
        )
    }

    /// Advances active crossfades by `delta_sec`.
    pub fn update_crossfades(&mut self, delta_sec: f32) {
        let step = clamp_duration(delta_sec);
        if step <= f32::EPSILON {
            return;
        }

        let mut updates: Vec<(u64, f32)> = Vec::new();
        let mut completed: Vec<CrossfadeState> = Vec::new();

        {
            let mut crossfades = self
                .crossfades
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut ordered_to_ids: Vec<u64> = crossfades.keys().copied().collect();
            ordered_to_ids.sort_unstable();

            let mut remove_ids: Vec<u64> = Vec::new();

            for to_id in ordered_to_ids {
                let Some(state) = crossfades.get_mut(&to_id) else {
                    continue;
                };
                let has_from = self.has_sink(state.from_id);
                let has_to = self.has_sink(state.to_id);

                if !has_to {
                    remove_ids.push(to_id);
                    continue;
                }

                if !has_from {
                    updates.push((state.to_id, state.to_target_volume));
                    completed.push(*state);
                    remove_ids.push(to_id);
                    continue;
                }

                state.elapsed_sec = (state.elapsed_sec + step).min(state.duration_sec);
                let progress = crossfade_progress(state.elapsed_sec, state.duration_sec);
                let (from_v, to_v) =
                    crossfade_pair(state.from_start_volume, state.to_target_volume, progress);
                updates.push((state.from_id, from_v));
                updates.push((state.to_id, to_v));

                if progress >= 1.0 {
                    completed.push(*state);
                    remove_ids.push(to_id);
                }
            }

            for remove_id in remove_ids {
                crossfades.remove(&remove_id);
            }
        }

        for (sink_id, volume) in updates {
            let _ = self.set_sink_volume(sink_id, volume);
        }

        for state in completed {
            let _ = self.stop(state.from_id);
            let _ = self.set_sink_volume(state.to_id, state.to_target_volume);
        }
    }

    /// Returns the number of currently active crossfades.
    pub fn active_crossfade_count(&self) -> usize {
        self.crossfades
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::{clamp_duration, crossfade_pair, crossfade_progress};

    #[test]
    fn test_crossfade_duration_clamping() {
        assert_eq!(clamp_duration(-1.0), 0.0);
        assert_eq!(clamp_duration(0.0), 0.0);
        assert_eq!(clamp_duration(1.25), 1.25);
        assert_eq!(clamp_duration(f32::NAN), 0.0);
        assert_eq!(clamp_duration(f32::INFINITY), 0.0);
    }

    #[test]
    fn test_crossfade_progress_math() {
        assert_eq!(crossfade_progress(0.0, 2.0), 0.0);
        assert_eq!(crossfade_progress(1.0, 2.0), 0.5);
        assert_eq!(crossfade_progress(3.0, 2.0), 1.0);
        assert_eq!(crossfade_progress(1.0, 0.0), 1.0);
    }

    #[test]
    fn test_crossfade_pair_math() {
        let (from_a, to_a) = crossfade_pair(1.0, 0.8, 0.0);
        assert!((from_a - 1.0).abs() < 0.0001);
        assert!((to_a - 0.0).abs() < 0.0001);

        let (from_b, to_b) = crossfade_pair(1.0, 0.8, 0.5);
        assert!((from_b - 0.5).abs() < 0.0001);
        assert!((to_b - 0.4).abs() < 0.0001);

        let (from_c, to_c) = crossfade_pair(1.0, 0.8, 1.0);
        assert!((from_c - 0.0).abs() < 0.0001);
        assert!((to_c - 0.8).abs() < 0.0001);
    }
}
