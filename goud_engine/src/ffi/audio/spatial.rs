//! Spatial and per-player audio FFI exports.

#[path = "spatial/shared.rs"]
mod shared;

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::components::AudioChannel;
use crate::ffi::context::GoudContextId;

use super::{ERR_AUDIO, ERR_I32};
use shared::{audio_asset_from_raw, update_spatial_source_3d, with_audio, with_audio_mut};

/// Plays audio with spatial attenuation in 2D (`z = 0`).
///
/// Returns a positive player ID on success and `-1` on failure.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_audio_play_spatial(
    context_id: GoudContextId,
    asset_data: *const u8,
    asset_len: usize,
    source_x: f32,
    source_y: f32,
    listener_x: f32,
    listener_y: f32,
    max_distance: f32,
    rolloff: f32,
) -> i64 {
    // SAFETY: Forwarding raw pointer contract from this function's safety requirements.
    unsafe {
        goud_audio_play_spatial_3d(
            context_id,
            asset_data,
            asset_len,
            source_x,
            source_y,
            0.0,
            listener_x,
            listener_y,
            0.0,
            max_distance,
            rolloff,
        )
    }
}

/// Plays audio with spatial attenuation in 3D.
///
/// Returns a positive player ID on success and `-1` on failure.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_audio_play_spatial_3d(
    context_id: GoudContextId,
    asset_data: *const u8,
    asset_len: usize,
    source_x: f32,
    source_y: f32,
    source_z: f32,
    listener_x: f32,
    listener_y: f32,
    listener_z: f32,
    max_distance: f32,
    rolloff: f32,
) -> i64 {
    // SAFETY: Caller guarantees `asset_data` points to `asset_len` valid bytes.
    let asset = match unsafe { audio_asset_from_raw(asset_data, asset_len) } {
        Ok(asset) => asset,
        Err(()) => return ERR_AUDIO,
    };

    match with_audio_mut(context_id, |audio| {
        audio.set_listener_position([listener_x, listener_y, listener_z]);
        let sink_id = audio.play_with_settings(&asset, 1.0, 1.0, false, AudioChannel::SFX)?;
        let tracked = audio.register_spatial_source(
            sink_id,
            [source_x, source_y, source_z],
            max_distance,
            rolloff,
            1.0,
        );

        if tracked {
            Ok(sink_id)
        } else {
            let _ = audio.stop(sink_id);
            Err(GoudError::InvalidHandle)
        }
    }) {
        Ok(Ok(player_id)) => player_id as i64,
        Ok(Err(e)) => {
            set_last_error(e);
            ERR_AUDIO
        }
        Err(()) => ERR_AUDIO,
    }
}

/// Updates attenuation for a playing spatial source in 2D (`z = 0`).
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_update_spatial_volume(
    context_id: GoudContextId,
    player_id: u64,
    source_x: f32,
    source_y: f32,
    listener_x: f32,
    listener_y: f32,
    max_distance: f32,
    rolloff: f32,
) -> i32 {
    goud_audio_update_spatial_volume_3d(
        context_id,
        player_id,
        source_x,
        source_y,
        0.0,
        listener_x,
        listener_y,
        0.0,
        max_distance,
        rolloff,
    )
}

/// Updates attenuation for a playing spatial source in 3D.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_update_spatial_volume_3d(
    context_id: GoudContextId,
    player_id: u64,
    source_x: f32,
    source_y: f32,
    source_z: f32,
    listener_x: f32,
    listener_y: f32,
    listener_z: f32,
    max_distance: f32,
    rolloff: f32,
) -> i32 {
    match with_audio(context_id, |audio| {
        update_spatial_source_3d(
            audio,
            player_id,
            [source_x, source_y, source_z],
            [listener_x, listener_y, listener_z],
            max_distance,
            rolloff,
        )
    }) {
        Ok(true) => 0,
        Ok(false) => {
            set_last_error(GoudError::InvalidHandle);
            ERR_I32
        }
        Err(()) => ERR_I32,
    }
}

/// Sets the spatial listener position in 2D (`z = 0`).
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_set_listener_position(
    context_id: GoudContextId,
    x: f32,
    y: f32,
) -> i32 {
    goud_audio_set_listener_position_3d(context_id, x, y, 0.0)
}

/// Sets the spatial listener position in 3D.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_set_listener_position_3d(
    context_id: GoudContextId,
    x: f32,
    y: f32,
    z: f32,
) -> i32 {
    match with_audio(context_id, |audio| audio.set_listener_position([x, y, z])) {
        Ok(()) => 0,
        Err(()) => ERR_I32,
    }
}

/// Sets (or starts tracking) a source position in 2D (`z = 0`).
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_set_source_position(
    context_id: GoudContextId,
    player_id: u64,
    x: f32,
    y: f32,
    max_distance: f32,
    rolloff: f32,
) -> i32 {
    goud_audio_set_source_position_3d(context_id, player_id, x, y, 0.0, max_distance, rolloff)
}

