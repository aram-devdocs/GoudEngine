use super::debugger_control::{
    goud_debugger_get_metrics_trace_json, goud_debugger_get_replay_status_json,
    goud_debugger_inject_key_event, goud_debugger_inject_mouse_button,
    goud_debugger_inject_mouse_position, goud_debugger_inject_scroll,
    goud_debugger_set_debug_draw_enabled, goud_debugger_set_paused, goud_debugger_set_time_scale,
    goud_debugger_start_recording, goud_debugger_start_replay, goud_debugger_step,
    goud_debugger_stop_replay, GoudDebuggerStepKind,
};
use crate::core::context_id::GoudContextId;
use crate::core::debugger::{
    control_state_for_route, dispatch_request_json_for_route, register_context, reset_for_tests,
    take_frame_control_for_route, test_lock, DebuggerConfig, RuntimeRouteId, RuntimeSurfaceKind,
};
use crate::ffi::input::{KEY_SPACE, MOUSE_BUTTON_LEFT};
use serde_json::{json, Value};

fn register_test_route(label: &str, index: u32) -> (GoudContextId, RuntimeRouteId) {
    let context_id = GoudContextId::new(index, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some(label.to_string()),
        },
    );
    (context_id, route)
}

fn read_json(
    function: unsafe extern "C" fn(GoudContextId, *mut u8, usize) -> i32,
    context_id: GoudContextId,
) -> Value {
    // SAFETY: Null buffer with zero length is the supported size query path.
    let required = unsafe { function(context_id, std::ptr::null_mut(), 0) };
    assert!(required < 0);
    let mut buf = vec![0_u8; (-required) as usize];
    // SAFETY: `buf` points to writable storage for `buf.len()` bytes.
    let written = unsafe { function(context_id, buf.as_mut_ptr(), buf.len()) };
    assert!(written > 0);
    serde_json::from_slice(&buf[..written as usize]).expect("json should parse")
}

fn replay_data_bytes(response: &Value) -> Vec<u8> {
    response["result"]["data"]
        .as_array()
        .expect("recording export should include bytes")
        .iter()
        .map(|value| {
            u8::try_from(value.as_u64().expect("replay byte should be u64"))
                .expect("replay byte should fit into u8")
        })
        .collect()
}

fn assert_single_queued_event(
    route: &RuntimeRouteId,
    device: &str,
    action: &str,
    key: Option<&str>,
    button: Option<&str>,
    position: Option<[f32; 2]>,
    delta: Option<[f32; 2]>,
) {
    let control = control_state_for_route(route).expect("control state should exist");
    assert_eq!(control.queued_input_events, 1);
    let frame = take_frame_control_for_route(route, 0.016).expect("frame plan should exist");
    assert_eq!(frame.synthetic_inputs.len(), 1);
    let event = &frame.synthetic_inputs[0];
    assert_eq!(event.device, device);
    assert_eq!(event.action, action);
    assert_eq!(event.key.as_deref(), key);
    assert_eq!(event.button.as_deref(), button);
    assert_eq!(event.position, position);
    assert_eq!(event.delta, delta);
}

#[test]
fn test_control_exports_update_route_control_state() {
    let _guard = test_lock();
    reset_for_tests();
    let (context_id, route) = register_test_route("ffi-control", 301);

    assert_eq!(goud_debugger_set_paused(context_id, true), 0);
    assert_eq!(
        goud_debugger_step(context_id, GoudDebuggerStepKind::Frame, 2),
        0
    );
    assert_eq!(
        goud_debugger_step(context_id, GoudDebuggerStepKind::Tick, 3),
        0
    );
    assert_eq!(goud_debugger_set_time_scale(context_id, 0.25), 0);
    assert_eq!(goud_debugger_set_debug_draw_enabled(context_id, true), 0);
    let control = control_state_for_route(&route).expect("control state should exist");
    assert!(control.paused);
    assert_eq!(control.frame_step_budget, 2);
    assert_eq!(control.tick_step_budget, 3);
    assert_eq!(control.time_scale, 0.25);
    assert!(control.debug_draw_enabled);
}

#[test]
fn test_input_injection_exports_queue_one_event_each() {
    let _guard = test_lock();
    reset_for_tests();
    let (context_id, route) = register_test_route("ffi-input", 302);

    assert_eq!(
        goud_debugger_inject_key_event(context_id, KEY_SPACE, true),
        0
    );
    assert_single_queued_event(&route, "keyboard", "press", Some("space"), None, None, None);
    assert_eq!(
        goud_debugger_inject_mouse_button(context_id, MOUSE_BUTTON_LEFT, false),
        0
    );
    assert_single_queued_event(&route, "mouse", "release", None, Some("left"), None, None);
    assert_eq!(
        goud_debugger_inject_mouse_position(context_id, 320.0, 240.0),
        0
    );
    assert_single_queued_event(
        &route,
        "mouse",
        "move",
        None,
        None,
        Some([320.0, 240.0]),
        None,
    );

    assert_eq!(goud_debugger_inject_scroll(context_id, 1.5, -2.0), 0);
    assert_single_queued_event(
        &route,
        "mouse",
        "scroll",
        None,
        None,
        None,
        Some([1.5, -2.0]),
    );
}

#[test]
fn test_metrics_and_replay_status_json_exports_return_non_empty_json() {
    let _guard = test_lock();
    reset_for_tests();
    let (context_id, _) = register_test_route("ffi-json", 303);
    let metrics = read_json(goud_debugger_get_metrics_trace_json, context_id);
    assert!(metrics["version"].is_number());
    let replay = read_json(goud_debugger_get_replay_status_json, context_id);
    assert!(replay["mode"].is_string());
}

#[test]
fn test_recording_and_replay_exports_succeed_for_valid_route() {
    let _guard = test_lock();
    reset_for_tests();
    let (context_id, route) = register_test_route("ffi-replay", 304);

    assert_eq!(goud_debugger_start_recording(context_id), 0);
    let status = read_json(goud_debugger_get_replay_status_json, context_id);
    assert_eq!(status["mode"], "recording");
    let recorded =
        dispatch_request_json_for_route(&route, &json!({ "verb": "stop_recording" }).to_string())
            .expect("dispatcher should return JSON");
    assert_eq!(recorded["ok"], true);
    let data = replay_data_bytes(&recorded);
    assert!(!data.is_empty());
    // SAFETY: `data` owns a readable byte slice for the duration of the call.
    let replay_result =
        unsafe { goud_debugger_start_replay(context_id, data.as_ptr(), data.len()) };
    assert_eq!(replay_result, 0);
    assert_eq!(goud_debugger_stop_replay(context_id), 0);
    let status = read_json(goud_debugger_get_replay_status_json, context_id);
    assert_eq!(status["mode"], "idle");
}
