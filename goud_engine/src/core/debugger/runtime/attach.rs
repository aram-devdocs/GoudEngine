use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use interprocess::local_socket::{
    prelude::*, GenericFilePath, ListenerNonblockingMode, ListenerOptions, NameType, Stream,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::state::{
    lock_runtime, sync_debugger_state, AttachServerState, AttachSessionState, DebuggerRuntimeState,
};
use crate::core::debugger::RuntimeRouteId;

const SNAPSHOT_SCHEMA: &str = "debugger_snapshot_v1";
const HEARTBEAT_INTERVAL_MS: u32 = 1_000;
const MAX_FRAME_BYTES: usize = 1024 * 1024;
const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(15);

/// Initial attach request for one debugger route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachHelloV1 {
    /// Fixed protocol version for RFC-0004 local attach.
    pub protocol_version: u32,
    /// Human-readable debugger client name.
    pub client_name: String,
    /// Operating-system process id for the attaching client.
    pub client_pid: u32,
    /// Route selected for this session.
    pub route_id: RuntimeRouteId,
}

/// Successful attach response for one bound route session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachAcceptedV1 {
    /// Fixed protocol version for RFC-0004 local attach.
    pub protocol_version: u32,
    /// Runtime-owned session identifier.
    pub session_id: u64,
    /// Route selected for this session.
    pub route_id: RuntimeRouteId,
    /// Snapshot schema identifier served by this runtime.
    pub snapshot_schema: String,
    /// Expected heartbeat interval for this session.
    pub heartbeat_interval_ms: u32,
}

fn protocol_error_response(code: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message,
        }
    })
}

fn attach_hello(
    runtime: &mut DebuggerRuntimeState,
    hello: AttachHelloV1,
) -> Result<AttachAcceptedV1, String> {
    if hello.protocol_version != 1 {
        return Err("version_mismatch".to_string());
    }

    let route_key = hello.route_id.context_id;
    let attachable = runtime
        .routes
        .get(&route_key)
        .map(|route| route.attachable)
        .ok_or_else(|| "route_not_found".to_string())?;
    if !attachable {
        return Err("route_not_attachable".to_string());
    }

    let session_id = runtime.next_session_id();
    let route = runtime
        .routes
        .get_mut(&route_key)
        .ok_or_else(|| "route_not_found".to_string())?;
    route.attached_clients = route.attached_clients.saturating_add(1);
    sync_debugger_state(route);
    runtime.sessions.insert(
        session_id,
        AttachSessionState {
            route_key,
            in_flight: false,
        },
    );

    Ok(AttachAcceptedV1 {
        protocol_version: 1,
        session_id,
        route_id: hello.route_id,
        snapshot_schema: SNAPSHOT_SCHEMA.to_string(),
        heartbeat_interval_ms: HEARTBEAT_INTERVAL_MS,
    })
}

fn detach_session(runtime: &mut DebuggerRuntimeState, session_id: u64) {
    let Some(session) = runtime.sessions.remove(&session_id) else {
        return;
    };
    if let Some(route) = runtime.routes.get_mut(&session.route_key) {
        route.attached_clients = route.attached_clients.saturating_sub(1);
        sync_debugger_state(route);
    }
}

fn handle_attach_hello(hello: AttachHelloV1) -> Result<AttachAcceptedV1, String> {
    let mut guard = lock_runtime();
    let runtime = guard
        .as_mut()
        .ok_or_else(|| "attach_disabled".to_string())?;
    attach_hello(runtime, hello)
}

fn session_heartbeat_ack(session_id: u64) -> Option<Value> {
    let guard = lock_runtime();
    guard.as_ref()?.sessions.get(&session_id)?;
    Some(json!({ "type": "heartbeat_ack" }))
}

fn dispatch_request_json_for_session(
    session_id: u64,
    request_json: &str,
) -> Result<Value, serde_json::Error> {
    let request_value: Value = serde_json::from_str(request_json)?;
    let route_key = {
        let mut guard = lock_runtime();
        let runtime = guard
            .as_mut()
            .ok_or_else(|| serde_json::Error::io(std::io::Error::other("runtime missing")))?;
        let Some(session) = runtime.sessions.get_mut(&session_id) else {
            return Ok(protocol_error_response(
                "attach_disabled",
                "attach session is no longer available",
            ));
        };
        if session.in_flight
            || request_value.get("pipelined").and_then(Value::as_bool) == Some(true)
        {
            detach_session(runtime, session_id);
            return Ok(protocol_error_response(
                "protocol_error",
                "only one in-flight request is allowed per attach session",
            ));
        }
        session.in_flight = true;
        session.route_key
    };

    let route_id = {
        let guard = lock_runtime();
        let runtime = guard
            .as_ref()
            .ok_or_else(|| serde_json::Error::io(std::io::Error::other("runtime missing")))?;
        runtime
            .routes
            .get(&route_key)
            .map(|route| route.snapshot.route_id.clone())
            .ok_or_else(|| serde_json::Error::io(std::io::Error::other("route missing")))?
    };

    let response = super::control::dispatch_request_json_for_route(&route_id, request_json)?;

    let mut guard = lock_runtime();
    if let Some(runtime) = guard.as_mut() {
        if let Some(session) = runtime.sessions.get_mut(&session_id) {
            session.in_flight = false;
        }
    }

    Ok(response)
}

fn read_frame(stream: &mut Stream) -> io::Result<Option<Vec<u8>>> {
    let mut len_buf = [0_u8; 4];
    match stream.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err),
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds maximum size",
        ));
    }
    let mut payload = vec![0_u8; len];
    stream.read_exact(&mut payload)?;
    Ok(Some(payload))
}

