use super::{
    active_route_count, current_manifest, default_capabilities, default_services, register_context,
    reset_for_tests, snapshot_for_route, test_lock, CapabilityStateV1, DebuggerConfig,
    RuntimeSurfaceKind, ROUTE_CAPABILITY_KEYS,
};
use crate::context_registry::GoudContextId;

#[test]
fn test_debugger_service_health_defaults_cover_all_required_services() {
    let _guard = test_lock();
    reset_for_tests();

    let services = default_services();
    let names: Vec<&str> = services
        .iter()
        .map(|service| service.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "renderer",
            "memory",
            "profiling",
            "physics",
            "audio",
            "network",
            "window",
            "assets",
            "capture",
            "replay",
            "debugger",
        ]
    );
    assert_eq!(services.last().unwrap().state, CapabilityStateV1::Ready);
}

#[test]
fn test_debugger_default_capabilities_cover_all_wire_keys() {
    let _guard = test_lock();
    let capabilities = default_capabilities();
    for key in ROUTE_CAPABILITY_KEYS {
        assert!(
            capabilities.contains_key(key),
            "missing capability key {key}"
        );
    }
    assert_eq!(capabilities.len(), ROUTE_CAPABILITY_KEYS.len());
    assert_eq!(
        capabilities.get("snapshots"),
        Some(&CapabilityStateV1::Ready)
    );
}

#[test]
fn test_debugger_manifest_stays_unpublished_without_attachable_routes() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(9, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("hidden".to_string()),
        },
    );

    assert_eq!(active_route_count(), 1);
    assert!(snapshot_for_route(&route).is_some());
    assert!(current_manifest().is_none());
}
