use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use super::config::DebuggerConfig;
use super::snapshot::{
    default_capabilities, DebuggerSnapshotV1, LocalEndpointV1, RouteSummaryV1, RuntimeManifestV1,
};
use super::types::{RuntimeRouteId, RuntimeSurfaceKind};
use crate::context_registry::GoudContextId;

#[derive(Debug, Clone)]
struct RouteState {
    label: Option<String>,
    attachable: bool,
    snapshot: DebuggerSnapshotV1,
    capabilities: std::collections::BTreeMap<String, super::types::CapabilityStateV1>,
}

#[derive(Debug)]
struct DebuggerRuntimeState {
    routes: HashMap<u64, RouteState>,
    process_nonce: u64,
    published_at_unix_ms: u64,
}

static DEBUGGER_RUNTIME: OnceLock<Mutex<Option<DebuggerRuntimeState>>> = OnceLock::new();
static PROCESS_NONCE: OnceLock<u64> = OnceLock::new();

fn runtime_cell() -> &'static Mutex<Option<DebuggerRuntimeState>> {
    DEBUGGER_RUNTIME.get_or_init(|| Mutex::new(None))
}

fn process_nonce() -> u64 {
    *PROCESS_NONCE.get_or_init(|| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        (std::process::id() as u64).rotate_left(32) ^ now
    })
}

fn endpoint_for_process(process_nonce: u64) -> LocalEndpointV1 {
    #[cfg(windows)]
    {
        LocalEndpointV1 {
            transport: "named_pipe".to_string(),
            location: format!(
                r"\\.\pipe\goudengine-{}-{}",
                std::process::id(),
                process_nonce
            ),
        }
    }
    #[cfg(not(windows))]
    {
        let root = std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| std::env::temp_dir().display().to_string());
        LocalEndpointV1 {
            transport: "unix".to_string(),
            location: format!(
                "{root}/goudengine-{}-{}.sock",
                std::process::id(),
                process_nonce
            ),
        }
    }
}

fn monotonic_now_ms(previous: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    now.max(previous.saturating_add(1))
}

impl DebuggerRuntimeState {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
            process_nonce: process_nonce(),
            published_at_unix_ms: 0,
        }
    }

    fn touch_manifest(&mut self) {
        self.published_at_unix_ms = monotonic_now_ms(self.published_at_unix_ms);
    }
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
    let key = route_id.context_id;
    runtime.routes.entry(key).or_insert_with(|| RouteState {
        label: config.route_label.clone(),
        attachable: config.publish_local_attach,
        snapshot: DebuggerSnapshotV1::skeleton(route_id.clone()),
        capabilities: default_capabilities(),
    });
    runtime.touch_manifest();
    route_id
}

/// Removes a debugger route and tears the process-wide runtime down when empty.
pub fn unregister_context(context_id: GoudContextId) {
    let raw_context_id = ((context_id.generation() as u64) << 32) | context_id.index() as u64;
    let mut guard = runtime_cell()
        .lock()
        .expect("debugger runtime mutex poisoned");
    let Some(runtime) = guard.as_mut() else {
        return;
    };

    runtime.routes.remove(&raw_context_id);

    if runtime.routes.is_empty() {
        *guard = None;
    } else {
        runtime.touch_manifest();
    }
}

/// Returns the registered route for a context, if one exists.
pub fn route_for_context(context_id: GoudContextId) -> Option<RuntimeRouteId> {
    let raw_context_id = ((context_id.generation() as u64) << 32) | context_id.index() as u64;
    runtime_cell().lock().ok().and_then(|guard| {
        guard
            .as_ref()?
            .routes
            .get(&raw_context_id)
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

/// Mutates one route snapshot. Later adapters use this to update runtime-owned data.
pub fn with_snapshot_mut<R>(
    route_id: &RuntimeRouteId,
    f: impl FnOnce(&mut DebuggerSnapshotV1) -> R,
) -> Option<R> {
    runtime_cell().lock().ok().and_then(|mut guard| {
        let route = guard.as_mut()?.routes.get_mut(&route_id.context_id)?;
        Some(f(&mut route.snapshot))
    })
}

/// Updates the runtime-owned frame state for a route.
pub fn begin_frame(route_id: &RuntimeRouteId, index: u64, delta_seconds: f32, total_seconds: f64) {
    let _ = with_snapshot_mut(route_id, |snapshot| {
        snapshot.frame.index = index;
        snapshot.frame.delta_seconds = delta_seconds;
        snapshot.frame.total_seconds = total_seconds;
    });
}

/// Frame-end hook reserved for later adapter flushing.
pub fn end_frame(_route_id: &RuntimeRouteId) {}

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
}
