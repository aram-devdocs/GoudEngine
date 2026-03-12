use super::super::{
    control_state_for_route, dispatch_request_json_for_route, register_context, reset_for_tests,
    snapshot_for_route, take_frame_control_for_route, test_lock, DebuggerConfig,
    RouteControlStateV1, RuntimeRouteId, RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use serde_json::json;

fn dispatch(route_id: &RuntimeRouteId, request: serde_json::Value) -> serde_json::Value {
    dispatch_request_json_for_route(route_id, &request.to_string())
        .expect("dispatcher should return JSON")
}

#[test]
fn test_request_dispatcher_updates_route_local_control_state_and_snapshot() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(41, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("control".to_string()),
        },
    );

    let paused = dispatch(&route, json!({ "verb": "set_paused", "paused": true }));
    assert_eq!(paused["ok"], true);

    let scaled = dispatch(
        &route,
        json!({ "verb": "set_time_scale", "time_scale": 0.25 }),
    );
    assert_eq!(scaled["ok"], true);

    let draw = dispatch(
        &route,
        json!({ "verb": "set_debug_draw_enabled", "enabled": true }),
    );
    assert_eq!(draw["ok"], true);

    let selected = dispatch(
        &route,
        json!({ "verb": "set_selected_entity", "entity_id": 77_u64 }),
    );
    assert_eq!(selected["ok"], true);

    let control = control_state_for_route(&route).expect("control state should exist");
    assert_eq!(
        control,
        RouteControlStateV1 {
            paused: true,
            time_scale: 0.25,
            frame_step_budget: 0,
            tick_step_budget: 0,
            debug_draw_enabled: true,
            queued_input_events: 0,
        }
    );

    let snapshot = snapshot_for_route(&route).expect("snapshot should exist");
    assert!(snapshot.debugger.paused);
    assert_eq!(snapshot.debugger.time_scale, 0.25);
    assert_eq!(snapshot.selection.entity_id, Some(77));
}

#[test]
fn test_step_budget_is_consumed_by_runtime_owned_frame_control() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(42, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("step-budget".to_string()),
        },
    );

    let _ = dispatch(&route, json!({ "verb": "set_paused", "paused": true }));
    let _ = dispatch(&route, json!({ "verb": "step", "frames": 2_u32 }));

    let first = take_frame_control_for_route(&route, 0.016).expect("first frame plan");
    let second = take_frame_control_for_route(&route, 0.016).expect("second frame plan");
    let third = take_frame_control_for_route(&route, 0.016).expect("third frame plan");

    assert!(first.advance_frame);
    assert!(second.advance_frame);
    assert!(!third.advance_frame);
    assert_eq!(third.effective_delta_seconds, 0.0);
}

#[test]
fn test_inject_input_queues_synthetic_events_until_runtime_consumes_them() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(43, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("input".to_string()),
        },
    );

    let response = dispatch(
        &route,
        json!({
            "verb": "inject_input",
            "events": [
                {
                    "device": "keyboard",
                    "action": "press",
                    "key": "space"
                }
            ]
        }),
    );
    assert_eq!(response["ok"], true);

    let queued = control_state_for_route(&route).expect("control state should exist");
    assert_eq!(queued.queued_input_events, 1);

    let frame = take_frame_control_for_route(&route, 0.016).expect("frame plan should exist");
    assert_eq!(frame.synthetic_inputs.len(), 1);

    let drained = control_state_for_route(&route).expect("control state should exist");
    assert_eq!(drained.queued_input_events, 0);
}

#[test]
fn test_unsupported_commands_are_capability_gated_and_service_health_driven() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(44, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("gated".to_string()),
        },
    );

    let capture = dispatch(&route, json!({ "verb": "capture_frame" }));
    assert_eq!(capture["ok"], false);
    assert_eq!(capture["error"]["code"], "unsupported");
    assert_eq!(capture["error"]["capability"], "capture");

    let replay = dispatch(&route, json!({ "verb": "start_replay" }));
    assert_eq!(replay["ok"], false);
    assert_eq!(replay["error"]["code"], "unsupported");
    assert_eq!(replay["error"]["service"], "replay");

    let snapshot = dispatch(&route, json!({ "verb": "get_snapshot" }));
    assert_eq!(snapshot["ok"], true);
    assert_eq!(
        snapshot["result"]["route_id"]["context_id"],
        route.context_id
    );
}
