use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::types::{
    CapabilityStateV1, RuntimeRouteId, REQUIRED_SERVICE_OWNERS, ROUTE_CAPABILITY_KEYS,
};

/// Shared health entry for one subsystem or debugger-owned service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceHealthV1 {
    pub name: String,
    pub state: CapabilityStateV1,
    pub owner: String,
    pub detail: Option<String>,
    pub updated_frame: u64,
}

/// Route-local frame state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameStateV1 {
    pub index: u64,
    pub delta_seconds: f32,
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
    pub scene_id: String,
    pub entity_id: Option<u64>,
}

/// Current scene summary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneStateV1 {
    pub active_scene: String,
    pub entity_count: u32,
}

/// Inspector view of one entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityStateV1 {
    pub entity_id: u64,
    pub scene_id: String,
    pub name: Option<String>,
    pub component_types: Vec<String>,
    pub components: BTreeMap<String, serde_json::Value>,
}

impl EntityStateV1 {
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
    pub draw_calls: u32,
    pub triangles: u32,
    pub texture_binds: u32,
    pub shader_binds: u32,
}

/// Aggregated memory statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStatsV1 {
    pub tracked_bytes: u64,
    pub peak_bytes: u64,
}

/// Memory usage for one debugger-owned category.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCategoryStatsV1 {
    pub current_bytes: u64,
    pub peak_bytes: u64,
}

/// Snapshot-friendly memory usage totals grouped by subsystem.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySummaryV1 {
    pub rendering: MemoryCategoryStatsV1,
    pub assets: MemoryCategoryStatsV1,
    pub ecs: MemoryCategoryStatsV1,
    pub ui: MemoryCategoryStatsV1,
    pub audio: MemoryCategoryStatsV1,
    pub network: MemoryCategoryStatsV1,
    pub debugger: MemoryCategoryStatsV1,
    pub other: MemoryCategoryStatsV1,
    pub total_current_bytes: u64,
    pub total_peak_bytes: u64,
}

/// One CPU timing sample emitted by the runtime profiler.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilerSampleV1 {
    pub sample_kind: String,
    pub stage: String,
    pub name: String,
    pub duration_cpu_micros: u64,
}

/// Aggregated network statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkStatsV1 {
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Snapshot stats section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotStatsV1 {
    pub render: RenderStatsV1,
    pub memory: MemoryStatsV1,
    pub network: NetworkStatsV1,
}

/// Aggregated diagnostics suitable for overlays and tooling.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsStateV1 {
    pub errors: Vec<String>,
    pub last_fault: Option<String>,
}

/// Runtime-owned debugger state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebuggerStateV1 {
    pub paused: bool,
    pub time_scale: f32,
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
    pub snapshot_version: u32,
    pub route_id: RuntimeRouteId,
    pub frame: FrameStateV1,
    pub selection: SelectionStateV1,
    pub scene: SceneStateV1,
    pub entities: Vec<EntityStateV1>,
    pub profiler_samples: Vec<ProfilerSampleV1>,
    pub services: Vec<ServiceHealthV1>,
    pub stats: SnapshotStatsV1,
    pub memory_summary: MemorySummaryV1,
    pub diagnostics: DiagnosticsStateV1,
    pub debugger: DebuggerStateV1,
}

impl DebuggerSnapshotV1 {
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

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn service_mut(&mut self, name: &str) -> Option<&mut ServiceHealthV1> {
        self.services.iter_mut().find(|service| service.name == name)
    }
}

/// Transport discovery endpoint for a debugger manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalEndpointV1 {
    pub transport: String,
    pub location: String,
}

/// Manifest route summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSummaryV1 {
    pub route_id: RuntimeRouteId,
    pub label: Option<String>,
    pub attachable: bool,
    pub capabilities: BTreeMap<String, CapabilityStateV1>,
}

/// Process-level manifest for debugger route discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeManifestV1 {
    pub manifest_version: u32,
    pub pid: u32,
    pub process_nonce: u64,
    pub executable: String,
    pub endpoint: LocalEndpointV1,
    pub routes: Vec<RouteSummaryV1>,
    pub published_at_unix_ms: u64,
}

impl RuntimeManifestV1 {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

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
