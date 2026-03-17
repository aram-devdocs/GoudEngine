#![cfg(feature = "native")]

use super::super::{
    dispatch_request_json_for_route, register_context, reset_for_tests, scoped_route,
    snapshot_for_route, take_frame_control_for_route, test_lock, DebuggerConfig, RuntimeRouteId,
    RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use crate::core::input_manager::InputManager;
use crate::core::math::Vec2;
use crate::core::providers::input_types::KeyCode as Key;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct RecordedInputEvent {
    frame_index: u64,
    event: super::super::SyntheticInputEventV1,
}

fn dispatch(route_id: &RuntimeRouteId, request: Value) -> Value {
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

fn result_data_bytes(response: &Value) -> Vec<u8> {
    response["result"]["data"]
        .as_array()
        .expect("result.data should be an array")
        .iter()
        .map(|value| {
            u8::try_from(value.as_u64().expect("each byte should be u64"))
                .expect("byte should fit into u8")
        })
        .collect()
}

#[test]
fn test_recording_captures_key_and_mouse_move_and_exports_data() {
    let _guard = test_lock();
    reset_for_tests();
    let runtime_dir = test_runtime_dir("replay");
    let _ = fs::remove_dir_all(&runtime_dir);
    fs::create_dir_all(&runtime_dir).expect("runtime dir should be created");
    std::env::set_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR", &runtime_dir);

    let route = register_context(
        GoudContextId::new(81, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("replay-record".to_string()),
        },
    );

    let _ = dispatch(&route, json!({ "verb": "start_recording" }));
    let mut input = InputManager::new();
    scoped_route(Some(route.clone()), || {
        input.press_key(Key::Space);
        input.set_mouse_position(Vec2::new(320.0, 240.0));
    });

    let response = dispatch(&route, json!({ "verb": "stop_recording" }));
    std::env::remove_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR");
    assert_eq!(response["ok"], true);
    let artifact_id = response["result"]["artifact_id"]
        .as_str()
        .expect("stop_recording should include artifact_id");
    assert!(!artifact_id.is_empty());
    let manifest_json = response["result"]["manifest_json"]
        .as_str()
        .expect("manifest_json should be a string");
    assert!(manifest_json.contains("determinism"));

    let data = result_data_bytes(&response);
    assert!(!data.is_empty());

    let recorded: Vec<RecordedInputEvent> =
        serde_json::from_slice(&data).expect("recorded data should decode");
    assert!(recorded.iter().any(|event| event.frame_index == 1
        && event.event.device == "keyboard"
        && event.event.action == "press"
        && event.event.key.as_deref() == Some("space")));
    assert!(recorded.iter().any(|event| event.frame_index == 1
        && event.event.device == "mouse"
        && event.event.action == "move"
        && event.event.position == Some([320.0, 240.0])));

    let route_bucket = format!("route-{}", route.context_id);
    let artifact_dir = format!("{runtime_dir}/artifacts/{route_bucket}/recording/{artifact_id}");
    assert!(fs::metadata(format!("{artifact_dir}/manifest.json")).is_ok());
    assert!(fs::metadata(format!("{artifact_dir}/data.bin")).is_ok());
}

#[test]
fn test_start_replay_surfaces_events_on_matching_frame() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(82, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("replay-play".to_string()),
        },
    );

    let _ = dispatch(&route, json!({ "verb": "start_recording" }));
    let mut input = InputManager::new();
    scoped_route(Some(route.clone()), || {
        input.press_key(Key::Space);
        input.set_mouse_position(Vec2::new(640.0, 360.0));
    });
    let recorded = dispatch(&route, json!({ "verb": "stop_recording" }));
    let data = result_data_bytes(&recorded);

    let replay_start = dispatch(&route, json!({ "verb": "start_replay", "data": data }));
    assert_eq!(replay_start["ok"], true);

    let current_frame = snapshot_for_route(&route)
        .expect("snapshot should exist")
        .frame
        .index;
    assert_eq!(current_frame, 0);

    let frame_plan = take_frame_control_for_route(&route, 0.016).expect("frame plan should exist");
    assert!(frame_plan.synthetic_inputs.iter().any(|event| {
        event.device == "keyboard"
            && event.action == "press"
            && event.key.as_deref() == Some("space")
    }));
    assert!(frame_plan.synthetic_inputs.iter().any(|event| {
        event.device == "mouse" && event.action == "move" && event.position == Some([640.0, 360.0])
    }));
}

#[test]
fn test_get_replay_status_returns_mode_and_detail_json_fields() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(83, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("replay-status".to_string()),
        },
    );

    let idle = dispatch(&route, json!({ "verb": "get_replay_status" }));
    assert_eq!(idle["ok"], true);
    assert_eq!(idle["result"]["mode"], "idle");
    assert!(idle["result"]["detail"].is_string());

    let _ = dispatch(&route, json!({ "verb": "start_recording" }));
    let recording = dispatch(&route, json!({ "verb": "get_replay_status" }));
    assert_eq!(recording["ok"], true);
    assert_eq!(recording["result"]["mode"], "recording");
    assert!(recording["result"]["detail"].is_string());
}
