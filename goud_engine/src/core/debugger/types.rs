use serde::{Deserialize, Serialize};

use crate::core::context_id::GoudContextId;

/// Stable route identity for a debuggable context within one process lifetime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeRouteId {
    pub process_nonce: u64,
    pub context_id: u64,
    pub surface_kind: RuntimeSurfaceKind,
}

impl RuntimeRouteId {
    pub(crate) fn for_context(
        process_nonce: u64,
        context_id: GoudContextId,
        surface_kind: RuntimeSurfaceKind,
    ) -> Self {
        Self {
            process_nonce,
            context_id: ((context_id.generation() as u64) << 32) | context_id.index() as u64,
            surface_kind,
        }
    }
}

/// Identifies the kind of debuggable surface behind a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSurfaceKind {
    WindowedGame,
    HeadlessContext,
    ToolContext,
}

/// Shared availability state used by route capabilities and service health.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStateV1 {
    Ready,
    Disabled,
    Unavailable,
    Faulted,
}

/// Route-scoped capability wire keys required by RFC-0004.
pub const ROUTE_CAPABILITY_KEYS: [&str; 9] = [
    "snapshots",
    "profiling",
    "render_stats",
    "memory_stats",
    "entity_inspection",
    "debug_draw",
    "control_plane",
    "replay",
    "capture",
];

/// Required snapshot service definitions `(name, owner)`.
pub const REQUIRED_SERVICE_OWNERS: [(&str, &str); 11] = [
    ("renderer", "renderer_adapter"),
    ("memory", "memory_adapter"),
    ("profiling", "debugger_runtime"),
    ("physics", "physics_adapter"),
    ("audio", "audio_adapter"),
    ("network", "network_adapter"),
    ("window", "window_adapter"),
    ("assets", "asset_manager"),
    ("capture", "capture_subsystem"),
    ("replay", "replay_subsystem"),
    ("debugger", "debugger_runtime"),
];
