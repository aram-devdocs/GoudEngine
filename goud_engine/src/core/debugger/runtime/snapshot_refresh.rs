use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::core::debugger::RuntimeRouteId;

type SnapshotRefreshHook = Arc<dyn Fn(&RuntimeRouteId) + Send + Sync>;

static SNAPSHOT_REFRESH_HOOKS: OnceLock<Mutex<HashMap<u128, SnapshotRefreshHook>>> =
    OnceLock::new();

fn hook_registry() -> &'static Mutex<HashMap<u128, SnapshotRefreshHook>> {
    SNAPSHOT_REFRESH_HOOKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn packed_route_identity(route_id: &RuntimeRouteId) -> u128 {
    ((route_id.process_nonce as u128) << 64) | route_id.context_id as u128
}

/// Registers or replaces a snapshot refresh hook for a route.
pub fn register_snapshot_refresh_hook_for_route(
    route_id: RuntimeRouteId,
    hook: impl Fn(&RuntimeRouteId) + Send + Sync + 'static,
) {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(packed_route_identity(&route_id), Arc::new(hook));
}

/// Unregisters the snapshot refresh hook for a route.
pub fn unregister_snapshot_refresh_hook_for_route(route_id: &RuntimeRouteId) {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&packed_route_identity(route_id));
}

/// Calls the registered refresh hook, if any, to populate entity/scene data before snapshot read.
pub(crate) fn refresh_snapshot_for_route(route_id: &RuntimeRouteId) {
    let hook = hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&packed_route_identity(route_id))
        .cloned();
    if let Some(hook) = hook {
        hook(route_id);
    }
}

#[cfg(test)]
pub(super) fn clear_snapshot_refresh_hooks_for_tests() {
    hook_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}
