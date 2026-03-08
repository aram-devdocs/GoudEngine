//! Audio playback FFI functions: play, play_on_channel, play_with_settings.

use crate::assets::loaders::audio::asset::AudioData;
use crate::assets::loaders::audio::format::AudioFormat;
use crate::assets::loaders::AudioAsset;
use crate::assets::AudioManager;
use crate::core::error::{set_last_error, GoudError};
use crate::ecs::components::AudioChannel;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::ERR_AUDIO;

/// Plays audio from raw in-memory bytes on the default SFX channel.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `asset_data` - Pointer to encoded audio file bytes (WAV/OGG/MP3/FLAC)
/// * `asset_len` - Length of the audio data in bytes
///
/// # Returns
///
/// A positive player ID on success, or a negative value on error.
///
/// # Ownership
///
/// The caller retains ownership of `asset_data`. The engine copies
/// the bytes internally.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` valid bytes, or be null
/// (which triggers an error return).
#[no_mangle]
pub unsafe extern "C" fn goud_audio_play(
    context_id: GoudContextId,
    asset_data: *const u8,
    asset_len: usize,
) -> i64 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_AUDIO;
    }
    if asset_data.is_null() {
        set_last_error(GoudError::InvalidState(
            "asset_data pointer is null".to_string(),
        ));
        return ERR_AUDIO;
    }

    // SAFETY: Caller guarantees asset_data points to asset_len valid bytes.
    let bytes = std::slice::from_raw_parts(asset_data, asset_len).to_vec();
    let asset = AudioAsset::new(AudioData::InMemory(bytes), 0, 0, AudioFormat::Unknown, 0.0);

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_AUDIO;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_AUDIO;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_AUDIO;
        }
    };

    match audio.play(&asset) {
        Ok(id) => id as i64,
        Err(e) => {
            set_last_error(e);
            ERR_AUDIO
        }
    }
}

/// Plays audio on a specific channel.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `asset_data` - Pointer to encoded audio file bytes
/// * `asset_len` - Length of the audio data in bytes
/// * `channel` - Audio channel ID (0=Music, 1=SFX, 2=Voice, 3=Ambience,
///   4=UI, 5+=Custom)
///
/// # Returns
///
/// A positive player ID on success, or a negative value on error.
///
/// # Ownership
///
/// The caller retains ownership of `asset_data`.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` valid bytes, or be null.
#[no_mangle]
pub unsafe extern "C" fn goud_audio_play_on_channel(
    context_id: GoudContextId,
    asset_data: *const u8,
    asset_len: usize,
    channel: u8,
) -> i64 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_AUDIO;
    }
    if asset_data.is_null() {
        set_last_error(GoudError::InvalidState(
            "asset_data pointer is null".to_string(),
        ));
        return ERR_AUDIO;
    }

    // SAFETY: Caller guarantees asset_data points to asset_len valid bytes.
    let bytes = std::slice::from_raw_parts(asset_data, asset_len).to_vec();
    let asset = AudioAsset::new(AudioData::InMemory(bytes), 0, 0, AudioFormat::Unknown, 0.0);
    let ch = AudioChannel::from_id(channel);

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_AUDIO;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_AUDIO;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_AUDIO;
        }
    };

    match audio.play_on_channel(&asset, ch) {
        Ok(id) => id as i64,
        Err(e) => {
            set_last_error(e);
            ERR_AUDIO
        }
    }
}

/// Plays audio with full control over volume, speed, looping, and channel.
///
/// # Arguments
///
/// * `context_id` - Engine context handle
/// * `asset_data` - Pointer to encoded audio file bytes
/// * `asset_len` - Length of the audio data in bytes
/// * `volume` - Individual volume multiplier (0.0-1.0, clamped)
/// * `speed` - Playback speed multiplier (0.1-10.0, affects pitch)
/// * `looping` - Whether to loop indefinitely
/// * `channel` - Audio channel ID
///
/// # Returns
///
/// A positive player ID on success, or a negative value on error.
///
/// # Ownership
///
/// The caller retains ownership of `asset_data`.
///
/// # Safety
///
/// `asset_data` must point to at least `asset_len` valid bytes, or be null.
#[no_mangle]
pub unsafe extern "C" fn goud_audio_play_with_settings(
    context_id: GoudContextId,
    asset_data: *const u8,
    asset_len: usize,
    volume: f32,
    speed: f32,
    looping: bool,
    channel: u8,
) -> i64 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return ERR_AUDIO;
    }
    if asset_data.is_null() {
        set_last_error(GoudError::InvalidState(
            "asset_data pointer is null".to_string(),
        ));
        return ERR_AUDIO;
    }

    // SAFETY: Caller guarantees asset_data points to asset_len valid bytes.
    let bytes = std::slice::from_raw_parts(asset_data, asset_len).to_vec();
    let asset = AudioAsset::new(AudioData::InMemory(bytes), 0, 0, AudioFormat::Unknown, 0.0);
    let ch = AudioChannel::from_id(channel);

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return ERR_AUDIO;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return ERR_AUDIO;
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return ERR_AUDIO;
        }
    };

    match audio.play_with_settings(&asset, volume, speed, looping, ch) {
        Ok(id) => id as i64,
        Err(e) => {
            set_last_error(e);
            ERR_AUDIO
        }
    }
}
