use super::super::{
    attach_hello_for_tests, attach_request_json_for_tests, attach_session_heartbeat_for_tests,
    register_context, reset_for_tests, snapshot_for_route, test_lock, AttachAcceptedV1,
    AttachHelloV1, DebuggerConfig, RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use serde_json::json;

#[test]
fn test_attach_handshake_binds_session_to_route_and_tracks_attached_clients() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(51, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("attach".to_string()),
        },
    );

    let accepted = attach_hello_for_tests(AttachHelloV1 {
        protocol_version: 1,
        client_name: "test-client".to_string(),
        client_pid: std::process::id(),
        route_id: route.clone(),
    })
    .expect("attach should succeed");

    assert_eq!(
        accepted,
        AttachAcceptedV1 {
            protocol_version: 1,
            session_id: accepted.session_id,
            route_id: route.clone(),
            snapshot_schema: "debugger_snapshot_v1".to_string(),
            heartbeat_interval_ms: accepted.heartbeat_interval_ms,
        }
    );

    let snapshot = snapshot_for_route(&route).expect("snapshot should exist");
    assert_eq!(snapshot.debugger.attached_clients, 1);
}

#[test]
fn test_attach_session_heartbeat_is_out_of_band() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(52, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("heartbeat".to_string()),
        },
    );

    let accepted = attach_hello_for_tests(AttachHelloV1 {
        protocol_version: 1,
        client_name: "heartbeat-client".to_string(),
        client_pid: std::process::id(),
        route_id: route,
    })
    .expect("attach should succeed");

    let ack = attach_session_heartbeat_for_tests(accepted.session_id)
        .expect("heartbeat should produce an ack");
    assert_eq!(ack, json!({ "type": "heartbeat_ack" }));
}

#[test]
fn test_attach_session_rejects_second_in_flight_request() {
    let _guard = test_lock();
    reset_for_tests();

    let route = register_context(
        GoudContextId::new(53, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("inflight".to_string()),
        },
    );

    let accepted = attach_hello_for_tests(AttachHelloV1 {
        protocol_version: 1,
        client_name: "pipeline-client".to_string(),
        client_pid: std::process::id(),
        route_id: route,
    })
    .expect("attach should succeed");

    let first = attach_request_json_for_tests(
        accepted.session_id,
        &json!({ "verb": "get_snapshot", "request_id": 1_u64 }).to_string(),
    )
    .expect("first request should be accepted");
    assert_eq!(first["ok"], true);

    let second = attach_request_json_for_tests(
        accepted.session_id,
        &json!({ "verb": "get_snapshot", "request_id": 2_u64, "pipelined": true }).to_string(),
    )
    .expect("second request should return a protocol error response");
    assert_eq!(second["ok"], false);
    assert_eq!(second["error"]["code"], "protocol_error");
}
