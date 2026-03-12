use serde::{Deserialize, Serialize};

use crate::core::context_id::GoudContextId;

/// Stable route identity for a debuggable context within one process lifetime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeRouteId {
    /// Per-process nonce used to distinguish routes across restarts.
    pub process_nonce: u64,
    /// Packed context identifier `(generation << 32) | index`.
    pub context_id: u64,
    /// Surface kind served by this route.
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
    /// Route backed by a windowed game instance.
    WindowedGame,
    /// Route backed by a headless context.
    HeadlessContext,
    /// Route backed by an editor or tooling context.
    ToolContext,
}

/// Shared availability state used by route capabilities and service health.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStateV1 {
    /// Feature or service is available now.
    Ready,
    /// Feature or service exists but is turned off.
    Disabled,
    /// Feature or service is not implemented on this route.
    Unavailable,
    /// Feature or service hit an error.
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
