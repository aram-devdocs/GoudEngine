use std::cell::RefCell;

use super::config::DebuggerConfig;
use super::snapshot::{DebuggerSnapshotV1, MemorySummaryV1, ProfilerSampleV1, RouteSummaryV1, RuntimeManifestV1};
use super::types::{CapabilityStateV1, RuntimeRouteId, RuntimeSurfaceKind};
use crate::context_registry::GoudContextId;

mod state;
use state::*;

thread_local! {
    static CURRENT_ROUTE: RefCell<Option<RuntimeRouteId>> = const { RefCell::new(None) };
}

/// Registers a debugger route for one context when debugger mode is enabled.
pub fn register_context(
    context_id: GoudContextId,
    surface_kind: RuntimeSurfaceKind,
    config: &DebuggerConfig,
) -> RuntimeRouteId {
    let mut guard = runtime_cell()
        .lock()
        .expect("debugger runtime mutex poisoned");
    let runtime = guard.get_or_insert_with(DebuggerRuntimeState::new);
    let route_id = RuntimeRouteId::for_context(runtime.process_nonce, context_id, surface_kind);
    runtime
        .routes
        .entry(route_id.context_id)
        .or_insert_with(|| initialize_route_state(route_id.clone(), surface_kind, config));
    runtime.touch_manifest();
    route_id
}

/// Removes a debugger route and tears the process-wide runtime down when empty.
pub fn unregister_context(context_id: GoudContextId) {
    let mut guard = runtime_cell()
        .lock()
        .expect("debugger runtime mutex poisoned");
    let Some(runtime) = guard.as_mut() else {
        return;
    };

    runtime.routes.remove(&raw_context_key(context_id));

    if runtime.routes.is_empty() {
        *guard = None;
    } else {
        runtime.touch_manifest();
    }
}

/// Returns the route currently scoped to this thread, if any.
pub fn current_route() -> Option<RuntimeRouteId> {
    CURRENT_ROUTE.with(|cell| cell.borrow().clone())
}

/// Runs `f` with an active route scoped to the current thread.
pub fn scoped_route<R>(route_id: Option<RuntimeRouteId>, f: impl FnOnce() -> R) -> R {
    CURRENT_ROUTE.with(|cell| {
        let previous = cell.replace(route_id);
        let result = f();
        cell.replace(previous);
        result
    })
}

/// Returns the registered route for a context, if one exists.
pub fn route_for_context(context_id: GoudContextId) -> Option<RuntimeRouteId> {
    runtime_cell().lock().ok().and_then(|guard| {
        guard
            .as_ref()?
            .routes
            .get(&raw_context_key(context_id))
            .map(|route| route.snapshot.route_id.clone())
    })
}

/// Returns the number of currently registered routes.
pub fn active_route_count() -> usize {
    runtime_cell()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|runtime| runtime.routes.len()))
        .unwrap_or(0)
}

/// Returns a cloned snapshot for one route.
pub fn snapshot_for_route(route_id: &RuntimeRouteId) -> Option<DebuggerSnapshotV1> {
    runtime_cell().lock().ok().and_then(|guard| {
        guard
            .as_ref()?
            .routes
            .get(&route_id.context_id)
            .map(|route| route.snapshot.clone())
    })
}

/// Returns a cloned snapshot for one context, if registered.
pub fn snapshot_for_context(context_id: GoudContextId) -> Option<DebuggerSnapshotV1> {
    route_for_context(context_id).and_then(|route_id| snapshot_for_route(&route_id))
}

/// Returns a serialized snapshot JSON document for one context, if registered.
pub fn snapshot_json_for_context(context_id: GoudContextId) -> Option<String> {
    snapshot_for_context(context_id).and_then(|snapshot| snapshot.to_json().ok())
}

/// Mutates one route snapshot.
pub fn with_snapshot_mut<R>(
    route_id: &RuntimeRouteId,
    f: impl FnOnce(&mut DebuggerSnapshotV1) -> R,
) -> Option<R> {
    with_route_state_mut(route_id, |route| f(&mut route.snapshot))
}

