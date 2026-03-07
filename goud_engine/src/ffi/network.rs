//! Networking FFI functions.
//!
//! Provides C-compatible functions for creating and managing network
//! connections using UDP or WebSocket transports. Network provider
//! instances are stored in a global registry keyed by opaque handles.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{Channel, ConnectionId, HostConfig, NetworkEvent};
#[cfg(any(feature = "net-udp", feature = "net-ws"))]
use crate::core::providers::ProviderLifecycle;
use crate::ffi::context::GoudContextId;
#[cfg(feature = "net-udp")]
use crate::libs::providers::impls::udp_network::UdpNetProvider;
#[cfg(feature = "net-ws")]
use crate::libs::providers::impls::ws_network::WsNetProvider;

// ============================================================================
// Network Handle Registry
// ============================================================================

/// Protocol selector passed from FFI callers.
const PROTOCOL_UDP: i32 = 0;
const PROTOCOL_WS: i32 = 1;

/// Error sentinel returned for handle-producing functions.
const ERR_HANDLE: i64 = -1;

/// Internal state for a network instance.
struct NetInstance {
    provider: Box<dyn NetworkProvider>,
    /// Buffered received-data events from the last `poll`.
    recv_queue: VecDeque<(u64, Vec<u8>)>, // (peer_id / conn_id, data)
}

// SAFETY: NetInstance is only accessed through the global Mutex, so all
// access is serialized. The trait object inside is not Sync on its own
// (WsNetProvider uses mpsc channels), but the Mutex serializes all access.
unsafe impl Send for NetInstance {}

static NET_REGISTRY: Mutex<Option<NetRegistryInner>> = Mutex::new(None);

struct NetRegistryInner {
    instances: HashMap<i64, NetInstance>,
    next_handle: i64,
}

impl NetRegistryInner {
    fn new() -> Self {
        Self {
            instances: HashMap::new(),
            next_handle: 1,
        }
    }

    fn insert(&mut self, instance: NetInstance) -> i64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.instances.insert(handle, instance);
        handle
    }
}

fn with_registry<F, R>(f: F) -> Result<R, i32>
where
    F: FnOnce(&mut NetRegistryInner) -> Result<R, i32>,
{
    let mut guard = NET_REGISTRY.lock().map_err(|_| {
        set_last_error(GoudError::InternalError(
            "Failed to lock network registry".to_string(),
        ));
        ERR_INTERNAL_ERROR
    })?;
    let reg = guard.get_or_insert_with(NetRegistryInner::new);
    f(reg)
}

fn with_instance<F, R>(handle: i64, f: F) -> Result<R, i32>
where
    F: FnOnce(&mut NetInstance) -> Result<R, i32>,
{
    with_registry(|reg| {
        let inst = reg.instances.get_mut(&handle).ok_or_else(|| {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown network handle {}",
                handle
            )));
            ERR_INVALID_STATE
        })?;
        f(inst)
    })
}

fn create_provider(protocol: i32) -> Result<Box<dyn NetworkProvider>, i32> {
    match protocol {
        PROTOCOL_UDP => create_udp_provider(),
        PROTOCOL_WS => create_ws_provider(),
        _ => {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown protocol: {}",
                protocol
            )));
            Err(ERR_INVALID_STATE)
        }
    }
}

#[cfg(feature = "net-udp")]
fn create_udp_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    let mut p = UdpNetProvider::new();
    p.init().map_err(|e| {
        let code = e.error_code();
        set_last_error(e);
        code
    })?;
    Ok(Box::new(p))
}

#[cfg(not(feature = "net-udp"))]
fn create_udp_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    set_last_error(GoudError::InvalidState(
        "UDP networking not available (net-udp feature disabled)".to_string(),
    ));
    Err(ERR_INVALID_STATE)
}

#[cfg(feature = "net-ws")]
fn create_ws_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    let mut p = WsNetProvider::new();
    p.init().map_err(|e| {
        let code = e.error_code();
        set_last_error(e);
        code
    })?;
    Ok(Box::new(p))
}

#[cfg(not(feature = "net-ws"))]
fn create_ws_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    set_last_error(GoudError::InvalidState(
        "WebSocket networking not available (net-ws feature disabled)".to_string(),
    ));
    Err(ERR_INVALID_STATE)
}

// ============================================================================
// Host / Connect / Disconnect
// ============================================================================

/// Creates a network host listening on the given port.
///
/// Returns a positive network handle on success, or a negative error code.
/// `protocol`: 0 = UDP, 1 = WebSocket.
#[no_mangle]
pub extern "C" fn goud_network_host(_context_id: GoudContextId, protocol: i32, port: u16) -> i64 {
    let provider = match create_provider(protocol) {
        Ok(p) => p,
        Err(_) => return ERR_HANDLE,
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

    with_registry(|reg| Ok(reg.insert(inst))).unwrap_or(ERR_HANDLE)
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
        Err(_) => return ERR_HANDLE,
    };

    if let Err(e) = provider.connect(&full_addr) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    let inst = NetInstance {
        provider,
        recv_queue: VecDeque::new(),
    };

    with_registry(|reg| Ok(reg.insert(inst))).unwrap_or(ERR_HANDLE)
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
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

// ============================================================================
// Send / Receive / Poll
// ============================================================================

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

// ============================================================================
// Statistics / Queries
// ============================================================================

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

/// Returns the number of active connections, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_network_peer_count(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| Ok(inst.provider.connections().len() as i32));
    result.unwrap_or_else(|e| e)
}