/// Sets (or starts tracking) a source position in 3D.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_set_source_position_3d(
    context_id: GoudContextId,
    player_id: u64,
    x: f32,
    y: f32,
    z: f32,
    max_distance: f32,
    rolloff: f32,
) -> i32 {
    match with_audio(context_id, |audio| {
        update_spatial_source_3d(
            audio,
            player_id,
            [x, y, z],
            audio.listener_position(),
            max_distance,
            rolloff,
        )
    }) {
        Ok(true) => 0,
        Ok(false) => {
            set_last_error(GoudError::InvalidHandle);
            ERR_I32
        }
        Err(()) => ERR_I32,
    }
}

/// Sets per-player volume.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_set_player_volume(
    context_id: GoudContextId,
    player_id: u64,
    volume: f32,
) -> i32 {
    match with_audio(context_id, |audio| audio.set_sink_volume(player_id, volume)) {
        Ok(true) => 0,
        Ok(false) => {
            set_last_error(GoudError::InvalidHandle);
            ERR_I32
        }
        Err(()) => ERR_I32,
    }
}

/// Sets per-player playback speed.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_set_player_speed(
    context_id: GoudContextId,
    player_id: u64,
    speed: f32,
) -> i32 {
    match with_audio(context_id, |audio| audio.set_sink_speed(player_id, speed)) {
        Ok(true) => 0,
        Ok(false) => {
            set_last_error(GoudError::InvalidHandle);
            ERR_I32
        }
        Err(()) => ERR_I32,
    }
}

/// Applies an immediate two-player crossfade mix in `[0, 1]`.
///
/// `mix = 0` keeps `from_player_id` at full volume and mutes `to_player_id`.
/// `mix = 1` mutes `from_player_id` and sets `to_player_id` to full volume.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_crossfade(
    context_id: GoudContextId,
    from_player_id: u64,
    to_player_id: u64,
    mix: f32,
) -> i32 {
    match with_audio(context_id, |audio| {
        audio.set_crossfade_mix(from_player_id, to_player_id, mix)
    }) {
        Ok(true) => 0,
        Ok(false) => {
            set_last_error(GoudError::InvalidHandle);
            ERR_I32
        }
        Err(()) => ERR_I32,
    }
}

/// Starts a timed crossfade from an active sink to a newly played asset.
///
/// Returns a positive destination player ID on success and `-1` on failure.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_audio_crossfade_to(
    context_id: GoudContextId,
    from_player_id: u64,
    asset_data: *const u8,
    asset_len: usize,
    duration_sec: f32,
    channel: u8,
) -> i64 {
    // SAFETY: Caller guarantees `asset_data` points to `asset_len` valid bytes.
    let asset = match unsafe { audio_asset_from_raw(asset_data, asset_len) } {
        Ok(asset) => asset,
        Err(()) => return ERR_AUDIO,
    };

    match with_audio_mut(context_id, |audio| {
        audio.crossfade_to(
            from_player_id,
            &asset,
            duration_sec,
            AudioChannel::from_id(channel),
        )
    }) {
        Ok(Ok(player_id)) => player_id as i64,
        Ok(Err(e)) => {
            set_last_error(e);
            ERR_AUDIO
        }
        Err(()) => ERR_AUDIO,
    }
}

/// Starts additive mixing by layering a secondary asset on top of a primary sink.
///
/// Returns a positive secondary player ID on success and `-1` on failure.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_audio_mix_with(
    context_id: GoudContextId,
    primary_player_id: u64,
    asset_data: *const u8,
    asset_len: usize,
    secondary_volume: f32,
    secondary_channel: u8,
) -> i64 {
    // SAFETY: Caller guarantees `asset_data` points to `asset_len` valid bytes.
    let asset = match unsafe { audio_asset_from_raw(asset_data, asset_len) } {
        Ok(asset) => asset,
        Err(()) => return ERR_AUDIO,
    };

    match with_audio_mut(context_id, |audio| {
        audio.mix_with(
            primary_player_id,
            &asset,
            secondary_volume,
            AudioChannel::from_id(secondary_channel),
        )
    }) {
        Ok(Ok(player_id)) => player_id as i64,
        Ok(Err(e)) => {
            set_last_error(e);
            ERR_AUDIO
        }
        Err(()) => ERR_AUDIO,
    }
}

/// Advances active timed crossfades.
///
/// Returns `0` on success and `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_update_crossfades(context_id: GoudContextId, delta_sec: f32) -> i32 {
    match with_audio_mut(context_id, |audio| audio.update_crossfades(delta_sec)) {
        Ok(()) => 0,
        Err(()) => ERR_I32,
    }
}

/// Returns the number of in-flight timed crossfades, or `-1` on failure.
#[no_mangle]
pub extern "C" fn goud_audio_active_crossfade_count(context_id: GoudContextId) -> i32 {
    match with_audio(context_id, |audio| audio.active_crossfade_count()) {
        Ok(count) => count as i32,
        Err(()) => ERR_I32,
    }
}
