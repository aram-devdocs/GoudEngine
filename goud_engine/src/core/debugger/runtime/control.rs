use serde_json::{json, Value};

use super::super::snapshot::DebuggerSnapshotV1;
use super::super::types::CapabilityStateV1;
use super::capture::{capture_frame_for_route, CaptureFrameError};
use super::metrics::{empty_metrics_export, metrics_trace_json_for_route};
use super::replay;
use super::state::{
    lock_runtime, set_route_capability, set_service_state, sync_debugger_state, FrameControlPlanV1,
    RouteControlStateV1, SyntheticInputEventV1,
};
use crate::core::debugger::RuntimeRouteId;

fn parse_vec2(value: Option<&Value>) -> Option<[f32; 2]> {
    let values = value?.as_array()?;
    if values.len() != 2 {
        return None;
    }
    Some([values[0].as_f64()? as f32, values[1].as_f64()? as f32])
}

fn parse_input_event(value: &Value) -> Option<SyntheticInputEventV1> {
    Some(SyntheticInputEventV1 {
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

fn parse_bytes(value: &Value) -> Option<Vec<u8>> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| item.as_u64().and_then(|v| u8::try_from(v).ok()))
            .collect(),
        Value::String(text) => Some(text.as_bytes().to_vec()),
        _ => None,
    }
}

fn ok_response(request: &Value, result: Value) -> Value {
    let mut response = json!({ "ok": true, "result": result });
    if let Some(request_id) = request.get("request_id") {
        response["request_id"] = request_id.clone();
    }
    response
}