/// Updates the runtime-owned frame state for a route and resets per-frame counters.
pub fn begin_frame(route_id: &RuntimeRouteId, index: u64, delta_seconds: f32, total_seconds: f64) {
    let _ = with_route_state_mut(route_id, |route| {
        route.snapshot.frame.index = index;
        route.snapshot.frame.delta_seconds = delta_seconds;
        route.snapshot.frame.total_seconds = total_seconds;
        route.snapshot.profiler_samples.clear();
        route.snapshot.stats.render = Default::default();
    });
}

/// Frame-end hook that keeps debugger-category memory totals current.
pub fn end_frame(route_id: &RuntimeRouteId) {
    let _ = with_route_state_mut(route_id, |route| {
        let debugger_bytes = serde_json::to_vec(&route.snapshot)
            .map(|bytes| bytes.len() as u64)
            .unwrap_or_default();
        let debugger = &mut route.snapshot.memory_summary.debugger;
        debugger.current_bytes = debugger_bytes;
        debugger.peak_bytes = debugger.peak_bytes.max(debugger_bytes);
        recalculate_memory_totals(&mut route.snapshot.memory_summary);
        route.snapshot.stats.memory.tracked_bytes = route.snapshot.memory_summary.total_current_bytes;
        route.snapshot.stats.memory.peak_bytes = route.snapshot.memory_summary.total_peak_bytes;
    });
}

/// Returns whether profiling is enabled for the given context.
pub fn profiler_enabled_for_context(context_id: GoudContextId) -> bool {
    with_route_state_mut_by_context(context_id, |route| route.profiling_enabled).unwrap_or(false)
}

/// Returns whether profiling is enabled for the given route.
pub fn profiler_enabled_for_route(route_id: &RuntimeRouteId) -> bool {
    with_route_state_mut(route_id, |route| route.profiling_enabled).unwrap_or(false)
}

/// Enables or disables route-local profiling for a context.
pub fn set_profiling_enabled_for_context(context_id: GoudContextId, enabled: bool) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
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
        true
    })
    .unwrap_or(false)
}

/// Stores the current FPS overlay statistics for a route-backed context.
pub fn update_fps_stats_for_context(
    context_id: GoudContextId,
    current_fps: f32,
    min_fps: f32,
    max_fps: f32,
    avg_fps: f32,
    frame_time_ms: f32,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.fps_stats =
            RuntimeFpsStats::from_values(current_fps, min_fps, max_fps, avg_fps, frame_time_ms);
        true
    })
    .unwrap_or(false)
}

/// Returns the current FPS overlay statistics for a route-backed context.
pub fn fps_stats_for_context(context_id: GoudContextId) -> Option<[f32; 5]> {
    with_route_state_mut_by_context(context_id, |route| route.fps_stats.as_array())
}

/// Sets the currently selected entity for one context.
pub fn set_selected_entity_for_context(
    context_id: GoudContextId,
    entity_id: Option<u64>,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        if route.snapshot.selection.scene_id.is_empty() {
            route.snapshot.selection.scene_id = route.snapshot.scene.active_scene.clone();
        }
        route.snapshot.selection.entity_id = entity_id;
        true
    })
    .unwrap_or(false)
}

/// Appends render stats to the current frame totals for one context.
pub fn update_render_stats_for_context(
    context_id: GoudContextId,
    draw_calls: u32,
    triangles: u32,
    texture_binds: u32,
    shader_binds: u32,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.snapshot.stats.render.draw_calls = route
            .snapshot
            .stats
            .render
            .draw_calls
            .saturating_add(draw_calls);
        route.snapshot.stats.render.triangles = route
            .snapshot
            .stats
            .render
            .triangles
            .saturating_add(triangles);
        route.snapshot.stats.render.texture_binds = route
            .snapshot
            .stats
            .render
            .texture_binds
            .saturating_add(texture_binds);
        route.snapshot.stats.render.shader_binds = route
            .snapshot
            .stats
            .render
            .shader_binds
            .saturating_add(shader_binds);
        set_route_capability(route, "render_stats", CapabilityStateV1::Ready);
        set_service_state(&mut route.snapshot, "renderer", CapabilityStateV1::Ready, None);
        true
    })
    .unwrap_or(false)
}

