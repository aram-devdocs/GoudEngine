use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::types::{
    CapabilityStateV1, RuntimeRouteId, REQUIRED_SERVICE_OWNERS, ROUTE_CAPABILITY_KEYS,
};

/// Shared health entry for one subsystem or debugger-owned service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealthV1 {
    /// Stable service wire key.
    pub name: String,
    /// Current availability state.
    pub state: CapabilityStateV1,
    /// Subsystem that owns this service.
    pub owner: String,
    /// Optional human-readable status detail.
    pub detail: Option<String>,
    /// Frame index when this entry last changed.
    pub updated_frame: u64,
}

/// Route-local frame state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameStateV1 {
    /// Frame number for this snapshot.
    pub index: u64,
    /// Seconds since the previous frame.
    pub delta_seconds: f32,
    /// Total elapsed seconds for the route.
    pub total_seconds: f64,
}

impl Default for FrameStateV1 {
    fn default() -> Self {
        Self {
            index: 0,
            delta_seconds: 0.0,
            total_seconds: 0.0,
        }
    }
}

/// Current inspector selection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionStateV1 {
    /// Scene that owns the current selection.
    pub scene_id: String,
    /// Selected entity in that scene, if any.
    pub entity_id: Option<u64>,
}

/// Current scene summary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneStateV1 {
    /// Active scene name for the route.
    pub active_scene: String,
    /// Number of live entities in the active scene.
    pub entity_count: u32,
}

/// Inspector view of one entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityStateV1 {
    /// Stable entity identifier.
    pub entity_id: u64,
    /// Scene that owns the entity.
    pub scene_id: String,
    /// Optional entity name component.
    pub name: Option<String>,
    /// Sorted component type names present on the entity.
    pub component_types: Vec<String>,
    /// Full component payloads for the selected entity only.
    pub components: BTreeMap<String, serde_json::Value>,
}

impl EntityStateV1 {
    /// Creates an inspector entry without full component payloads.
    pub fn summary_only(
        entity_id: u64,
        scene_id: impl Into<String>,
        name: Option<String>,
        component_types: Vec<String>,
    ) -> Self {
        Self {
            entity_id,
            scene_id: scene_id.into(),
            name,
            component_types,
            components: BTreeMap::new(),
        }
    }
}

/// Aggregated render statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderStatsV1 {
    /// Draw calls issued this frame.
    pub draw_calls: u32,
    /// Triangles submitted this frame.
    pub triangles: u32,
    /// Texture bind count for this frame.
    pub texture_binds: u32,
    /// Shader or pipeline bind count for this frame.
    pub shader_binds: u32,
}

/// Aggregated memory statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStatsV1 {
    /// Total tracked bytes at snapshot time.
    pub tracked_bytes: u64,
    /// Peak tracked bytes seen so far.
    pub peak_bytes: u64,
}

/// Memory usage for one debugger-owned category.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCategoryStatsV1 {
    /// Current bytes tracked for this category.
    pub current_bytes: u64,
    /// Peak bytes tracked for this category.
    pub peak_bytes: u64,
}

/// Snapshot-friendly memory usage totals grouped by subsystem.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySummaryV1 {
    /// Rendering subsystem memory totals.
    pub rendering: MemoryCategoryStatsV1,
    /// Asset subsystem memory totals.
    pub assets: MemoryCategoryStatsV1,
    /// ECS subsystem memory totals.
    pub ecs: MemoryCategoryStatsV1,
    /// UI subsystem memory totals.
    pub ui: MemoryCategoryStatsV1,
    /// Audio subsystem memory totals.
    pub audio: MemoryCategoryStatsV1,
    /// Network subsystem memory totals.
    pub network: MemoryCategoryStatsV1,
    /// Debugger-owned memory totals.
    pub debugger: MemoryCategoryStatsV1,
    /// Memory that does not fit another category.
    pub other: MemoryCategoryStatsV1,
    /// Sum of current bytes across all categories.
    pub total_current_bytes: u64,
    /// Sum of peak bytes across all categories.
    pub total_peak_bytes: u64,
}

/// One CPU timing sample emitted by the runtime profiler.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilerSampleV1 {
    /// Sample family such as `system` or `phase`.
    pub sample_kind: String,
    /// Stage that produced the sample.
    pub stage: String,
    /// Stable system or phase name.
    pub name: String,
    /// CPU time in microseconds.
    pub duration_cpu_micros: u64,
}

/// Aggregated network statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkStatsV1 {
    /// Bytes sent through runtime-owned providers.
    pub bytes_sent: u64,
    /// Bytes received through runtime-owned providers.
    pub bytes_received: u64,
}

/// Snapshot stats section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotStatsV1 {
    /// Per-frame render counters.
    pub render: RenderStatsV1,
    /// Aggregate memory counters.
    pub memory: MemoryStatsV1,
    /// Aggregate network counters.
    pub network: NetworkStatsV1,
}

