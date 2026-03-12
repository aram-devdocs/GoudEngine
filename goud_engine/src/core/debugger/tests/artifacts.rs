use super::super::{
    current_manifest, manifest_artifact_path_for_tests, register_context, reset_for_tests,
    set_profiling_enabled_for_context, test_lock, unregister_context, DebuggerConfig,
    RuntimeManifestV1, RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn manifest_from_disk() -> RuntimeManifestV1 {
    let path = manifest_artifact_path_for_tests().expect("manifest path should be published");
    let json = fs::read_to_string(path).expect("manifest file should be readable");
    serde_json::from_str(&json).expect("manifest JSON should parse")
}

#[test]
fn test_manifest_publication_writes_disk_artifact_with_strictly_monotonic_timestamp() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(31, 1);
    register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("artifact".to_string()),
        },
    );

    let first = manifest_from_disk();
    assert!(first.published_at_unix_ms > 0);
    assert_eq!(current_manifest(), Some(first.clone()));

    assert!(set_profiling_enabled_for_context(context_id, true));

    let second = manifest_from_disk();
    assert!(second.published_at_unix_ms > first.published_at_unix_ms);
    assert_eq!(current_manifest(), Some(second));
}

#[test]
fn test_manifest_artifact_is_removed_when_last_attachable_route_detaches() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(32, 1);
    register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("cleanup".to_string()),
        },
    );

    let path = manifest_artifact_path_for_tests().expect("manifest path should exist");
    assert!(path.exists());

    unregister_context(context_id);

    assert!(current_manifest().is_none());
    assert!(!path.exists(), "manifest artifact should be cleaned up");
}

#[cfg(unix)]
#[test]
fn test_manifest_publication_uses_owner_only_permissions() {
    let _guard = test_lock();
    reset_for_tests();

    register_context(
        GoudContextId::new(33, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("permissions".to_string()),
        },
    );

    let path = manifest_artifact_path_for_tests().expect("manifest path should exist");
    let file_mode = fs::metadata(&path)
        .expect("manifest metadata should exist")
        .permissions()
        .mode()
        & 0o777;
    let dir_mode = fs::metadata(path.parent().expect("manifest should have a parent"))
        .expect("runtime directory metadata should exist")
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(dir_mode, 0o700);
    assert_eq!(file_mode, 0o600);
}
