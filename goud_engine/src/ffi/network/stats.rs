use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};
use crate::core::providers::network_types::NetworkStats;
use crate::ffi::context::GoudContextId;

use super::registry::with_instance;

/// FFI-safe aggregate network statistics for a provider handle.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct FfiNetworkStats {
    /// Total bytes sent across all connections.
    pub bytes_sent: u64,
    /// Total bytes received across all connections.
    pub bytes_received: u64,
    /// Total packets sent across all connections.
    pub packets_sent: u64,
    /// Total packets received across all connections.
    pub packets_received: u64,
    /// Total packets lost across all connections.
    pub packets_lost: u64,
    /// Most recent RTT sample in milliseconds.
    pub rtt_ms: f32,
    /// Send bandwidth over the rolling 1-second window.
    pub send_bandwidth_bytes_per_sec: f32,
    /// Receive bandwidth over the rolling 1-second window.
    pub receive_bandwidth_bytes_per_sec: f32,
    /// Packet loss percentage over the rolling 1-second window.
    pub packet_loss_percent: f32,
    /// Rolling RTT jitter in milliseconds.
    pub jitter_ms: f32,
}

impl From<NetworkStats> for FfiNetworkStats {
    fn from(stats: NetworkStats) -> Self {
        Self {
            bytes_sent: stats.bytes_sent,
            bytes_received: stats.bytes_received,
            packets_sent: stats.packets_sent,
            packets_received: stats.packets_received,
            packets_lost: stats.packets_lost,
            rtt_ms: stats.rtt_ms,
            send_bandwidth_bytes_per_sec: stats.send_bandwidth_bytes_per_sec,
            receive_bandwidth_bytes_per_sec: stats.receive_bandwidth_bytes_per_sec,
            packet_loss_percent: stats.packet_loss_percent,
            jitter_ms: stats.jitter_ms,
        }
    }
}

/// Writes aggregate network statistics into caller-provided pointers.
/// Returns 0 on success, negative error code on failure.
///
/// # Safety
///
/// All four output pointers must point to valid `u64` values.
#[no_mangle]
pub unsafe extern "C" fn goud_network_get_stats(
    _context_id: GoudContextId,
    handle: i64,
    out_bytes_sent: *mut u64,
    out_bytes_recv: *mut u64,
    out_packets_sent: *mut u64,
    out_packets_recv: *mut u64,
) -> i32 {
    if out_bytes_sent.is_null()
        || out_bytes_recv.is_null()
        || out_packets_sent.is_null()
        || out_packets_recv.is_null()
    {
        set_last_error(GoudError::InvalidState(
            "One or more output pointers are null".to_string(),
        ));
        return ERR_INVALID_STATE;
    }

    let result = with_instance(handle, |inst| {
        let stats = inst.provider.stats();
        // SAFETY: Caller guarantees all output pointers are valid.
        *out_bytes_sent = stats.bytes_sent;
        *out_bytes_recv = stats.bytes_received;
        *out_packets_sent = stats.packets_sent;
        *out_packets_recv = stats.packets_received;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Writes aggregate network statistics into `out_stats`.
/// Returns 0 on success, negative error code on failure.
///
/// # Safety
///
/// `out_stats` must be non-null and point to writable storage for one
/// [`FfiNetworkStats`] value. Ownership is retained by the caller.
#[no_mangle]
pub unsafe extern "C" fn goud_network_get_stats_v2(
    _context_id: GoudContextId,
    handle: i64,
    out_stats: *mut FfiNetworkStats,
) -> i32 {
    if out_stats.is_null() {
        set_last_error(GoudError::InvalidState("out_stats is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_instance(handle, |inst| {
        let stats = FfiNetworkStats::from(inst.provider.stats());
        // SAFETY: Caller guarantees out_stats points to writable FfiNetworkStats storage.
        *out_stats = stats;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}
