use crate::assets::AudioManager;
use crate::core::error::{set_last_error, GoudError};
use crate::ecs::components::AudioChannel;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::ERR_I32;

/// Returns the current global audio volume.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
///
/// # Returns
///
/// The global volume (0.0-1.0), or `-1.0` on error.
pub(super) fn goud_audio_get_global_volume_impl(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return -1.0;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return -1.0,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return -1.0,
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => return -1.0,
    };

    audio.global_volume()
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
pub(super) fn goud_audio_get_channel_volume_impl(context_id: GoudContextId, channel: u8) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return -1.0;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return -1.0,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return -1.0,
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => return -1.0,
    };

    audio.get_channel_volume(AudioChannel::from_id(channel))
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
pub(super) fn goud_audio_is_playing_impl(context_id: GoudContextId, player_id: u64) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return ERR_I32;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return ERR_I32,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return ERR_I32,
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => return ERR_I32,
    };

    if audio.is_playing(player_id) {
        1
    } else {
        0
    }
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
pub(super) fn goud_audio_active_count_impl(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return ERR_I32;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return ERR_I32,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return ERR_I32,
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => return ERR_I32,
    };

    audio.active_count() as i32
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
pub(super) fn goud_audio_cleanup_finished_impl(context_id: GoudContextId) -> i32 {
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
    match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => {
            am.cleanup_finished();
            0
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            ERR_I32
        }
    }
}
