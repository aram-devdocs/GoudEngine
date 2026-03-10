use crate::ffi::context::GoudContextId;
use crate::sdk::network_debug_overlay::NetworkOverlayMetrics;

use super::registry::with_registry;

/// Snapshot used by the native network overlay renderer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NetworkOverlaySnapshot {
    /// Active network handle selected for this context.
    pub handle: i64,
    /// Metrics rendered in the overlay.
    pub metrics: NetworkOverlayMetrics,
}

/// Sets/clears an explicit active overlay handle override for this context.
///
/// This is an internal seam used by the native debug overlay path in this
/// batch. Passing `None` clears the override and falls back to the default
/// context-associated handle.
pub(crate) fn network_overlay_set_active_handle_override(
    context_id: GoudContextId,
    handle: Option<i64>,
) -> bool {
    with_registry(|reg| {
        if let Some(handle) = handle {
            if !reg.instances.contains_key(&handle) {
                return Ok(false);
            }
        }
        reg.set_overlay_override_handle_for_context(context_id, handle);
        Ok(true)
    })
    .unwrap_or(false)
}

/// Returns the active handle for a context using override-first semantics.
pub(crate) fn network_overlay_handle_for_context(context_id: GoudContextId) -> Option<i64> {
    with_registry(|reg| Ok(reg.active_handle_for_context(context_id)))
        .ok()
        .flatten()
}

/// Returns active-handle stats for the network overlay in this context.
pub(crate) fn network_overlay_snapshot_for_context(
    context_id: GoudContextId,
) -> Option<NetworkOverlaySnapshot> {
    with_registry(|reg| {
        let handle = match reg.active_handle_for_context(context_id) {
            Some(handle) => handle,
            None => return Ok(None),
        };
        let instance = match reg.instances.get(&handle) {
            Some(instance) => instance,
            None => return Ok(None),
        };

        let stats = instance.provider.stats();
        Ok(Some(NetworkOverlaySnapshot {
            handle,
            metrics: NetworkOverlayMetrics {
                rtt_ms: stats.rtt_ms,
                send_bandwidth_bytes_per_sec: stats.send_bandwidth_bytes_per_sec,
                receive_bandwidth_bytes_per_sec: stats.receive_bandwidth_bytes_per_sec,
                packet_loss_percent: stats.packet_loss_percent,
                jitter_ms: stats.jitter_ms,
            },
        }))
    })
    .ok()
    .flatten()
}
