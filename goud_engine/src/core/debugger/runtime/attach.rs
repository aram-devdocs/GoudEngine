use interprocess::local_socket::{GenericFilePath, NameType};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::{json, Value};

#[cfg(test)]
use super::state::{lock_runtime, AttachSessionState};
use super::state::{sync_debugger_state, DebuggerRuntimeState};
use crate::core::debugger::RuntimeRouteId;

#[cfg(test)]
const SNAPSHOT_SCHEMA: &str = "debugger_snapshot_v1";
#[cfg(test)]
const HEARTBEAT_INTERVAL_MS: u32 = 1_000;

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

#[cfg(test)]
fn protocol_error_response(code: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message,
        }
    })
}

pub(super) fn ensure_local_attach_server(_runtime: &mut DebuggerRuntimeState) {
    let _ = GenericFilePath::is_supported();
}

pub(super) fn detach_sessions_for_route(runtime: &mut DebuggerRuntimeState, route_key: u64) {
    let session_ids = runtime.session_ids_for_route(route_key);
    if let Some(route) = runtime.routes.get_mut(&route_key) {
        route.attached_clients = route
            .attached_clients
            .saturating_sub(session_ids.len() as u32);
        sync_debugger_state(route);
    }
    for session_id in session_ids {
        runtime.sessions.remove(&session_id);
    }
}

#[cfg(test)]
pub fn attach_hello_for_tests(hello: AttachHelloV1) -> Result<AttachAcceptedV1, String> {
    if hello.protocol_version != 1 {
        return Err("version_mismatch".to_string());
    }

    let mut guard = lock_runtime();
    let runtime = guard
        .as_mut()
        .ok_or_else(|| "attach_disabled".to_string())?;
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

#[cfg(test)]
pub fn attach_session_heartbeat_for_tests(session_id: u64) -> Option<Value> {
    let guard = lock_runtime();
    guard.as_ref()?.sessions.get(&session_id)?;
    Some(json!({ "type": "heartbeat_ack" }))
}

#[cfg(test)]
pub fn attach_request_json_for_tests(
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
            runtime.sessions.remove(&session_id);
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
