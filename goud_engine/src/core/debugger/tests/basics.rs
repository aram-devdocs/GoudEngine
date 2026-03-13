use super::super::{
    active_route_count, current_manifest, current_route, default_capabilities, default_services,
    register_context, reset_for_tests, scoped_route, set_profiling_enabled_for_context,
    set_selected_entity_for_context, snapshot_for_context, snapshot_for_route, test_lock,
    update_memory_category_for_context, update_render_stats_for_context, CapabilityStateV1,
    DebuggerConfig, RuntimeSurfaceKind, ROUTE_CAPABILITY_KEYS,
};
use crate::core::context_id::GoudContextId;
use std::panic::{catch_unwind, AssertUnwindSafe};

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

#[test]
fn test_debugger_runtime_disabled_path_keeps_runtime_unpublished_and_no_op() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(10, 1);

    assert_eq!(active_route_count(), 0);
    assert!(current_manifest().is_none());
    assert!(snapshot_for_context(context_id).is_none());
    assert!(!set_profiling_enabled_for_context(context_id, true));
    assert!(!set_selected_entity_for_context(context_id, Some(99)));
    assert!(!update_render_stats_for_context(context_id, 1, 2, 3, 4));
    assert!(!update_memory_category_for_context(context_id, "ecs", 512));
}

#[test]
fn test_debugger_runtime_collects_per_frame_render_and_memory_stats() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(11, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::WindowedGame,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("stats".to_string()),
        },
    );

    super::super::begin_frame(&route, 1, 0.016, 0.016);
    assert!(update_render_stats_for_context(context_id, 2, 6, 3, 1));
    assert!(update_memory_category_for_context(context_id, "ecs", 256));
    assert!(update_memory_category_for_context(context_id, "ecs", 128));
    super::super::end_frame(&route);

    let snapshot = snapshot_for_context(context_id).expect("snapshot should exist");
    assert_eq!(snapshot.stats.render.draw_calls, 2);
    assert_eq!(snapshot.stats.render.triangles, 6);
    assert_eq!(snapshot.stats.render.texture_binds, 3);
    assert_eq!(snapshot.stats.render.shader_binds, 1);
    assert_eq!(snapshot.memory_summary.ecs.current_bytes, 128);
    assert_eq!(snapshot.memory_summary.ecs.peak_bytes, 256);
    assert!(snapshot.stats.memory.peak_bytes >= 256);
}

#[test]
fn test_debugger_runtime_resets_profiler_samples_each_frame() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(12, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: None,
        },
    );

    assert!(set_profiling_enabled_for_context(context_id, true));
    super::super::begin_frame(&route, 1, 0.016, 0.016);
    super::super::set_system_sample(&route, "update", "ExampleSystem", 42);
    assert_eq!(
        snapshot_for_route(&route).unwrap().profiler_samples.len(),
        1
    );

    super::super::begin_frame(&route, 2, 0.016, 0.032);
    assert!(snapshot_for_route(&route)
        .unwrap()
        .profiler_samples
        .is_empty());
}

#[test]
fn test_debugger_runtime_records_active_phase_samples_only_when_enabled() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(13, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::WindowedGame,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: None,
        },
    );

    scoped_route(Some(route.clone()), || {
        super::super::record_phase_duration("window_events", 17);
    });
    assert!(snapshot_for_route(&route)
        .unwrap()
        .profiler_samples
        .is_empty());

    assert!(set_profiling_enabled_for_context(context_id, true));
    scoped_route(Some(route.clone()), || {
        super::super::record_phase_duration("window_events", 17);
    });
    let snapshot = snapshot_for_route(&route).expect("snapshot should exist");
    assert_eq!(snapshot.profiler_samples.len(), 1);
    assert_eq!(snapshot.profiler_samples[0].sample_kind, "phase");
    assert_eq!(snapshot.profiler_samples[0].name, "window_events");
}

#[test]
fn test_debugger_runtime_selection_is_route_local() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(14, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: None,
        },
    );

    assert!(set_selected_entity_for_context(context_id, Some(77)));
    let snapshot = snapshot_for_route(&route).expect("snapshot should exist");
    assert_eq!(snapshot.selection.entity_id, Some(77));

    assert!(set_selected_entity_for_context(context_id, None));
    let snapshot = snapshot_for_route(&route).expect("snapshot should exist");
    assert_eq!(snapshot.selection.entity_id, None);
}

#[test]
fn test_debugger_runtime_selection_uses_active_scene_when_available() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(15, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: None,
        },
    );

    let _ = super::super::with_snapshot_mut(&route, |snapshot| {
        snapshot.scene.active_scene = "secondary".to_string();
    });

    assert!(set_selected_entity_for_context(context_id, Some(77)));
    let snapshot = snapshot_for_route(&route).expect("snapshot should exist");
    assert_eq!(snapshot.selection.scene_id, "secondary");
    assert_eq!(snapshot.selection.entity_id, Some(77));
}

#[test]
fn test_scoped_route_restores_thread_local_after_panic() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(16, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: None,
        },
    );

    let result = catch_unwind(AssertUnwindSafe(|| {
        scoped_route(Some(route.clone()), || panic!("boom"));
    }));

    assert!(result.is_err());
    assert_eq!(current_route(), None);
}
