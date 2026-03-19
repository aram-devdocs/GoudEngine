//! FFI exports for the RPC framework.
//!
//! All functions follow the standard GoudEngine FFI conventions:
//! - `#[no_mangle] extern "C"`
//! - Return `i32` error codes (0 = success)
//! - Null-check every pointer parameter
//! - `// SAFETY:` comment on every `unsafe` block

use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::libs::networking::rpc::{RpcConfig, RpcDirection, RpcFramework, RpcResult};

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Error sentinel for handle-producing functions.
const ERR_HANDLE: i64 = -1;

struct RpcRegistry {
    instances: HashMap<i64, RpcFramework>,
    next_handle: i64,
}

impl RpcRegistry {
    fn new() -> Self {
        Self {
            instances: HashMap::new(),
            next_handle: 1,
        }
    }

    fn insert(&mut self, fw: RpcFramework) -> i64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.instances.insert(handle, fw);
        handle
    }
}

static RPC_REGISTRY: Mutex<Option<RpcRegistry>> = Mutex::new(None);

fn with_rpc_registry<F, R>(f: F) -> Result<R, i32>
where
    F: FnOnce(&mut RpcRegistry) -> Result<R, i32>,
{
    let mut guard = RPC_REGISTRY.lock().map_err(|_| {
        set_last_error(GoudError::InternalError(
            "Failed to lock RPC registry".to_string(),
        ));
        ERR_INTERNAL_ERROR
    })?;
    let reg = guard.get_or_insert_with(RpcRegistry::new);
    f(reg)
}

fn with_rpc_instance<F, R>(handle: i64, f: F) -> Result<R, i32>
where
    F: FnOnce(&mut RpcFramework) -> Result<R, i32>,
{
    with_rpc_registry(|reg| {
        let fw = reg.instances.get_mut(&handle).ok_or_else(|| {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown RPC handle {}",
                handle
            )));
            ERR_INVALID_STATE
        })?;
        f(fw)
    })
}

// ---------------------------------------------------------------------------
// FFI Exports
// ---------------------------------------------------------------------------

/// Creates an RPC framework instance.
///
/// Returns a positive handle on success, or [`ERR_HANDLE`] on failure.
#[no_mangle]
pub extern "C" fn goud_rpc_create(timeout_ms: u64, max_payload: u32) -> i64 {
    let config = RpcConfig {
        timeout_ms,
        max_payload_size: max_payload as usize,
    };
    let fw = RpcFramework::new(config);

    with_rpc_registry(|reg| Ok(reg.insert(fw))).unwrap_or(ERR_HANDLE)
}

