use serde_json::{json, Value};

use super::super::types::CapabilityStateV1;
use super::dispatch::{
    capture_frame_response_for_route, error_response, metrics_trace_response_for_route,
    ok_response, parse_bytes, parse_input_event, snapshot_value,
};
use super::replay;
use super::state::{
    lock_runtime, set_route_capability, set_service_state, sync_debugger_state,
    with_route_state_mut, FrameControlPlanV1, RouteControlStateV1,
};
use crate::core::debugger::RuntimeRouteId;

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
    if verb == "get_diagnostics" {
        super::snapshot_refresh::refresh_snapshot_for_route(route_id);
        let guard = lock_runtime();
        let diag = guard
            .as_ref()
            .and_then(|rt| rt.routes.get(&route_id.context_id))
            .map(|route| route.snapshot.provider_diagnostics.clone())
            .unwrap_or_default();
        return Ok(ok_response(&request, json!({ "diagnostics": diag })));
    }
    if verb == "get_diagnostics_for" {
        let key = request.get("key").and_then(Value::as_str).unwrap_or("");
        super::snapshot_refresh::refresh_snapshot_for_route(route_id);
        let guard = lock_runtime();
        let val = guard
            .as_ref()
            .and_then(|rt| rt.routes.get(&route_id.context_id))
            .and_then(|route| route.snapshot.provider_diagnostics.get(key).cloned())
            .unwrap_or(Value::Null);
        return Ok(ok_response(
            &request,
            json!({ "key": key, "diagnostics": val }),
        ));
    }
    if verb == "get_logs" {
        // Log capture will be fully wired in a later phase.
        return Ok(ok_response(&request, json!({ "entries": [] })));
    }
    if verb == "get_scene_hierarchy" {
        super::snapshot_refresh::refresh_snapshot_for_route(route_id);
        let guard = lock_runtime();
        let entities = guard
            .as_ref()
            .and_then(|rt| rt.routes.get(&route_id.context_id))
            .map(|route| route.snapshot.entities.clone())
            .unwrap_or_default();
        return Ok(ok_response(&request, json!({ "entities": entities })));
    }
    if verb == "start_diagnostics_recording" {
        let duration_seconds = request
            .get("duration_seconds")
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32;
        let max_frames = request
            .get("max_frames")
            .and_then(Value::as_u64)
            .unwrap_or(3600) as usize;
        let result = with_route_state_mut(route_id, |route| {
            let recording_id =
                super::metrics::start_diagnostics_recording(route, duration_seconds, max_frames);
            let status = super::metrics::diagnostics_recording_status(route);
            json!({ "recording_id": recording_id, "status": status })
        });
        return Ok(ok_response(
            &request,
            result.unwrap_or_else(|| json!({ "error": "route not found" })),
        ));
    }
    if verb == "stop_diagnostics_recording" {
        let result = with_route_state_mut(route_id, |route| {
            let recording_id = super::metrics::stop_diagnostics_recording(route);
            let frame_count = route.diagnostics_recording.frames.len();
            json!({ "recording_id": recording_id, "frame_count": frame_count })
        });
        return Ok(ok_response(
            &request,
            result.unwrap_or_else(|| json!({ "error": "route not found" })),
        ));
    }
    if verb == "get_diagnostics_recording" {
        let slice_count = request
            .get("slice_count")
            .and_then(Value::as_u64)
            .unwrap_or(100)
            .clamp(1, 1000) as u32;
        let guard = lock_runtime();
        let export = guard
            .as_ref()
            .and_then(|rt| rt.routes.get(&route_id.context_id))
            .and_then(|route| {
                super::metrics::export_diagnostics_recording_sliced(route, slice_count)
            });
        return Ok(match export {
            Some(export) => ok_response(&request, serde_json::to_value(export).unwrap_or_default()),
            None => error_response(
                &request,
                "no_recording",
                "no diagnostics recording data available".to_string(),
                None,
                None,
            ),
        });
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
