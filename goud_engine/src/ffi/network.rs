//! Networking FFI functions.
//!
//! Provides C-compatible functions for creating and managing network
//! connections using UDP, WebSocket, or TCP transports. Network provider
//! instances are stored in a global registry keyed by opaque handles.

use std::collections::VecDeque;

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::core::providers::network_types::{
    Channel, ConnectionId, HostConfig, NetworkEvent, NetworkSimulationConfig,
};
use crate::ffi::context::GoudContextId;
use crate::ffi::window::with_window_state;

mod overlay;
mod provider_factory;
pub(crate) mod registry;

#[allow(unused_imports)]
pub(crate) use overlay::NetworkOverlaySnapshot;
pub(crate) use overlay::{
    network_overlay_handle_for_context, network_overlay_set_active_handle_override,
    network_overlay_snapshot_for_context,
};
use provider_factory::create_provider;
use registry::{with_instance, with_registry, NetInstance, ERR_HANDLE};

#[cfg(test)]
#[path = "network/tests.rs"]
mod tests;

/// Creates a network host listening on the given port.
///
/// Returns a positive network handle on success, or a negative error code.
/// `protocol`: 0 = UDP, 1 = WebSocket, 2 = TCP.
#[no_mangle]
pub extern "C" fn goud_network_host(_context_id: GoudContextId, protocol: i32, port: u16) -> i64 {
    let context_id = _context_id;
    let provider = match create_provider(protocol) {
        Ok(p) => p,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to create network provider".to_string(),
            ));
            return ERR_HANDLE;
        }
    };

    let mut inst = NetInstance {
        provider,
        recv_queue: VecDeque::new(),
    };

    let config = HostConfig {
        bind_address: "0.0.0.0".to_string(),
        port,
        max_connections: 32,
        tls_cert_path: None,
        tls_key_path: None,
    };

    if let Err(e) = inst.provider.host(&config) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    with_registry(|reg| {
        let handle = reg.insert(inst);
        reg.set_default_handle_for_context(context_id, handle);
        Ok(handle)
    })
    .unwrap_or(ERR_HANDLE)
}

/// Connects to a remote host. Returns a positive handle or negative error.
///
/// # Safety
///
/// `addr_ptr` must point to valid UTF-8 of `addr_len` bytes. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_network_connect(
    _context_id: GoudContextId,
    protocol: i32,
    addr_ptr: *const u8,
    addr_len: i32,
    port: u16,
) -> i64 {
    let context_id = _context_id;
    if addr_ptr.is_null() || addr_len <= 0 {
        set_last_error(GoudError::InvalidState(
            "addr_ptr is null or empty".to_string(),
        ));
        return ERR_HANDLE;
    }

    // SAFETY: Caller guarantees addr_ptr is valid for addr_len bytes.
    let addr_bytes = std::slice::from_raw_parts(addr_ptr, addr_len as usize);
    let addr_str = match std::str::from_utf8(addr_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "address is not valid UTF-8".to_string(),
            ));
            return ERR_HANDLE;
        }
    };

    // Build the full address. If the caller provided just an IP, append the port.
    let full_addr = if addr_str.contains(':') {
        addr_str.to_string()
    } else {
        format!("{}:{}", addr_str, port)
    };

    let mut provider = match create_provider(protocol) {
        Ok(p) => p,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to create network provider".to_string(),
            ));
            return ERR_HANDLE;
        }
    };

    if let Err(e) = provider.connect(&full_addr) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    let inst = NetInstance {
        provider,
        recv_queue: VecDeque::new(),
    };

    with_registry(|reg| {
        let handle = reg.insert(inst);
        reg.set_default_handle_for_context(context_id, handle);
        Ok(handle)
    })
    .unwrap_or(ERR_HANDLE)
}

