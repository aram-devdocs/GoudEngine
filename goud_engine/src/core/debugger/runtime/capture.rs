use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use serde::Serialize;
use serde_json::json;

use super::metrics::{empty_metrics_export, metrics_trace_json_for_route};
use super::state::lock_runtime;
use crate::core::debugger::RuntimeRouteId;

type CaptureHook = Arc<dyn Fn() -> Result<RawFramebufferReadbackV1, String> + Send + Sync>;

/// Shared state for deferred framebuffer capture coordination.
pub(crate) struct DeferredCaptureState {
    pub(crate) requested: bool,
    pub(crate) result: Option<Result<RawFramebufferReadbackV1, String>>,
}

pub(crate) type DeferredCapture = Arc<(Mutex<DeferredCaptureState>, Condvar)>;

/// Raw framebuffer readback payload for one route-local capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFramebufferReadbackV1 {
    /// Captured image width in pixels.
    pub width: u32,
    /// Captured image height in pixels.
    pub height: u32,
    /// Packed RGBA8 pixel bytes in row-major order.
    pub rgba8: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureArtifactV1 {
    pub image_png: Vec<u8>,
    pub metadata_json: String,
    pub snapshot_json: String,
    pub metrics_trace_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureFrameError {
    RouteNotFound,
    Unsupported(String),
    CaptureFailed(String),
}

static CAPTURE_HOOKS: OnceLock<Mutex<HashMap<u128, CaptureHook>>> = OnceLock::new();

fn hook_registry() -> &'static Mutex<HashMap<u128, CaptureHook>> {
    CAPTURE_HOOKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn packed_route_identity(route_id: &RuntimeRouteId) -> u128 {
    ((route_id.process_nonce as u128) << 64) | route_id.context_id as u128
}

fn hook_for_route(route_id: &RuntimeRouteId) -> Option<CaptureHook> {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&packed_route_identity(route_id))
        .cloned()
}

/// Registers or replaces a route-local capture hook.
pub fn register_capture_hook_for_route(
    route_id: RuntimeRouteId,
    hook: impl Fn() -> Result<RawFramebufferReadbackV1, String> + Send + Sync + 'static,
) {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(packed_route_identity(&route_id), Arc::new(hook));
}

/// Unregisters the route-local capture hook, if present.
pub fn unregister_capture_hook_for_route(route_id: &RuntimeRouteId) {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&packed_route_identity(route_id));
}

pub(crate) fn new_deferred_capture() -> DeferredCapture {
    Arc::new((
        Mutex::new(DeferredCaptureState {
            requested: false,
            result: None,
        }),
        Condvar::new(),
    ))
}

pub(crate) fn register_deferred_capture_hook_for_route(
    route_id: RuntimeRouteId,
    deferred: DeferredCapture,
) {
    register_capture_hook_for_route(route_id, move || wait_for_deferred_capture(&deferred));
}

fn wait_for_deferred_capture(
    deferred: &DeferredCapture,
) -> Result<RawFramebufferReadbackV1, String> {
    let (lock, cvar) = &**deferred;
    let mut guard = lock
        .lock()
        .map_err(|e| format!("capture lock poisoned: {e}"))?;
    guard.requested = true;
    guard.result = None;
    let timeout = Duration::from_secs(5);
    loop {
        let (new_guard, wait_result) = cvar
            .wait_timeout(guard, timeout)
            .map_err(|e| format!("capture condvar error: {e}"))?;
        guard = new_guard;
        if guard.result.is_some() {
            break;
        }
        if wait_result.timed_out() {
            guard.requested = false;
            return Err("capture timed out waiting for main thread readback".to_string());
        }
    }
    guard.requested = false;
    guard
        .result
        .take()
        .ok_or_else(|| "capture hook resumed without a readback result".to_string())?
}

fn validate_readback(readback: &RawFramebufferReadbackV1) -> Result<(), CaptureFrameError> {
    let expected_len = (readback.width as usize)
        .saturating_mul(readback.height as usize)
        .saturating_mul(4);
    if expected_len == 0 {
        return Err(CaptureFrameError::CaptureFailed(
            "capture readback produced an empty image".to_string(),
        ));
    }
    if readback.rgba8.len() != expected_len {
        return Err(CaptureFrameError::CaptureFailed(format!(
            "capture readback size mismatch: expected {expected_len} bytes, got {} bytes",
            readback.rgba8.len()
        )));
    }
    Ok(())
}

/// Captures one frame artifact for the given route.
pub fn capture_frame_for_route(
    route_id: &RuntimeRouteId,
) -> Result<CaptureArtifactV1, CaptureFrameError> {
    let snapshot = {
        let guard = lock_runtime();
        let runtime = guard.as_ref().ok_or(CaptureFrameError::RouteNotFound)?;
        let route = runtime
            .routes
            .get(&route_id.context_id)
            .ok_or(CaptureFrameError::RouteNotFound)?;
        route.snapshot.clone()
    };

    let hook = hook_for_route(route_id).ok_or_else(|| {
        CaptureFrameError::Unsupported("capture hook is not registered for this route".to_string())
    })?;

    let readback = hook().map_err(CaptureFrameError::CaptureFailed)?;
    validate_readback(&readback)?;

    let mut image_png = Vec::new();
    PngEncoder::new(&mut image_png)
        .write_image(
            &readback.rgba8,
            readback.width,
            readback.height,
            ColorType::Rgba8.into(),
        )
        .map_err(|err| CaptureFrameError::CaptureFailed(format!("png encode failed: {err}")))?;

    let metadata_json = serde_json::to_string(&json!({
        "route_id": snapshot.route_id,
        "surface_kind": route_id.surface_kind,
        "frame_index": snapshot.frame.index,
        "scene_active_name": snapshot.scene.active_scene,
        "debugger": {
            "paused": snapshot.debugger.paused,
            "time_scale": snapshot.debugger.time_scale,
        },
        "image": {
            "width": readback.width,
            "height": readback.height,
        }
    }))
    .map_err(|err| {
        CaptureFrameError::CaptureFailed(format!("metadata serialization failed: {err}"))
    })?;
    let snapshot_json = serde_json::to_string(&snapshot).map_err(|err| {
        CaptureFrameError::CaptureFailed(format!("snapshot serialization failed: {err}"))
    })?;
    let metrics_trace_json = metrics_trace_json_for_route(route_id).unwrap_or_else(|| {
        serde_json::to_string(&empty_metrics_export(route_id)).unwrap_or_else(|_| "{}".to_string())
    });

    Ok(CaptureArtifactV1 {
        image_png,
        metadata_json,
        snapshot_json,
        metrics_trace_json,
    })
}

#[cfg(test)]
pub(super) fn clear_capture_hooks_for_tests() {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}
