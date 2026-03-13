use super::super::{
    begin_frame, dispatch_request_json_for_route, end_frame, register_context, reset_for_tests,
    test_lock, DebuggerConfig, RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use serde_json::json;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn dispatch(
    route_id: &super::super::RuntimeRouteId,
    request: serde_json::Value,
) -> serde_json::Value {
    dispatch_request_json_for_route(route_id, &request.to_string())
        .expect("dispatcher should return JSON")
}

fn test_runtime_dir(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("/tmp/goud-debugger-{prefix}-{}-{nanos}", std::process::id())
}

#[test]
fn test_metrics_trace_frames_are_capped_to_300() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(51, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("metrics-cap".to_string()),
        },
    );

    for index in 1..=350_u64 {
        begin_frame(&route, index, 0.016, index as f64 * 0.016);
        end_frame(&route);
    }

    let response = dispatch(&route, json!({ "verb": "get_metrics_trace" }));
    assert_eq!(response["ok"], true);
    let frames = response["result"]["frames"]
        .as_array()
        .expect("frames should be an array");
    assert_eq!(frames.len(), 300);
    assert_eq!(frames.first().unwrap()["frame"]["index"], 51);
    assert_eq!(frames.last().unwrap()["frame"]["index"], 350);
}

#[test]
fn test_get_metrics_trace_returns_versioned_object_shape() {
    let _guard = test_lock();
    reset_for_tests();
    let runtime_dir = test_runtime_dir("metrics");
    let _ = fs::remove_dir_all(&runtime_dir);
    fs::create_dir_all(&runtime_dir).expect("runtime dir should be created");
    std::env::set_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR", &runtime_dir);

    let route = register_context(
        GoudContextId::new(52, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("metrics-shape".to_string()),
        },
    );

    let response = dispatch(&route, json!({ "verb": "get_metrics_trace" }));
    std::env::remove_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR");
    assert_eq!(response["ok"], true);
    let result = response["result"]
        .as_object()
        .expect("metrics trace should be an object");
    let artifact_id = result
        .get("artifact_id")
        .and_then(|value| value.as_str())
        .expect("metrics result should include artifact_id");
    assert!(!artifact_id.is_empty());

    assert!(result.contains_key("version"));
    assert!(result.contains_key("route"));
    assert!(result.contains_key("frames"));
    assert!(result.contains_key("events"));

    let route_bucket = format!("route-{}", route.context_id);
    let metrics_file = format!("{runtime_dir}/artifacts/{route_bucket}/metrics/{artifact_id}.json");
    assert!(fs::metadata(metrics_file).is_ok());
}
