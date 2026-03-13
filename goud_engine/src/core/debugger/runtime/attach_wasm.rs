use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::{json, Value};

use super::state::DebuggerRuntimeState;
use crate::core::debugger::RuntimeRouteId;

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

pub fn ensure_local_attach_server(_runtime: &mut DebuggerRuntimeState) {}

pub fn detach_sessions_for_route(_runtime: &mut DebuggerRuntimeState, _route_key: u64) {}

pub fn stop_local_attach_server(_runtime: &mut DebuggerRuntimeState) {}

#[cfg(test)]
pub fn attach_hello_for_tests(_hello: AttachHelloV1) -> Result<AttachAcceptedV1, String> {
    Err("attach transport is unavailable on wasm32".to_string())
}

#[cfg(test)]
pub fn attach_request_json_for_tests(_session_id: u64, _request_json: &str) -> Value {
    json!({
        "ok": false,
        "error": {
            "code": "attach_disabled",
            "message": "attach transport is unavailable on wasm32",
        }
    })
}

#[cfg(test)]
pub fn attach_session_heartbeat_for_tests(_session_id: u64) -> Option<Value> {
    None
}
