use crate::core::debugger::{self, RuntimeRouteId};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::input::{
    GoudKeyCode, GoudMouseButton, KEY_A, KEY_D, KEY_DOWN, KEY_ENTER, KEY_ESCAPE, KEY_LEFT,
    KEY_RIGHT, KEY_S, KEY_SPACE, KEY_TAB, KEY_UP, KEY_W, MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE,
    MOUSE_BUTTON_RIGHT,
};
use crate::ffi::types::FfiVec2;
use serde_json::{json, Value};

use super::debugger_runtime::write_string_result;

/// FFI-safe step mode for debugger-controlled execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoudDebuggerStepKind {
    /// Advances execution by rendered frames.
    Frame = 0,
    /// Advances execution by simulation ticks.
    Tick = 1,
}

fn route_for_context(context_id: GoudContextId) -> Result<RuntimeRouteId, i32> {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return Err(-1);
    }

    debugger::route_for_context(context_id).ok_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        -1
    })
}

fn response_error(response: &Value) -> GoudError {
    let code = response
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(Value::as_str)
        .unwrap_or("internal_error");
    let message = response
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("debugger request failed")
        .to_string();

    match code {
        "route_not_found" => GoudError::InvalidContext,
        "protocol_error" | "unsupported" | "capture_failed" => GoudError::InvalidState(message),
        _ => GoudError::InternalError(message),
    }
}

fn dispatch_result(context_id: GoudContextId, request: Value) -> Result<Value, i32> {
    let route_id = route_for_context(context_id)?;
    let request_json = serde_json::to_string(&request).map_err(|err| {
        set_last_error(GoudError::InternalError(format!(
            "failed to serialize debugger request: {err}"
        )));
        -1
    })?;
    let response =
        debugger::dispatch_request_json_for_route(&route_id, &request_json).map_err(|err| {
            set_last_error(GoudError::InternalError(format!(
                "failed to dispatch debugger request: {err}"
            )));
            -1
        })?;

    if response.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    } else {
        set_last_error(response_error(&response));
        Err(-1)
    }
}

fn dispatch_status(context_id: GoudContextId, request: Value) -> i32 {
    match dispatch_result(context_id, request) {
        Ok(_) => 0,
        Err(code) => code,
    }
}

fn write_json_result(
    context_id: GoudContextId,
    request: Value,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    let result = match dispatch_result(context_id, request) {
        Ok(result) => result,
        Err(code) => return code,
    };
    let result_json = match serde_json::to_string(&result) {
        Ok(json) => json,
        Err(err) => {
            set_last_error(GoudError::InternalError(format!(
                "failed to encode debugger response: {err}"
            )));
            return -1;
        }
    };

    // SAFETY: The caller provides a writable buffer or uses the null/zero size query path.
    unsafe { write_string_result(&result_json, buf, buf_len) }
}

fn key_name(key: GoudKeyCode) -> Option<&'static str> {
    match key {
        KEY_SPACE => Some("space"),
        KEY_ENTER => Some("enter"),
        KEY_ESCAPE => Some("escape"),
        KEY_TAB => Some("tab"),
        KEY_LEFT => Some("left"),
        KEY_RIGHT => Some("right"),
        KEY_UP => Some("up"),
        KEY_DOWN => Some("down"),
        KEY_A => Some("a"),
        KEY_D => Some("d"),
        KEY_S => Some("s"),
        KEY_W => Some("w"),
        _ => None,
    }
}

fn mouse_button_name(button: GoudMouseButton) -> Option<&'static str> {
    match button {
        MOUSE_BUTTON_LEFT => Some("left"),
        MOUSE_BUTTON_RIGHT => Some("right"),
        MOUSE_BUTTON_MIDDLE => Some("middle"),
        _ => None,
    }
}

fn invalid_state(message: impl Into<String>) -> i32 {
    set_last_error(GoudError::InvalidState(message.into()));
    -2
}

fn missing_result_field(field: &str) -> i32 {
    set_last_error(GoudError::InternalError(format!(
        "debugger response missing required field '{field}'"
    )));
    -1
}

