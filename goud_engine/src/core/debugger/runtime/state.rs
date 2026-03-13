use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::core::context_id::GoudContextId;

use super::super::config::DebuggerConfig;
use super::super::snapshot::{
    default_capabilities, DebuggerSnapshotV1, LocalEndpointV1, MemoryCategoryStatsV1,
    MemorySummaryV1,
};
use super::super::types::{CapabilityStateV1, RuntimeRouteId, RuntimeSurfaceKind};
use super::debug_draw::DebugDrawPayloadV1;
use super::metrics::RouteMetricsState;
use super::replay::RouteReplayState;

/// One normalized synthetic input event queued by the debugger runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntheticInputEventV1 {
    /// Input device family such as `keyboard` or `mouse`.
    pub device: String,
    /// Requested action such as `press` or `release`.
    pub action: String,
    /// Stable key name when the event targets a keyboard key.
    pub key: Option<String>,
    /// Stable button name when the event targets a mouse button.
    pub button: Option<String>,
    /// Normalized absolute pointer position for mouse movement events.
    pub position: Option<[f32; 2]>,
    /// Normalized movement or scroll delta payload.
    pub delta: Option<[f32; 2]>,
}

/// Route-local control state surfaced to debugger clients.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct RouteControlStateV1 {
    /// Whether route time is currently paused.
    pub paused: bool,
    /// Runtime-owned time-scale multiplier for this route.
    pub time_scale: f32,
    /// Remaining whole-frame step budget.
    pub frame_step_budget: u64,
    /// Remaining tick-step budget for fixed-step coordinators.
    pub tick_step_budget: u64,
    /// Whether debugger-controlled debug draw is enabled.
    pub debug_draw_enabled: bool,
    /// Number of queued synthetic inputs waiting to be consumed.
    pub queued_input_events: usize,
}

/// One frame's worth of runtime-owned control decisions for a route.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameControlPlanV1 {
    /// Whether the route should advance this frame.
    pub advance_frame: bool,
    /// Effective delta time after pause/time-scale control.
    pub effective_delta_seconds: f32,
    /// Whether debug draw should be enabled for the frame.
    pub debug_draw_enabled: bool,
    /// Synthetic inputs to inject before the frame update runs.
    pub synthetic_inputs: Vec<SyntheticInputEventV1>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct RouteControlState {
    pub(super) paused: bool,
    pub(super) time_scale: f32,
    pub(super) frame_step_budget: u64,
    pub(super) tick_step_budget: u64,
    pub(super) debug_draw_enabled: bool,
    pub(super) synthetic_inputs: VecDeque<SyntheticInputEventV1>,
}

impl RouteControlState {
    pub(super) fn new() -> Self {
        Self {
            time_scale: 1.0,
            ..Self::default()
        }
    }

    pub(super) fn snapshot(&self) -> RouteControlStateV1 {
        RouteControlStateV1 {
            paused: self.paused,
            time_scale: self.time_scale,
            frame_step_budget: self.frame_step_budget,
            tick_step_budget: self.tick_step_budget,
            debug_draw_enabled: self.debug_draw_enabled,
            queued_input_events: self.synthetic_inputs.len(),
        }
    }

    pub(super) fn take_frame_plan(&mut self, raw_delta_seconds: f32) -> FrameControlPlanV1 {
        let advance_frame = !self.paused || self.frame_step_budget > 0;
        if self.paused && self.frame_step_budget > 0 {
            self.frame_step_budget = self.frame_step_budget.saturating_sub(1);
        }

        FrameControlPlanV1 {
            advance_frame,
            effective_delta_seconds: if advance_frame {
                raw_delta_seconds.max(0.0) * self.time_scale.max(0.0)
            } else {
                0.0
            },
            debug_draw_enabled: self.debug_draw_enabled,
            synthetic_inputs: self.synthetic_inputs.drain(..).collect(),
        }
    }
}

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
    pub(super) debug_draw: DebugDrawPayloadV1,
    pub(super) capabilities: BTreeMap<String, CapabilityStateV1>,
    pub(super) fps_stats: RuntimeFpsStats,
    pub(super) control: RouteControlState,
    pub(super) replay: RouteReplayState,
    pub(super) metrics: RouteMetricsState,
    pub(super) attached_clients: u32,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeArtifactsState {
    pub(super) runtime_dir: PathBuf,
    pub(super) artifacts_root_dir: PathBuf,
    pub(super) manifest_path: PathBuf,
    pub(super) endpoint: LocalEndpointV1,
    pub(super) next_artifact_sequence: u64,
    pub(super) artifact_paths: HashMap<String, PathBuf>,
    pub(super) route_buckets: HashMap<(u64, String), VecDeque<String>>,
}