/// Disconnects and destroys a network instance.
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_network_disconnect(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_registry(|reg| {
        let mut inst = reg.instances.remove(&handle).ok_or_else(|| {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown network handle {}",
                handle
            )));
            ERR_INVALID_STATE
        })?;
        let _ = inst.provider.disconnect_all();
        inst.provider.shutdown();
        reg.clear_associations_for_handle(handle);
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Sends data to a specific peer on the given channel.
/// Returns 0 on success, negative error code on failure.
///
/// # Safety
///
/// `data_ptr` must point to `data_len` valid bytes. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_network_send(
    _context_id: GoudContextId,
    handle: i64,
    peer_id: u64,
    data_ptr: *const u8,
    data_len: i32,
    channel: u8,
) -> i32 {
    if data_ptr.is_null() || data_len < 0 {
        set_last_error(GoudError::InvalidState(
            "data_ptr is null or data_len is negative".to_string(),
        ));
        return ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees data_ptr is valid for data_len bytes.
    let data = if data_len > 0 {
        std::slice::from_raw_parts(data_ptr, data_len as usize)
    } else {
        &[]
    };

    let result = with_instance(handle, |inst| {
        inst.provider
            .send(ConnectionId(peer_id), Channel(channel), data)
            .map_err(|e| {
                let code = e.error_code();
                set_last_error(e);
                code
            })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Receives the next buffered message (queued by `goud_network_poll`).
/// Returns bytes written to `out_buf` (0 if empty), or negative error.
///
/// # Safety
///
/// `out_buf` must be valid for `buf_len` bytes. `out_peer_id` must
/// point to a valid `u64`. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_network_receive(
    _context_id: GoudContextId,
    handle: i64,
    out_buf: *mut u8,
    buf_len: i32,
    out_peer_id: *mut u64,
) -> i32 {
    if out_buf.is_null() || buf_len <= 0 {
        set_last_error(GoudError::InvalidState(
            "out_buf is null or buf_len <= 0".to_string(),
        ));
        return ERR_INVALID_STATE;
    }
    if out_peer_id.is_null() {
        set_last_error(GoudError::InvalidState("out_peer_id is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_instance(handle, |inst| {
        if inst.recv_queue.is_empty() {
            return Ok(0);
        }
        let (peer, data) = inst.recv_queue.pop_front().unwrap();

        let copy_len = data.len().min(buf_len as usize);
        // SAFETY: Caller guarantees out_buf is valid for buf_len bytes,
        // and copy_len <= buf_len.
        std::ptr::copy_nonoverlapping(data.as_ptr(), out_buf, copy_len);
        // SAFETY: Caller guarantees out_peer_id points to a valid u64.
        *out_peer_id = peer;

        Ok(copy_len as i32)
    });
    result.unwrap_or_else(|e| e)
}

/// Polls the network instance for new events, buffering received data
/// for retrieval via `goud_network_receive`.
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_network_poll(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| {
        inst.provider.update(0.0).map_err(|e| {
            let code = e.error_code();
            set_last_error(e);
            code
        })?;

        let events = inst.provider.drain_events();
        for event in events {
            if let NetworkEvent::Received { conn, data, .. } = event {
                inst.recv_queue.push_back((conn.0, data));
            }
        }
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

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

impl From<crate::core::providers::network_types::NetworkStats> for FfiNetworkStats {
    fn from(stats: crate::core::providers::network_types::NetworkStats) -> Self {
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

/// Returns the number of active connections, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_network_peer_count(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| Ok(inst.provider.connections().len() as i32));
    result.unwrap_or_else(|e| e)
}

/// Sets an explicit overlay-handle override for this context.
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_network_set_overlay_handle(context_id: GoudContextId, handle: i64) -> i32 {
    match with_registry(|reg| Ok(reg.instances.contains_key(&handle))) {
        Ok(true) => {
            if network_overlay_set_active_handle_override(context_id, Some(handle)) {
                let _ = with_window_state(context_id, |state| {
                    state.network_overlay.set_active_handle(Some(handle));
                });
                0
            } else {
                ERR_INTERNAL_ERROR
            }
        }
        Ok(false) => {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown network handle {}",
                handle
            )));
            ERR_INVALID_STATE
        }
        Err(code) => code,
    }
}

/// Clears any explicit overlay-handle override for this context.
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_network_clear_overlay_handle(context_id: GoudContextId) -> i32 {
    if network_overlay_set_active_handle_override(context_id, None) {
        let _ = with_window_state(context_id, |state| {
            state.network_overlay.set_active_handle(None);
        });
        0
    } else {
        ERR_INTERNAL_ERROR
    }
}

#[cfg_attr(any(debug_assertions, test), allow(dead_code))]
fn simulation_controls_unavailable() -> i32 {
    set_last_error(GoudError::InvalidState(
        "Network simulation controls are only available in debug/test builds".to_string(),
    ));
    ERR_INVALID_STATE
}

/// Applies a debug-only network simulation config to the provider handle.
/// Returns 0 on success, negative error code on failure.
#[cfg(any(debug_assertions, test))]
#[no_mangle]
pub extern "C" fn goud_network_set_simulation(
    _context_id: GoudContextId,
    handle: i64,
    config: NetworkSimulationConfig,
) -> i32 {
    if let Err(message) = config.validate() {
        set_last_error(GoudError::InvalidState(message));
        return ERR_INVALID_STATE;
    }

    let result = with_instance(handle, |inst| {
        inst.provider.set_simulation_config(config).map_err(|e| {
            let code = e.error_code();
            set_last_error(e);
            code
        })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Applies a debug-only network simulation config to the provider handle.
/// Returns `ERR_INVALID_STATE` in release builds because simulation hooks are
/// not compiled into release networking providers.
#[cfg(not(any(debug_assertions, test)))]
#[no_mangle]
pub extern "C" fn goud_network_set_simulation(
    _context_id: GoudContextId,
    handle: i64,
    config: NetworkSimulationConfig,
) -> i32 {
    let _ = (handle, config);
    simulation_controls_unavailable()
}

/// Clears any debug-only network simulation config from the provider handle.
/// Returns 0 on success, negative error code on failure.
#[cfg(any(debug_assertions, test))]
#[no_mangle]
pub extern "C" fn goud_network_clear_simulation(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| {
        inst.provider.clear_simulation_config().map_err(|e| {
            let code = e.error_code();
            set_last_error(e);
            code
        })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Clears any debug-only network simulation config from the provider handle.
/// Returns `ERR_INVALID_STATE` in release builds because simulation hooks are
/// not compiled into release networking providers.
#[cfg(not(any(debug_assertions, test)))]
#[no_mangle]
pub extern "C" fn goud_network_clear_simulation(_context_id: GoudContextId, handle: i64) -> i32 {
    let _ = handle;
    simulation_controls_unavailable()
}