/// Aggregated diagnostics suitable for overlays and tooling.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsStateV1 {
    /// Current non-fatal errors.
    pub errors: Vec<String>,
    /// Most recent fault, if one exists.
    pub last_fault: Option<String>,
}

/// Runtime-owned debugger state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebuggerStateV1 {
    /// Whether route time is paused.
    pub paused: bool,
    /// Time scale requested by the debugger.
    pub time_scale: f32,
    /// Number of attached local clients.
    pub attached_clients: u32,
}

impl Default for DebuggerStateV1 {
    fn default() -> Self {
        Self {
            paused: false,
            time_scale: 1.0,
            attached_clients: 0,
        }
    }
}

/// Canonical route snapshot schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebuggerSnapshotV1 {
    /// Snapshot schema version.
    pub snapshot_version: u32,
    /// Route identity for this snapshot.
    pub route_id: RuntimeRouteId,
    /// Frame timing information.
    pub frame: FrameStateV1,
    /// Current inspector selection.
    pub selection: SelectionStateV1,
    /// Active scene summary.
    pub scene: SceneStateV1,
    /// Entity summaries for the active route.
    pub entities: Vec<EntityStateV1>,
    /// Profiler samples collected this frame.
    pub profiler_samples: Vec<ProfilerSampleV1>,
    /// Service health entries for the route.
    pub services: Vec<ServiceHealthV1>,
    /// Aggregated stats section.
    pub stats: SnapshotStatsV1,
    /// Memory totals grouped by subsystem.
    pub memory_summary: MemorySummaryV1,
    /// Diagnostics collected for the route.
    pub diagnostics: DiagnosticsStateV1,
    /// Debugger-owned state for the route.
    pub debugger: DebuggerStateV1,
}

impl DebuggerSnapshotV1 {
    /// Creates an empty snapshot for a registered route.
    pub fn skeleton(route_id: RuntimeRouteId) -> Self {
        Self {
            snapshot_version: 1,
            route_id,
            frame: FrameStateV1::default(),
            selection: SelectionStateV1::default(),
            scene: SceneStateV1::default(),
            entities: Vec::new(),
            profiler_samples: Vec::new(),
            services: default_services(),
            stats: SnapshotStatsV1::default(),
            memory_summary: MemorySummaryV1::default(),
            diagnostics: DiagnosticsStateV1::default(),
            debugger: DebuggerStateV1::default(),
        }
    }

    /// Serializes this snapshot to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Finds one service entry by wire key.
    pub fn service_mut(&mut self, name: &str) -> Option<&mut ServiceHealthV1> {
        self.services
            .iter_mut()
            .find(|service| service.name == name)
    }
}

/// Transport discovery endpoint for a debugger manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalEndpointV1 {
    /// Transport name, such as `local`.
    pub transport: String,
    /// Local endpoint location string.
    pub location: String,
}

/// Manifest route summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSummaryV1 {
    /// Stable route identity.
    pub route_id: RuntimeRouteId,
    /// Optional human-facing route label.
    pub label: Option<String>,
    /// Whether local attach is allowed for this route.
    pub attachable: bool,
    /// Capability map keyed by RFC wire name.
    pub capabilities: BTreeMap<String, CapabilityStateV1>,
}

/// Process-level manifest for debugger route discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeManifestV1 {
    /// Manifest schema version.
    pub manifest_version: u32,
    /// Process identifier.
    pub pid: u32,
    /// Per-process nonce shared by all routes.
    pub process_nonce: u64,
    /// Executable path or process label.
    pub executable: String,
    /// Local attach endpoint information.
    pub endpoint: LocalEndpointV1,
    /// Routes currently published by the process.
    pub routes: Vec<RouteSummaryV1>,
    /// Publication timestamp in Unix milliseconds.
    pub published_at_unix_ms: u64,
}

impl RuntimeManifestV1 {
    /// Serializes this manifest to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Returns the default capability map for a new route.
pub fn default_capabilities() -> BTreeMap<String, CapabilityStateV1> {
    ROUTE_CAPABILITY_KEYS
        .into_iter()
        .map(|key| {
            let state = match key {
                "snapshots" => CapabilityStateV1::Ready,
                "debug_draw" | "replay" | "capture" => CapabilityStateV1::Unavailable,
                _ => CapabilityStateV1::Disabled,
            };
            (key.to_string(), state)
        })
        .collect()
}

/// Returns the default service health list for a new route.
pub fn default_services() -> Vec<ServiceHealthV1> {
    REQUIRED_SERVICE_OWNERS
        .into_iter()
        .map(|(name, owner)| ServiceHealthV1 {
            name: name.to_string(),
            state: match name {
                "debugger" => CapabilityStateV1::Ready,
                "capture" | "replay" | "renderer" | "physics" | "audio" | "network" | "window"
                | "assets" => CapabilityStateV1::Unavailable,
                _ => CapabilityStateV1::Disabled,
            },
            owner: owner.to_string(),
            detail: None,
            updated_frame: 0,
        })
        .collect()
}