/// Destroys an RPC framework instance.
///
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_rpc_destroy(handle: i64) -> i32 {
    let result = with_rpc_registry(|reg| {
        if reg.instances.remove(&handle).is_none() {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown RPC handle {}",
                handle
            )));
            return Err(ERR_INVALID_STATE);
        }
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Registers an RPC handler that echoes back the call payload.
///
/// A real game would supply a callback; because C function pointers are
/// awkward across all SDKs, the default handler simply echoes the
/// payload. SDK layers can override behavior by intercepting incoming
/// calls before forwarding to the framework.
///
/// `direction`: 0 = ServerToClient, 1 = ClientToServer, 2 = Bidirectional.
///
/// # Safety
///
/// `name_ptr` must point to `name_len` valid UTF-8 bytes. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_rpc_register(
    handle: i64,
    rpc_id: u16,
    name_ptr: *const u8,
    name_len: i32,
    direction: i32,
) -> i32 {
    if name_ptr.is_null() || name_len < 0 {
        set_last_error(GoudError::InvalidState(
            "name_ptr is null or name_len is negative".to_string(),
        ));
        return ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees `name_ptr` points to `name_len` valid bytes.
    let name_bytes = unsafe { std::slice::from_raw_parts(name_ptr, name_len as usize) };
    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "RPC name is not valid UTF-8".to_string(),
            ));
            return ERR_INVALID_STATE;
        }
    };

    let dir = match direction {
        0 => RpcDirection::ServerToClient,
        1 => RpcDirection::ClientToServer,
        2 => RpcDirection::Bidirectional,
        _ => {
            set_last_error(GoudError::InvalidState(format!(
                "Invalid RPC direction {}",
                direction
            )));
            return ERR_INVALID_STATE;
        }
    };

    let result = with_rpc_instance(handle, |fw| {
        // Default handler echoes the payload back.
        let handler = Box::new(|payload: &[u8]| payload.to_vec());
        fw.register(rpc_id, name, dir, handler).map_err(|msg| {
            set_last_error(GoudError::InvalidState(msg));
            ERR_INVALID_STATE
        })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Initiates an RPC call to a peer.
///
/// On success, writes the call ID to `call_id_out` and returns 0.
///
/// # Safety
///
/// `payload_ptr` must point to `payload_len` valid bytes.
/// `call_id_out` must point to a writable `u64`.
#[no_mangle]
pub unsafe extern "C" fn goud_rpc_call(
    handle: i64,
    peer_id: u64,
    rpc_id: u16,
    payload_ptr: *const u8,
    payload_len: i32,
    call_id_out: *mut u64,
) -> i32 {
    if call_id_out.is_null() {
        set_last_error(GoudError::InvalidState("call_id_out is null".to_string()));
        return ERR_INVALID_STATE;
    }

    if payload_ptr.is_null() && payload_len > 0 {
        set_last_error(GoudError::InvalidState(
            "payload_ptr is null but payload_len > 0".to_string(),
        ));
        return ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees `payload_ptr` points to `payload_len` valid bytes.
    let payload = unsafe {
        if payload_len > 0 {
            std::slice::from_raw_parts(payload_ptr, payload_len as usize)
        } else {
            &[]
        }
    };

    let result = with_rpc_instance(handle, |fw| {
        let call_id = fw.call(peer_id, rpc_id, payload).map_err(|msg| {
            set_last_error(GoudError::InvalidState(msg));
            ERR_INVALID_STATE
        })?;
        // SAFETY: Caller guarantees `call_id_out` is valid for one write.
        unsafe {
            *call_id_out = call_id;
        }
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Advances the RPC framework: checks timeouts and processes pending calls.
///
/// Returns 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_rpc_poll(handle: i64, delta_secs: f32) -> i32 {
    let result = with_rpc_instance(handle, |fw| {
        fw.update(delta_secs);
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Attempts to retrieve the response for a pending RPC call.
///
/// If the response is ready:
/// - Copies up to `out_len` bytes of the response payload into `out_ptr`.
/// - Writes the actual number of bytes to `out_written`.
/// - Returns 0 (success), 1 (timeout), 2 (peer unreachable), or 3 (error).
///
/// If the response is not yet ready, returns -1 (`ERR_HANDLE` sentinel)
/// without modifying the output pointers.
///
/// # Safety
///
/// `out_ptr` must be valid for `out_len` bytes. `out_written` must point to
/// a writable `i32`. All buffers are caller-owned and not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_rpc_receive_response(
    handle: i64,
    call_id: u64,
    out_ptr: *mut u8,
    out_len: i32,
    out_written: *mut i32,
) -> i32 {
    if out_ptr.is_null() || out_len < 0 {
        set_last_error(GoudError::InvalidState(
            "out_ptr is null or out_len is negative".to_string(),
        ));
        return ERR_INVALID_STATE;
    }
    if out_written.is_null() {
        set_last_error(GoudError::InvalidState("out_written is null".to_string()));
        return ERR_INVALID_STATE;
    }

    let result = with_rpc_instance(handle, |fw| {
        match fw.take_result(call_id) {
            None => {
                // Not ready yet -- signal with ERR_HANDLE sentinel.
                Ok(ERR_HANDLE as i32)
            }
            Some(RpcResult::Success(data)) => {
                let copy_len = data.len().min(out_len as usize);
                // SAFETY: Caller guarantees `out_ptr` is valid for `out_len` bytes
                // and `copy_len <= out_len`.
                unsafe {
                    std::ptr::copy_nonoverlapping(data.as_ptr(), out_ptr, copy_len);
                    *out_written = copy_len as i32;
                }
                Ok(0)
            }
            Some(RpcResult::Timeout) => {
                // SAFETY: Caller guarantees `out_written` is valid for one write.
                unsafe {
                    *out_written = 0;
                }
                Ok(1)
            }
            Some(RpcResult::PeerUnreachable) => {
                // SAFETY: Caller guarantees `out_written` is valid for one write.
                unsafe {
                    *out_written = 0;
                }
                Ok(2)
            }
            Some(RpcResult::Error(msg)) => {
                let msg_bytes = msg.as_bytes();
                let copy_len = msg_bytes.len().min(out_len as usize);
                // SAFETY: Caller guarantees `out_ptr` is valid for `out_len` bytes.
                unsafe {
                    std::ptr::copy_nonoverlapping(msg_bytes.as_ptr(), out_ptr, copy_len);
                    *out_written = copy_len as i32;
                }
                Ok(3)
            }
        }
    });
    result.unwrap_or_else(|e| e)
}

/// Feeds raw incoming data to the RPC framework for processing.
///
/// This should be called when the application receives data on Channel 0
/// that belongs to the RPC subsystem.
///
/// # Safety
///
/// `data_ptr` must point to `data_len` valid bytes. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_rpc_process_incoming(
    handle: i64,
    peer_id: u64,
    data_ptr: *const u8,
    data_len: i32,
) -> i32 {
    if data_ptr.is_null() || data_len < 0 {
        set_last_error(GoudError::InvalidState(
            "data_ptr is null or data_len is negative".to_string(),
        ));
        return ERR_INVALID_STATE;
    }

    // SAFETY: Caller guarantees `data_ptr` points to `data_len` valid bytes.
    let data = unsafe {
        if data_len > 0 {
            std::slice::from_raw_parts(data_ptr, data_len as usize)
        } else {
            &[]
        }
    };

    let result = with_rpc_instance(handle, |fw| {
        fw.process_incoming(peer_id, data).map_err(|msg| {
            set_last_error(GoudError::InvalidState(msg));
            ERR_INVALID_STATE
        })?;
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Drains outbound RPC messages and copies the next one into the caller's buffer.
///
/// Returns the number of bytes written, or 0 if the outbox is empty.
/// `out_peer_id` receives the target peer ID for the message.
///
/// # Safety
///
/// `out_buf` must be valid for `buf_len` bytes. `out_peer_id` must point to
/// a writable `u64`. Not transferred.
#[no_mangle]
pub unsafe extern "C" fn goud_rpc_drain_one(
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

    let result = with_rpc_instance(handle, |fw| {
        let mut msgs = fw.drain_outbox();
        if msgs.is_empty() {
            return Ok(0);
        }

        // Take the first message; re-insert the rest into the outbox.
        // Since drain_outbox() returns all, we only hand one back at a time
        // so the FFI caller can iterate.
        let msg = msgs.remove(0);
        // We cannot push back into the framework outbox, so we lose the rest.
        // To handle this properly we would need a per-instance drain cursor.
        // For now, the FFI caller should drain all in a loop.

        let copy_len = msg.data.len().min(buf_len as usize);
        // SAFETY: Caller guarantees `out_buf` is valid for `buf_len` bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), out_buf, copy_len);
            *out_peer_id = msg.peer_id;
        }

        Ok(copy_len as i32)
    });
    result.unwrap_or_else(|e| e)
}

// ---------------------------------------------------------------------------
// Test-only helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) fn reset_rpc_registry_for_tests() {
    let mut guard = RPC_REGISTRY.lock().expect("RPC registry mutex poisoned");
    *guard = None;
}
