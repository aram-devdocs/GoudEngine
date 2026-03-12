use super::super::{
    dispatch_request_json_for_route, register_capture_hook_for_route, register_context,
    reset_for_tests, test_lock, DebuggerConfig, RawFramebufferReadbackV1, RuntimeRouteId,
    RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use serde_json::json;

fn dispatch(route_id: &RuntimeRouteId, request: serde_json::Value) -> serde_json::Value {
    dispatch_request_json_for_route(route_id, &request.to_string())
        .expect("dispatcher should return JSON")
}

#[test]
fn test_capture_frame_returns_capture_artifact_object_for_windowed_route() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(61, 1),
        RuntimeSurfaceKind::WindowedGame,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("capture-windowed".to_string()),
        },
    );

    register_capture_hook_for_route(route.clone(), || {
        Ok(RawFramebufferReadbackV1 {
            width: 1,
            height: 1,
            rgba8: vec![255, 0, 0, 255],
        })
    });

    let response = dispatch(&route, json!({ "verb": "capture_frame" }));
    assert_eq!(response["ok"], true);
    let result = &response["result"];
    assert!(result["image_png"].is_array());
    assert!(result["image_png"]
        .as_array()
        .is_some_and(|bytes| !bytes.is_empty()));
    assert!(result["metadata_json"].is_string());
    assert!(result["snapshot_json"].is_string());
    assert!(result["metrics_trace_json"].is_string());
}

#[test]
fn test_capture_frame_for_headless_route_stays_unsupported() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(62, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("capture-headless".to_string()),
        },
    );

    let response = dispatch(&route, json!({ "verb": "capture_frame" }));
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "unsupported");
    assert_eq!(response["error"]["capability"], "capture");
}
