use std::collections::VecDeque;

use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};
use crate::core::providers::network_types::{Channel, ConnectionId, HostConfig, NetworkEvent};
use crate::ffi::context::GoudContextId;

use super::provider_factory::create_provider;
use super::registry::{with_instance, with_registry, NetInstance, ERR_HANDLE};

/// Creates a network host listening on the given port.
///
/// Returns a positive network handle on success, or a negative error code.
/// `protocol`: 0 = UDP, 1 = WebSocket, 2 = TCP.
#[no_mangle]
pub extern "C" fn goud_network_host(context_id: GoudContextId, protocol: i32, port: u16) -> i64 {
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

fn parse_connect_address(addr_ptr: *const u8, addr_len: i32, port: u16) -> Result<String, i32> {
    if addr_ptr.is_null() || addr_len <= 0 {
        set_last_error(GoudError::InvalidState(
            "addr_ptr is null or empty".to_string(),
        ));
        return Err(ERR_INVALID_STATE);
    }

    // SAFETY: The caller guarantees that `addr_ptr` points to `addr_len` readable bytes.
    let addr_bytes = unsafe { std::slice::from_raw_parts(addr_ptr, addr_len as usize) };
    let addr_str = match std::str::from_utf8(addr_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "address is not valid UTF-8".to_string(),
            ));
            return Err(ERR_INVALID_STATE);
        }
    };

    Ok(if addr_str.contains(':') {
        addr_str.to_string()
    } else {
        format!("{}:{}", addr_str, port)
    })
}

fn connect_instance(
    context_id: GoudContextId,
    protocol: i32,
    full_addr: &str,
) -> Result<(i64, u64), i32> {
    let mut provider = create_provider(protocol)?;
    let peer_id = provider.connect(full_addr).map_err(|e| {
        let code = e.error_code();
        set_last_error(e);
        code
    })?;

    let inst = NetInstance {
        provider,
        recv_queue: VecDeque::new(),
    };

    with_registry(|reg| {
        let handle = reg.insert(inst);
        reg.set_default_handle_for_context(context_id, handle);
        Ok((handle, peer_id.0))
    })
}

/// Connects to a remote host. Returns a positive handle or negative error.
///
/// # Safety
///
/// `addr_ptr` must point to valid UTF-8 of `addr_len` bytes. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_network_connect(
    context_id: GoudContextId,
    protocol: i32,
    addr_ptr: *const u8,
    addr_len: i32,
    port: u16,
) -> i64 {
    let full_addr = match parse_connect_address(addr_ptr, addr_len, port) {
        Ok(addr) => addr,
        Err(_) => return ERR_HANDLE,
    };

    match connect_instance(context_id, protocol, &full_addr) {
        Ok((handle, _peer_id)) => handle,
        Err(_) => ERR_HANDLE,
    }
}

/// Connects to a remote host and preserves the provider-assigned peer ID.
/// Returns 0 on success, or a negative error code on failure.
///
/// # Safety
///
/// `addr_ptr` must point to valid UTF-8 of `addr_len` bytes. `out_handle` and
/// `out_peer_id` must point to writable storage retained by the caller.
#[no_mangle]
pub unsafe extern "C" fn goud_network_connect_with_peer(
    context_id: GoudContextId,
    protocol: i32,
    addr_ptr: *const u8,
    addr_len: i32,
    port: u16,
    out_handle: *mut i64,
    out_peer_id: *mut u64,
) -> i32 {
    if out_handle.is_null() {
        set_last_error(GoudError::InvalidState("out_handle is null".to_string()));
        return ERR_INVALID_STATE;
    }
    if out_peer_id.is_null() {
        set_last_error(GoudError::InvalidState("out_peer_id is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let full_addr = match parse_connect_address(addr_ptr, addr_len, port) {
        Ok(addr) => addr,
        Err(code) => return code,
    };

    let (handle, peer_id) = match connect_instance(context_id, protocol, &full_addr) {
        Ok(result) => result,
        Err(code) => return code,
    };

    // SAFETY: The caller guarantees that both output pointers are valid for one write.
    unsafe {
        *out_handle = handle;
        *out_peer_id = peer_id;
    }
    0
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

    let data = {
        // SAFETY: Caller guarantees `data_ptr` is valid for `data_len` bytes.
        unsafe {
            if data_len > 0 {
                std::slice::from_raw_parts(data_ptr, data_len as usize)
            } else {
                &[]
            }
        }
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
        {
            // SAFETY: Caller guarantees `out_buf` is valid for `buf_len` bytes,
            // and `copy_len <= buf_len`.
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), out_buf, copy_len);
            }
        }
        {
            // SAFETY: Caller guarantees `out_peer_id` points to a valid writable `u64`.
            unsafe {
                *out_peer_id = peer;
            }
        }

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
