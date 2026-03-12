use crate::core::debugger::{active_route_count, current_manifest, reset_for_tests, test_lock};
use crate::ffi::context::{
    goud_context_create_with_config, goud_context_destroy, GoudContextConfig, GoudDebuggerConfig,
};
use std::ffi::CString;

#[test]
fn test_debugger_ffi_context_create_with_config_registers_route() {
    let _guard = test_lock();
    reset_for_tests();

    let label = CString::new("ffi-context").unwrap();
    let config = GoudContextConfig {
        debugger: GoudDebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: label.as_ptr(),
        },
    };

    // SAFETY: `config` points to a valid FFI struct for the duration of the call.
    let context_id = unsafe { goud_context_create_with_config(&config) };
    assert!(!context_id.is_invalid());
    assert_eq!(active_route_count(), 1);
    let manifest = current_manifest().expect("manifest should exist");
    assert_eq!(manifest.routes.len(), 1);
    assert_eq!(manifest.routes[0].label.as_deref(), Some("ffi-context"));

    assert!(goud_context_destroy(context_id));
    assert_eq!(active_route_count(), 0);
    assert!(current_manifest().is_none());
}

#[test]
fn test_debugger_ffi_context_create_with_disabled_config_keeps_runtime_unpublished() {
    let _guard = test_lock();
    reset_for_tests();

    let label = CString::new("disabled-ffi-context").unwrap();
    let config = GoudContextConfig {
        debugger: GoudDebuggerConfig {
            enabled: false,
            publish_local_attach: true,
            route_label: label.as_ptr(),
        },
    };

    // SAFETY: `config` points to a valid FFI struct for the duration of the call.
    let context_id = unsafe { goud_context_create_with_config(&config) };
    assert!(!context_id.is_invalid());
    assert_eq!(active_route_count(), 0);
    assert!(current_manifest().is_none());

    assert!(goud_context_destroy(context_id));
    assert_eq!(active_route_count(), 0);
    assert!(current_manifest().is_none());
}