/// Updates one tracked memory category for the given context.
pub fn update_memory_category_for_context(
    context_id: GoudContextId,
    category: &str,
    current_bytes: u64,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        let Some(category_stats) = memory_category_mut(&mut route.snapshot.memory_summary, category)
        else {
            return false;
        };
        category_stats.current_bytes = current_bytes;
        category_stats.peak_bytes = category_stats.peak_bytes.max(current_bytes);
        recalculate_memory_totals(&mut route.snapshot.memory_summary);
        route.snapshot.stats.memory.tracked_bytes = route.snapshot.memory_summary.total_current_bytes;
        route.snapshot.stats.memory.peak_bytes = route.snapshot.memory_summary.total_peak_bytes;
        set_route_capability(route, "memory_stats", CapabilityStateV1::Ready);
        set_service_state(&mut route.snapshot, "memory", CapabilityStateV1::Ready, None);
        true
    })
    .unwrap_or(false)
}

/// Returns the memory summary for one context, if registered.
pub fn get_memory_summary_for_context(context_id: GoudContextId) -> Option<MemorySummaryV1> {
    with_route_state_mut_by_context(context_id, |route| route.snapshot.memory_summary)
}

/// Stores the latest network snapshot totals for a context.
pub fn set_snapshot_network_stats_for_context(
    context_id: GoudContextId,
    bytes_sent: u64,
    bytes_received: u64,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.snapshot.stats.network.bytes_sent = bytes_sent;
        route.snapshot.stats.network.bytes_received = bytes_received;
        set_service_state(&mut route.snapshot, "network", CapabilityStateV1::Ready, None);
        true
    })
    .unwrap_or(false)
}

/// Updates one service entry on the context snapshot.
pub fn set_service_state_for_context(
    context_id: GoudContextId,
    name: &str,
    state: CapabilityStateV1,
    detail: Option<String>,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        set_service_state(&mut route.snapshot, name, state, detail);
        true
    })
    .unwrap_or(false)
}

/// Records one named system sample on a specific route.
pub fn set_system_sample(
    route_id: &RuntimeRouteId,
    stage: &str,
    system_name: &str,
    duration_cpu_micros: u64,
) {
    let _ = with_route_state_mut(route_id, |route| {
        if !route.profiling_enabled {
            return;
        }
        route.snapshot.profiler_samples.push(ProfilerSampleV1 {
            sample_kind: "system".to_string(),
            stage: stage.to_string(),
            name: system_name.to_string(),
            duration_cpu_micros,
        });
        set_route_capability(route, "profiling", CapabilityStateV1::Ready);
        set_service_state(&mut route.snapshot, "profiling", CapabilityStateV1::Ready, None);
    });
}

/// Records one named synthetic phase on the current active route.
pub fn record_phase_duration(name: &str, duration_cpu_micros: u64) {
    let Some(route_id) = current_route() else {
        return;
    };

    let _ = with_route_state_mut(&route_id, |route| {
        if !route.profiling_enabled {
            return;
        }
        route.snapshot.profiler_samples.push(ProfilerSampleV1 {
            sample_kind: "phase".to_string(),
            stage: "runtime".to_string(),
            name: name.to_string(),
            duration_cpu_micros,
        });
        set_route_capability(route, "profiling", CapabilityStateV1::Ready);
        set_service_state(&mut route.snapshot, "profiling", CapabilityStateV1::Ready, None);
    });
}

/// Returns the current manifest when at least one route is attachable.
pub fn current_manifest() -> Option<RuntimeManifestV1> {
    let guard = runtime_cell().lock().ok()?;
    let runtime = guard.as_ref()?;
    if !runtime.routes.values().any(|route| route.attachable) {
        return None;
    }

    let executable = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut routes: Vec<RouteSummaryV1> = runtime
        .routes
        .values()
        .map(|route| RouteSummaryV1 {
            route_id: route.snapshot.route_id.clone(),
            label: route.label.clone(),
            attachable: route.attachable,
            capabilities: route.capabilities.clone(),
        })
        .collect();
    routes.sort_by_key(|route| route.route_id.context_id);

    Some(RuntimeManifestV1 {
        manifest_version: 1,
        pid: std::process::id(),
        process_nonce: runtime.process_nonce,
        executable,
        endpoint: endpoint_for_process(runtime.process_nonce),
        routes,
        published_at_unix_ms: runtime.published_at_unix_ms,
    })
}

#[cfg(test)]
pub(crate) fn reset_for_tests() {
    if let Ok(mut guard) = runtime_cell().lock() {
        *guard = None;
    }
    CURRENT_ROUTE.with(|cell| {
        cell.replace(None);
    });
}