#[derive(Debug)]
pub(super) struct AttachServerState {
    pub(super) endpoint_location: String,
    pub(super) shutdown: Arc<AtomicBool>,
    pub(super) accept_thread: Option<JoinHandle<()>>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone)]
pub(super) struct AttachSessionState {
    pub(super) route_key: u64,
    pub(super) in_flight: bool,
}

#[derive(Debug)]
pub(super) struct DebuggerRuntimeState {
    pub(super) routes: HashMap<u64, RouteState>,
    pub(super) sessions: HashMap<u64, AttachSessionState>,
    pub(super) process_nonce: u64,
    pub(super) published_at_unix_ms: u64,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) next_session_id: u64,
    pub(super) artifacts: Option<RuntimeArtifactsState>,
    pub(super) attach_server: Option<AttachServerState>,
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

pub(super) fn sync_debugger_state(route: &mut RouteState) {
    route.snapshot.debugger.paused = route.control.paused;
    route.snapshot.debugger.time_scale = route.control.time_scale;
    route.snapshot.debugger.attached_clients = route.attached_clients;
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
    capabilities.insert("replay".to_string(), CapabilityStateV1::Ready);
    capabilities.insert(
        "capture".to_string(),
        match surface_kind {
            RuntimeSurfaceKind::WindowedGame => CapabilityStateV1::Ready,
            _ => CapabilityStateV1::Unavailable,
        },
    );
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
    set_service_state(
        &mut snapshot,
        "replay",
        CapabilityStateV1::Ready,
        Some(
            "idle; normalized input timing only; physics and render output can still diverge by platform/frame pacing"
                .to_string(),
        ),
    );
    set_service_state(
        &mut snapshot,
        "capture",
        match surface_kind {
            RuntimeSurfaceKind::WindowedGame => CapabilityStateV1::Ready,
            _ => CapabilityStateV1::Unavailable,
        },
        Some(match surface_kind {
            RuntimeSurfaceKind::WindowedGame => {
                "framebuffer capture ready for this route".to_string()
            }
            _ => "capture is unavailable for this route surface".to_string(),
        }),
    );

    let mut route = RouteState {
        label: config.route_label.clone(),
        attachable: config.publish_local_attach,
        profiling_enabled: false,
        snapshot,
        debug_draw: DebugDrawPayloadV1::default(),
        capabilities,
        fps_stats: RuntimeFpsStats::default(),
        control: RouteControlState::new(),
        replay: RouteReplayState::default(),
        metrics: RouteMetricsState::default(),
        attached_clients: 0,
    };
    sync_debugger_state(&mut route);
    route
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
            sessions: HashMap::new(),
            process_nonce: process_nonce(),
            published_at_unix_ms: 0,
            next_session_id: 1,
            artifacts: None,
            attach_server: None,
        }
    }

    pub(super) fn touch_manifest(&mut self) {
        self.published_at_unix_ms = monotonic_now_ms(self.published_at_unix_ms);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn next_session_id(&mut self) -> u64 {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.saturating_add(1);
        session_id
    }

    pub(super) fn attached_route_count(&self) -> usize {
        self.routes
            .values()
            .filter(|route| route.attachable)
            .count()
    }

    pub(super) fn session_ids_for_route(&self, route_key: u64) -> HashSet<u64> {
        self.sessions
            .iter()
            .filter_map(|(session_id, session)| {
                (session.route_key == route_key).then_some(*session_id)
            })
            .collect()
    }
}
