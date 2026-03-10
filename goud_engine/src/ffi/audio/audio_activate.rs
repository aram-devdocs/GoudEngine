use crate::core::error::{set_last_error, GoudError};
use crate::ffi::audio::ERR_I32;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// Validates a native context for audio activation.
///
/// Native backends do not require an activation handshake, so this intentionally
/// stops after context validation. Browser/WebAudio activation lives in
/// `crate::wasm::audio::activation_playback::WasmGame::audio_activate`.
pub(crate) fn goud_audio_activate_impl(context_id: GoudContextId) -> i32 {
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

    if registry.get(context_id).is_none() {
        set_last_error(GoudError::InvalidContext);
        return ERR_I32;
    }

    0
}
