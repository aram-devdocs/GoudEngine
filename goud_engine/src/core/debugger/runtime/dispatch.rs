use serde_json::{json, Value};

use super::super::snapshot::DebuggerSnapshotV1;
use super::super::types::CapabilityStateV1;
use super::capture::{capture_frame_for_route, CaptureFrameError};
use super::metrics::{empty_metrics_export, metrics_trace_json_for_route};
use super::state::lock_runtime;
use crate::core::debugger::RuntimeRouteId;

pub(super) fn parse_vec2(value: Option<&Value>) -> Option<[f32; 2]> {
    let values = value?.as_array()?;
    if values.len() != 2 {
        return None;
    }
    Some([values[0].as_f64()? as f32, values[1].as_f64()? as f32])
}

pub(super) fn parse_input_event(value: &Value) -> Option<super::state::SyntheticInputEventV1> {
    Some(super::state::SyntheticInputEventV1 {
        device: value.get("device")?.as_str()?.to_string(),
        action: value.get("action")?.as_str()?.to_string(),
        key: value.get("key").and_then(Value::as_str).map(str::to_string),
        button: value
            .get("button")
            .and_then(Value::as_str)
            .map(str::to_string),
        position: parse_vec2(value.get("position")),
        delta: parse_vec2(value.get("delta")),
    })
}

pub(super) fn parse_bytes(value: &Value) -> Option<Vec<u8>> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| item.as_u64().and_then(|v| u8::try_from(v).ok()))
            .collect(),
        Value::String(text) => Some(text.as_bytes().to_vec()),
        _ => None,
    }
}

pub(super) fn ok_response(request: &Value, result: Value) -> Value {
    let mut response = json!({ "ok": true, "result": result });
    if let Some(request_id) = request.get("request_id") {
        response["request_id"] = request_id.clone();
    }
    response
}

pub(super) fn error_response(
    request: &Value,
    code: &str,
    message: String,
    capability: Option<&str>,
    service: Option<&str>,
) -> Value {
    let mut response = json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message,
        }
    });
    if let Some(capability) = capability {
        response["error"]["capability"] = json!(capability);
    }
    if let Some(service) = service {
        response["error"]["service"] = json!(service);
    }
    if let Some(request_id) = request.get("request_id") {
        response["request_id"] = request_id.clone();
    }
    response
}

pub(super) fn snapshot_value(snapshot: &DebuggerSnapshotV1) -> Value {
    serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}))
}

pub(super) fn metrics_trace_response_for_route(
    route_id: &RuntimeRouteId,
    request: &Value,
) -> Value {
    if let Some(metrics_json) = metrics_trace_json_for_route(route_id) {
        let artifact_id = super::artifacts::store_metrics_trace_for_route(route_id, &metrics_json);
        if let Ok(Value::Object(mut parsed)) = serde_json::from_str::<Value>(&metrics_json) {
            parsed.insert("artifact_id".to_string(), json!(artifact_id));
            return ok_response(request, Value::Object(parsed));
        }
    }

    let has_route = {
        let guard = lock_runtime();
        guard
            .as_ref()
            .and_then(|runtime| runtime.routes.get(&route_id.context_id))
            .is_some()
    };

    if has_route {
        let empty_export = empty_metrics_export(route_id);
        let metrics_json =
            serde_json::to_string(&empty_export).unwrap_or_else(|_| "{}".to_string());
        let artifact_id = super::artifacts::store_metrics_trace_for_route(route_id, &metrics_json);
        let mut result = serde_json::to_value(empty_export).unwrap_or_else(|_| json!({}));
        if let Value::Object(ref mut object) = result {
            object.insert("artifact_id".to_string(), json!(artifact_id));
        }
        return ok_response(request, result);
    }

    error_response(
        request,
        "route_not_found",
        "debugger route is no longer available".to_string(),
        None,
        None,
    )
}

pub(super) fn capture_frame_response_for_route(
    route_id: &RuntimeRouteId,
    request: &Value,
) -> Value {
    let route_capture_state = {
        let guard = lock_runtime();
        let Some(runtime) = guard.as_ref() else {
            return error_response(
                request,
                "route_not_found",
                "debugger route is no longer available".to_string(),
                None,
                None,
            );
        };
        let Some(route) = runtime.routes.get(&route_id.context_id) else {
            return error_response(
                request,
                "route_not_found",
                "debugger route is no longer available".to_string(),
                None,
                None,
            );
        };
        route.capabilities.get("capture").copied()
    };

    if route_capture_state != Some(CapabilityStateV1::Ready) {
        return error_response(
            request,
            "unsupported",
            "capture is not available for this route".to_string(),
            Some("capture"),
            Some("capture"),
        );
    }

    match capture_frame_for_route(route_id) {
        Ok(artifact) => {
            let artifact_id =
                super::artifacts::store_capture_artifact_for_route(route_id, &artifact);
            let mut artifact_json = serde_json::to_value(artifact).unwrap_or_else(|_| json!({}));
            if let Value::Object(ref mut object) = artifact_json {
                object.insert("artifact_id".to_string(), json!(artifact_id));
            }
            ok_response(request, artifact_json)
        }
        Err(CaptureFrameError::RouteNotFound) => error_response(
            request,
            "route_not_found",
            "debugger route is no longer available".to_string(),
            None,
            None,
        ),
        Err(CaptureFrameError::Unsupported(message)) => error_response(
            request,
            "unsupported",
            message,
            Some("capture"),
            Some("capture"),
        ),
        Err(CaptureFrameError::CaptureFailed(message)) => error_response(
            request,
            "capture_failed",
            message,
            Some("capture"),
            Some("capture"),
        ),
    }
}
