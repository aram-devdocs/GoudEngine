use crate::assets::loaders::audio::asset::AudioData;
use crate::assets::loaders::audio::format::AudioFormat;
use crate::assets::loaders::AudioAsset;
use crate::assets::AudioManager;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

pub(super) fn with_audio_mut<R>(
    context_id: GoudContextId,
    op: impl FnOnce(&mut AudioManager) -> R,
) -> Result<R, ()> {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return Err(());
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return Err(());
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return Err(());
        }
    };
    let audio = match context.world_mut().get_resource_mut::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return Err(());
        }
    };

    Ok(op(audio))
}

pub(super) fn with_audio<R>(
    context_id: GoudContextId,
    op: impl FnOnce(&AudioManager) -> R,
) -> Result<R, ()> {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return Err(());
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return Err(());
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return Err(());
        }
    };
    let audio = match context.world().get_resource::<AudioManager>() {
        Some(am) => am,
        None => {
            set_last_error(GoudError::InvalidState(
                "AudioManager resource not found".to_string(),
            ));
            return Err(());
        }
    };

    Ok(op(audio))
}

pub(super) unsafe fn audio_asset_from_raw(
    asset_data: *const u8,
    asset_len: usize,
) -> Result<AudioAsset, ()> {
    if asset_data.is_null() {
        set_last_error(GoudError::InvalidState(
            "asset_data pointer is null".to_string(),
        ));
        return Err(());
    }

    // SAFETY: Caller guarantees `asset_data` is valid for `asset_len` bytes.
    let bytes = unsafe { std::slice::from_raw_parts(asset_data, asset_len) }.to_vec();
    Ok(AudioAsset::new(
        AudioData::InMemory(bytes),
        0,
        0,
        AudioFormat::Unknown,
        0.0,
    ))
}

pub(super) fn update_spatial_source_3d(
    audio: &AudioManager,
    player_id: u64,
    source_position: [f32; 3],
    listener_position: [f32; 3],
    max_distance: f32,
    rolloff: f32,
) -> bool {
    if !audio.has_sink(player_id) {
        return false;
    }

    audio.set_listener_position(listener_position);

    if audio.set_source_position(player_id, source_position) {
        true
    } else {
        let base_volume = audio.sink_volume(player_id).unwrap_or(1.0);
        audio.register_spatial_source(
            player_id,
            source_position,
            max_distance,
            rolloff,
            base_volume,
        )
    }
}