fn required_result_field<'a>(result: &'a Value, field: &str) -> Result<&'a Value, i32> {
    result.get(field).ok_or_else(|| missing_result_field(field))
}

fn required_result_string<'a>(result: &'a Value, field: &str) -> Result<&'a str, i32> {
    required_result_field(result, field)?
        .as_str()
        .ok_or_else(|| missing_result_field(field))
}

fn write_transformed_json_result(
    context_id: GoudContextId,
    request: Value,
    buf: *mut u8,
    buf_len: usize,
    transform: impl FnOnce(&Value) -> Result<Value, i32>,
) -> i32 {
    let result = match dispatch_result(context_id, request) {
        Ok(result) => result,
        Err(code) => return code,
    };
    let transformed = match transform(&result) {
        Ok(transformed) => transformed,
        Err(code) => return code,
    };
    let result_json = match serde_json::to_string(&transformed) {
        Ok(json) => json,
        Err(err) => {
            set_last_error(GoudError::InternalError(format!(
                "failed to encode debugger response: {err}"
            )));
            return -1;
        }
    };

    // SAFETY: The caller provides a writable buffer or uses the null/zero size query path.
    unsafe { write_string_result(&result_json, buf, buf_len) }
}

#[no_mangle]
/// Sets the debugger paused state for one route-scoped context.
pub extern "C" fn goud_debugger_set_paused(context_id: GoudContextId, paused: bool) -> i32 {
    dispatch_status(
        context_id,
        json!({ "verb": "set_paused", "paused": paused }),
    )
}

#[no_mangle]
/// Spends one debugger step budget in frame or tick units for the selected route.
pub extern "C" fn goud_debugger_step(
    context_id: GoudContextId,
    kind: GoudDebuggerStepKind,
    count: u32,
) -> i32 {
    let request = match kind {
        GoudDebuggerStepKind::Frame => json!({ "verb": "step", "frames": count }),
        GoudDebuggerStepKind::Tick => json!({ "verb": "step", "frames": 0, "ticks": count }),
    };
    dispatch_status(context_id, request)
}

#[no_mangle]
/// Updates the debugger-owned time-scale multiplier for one route.
pub extern "C" fn goud_debugger_set_time_scale(context_id: GoudContextId, scale: f32) -> i32 {
    dispatch_status(
        context_id,
        json!({ "verb": "set_time_scale", "time_scale": scale }),
    )
}

#[no_mangle]
/// Enables or disables debugger-owned debug draw primitives for one route.
pub extern "C" fn goud_debugger_set_debug_draw_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    dispatch_status(
        context_id,
        json!({ "verb": "set_debug_draw_enabled", "enabled": enabled }),
    )
}

#[no_mangle]
/// Injects one normalized keyboard event into the debugger synthetic input queue.
pub extern "C" fn goud_debugger_inject_key_event(
    context_id: GoudContextId,
    key: GoudKeyCode,
    pressed: bool,
) -> i32 {
    let Some(key) = key_name(key) else {
        return invalid_state(format!("unsupported debugger key code: {key}"));
    };

    dispatch_status(
        context_id,
        json!({
            "verb": "inject_input",
            "events": [{
                "device": "keyboard",
                "action": if pressed { "press" } else { "release" },
                "key": key,
            }],
        }),
    )
}

#[no_mangle]
/// Injects one normalized mouse button event into the debugger synthetic input queue.
pub extern "C" fn goud_debugger_inject_mouse_button(
    context_id: GoudContextId,
    button: GoudMouseButton,
    pressed: bool,
) -> i32 {
    let Some(button) = mouse_button_name(button) else {
        return invalid_state(format!("unsupported debugger mouse button: {button}"));
    };

    dispatch_status(
        context_id,
        json!({
            "verb": "inject_input",
            "events": [{
                "device": "mouse",
                "action": if pressed { "press" } else { "release" },
                "button": button,
            }],
        }),
    )
}

#[no_mangle]
/// Injects one normalized mouse move event into the debugger synthetic input queue.
pub extern "C" fn goud_debugger_inject_mouse_position(
    context_id: GoudContextId,
    position: FfiVec2,
) -> i32 {
    dispatch_status(
        context_id,
        json!({
            "verb": "inject_input",
            "events": [{
                "device": "mouse",
                "action": "move",
                "position": [position.x, position.y],
            }],
        }),
    )
}