fn write_frame(stream: &mut Stream, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_vec(value).map_err(io::Error::other)?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds maximum size",
        ));
    }
    let len = u32::try_from(payload.len()).map_err(io::Error::other)?;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&payload)?;
    Ok(())
}

fn handle_client_connection(mut stream: Stream) -> io::Result<()> {
    let hello_frame = match read_frame(&mut stream)? {
        Some(frame) => frame,
        None => return Ok(()),
    };
    let hello: AttachHelloV1 = match serde_json::from_slice(&hello_frame) {
        Ok(hello) => hello,
        Err(_) => {
            let _ = write_frame(
                &mut stream,
                &protocol_error_response("protocol_error", "invalid attach hello"),
            );
            return Ok(());
        }
    };

    let accepted = match handle_attach_hello(hello) {
        Ok(accepted) => accepted,
        Err(code) => {
            let _ = write_frame(
                &mut stream,
                &protocol_error_response(&code, "attach hello rejected"),
            );
            return Ok(());
        }
    };
    let session_id = accepted.session_id;
    write_frame(
        &mut stream,
        &serde_json::to_value(accepted).map_err(io::Error::other)?,
    )?;

    loop {
        let Some(frame) = read_frame(&mut stream)? else {
            break;
        };
        let value: Value = match serde_json::from_slice(&frame) {
            Ok(value) => value,
            Err(_) => {
                let _ = write_frame(
                    &mut stream,
                    &protocol_error_response("protocol_error", "invalid request frame"),
                );
                break;
            }
        };

        if value.get("type").and_then(Value::as_str) == Some("heartbeat") {
            if let Some(ack) = session_heartbeat_ack(session_id) {
                let _ = write_frame(&mut stream, &ack);
            } else {
                let _ = write_frame(
                    &mut stream,
                    &protocol_error_response(
                        "attach_disabled",
                        "attach session is no longer available",
                    ),
                );
                break;
            }
            continue;
        }

        let request_json = String::from_utf8(frame).map_err(io::Error::other)?;
        let response = match dispatch_request_json_for_session(session_id, &request_json) {
            Ok(response) => response,
            Err(_) => protocol_error_response("protocol_error", "invalid request payload"),
        };

        let should_close = response
            .get("error")
            .and_then(|error| error.get("code"))
            .and_then(Value::as_str)
            == Some("protocol_error");
        let _ = write_frame(&mut stream, &response);
        if should_close {
            break;
        }
    }

    let mut guard = lock_runtime();
    if let Some(runtime) = guard.as_mut() {
        detach_session(runtime, session_id);
    }
    Ok(())
}

fn start_server(endpoint_location: &str) -> io::Result<AttachServerState> {
    let name = Path::new(endpoint_location).to_fs_name::<GenericFilePath>()?;
    let listener = ListenerOptions::new()
        .name(name)
        .nonblocking(ListenerNonblockingMode::Accept)
        .create_sync()?;

    let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_shutdown = shutdown.clone();
    let accept_thread = thread::spawn(move || {
        while !thread_shutdown.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok(stream) => {
                    thread::spawn(move || {
                        let _ = stream.set_nonblocking(false);
                        let _ = handle_client_connection(stream);
                    });
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(ACCEPT_POLL_INTERVAL);
                }
                Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    });

    Ok(AttachServerState {
        endpoint_location: endpoint_location.to_string(),
        shutdown,
        accept_thread: Some(accept_thread),
    })
}

pub(super) fn ensure_local_attach_server(runtime: &mut DebuggerRuntimeState) {
    if !GenericFilePath::is_supported() {
        stop_local_attach_server(runtime);
        return;
    }

    let endpoint_location = runtime
        .artifacts
        .as_ref()
        .map(|artifacts| artifacts.endpoint.location.clone());
    let Some(endpoint_location) = endpoint_location else {
        stop_local_attach_server(runtime);
        return;
    };

    if let Some(server) = runtime.attach_server.as_ref() {
        if server.endpoint_location == endpoint_location {
            return;
        }
    }

    stop_local_attach_server(runtime);
    match start_server(&endpoint_location) {
        Ok(server) => {
            runtime.attach_server = Some(server);
        }
        Err(_err) => {}
    }
}

pub(super) fn stop_local_attach_server(runtime: &mut DebuggerRuntimeState) {
    let Some(mut server) = runtime.attach_server.take() else {
        return;
    };
    server.shutdown.store(true, Ordering::Relaxed);
    if let Some(join) = server.accept_thread.take() {
        let _ = join.join();
    }
}

pub(super) fn detach_sessions_for_route(runtime: &mut DebuggerRuntimeState, route_key: u64) {
    let session_ids = runtime.session_ids_for_route(route_key);
    for session_id in session_ids {
        detach_session(runtime, session_id);
    }
}

#[cfg(test)]
pub fn attach_hello_for_tests(hello: AttachHelloV1) -> Result<AttachAcceptedV1, String> {
    handle_attach_hello(hello)
}

#[cfg(test)]
pub fn attach_session_heartbeat_for_tests(session_id: u64) -> Option<Value> {
    session_heartbeat_ack(session_id)
}

#[cfg(test)]
pub fn attach_request_json_for_tests(
    session_id: u64,
    request_json: &str,
) -> Result<Value, serde_json::Error> {
    dispatch_request_json_for_session(session_id, request_json)
}
