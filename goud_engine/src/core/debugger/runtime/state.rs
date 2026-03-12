use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::context_id::GoudContextId;

use super::super::config::DebuggerConfig;
use super::super::snapshot::{
    default_capabilities, DebuggerSnapshotV1, LocalEndpointV1, MemoryCategoryStatsV1,
    MemorySummaryV1,
};
use super::super::types::{CapabilityStateV1, RuntimeRouteId, RuntimeSurfaceKind};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct RuntimeFpsStats {
    current_fps: f32,
    min_fps: f32,
    max_fps: f32,
    avg_fps: f32,
    frame_time_ms: f32,
}

impl RuntimeFpsStats {
    pub(super) fn from_values(
        current_fps: f32,
        min_fps: f32,
        max_fps: f32,
        avg_fps: f32,
        frame_time_ms: f32,
    ) -> Self {
        Self {
            current_fps,
            min_fps,
            max_fps,
            avg_fps,
            frame_time_ms,
        }
    }

    pub(super) fn as_array(self) -> [f32; 5] {
        [
            self.current_fps,
            self.min_fps,
            self.max_fps,
            self.avg_fps,
            self.frame_time_ms,
        ]
    }
}

#[derive(Debug, Clone)]
pub(super) struct RouteState {
    pub(super) label: Option<String>,
    pub(super) attachable: bool,
    pub(super) profiling_enabled: bool,
    pub(super) snapshot: DebuggerSnapshotV1,
    pub(super) capabilities: BTreeMap<String, CapabilityStateV1>,
    pub(super) fps_stats: RuntimeFpsStats,
}

#[derive(Debug)]
pub(super) struct DebuggerRuntimeState {
    pub(super) routes: HashMap<u64, RouteState>,
    pub(super) process_nonce: u64,
    pub(super) published_at_unix_ms: u64,
}

static DEBUGGER_RUNTIME: OnceLock<Mutex<Option<DebuggerRuntimeState>>> = OnceLock::new();
static PROCESS_NONCE: OnceLock<u64> = OnceLock::new();

pub(super) fn runtime_cell() -> &'static Mutex<Option<DebuggerRuntimeState>> {
    DEBUGGER_RUNTIME.get_or_init(|| Mutex::new(None))
}

pub(super) fn lock_runtime() -> MutexGuard<'static, Option<DebuggerRuntimeState>> {
    runtime_cell()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

pub(super) fn raw_context_key(context_id: GoudContextId) -> u64 {
    ((context_id.generation() as u64) << 32) | context_id.index() as u64
}

pub(super) fn endpoint_for_process(process_nonce: u64) -> LocalEndpointV1 {
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

pub(super) fn recalculate_memory_totals(summary: &mut MemorySummaryV1) {
    let all = [
        summary.rendering,
        summary.assets,
        summary.ecs,
        summary.ui,
        summary.audio,
        summary.network,
        summary.debugger,
        summary.other,
    ];
    summary.total_current_bytes = all.iter().map(|stats| stats.current_bytes).sum();
    summary.total_peak_bytes = all.iter().map(|stats| stats.peak_bytes).sum();
}

pub(super) fn memory_category_mut<'a>(
    summary: &'a mut MemorySummaryV1,
    category: &str,
) -> Option<&'a mut MemoryCategoryStatsV1> {
    match category {
        "rendering" => Some(&mut summary.rendering),
        "assets" => Some(&mut summary.assets),
        "ecs" => Some(&mut summary.ecs),
        "ui" => Some(&mut summary.ui),
        "audio" => Some(&mut summary.audio),
        "network" => Some(&mut summary.network),
        "debugger" => Some(&mut summary.debugger),
        "other" => Some(&mut summary.other),
        _ => None,
    }
}

pub(super) fn set_route_capability(route: &mut RouteState, key: &str, state: CapabilityStateV1) {
    if let Some(capability) = route.capabilities.get_mut(key) {
        *capability = state;
    }
}

pub(super) fn set_service_state(
    snapshot: &mut DebuggerSnapshotV1,
    name: &str,
    state: CapabilityStateV1,
    detail: Option<String>,
) {
    let frame_index = snapshot.frame.index;
    if let Some(service) = snapshot.service_mut(name) {
        service.state = state;
        service.detail = detail;
        service.updated_frame = frame_index;
    }
}

pub(super) fn initialize_route_state(
    route_id: RuntimeRouteId,
    surface_kind: RuntimeSurfaceKind,
    config: &DebuggerConfig,
) -> RouteState {
    let mut snapshot = DebuggerSnapshotV1::skeleton(route_id);
    let mut capabilities = default_capabilities();

    capabilities.insert("entity_inspection".to_string(), CapabilityStateV1::Ready);
    capabilities.insert("memory_stats".to_string(), CapabilityStateV1::Ready);
    capabilities.insert("control_plane".to_string(), CapabilityStateV1::Ready);
    capabilities.insert(
        "render_stats".to_string(),
        match surface_kind {
            RuntimeSurfaceKind::WindowedGame => CapabilityStateV1::Ready,
            _ => CapabilityStateV1::Unavailable,
        },
    );

    set_service_state(
        &mut snapshot,
        "memory",
        CapabilityStateV1::Ready,
        Some("engine-managed memory tracking active".to_string()),
    );
    set_service_state(
        &mut snapshot,
        "profiling",
        CapabilityStateV1::Disabled,
        Some("profiling disabled for this route".to_string()),
    );
    set_service_state(
        &mut snapshot,
        "renderer",
        match surface_kind {
            RuntimeSurfaceKind::WindowedGame => CapabilityStateV1::Ready,
            _ => CapabilityStateV1::Unavailable,
        },
        None,
    );
    set_service_state(
        &mut snapshot,
        "window",
        match surface_kind {
            RuntimeSurfaceKind::WindowedGame => CapabilityStateV1::Ready,
            _ => CapabilityStateV1::Unavailable,
        },
        None,
    );

    RouteState {
        label: config.route_label.clone(),
        attachable: config.publish_local_attach,
        profiling_enabled: false,
        snapshot,
        capabilities,
        fps_stats: RuntimeFpsStats::default(),
    }
}

pub(super) fn with_route_state_mut<R>(
    route_id: &RuntimeRouteId,
    f: impl FnOnce(&mut RouteState) -> R,
) -> Option<R> {
    let mut guard = lock_runtime();
    let route = guard.as_mut()?.routes.get_mut(&route_id.context_id)?;
    Some(f(route))
}

pub(super) fn with_route_state_mut_by_context<R>(
    context_id: GoudContextId,
    f: impl FnOnce(&mut RouteState) -> R,
) -> Option<R> {
    let key = raw_context_key(context_id);
    let mut guard = lock_runtime();
    let route = guard.as_mut()?.routes.get_mut(&key)?;
    Some(f(route))
}

impl DebuggerRuntimeState {
    pub(super) fn new() -> Self {
        Self {
            routes: HashMap::new(),
            process_nonce: process_nonce(),
            published_at_unix_ms: 0,
        }
    }

    pub(super) fn touch_manifest(&mut self) {
        self.published_at_unix_ms = monotonic_now_ms(self.published_at_unix_ms);
    }
}