#[no_mangle]
/// Injects one normalized scroll event into the debugger synthetic input queue.
pub extern "C" fn goud_debugger_inject_scroll(context_id: GoudContextId, delta: FfiVec2) -> i32 {
    dispatch_status(
        context_id,
        json!({
            "verb": "inject_input",
            "events": [{
                "device": "mouse",
                "action": "scroll",
                "delta": [delta.x, delta.y],
            }],
        }),
    )
}

#[no_mangle]
/// Writes the current metrics trace JSON into a caller-owned buffer.
///
/// # Safety
///
/// `buf` must be valid for `buf_len` writable bytes, or null with zero length to query size.
pub unsafe extern "C" fn goud_debugger_get_metrics_trace_json(
    context_id: GoudContextId,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    write_json_result(
        context_id,
        json!({ "verb": "get_metrics_trace" }),
        buf,
        buf_len,
    )
}

#[no_mangle]
/// Writes the current replay status JSON into a caller-owned buffer.
///
/// # Safety
///
/// `buf` must be valid for `buf_len` writable bytes, or null with zero length to query size.
pub unsafe extern "C" fn goud_debugger_get_replay_status_json(
    context_id: GoudContextId,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    write_json_result(
        context_id,
        json!({ "verb": "get_replay_status" }),
        buf,
        buf_len,
    )
}

#[no_mangle]
/// Captures the current debugger frame artifact as a JSON envelope.
///
/// # Safety
///
/// `buf` must be valid for `buf_len` writable bytes, or null with zero length to query size.
pub unsafe extern "C" fn goud_debugger_capture_frame_json(
    context_id: GoudContextId,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    write_transformed_json_result(
        context_id,
        json!({ "verb": "capture_frame" }),
        buf,
        buf_len,
        |result| {
            Ok(json!({
                "imagePng": required_result_field(result, "image_png")?.clone(),
                "metadataJson": required_result_string(result, "metadata_json")?,
                "snapshotJson": required_result_string(result, "snapshot_json")?,
                "metricsTraceJson": required_result_string(result, "metrics_trace_json")?,
            }))
        },
    )
}

#[no_mangle]
/// Starts debugger-owned input recording for one route.
pub extern "C" fn goud_debugger_start_recording(context_id: GoudContextId) -> i32 {
    dispatch_status(context_id, json!({ "verb": "start_recording" }))
}

#[no_mangle]
/// Stops debugger-owned input recording and exports the replay artifact as JSON.
///
/// # Safety
///
/// `buf` must be valid for `buf_len` writable bytes, or null with zero length to query size.
pub unsafe extern "C" fn goud_debugger_stop_recording_json(
    context_id: GoudContextId,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    write_transformed_json_result(
        context_id,
        json!({ "verb": "stop_recording" }),
        buf,
        buf_len,
        |result| {
            Ok(json!({
                "manifestJson": required_result_string(result, "manifest_json")?,
                "data": required_result_field(result, "data")?.clone(),
            }))
        },
    )
}

#[no_mangle]
/// Starts replay mode with caller-provided recording bytes.
///
/// # Safety
///
/// `data_ptr` must point to `data_len` readable bytes when `data_len` is non-zero.
pub unsafe extern "C" fn goud_debugger_start_replay(
    context_id: GoudContextId,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    if data_ptr.is_null() && data_len > 0 {
        return invalid_state("data_ptr is null");
    }

    let data = if data_len == 0 {
        Vec::new()
    } else {
        // SAFETY: `data_ptr` is non-null when `data_len` is non-zero and points to readable bytes.
        unsafe { std::slice::from_raw_parts(data_ptr, data_len) }.to_vec()
    };

    dispatch_status(context_id, json!({ "verb": "start_replay", "data": data }))
}

#[no_mangle]
/// Stops any active debugger replay session for one route.
pub extern "C" fn goud_debugger_stop_replay(context_id: GoudContextId) -> i32 {
    dispatch_status(context_id, json!({ "verb": "stop_replay" }))
}
