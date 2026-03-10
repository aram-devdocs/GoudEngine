use crate::assets::AudioManager;
use crate::core::error::{set_last_error, GoudError};
use crate::ecs::components::AudioChannel;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::ERR_I32;

#[path = "audio_activate.rs"]
mod audio_activate;
#[path = "query.rs"]
mod query;
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

use audio_activate::goud_audio_activate_impl;
use query::{
    goud_audio_active_count_impl, goud_audio_cleanup_finished_impl,
    goud_audio_get_channel_volume_impl, goud_audio_get_global_volume_impl,
    goud_audio_is_playing_impl,
};

/// Activates audio playback on platforms that require explicit initialization.
///
/// For native contexts this is intentionally validation-only.
///
/// Browser/WebAudio activation lives in
/// `crate::wasm::audio::activation_playback::WasmGame::audio_activate`.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_activate(context_id: GoudContextId) -> i32 {
    goud_audio_activate_impl(context_id)
}

/// Stops audio playback for the given player ID.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `player_id` - ID returned from a play function
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_stop(context_id: GoudContextId, player_id: u64) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_I32;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_I32;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_I32;
        }
    };

    audio.stop(player_id);
    0
}

/// Pauses audio playback for the given player ID.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `player_id` - ID returned from a play function
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_pause(context_id: GoudContextId, player_id: u64) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_I32;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_I32;
        }
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_I32;
        }
    };

    audio.pause(player_id);
    0
}

/// Resumes audio playback for the given player ID.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `player_id` - ID returned from a play function
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_resume(context_id: GoudContextId, player_id: u64) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_I32;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_I32;
        }
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_I32;
        }
    };

    audio.resume(player_id);
    0
}

/// Stops all currently playing audio.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_stop_all(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_I32;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_I32;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_I32;
        }
    };

    audio.stop_all();
    0
}

// ============================================================================
// Volume Control
// ============================================================================

/// Sets the global audio volume.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `volume` - Volume level (0.0 = mute, 1.0 = full; clamped)
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_set_global_volume(context_id: GoudContextId, volume: f32) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_I32;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_I32;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_I32;
        }
    };

    audio.set_global_volume(volume);
    0
}

/// Sets the volume for a specific audio channel.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `channel` - Audio channel ID
/// * `volume` - Volume level (0.0-1.0, clamped)
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_set_channel_volume(
    context_id: GoudContextId,
    channel: u8,
    volume: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_I32;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_I32;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_I32;
        }
    };

    audio.set_channel_volume(AudioChannel::from_id(channel), volume);
    0
}

/// Returns the current global audio volume.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
///
/// # Returns
///
/// The global volume (0.0-1.0), or `-1.0` on error.
#[no_mangle]
pub extern "C" fn goud_audio_get_global_volume(context_id: GoudContextId) -> f32 {
    goud_audio_get_global_volume_impl(context_id)
}

/// Returns the current volume for a specific audio channel.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `channel` - Audio channel ID
///
/// # Returns
///
/// The channel volume (0.0-1.0), or `-1.0` on error.
#[no_mangle]
pub extern "C" fn goud_audio_get_channel_volume(context_id: GoudContextId, channel: u8) -> f32 {
    goud_audio_get_channel_volume_impl(context_id, channel)
}

/// Checks whether a specific player is currently playing audio.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `player_id` - ID returned from a play function
///
/// # Returns
///
/// `1` if playing, `0` if not playing (or paused/finished), `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_is_playing(context_id: GoudContextId, player_id: u64) -> i32 {
    goud_audio_is_playing_impl(context_id, player_id)
}

/// Returns the number of active audio players.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
///
/// # Returns
///
/// The number of active players, or `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_active_count(context_id: GoudContextId) -> i32 {
    goud_audio_active_count_impl(context_id)
}

/// Cleans up finished audio players.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
///
/// # Returns
///
/// `0` on success, `-1` on error.
#[no_mangle]
pub extern "C" fn goud_audio_cleanup_finished(context_id: GoudContextId) -> i32 {
    goud_audio_cleanup_finished_impl(context_id)
}