fn error_response(
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

fn snapshot_value(snapshot: &DebuggerSnapshotV1) -> Value {
    serde_json::to_value(snapshot).unwrap_or_else(|_| json!({}))
}

fn metrics_trace_response_for_route(route_id: &RuntimeRouteId, request: &Value) -> Value {
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

fn capture_frame_response_for_route(route_id: &RuntimeRouteId, request: &Value) -> Value {
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

/// Returns the current route-local debugger control state.
pub fn control_state_for_route(route_id: &RuntimeRouteId) -> Option<RouteControlStateV1> {
    let guard = lock_runtime();
    let route = guard.as_ref()?.routes.get(&route_id.context_id)?;
    Some(route.control.snapshot())
}

/// Consumes one frame of runtime-owned control state for a route.
pub fn take_frame_control_for_route(
    route_id: &RuntimeRouteId,
    raw_delta_seconds: f32,
) -> Option<FrameControlPlanV1> {
    let mut guard = lock_runtime();
    let runtime = guard.as_mut()?;
    let route = runtime.routes.get_mut(&route_id.context_id)?;
    let mut plan = route.control.take_frame_plan(raw_delta_seconds);
    let target_frame_index = route.snapshot.frame.index.saturating_add(1);
    let replay_events = replay::take_replay_events_for_frame(route, target_frame_index);
    plan.synthetic_inputs.extend(replay_events);
    sync_debugger_state(route);
    Some(plan)
}

/// Dispatches one debugger JSON request against a bound route.
pub fn dispatch_request_json_for_route(
    route_id: &RuntimeRouteId,
    request_json: &str,
) -> Result<Value, serde_json::Error> {
    let request: Value = serde_json::from_str(request_json)?;
    let verb = request
        .get("verb")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if verb == "get_metrics_trace" {
        return Ok(metrics_trace_response_for_route(route_id, &request));
    }
    if verb == "capture_frame" {
        return Ok(capture_frame_response_for_route(route_id, &request));
    }
    if verb == "get_snapshot" {
        super::snapshot_refresh::refresh_snapshot_for_route(route_id);
        let guard = lock_runtime();
        let Some(runtime) = guard.as_ref() else {
            return Ok(error_response(
                &request,
                "route_not_found",
                "debugger route is no longer available".to_string(),
                None,
                None,
            ));
        };
        let Some(route) = runtime.routes.get(&route_id.context_id) else {
            return Ok(error_response(
                &request,
                "route_not_found",
                "debugger route is no longer available".to_string(),
                None,
                None,
            ));
        };
        return Ok(ok_response(&request, snapshot_value(&route.snapshot)));
    }
    let mut should_publish_manifest = false;

    let response = {
        let mut guard = lock_runtime();
        let Some(runtime) = guard.as_mut() else {
            return Ok(error_response(
                &request,
                "route_not_found",
                "debugger route is no longer available".to_string(),
                None,
                None,
            ));
        };
        let Some(route) = runtime.routes.get_mut(&route_id.context_id) else {
            return Ok(error_response(
                &request,
                "route_not_found",
                "debugger route is no longer available".to_string(),
                None,
                None,
            ));
        };

        let response = match verb.as_str() {
            "set_selected_entity" => {
                let entity_id = request.get("entity_id").and_then(Value::as_u64);
                if !route.snapshot.scene.active_scene.is_empty() {
                    route.snapshot.selection.scene_id = route.snapshot.scene.active_scene.clone();
                }
                route.snapshot.selection.entity_id = entity_id;
                ok_response(&request, snapshot_value(&route.snapshot))
            }
            "clear_selected_entity" => {
                route.snapshot.selection.entity_id = None;
                ok_response(&request, snapshot_value(&route.snapshot))
            }
            "set_profiling_enabled" => {
                let enabled = request
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                route.profiling_enabled = enabled;
                set_route_capability(
                    route,
                    "profiling",
                    if enabled {
                        CapabilityStateV1::Ready
                    } else {
                        CapabilityStateV1::Disabled
                    },
                );
                set_service_state(
                    &mut route.snapshot,
                    "profiling",
                    if enabled {
                        CapabilityStateV1::Ready
                    } else {
                        CapabilityStateV1::Disabled
                    },
                    Some(if enabled {
                        "profiling enabled".to_string()
                    } else {
                        "profiling disabled for this route".to_string()
                    }),
                );
                should_publish_manifest = true;
                ok_response(&request, snapshot_value(&route.snapshot))
            }
            "set_paused" => {
                route.control.paused = request
                    .get("paused")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                sync_debugger_state(route);
                ok_response(&request, json!(route.control.snapshot()))
            }
            "step" => {
                let frames = request.get("frames").and_then(Value::as_u64).unwrap_or(1);
                let ticks = request.get("ticks").and_then(Value::as_u64).unwrap_or(0);
                route.control.frame_step_budget =
                    route.control.frame_step_budget.saturating_add(frames);
                route.control.tick_step_budget =
                    route.control.tick_step_budget.saturating_add(ticks);
                sync_debugger_state(route);
                ok_response(&request, json!(route.control.snapshot()))
            }
            "set_time_scale" => {
                let time_scale = request
                    .get("time_scale")
                    .and_then(Value::as_f64)
                    .unwrap_or(1.0)
                    .clamp(0.0, 10.0) as f32;
                route.control.time_scale = time_scale;
                sync_debugger_state(route);
                ok_response(&request, json!(route.control.snapshot()))
            }
            "set_debug_draw_enabled" => {
                let enabled = request
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                route.control.debug_draw_enabled = enabled;
                set_route_capability(
                    route,
                    "debug_draw",
                    if enabled {
                        CapabilityStateV1::Ready
                    } else {
                        CapabilityStateV1::Disabled
                    },
                );
                should_publish_manifest = true;
                ok_response(&request, json!(route.control.snapshot()))
            }
            "inject_input" => {
                let events = request
                    .get("events")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for event in events.iter().filter_map(parse_input_event) {
                    route.control.synthetic_inputs.push_back(event);
                }
                ok_response(&request, json!(route.control.snapshot()))
            }
            "get_replay_status" => ok_response(&request, replay::replay_status_json(route)),
            "start_recording" => {
                replay::start_recording(route);
                ok_response(&request, replay::replay_status_json(route))
            }
            "stop_recording" => {
                let export = replay::stop_recording(route);
                let artifact_id = super::artifacts::store_recording_artifact_in_state(
                    runtime.artifacts.as_mut(),
                    &route.snapshot.route_id,
                    &export,
                );
                let mut result = json!(export);
                if let Value::Object(ref mut object) = result {
                    object.insert("artifact_id".to_string(), json!(artifact_id));
                }
                ok_response(&request, result)
            }
            "start_replay" => {
                let Some(data_value) = request.get("data") else {
                    return Ok(error_response(
                        &request,
                        "protocol_error",
                        "start_replay requires replay data bytes".to_string(),
                        None,
                        None,
                    ));
                };
                let Some(data) = parse_bytes(data_value) else {
                    return Ok(error_response(
                        &request,
                        "protocol_error",
                        "start_replay data must be a byte array or string".to_string(),
                        None,
                        None,
                    ));
                };

                match replay::start_replay(route, &data) {
                    Ok(()) => ok_response(&request, replay::replay_status_json(route)),
                    Err(message) => {
                        error_response(&request, "protocol_error", message, None, Some("replay"))
                    }
                }
            }
            "stop_replay" => {
                replay::stop_replay(route);
                ok_response(&request, replay::replay_status_json(route))
            }
            _ => error_response(
                &request,
                "protocol_error",
                format!("unsupported debugger verb: {verb}"),
                None,
                None,
            ),
        };

        if should_publish_manifest {
            runtime.touch_manifest();
        }
        response
    };

    if should_publish_manifest {
        let mut guard = lock_runtime();
        if let Some(runtime) = guard.as_mut() {
            super::artifacts::sync_manifest(runtime);
        }
    }

    Ok(response)
}
