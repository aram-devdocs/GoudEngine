use super::*;
use crate::core::error::last_error_message;

#[test]
fn test_set_render_backend_rejects_unknown_code() {
    let handle = goud_engine_config_create();
    assert!(!handle.is_null());

    // SAFETY: `handle` was returned by `goud_engine_config_create` above.
    let ok = unsafe { goud_engine_config_set_render_backend(handle, 99) };
    assert!(!ok);
    assert!(last_error_message()
        .expect("expected error message")
        .contains("invalid render backend"),);

    // SAFETY: `handle` was allocated by `goud_engine_config_create` above.
    unsafe { goud_engine_config_destroy(handle) };
}

#[test]
fn test_set_window_backend_rejects_unknown_code() {
    let handle = goud_engine_config_create();
    assert!(!handle.is_null());

    // SAFETY: `handle` was returned by `goud_engine_config_create` above.
    let ok = unsafe { goud_engine_config_set_window_backend(handle, 99) };
    assert!(!ok);
    assert!(last_error_message()
        .expect("expected error message")
        .contains("invalid window backend"),);

    // SAFETY: `handle` was allocated by `goud_engine_config_create` above.
    unsafe { goud_engine_config_destroy(handle) };
}

#[cfg(feature = "native")]
#[test]
fn test_engine_create_rejects_mixed_native_backend_pair() {
    let handle = goud_engine_config_create();
    assert!(!handle.is_null());

    // SAFETY: `handle` is valid for all setter calls below.
    unsafe {
        assert!(goud_engine_config_set_window_backend(
            handle,
            WindowBackendKind::Winit as u32,
        ));
        assert!(goud_engine_config_set_render_backend(
            handle,
            RenderBackendKind::OpenGlLegacy as u32,
        ));
    }

    // SAFETY: `handle` is consumed by `goud_engine_create`.
    let context_id = unsafe { goud_engine_create(handle) };
    assert_eq!(context_id, crate::ffi::context::GOUD_INVALID_CONTEXT_ID);
    assert!(last_error_message()
        .expect("expected error message")
        .contains("invalid native backend pair"),);
}
