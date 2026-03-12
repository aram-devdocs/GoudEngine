use serde_json::{json, Value};

use super::super::snapshot::DebuggerSnapshotV1;
use super::super::types::CapabilityStateV1;
use super::state::{
    lock_runtime, set_route_capability, set_service_state, sync_debugger_state, FrameControlPlanV1,
    RouteControlStateV1, SyntheticInputEventV1,
};
use crate::core::debugger::RuntimeRouteId;

fn parse_input_event(value: &Value) -> Option<SyntheticInputEventV1> {
    Some(SyntheticInputEventV1 {
        device: value.get("device")?.as_str()?.to_string(),
        action: value.get("action")?.as_str()?.to_string(),
        key: value.get("key").and_then(Value::as_str).map(str::to_string),
        button: value
            .get("button")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
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
    let plan = route.control.take_frame_plan(raw_delta_seconds);
    sync_debugger_state(route);
    Some(plan)
}

/// Dispatches one debugger JSON request against a bound route.
pub fn dispatch_request_json_for_route(
    route_id: &RuntimeRouteId,
    request_json: &str,
) -> Result<Value, serde_json::Error> {
    let request: Value = serde_json::from_str(request_json)?;
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

        let verb = request
            .get("verb")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        let response = match verb.as_str() {
            "get_snapshot" => ok_response(&request, snapshot_value(&route.snapshot)),
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
            "get_metrics_trace" => ok_response(&request, json!(route.snapshot.profiler_samples)),
            "get_replay_status" => {
                let service = route
                    .snapshot
                    .services
                    .iter()
                    .find(|service| service.name == "replay");
                ok_response(
                    &request,
                    json!({
                        "state": service.map(|service| service.state),
                        "detail": service.and_then(|service| service.detail.clone()),
                    }),
                )
            }
            "capture_frame" => error_response(
                &request,
                "unsupported",
                "capture is not available for this route".to_string(),
                Some("capture"),
                Some("capture"),
            ),
            "start_recording" | "stop_recording" | "start_replay" | "stop_replay" => {
                error_response(
                    &request,
                    "unsupported",
                    "replay controls are not available for this route".to_string(),
                    Some("replay"),
                    Some("replay"),
                )
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
