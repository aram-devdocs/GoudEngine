use super::super::{
    attach_hello_for_tests, attach_request_json_for_tests, attach_session_heartbeat_for_tests,
    current_manifest, register_context, reset_for_tests, snapshot_for_route,
    stop_attach_server_for_tests, test_lock, AttachAcceptedV1, AttachHelloV1, DebuggerConfig,
    RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use serde_json::json;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

fn write_frame(stream: &mut Stream, value: &serde_json::Value) {
    let payload = serde_json::to_vec(value).expect("frame JSON should serialize");
    let len = u32::try_from(payload.len()).expect("frame length should fit in u32");
    stream
        .write_all(&len.to_le_bytes())
        .expect("frame length should write");
    stream
        .write_all(&payload)
        .expect("frame payload should write");
}

fn read_frame(stream: &mut Stream) -> serde_json::Value {
    let mut len_buf = [0_u8; 4];
    stream
        .read_exact(&mut len_buf)
        .expect("frame length should read");
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0_u8; len];
    stream
        .read_exact(&mut payload)
        .expect("frame payload should read");
    serde_json::from_slice(&payload).expect("frame JSON should parse")
}

fn connect_with_retry(location: &str) -> std::io::Result<Stream> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut last_error = std::io::Error::other("connect retry exhausted");
    while Instant::now() < deadline {
        let socket_name = Path::new(location)
            .to_fs_name::<GenericFilePath>()
            .expect("socket name should map");
        match Stream::connect(socket_name) {
            Ok(stream) => return Ok(stream),
            Err(err) => {
                last_error = err;
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    Err(last_error)
}

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
    // Stop the IPC server to prevent background thread interference with session state.
    // This test exercises the direct function-call heartbeat path, not the IPC path.
    stop_attach_server_for_tests();

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

#[test]
fn test_attach_ipc_handshake_heartbeat_and_request_over_local_socket() {
    let _guard = test_lock();
    reset_for_tests();
    if !GenericFilePath::is_supported() {
        eprintln!("skipping ipc attach test: local socket transport is unsupported");
        return;
    }

    let runtime_dir = format!("/tmp/ge209-{}", std::process::id());
    let _ = fs::remove_dir_all(&runtime_dir);
    fs::create_dir_all(&runtime_dir).expect("runtime dir should be created");
    std::env::set_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR", &runtime_dir);

    let route = register_context(
        GoudContextId::new(54, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("ipc".to_string()),
        },
    );
    let expected_route = route.clone();

    let manifest = current_manifest().expect("manifest should be present");

    let mut stream = match connect_with_retry(&manifest.endpoint.location) {
        Ok(stream) => stream,
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::PermissionDenied | ErrorKind::NotFound | ErrorKind::AddrNotAvailable
            ) =>
        {
            std::env::remove_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR");
            eprintln!(
                "skipping ipc attach test in restricted environment: {err}; endpoint={}",
                manifest.endpoint.location
            );
            return;
        }
        Err(err) => panic!(
            "local attach socket should accept: {err}; endpoint={}",
            manifest.endpoint.location
        ),
    };

    write_frame(
        &mut stream,
        &json!({
            "protocol_version": 1_u32,
            "client_name": "ipc-test",
            "client_pid": std::process::id(),
            "route_id": route,
        }),
    );
    let accepted = read_frame(&mut stream);
    std::env::remove_var("GOUDENGINE_DEBUGGER_RUNTIME_DIR");
    assert_eq!(accepted["protocol_version"], 1);
    let accepted_route: super::super::RuntimeRouteId =
        serde_json::from_value(accepted["route_id"].clone()).expect("route id should deserialize");
    assert_eq!(accepted_route, expected_route);
    assert_eq!(accepted["snapshot_schema"], "debugger_snapshot_v1");
    assert_eq!(accepted["heartbeat_interval_ms"], 1_000_u32);

    let snapshot = snapshot_for_route(&accepted_route).expect("snapshot should exist");
    assert_eq!(snapshot.debugger.attached_clients, 1);

    write_frame(&mut stream, &json!({ "type": "heartbeat" }));
    let ack = read_frame(&mut stream);
    assert_eq!(ack, json!({ "type": "heartbeat_ack" }));

    write_frame(
        &mut stream,
        &json!({ "verb": "get_snapshot", "request_id": 11_u64 }),
    );
    let response = read_frame(&mut stream);
    assert_eq!(response["ok"], true);
    assert_eq!(response["request_id"], 11_u64);
}
