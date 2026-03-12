#![allow(missing_docs)]

#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

mod config;
mod runtime;
mod snapshot;
mod types;

pub use config::{ContextConfig, DebuggerConfig};
pub use runtime::{
    active_route_count, begin_frame, current_manifest, end_frame, register_context,
    route_for_context, snapshot_for_route, unregister_context, with_snapshot_mut,
};
pub use snapshot::{
    default_capabilities, default_services, DebuggerSnapshotV1, DebuggerStateV1,
    DiagnosticsStateV1, EntityStateV1, FrameStateV1, LocalEndpointV1, MemoryStatsV1,
    NetworkStatsV1, RenderStatsV1, RouteSummaryV1, RuntimeManifestV1, SceneStateV1,
    SelectionStateV1, ServiceHealthV1, SnapshotStatsV1,
};
pub use types::{
    CapabilityStateV1, RuntimeRouteId, RuntimeSurfaceKind, REQUIRED_SERVICE_OWNERS,
    ROUTE_CAPABILITY_KEYS,
};

#[cfg(test)]
pub(crate) use runtime::reset_for_tests;

#[cfg(test)]
pub(crate) fn test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

#[cfg(test)]
mod tests;
