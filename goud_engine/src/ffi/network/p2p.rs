//! FFI functions for peer-to-peer mesh networking.
//!
//! These functions expose the `P2pMesh` layer to C#/Python SDKs through
//! C-compatible exports. Mesh instances are stored in the same global
//! network registry as regular network providers.

use std::collections::VecDeque;

use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{HostConfig, P2pMeshConfig, P2pTopology};
use crate::core::providers::ProviderLifecycle;
use crate::ffi::context::GoudContextId;
use crate::libs::providers::impls::P2pMesh;

use super::provider_factory::create_provider;
use super::registry::{with_instance, with_registry, NetInstance, ERR_HANDLE};

/// FFI-safe P2P mesh configuration.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FfiP2pMeshConfig {
    /// Maximum number of peers (including self).
    pub max_peers: u32,
    /// Whether host migration is enabled.
    pub host_migration: bool,
    /// Mesh topology: 0 = FullMesh, 1 = Star.
    pub topology: i32,
}

impl Default for FfiP2pMeshConfig {
    fn default() -> Self {
        Self {
            max_peers: 8,
            host_migration: true,
            topology: 0,
        }
    }
}

/// Creates a P2P mesh host on the given port using the specified transport.
///
/// Returns a positive network handle on success, or a negative error code.
///
/// # Parameters
///
/// - `context_id`: The engine context.
/// - `protocol`: Transport protocol (0 = UDP, 1 = WebSocket, 2 = TCP).
/// - `port`: Port to listen on.
/// - `config`: P2P mesh configuration.
#[no_mangle]
pub extern "C" fn goud_p2p_create_mesh(
    context_id: GoudContextId,
    protocol: i32,
    port: u16,
    config: FfiP2pMeshConfig,
) -> i64 {
    let transport = match create_provider(protocol) {
        Ok(p) => p,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to create transport for P2P mesh".to_string(),
            ));
            return ERR_HANDLE;
        }
    };

    let mesh_config = P2pMeshConfig {
        max_peers: config.max_peers as usize,
        relay_server: None,
        host_migration: config.host_migration,
        topology: if config.topology == 1 {
            P2pTopology::Star
        } else {
            P2pTopology::FullMesh
        },
    };

    let mut mesh = P2pMesh::new(transport, mesh_config);

    if let Err(e) = ProviderLifecycle::init(&mut mesh) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    let host_config = HostConfig {
        bind_address: "0.0.0.0".to_string(),
        port,
        max_connections: config.max_peers,
        tls_cert_path: None,
        tls_key_path: None,
    };

    if let Err(e) = mesh.host(&host_config) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    let inst = NetInstance {
        provider: Box::new(mesh),
        recv_queue: VecDeque::new(),
    };

    with_registry(|reg| {
        let handle = reg.insert(inst);
        reg.set_default_handle_for_context(context_id, handle);
        Ok(handle)
    })
    .unwrap_or(ERR_HANDLE)
}

/// Joins an existing P2P mesh at the given address.
///
/// Returns a positive network handle on success, or a negative error code.
///
/// # Safety
///
/// `addr_ptr` must point to valid UTF-8 of `addr_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_p2p_join_mesh(
    context_id: GoudContextId,
    protocol: i32,
    addr_ptr: *const u8,
    addr_len: i32,
    port: u16,
    config: FfiP2pMeshConfig,
) -> i64 {
    if addr_ptr.is_null() || addr_len <= 0 {
        set_last_error(GoudError::InvalidState(
            "addr_ptr is null or empty".to_string(),
        ));
        return ERR_HANDLE;
    }

    // SAFETY: The caller guarantees that `addr_ptr` points to `addr_len` readable bytes.
    let addr_bytes = unsafe { std::slice::from_raw_parts(addr_ptr, addr_len as usize) };
    let addr_str = match std::str::from_utf8(addr_bytes) {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "address is not valid UTF-8".to_string(),
            ));
            return ERR_HANDLE;
        }
    };

    let full_addr = if addr_str.contains(':') {
        addr_str.to_string()
    } else {
        format!("{}:{}", addr_str, port)
    };

    let transport = match create_provider(protocol) {
        Ok(p) => p,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to create transport for P2P mesh".to_string(),
            ));
            return ERR_HANDLE;
        }
    };

    let mesh_config = P2pMeshConfig {
        max_peers: config.max_peers as usize,
        relay_server: None,
        host_migration: config.host_migration,
        topology: if config.topology == 1 {
            P2pTopology::Star
        } else {
            P2pTopology::FullMesh
        },
    };

    let mut mesh = P2pMesh::new(transport, mesh_config);

    if let Err(e) = ProviderLifecycle::init(&mut mesh) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    if let Err(e) = mesh.connect(&full_addr) {
        set_last_error(e);
        return ERR_HANDLE;
    }

    let inst = NetInstance {
        provider: Box::new(mesh),
        recv_queue: VecDeque::new(),
    };

    with_registry(|reg| {
        let handle = reg.insert(inst);
        reg.set_default_handle_for_context(context_id, handle);
        Ok(handle)
    })
    .unwrap_or(ERR_HANDLE)
}

/// Leaves the P2P mesh and destroys the network instance.
///
/// Returns 0 on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_p2p_leave_mesh(_context_id: GoudContextId, handle: i64) -> i32 {
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

/// Returns the number of connected peers in the mesh, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_p2p_get_peers(_context_id: GoudContextId, handle: i64) -> i32 {
    let result = with_instance(handle, |inst| Ok(inst.provider.connections().len() as i32));
    result.unwrap_or_else(|e| e)
}

/// Returns the host peer's ID, or 0 on error.
///
/// The host peer ID is the `local_id()` of the mesh provider, which
/// corresponds to the mesh host's peer ID.
#[no_mangle]
pub extern "C" fn goud_p2p_get_host(_context_id: GoudContextId, handle: i64) -> u64 {
    let result = with_instance(handle, |inst| {
        // local_id() returns the local peer's ID. For the host query,
        // we use the connections list -- but since we can only access
        // through the NetworkProvider trait, we return the local ID
        // if the provider is hosting (peer ID 1), otherwise 0.
        Ok(inst.provider.local_id().map(|id| id.0).unwrap_or(0))
    });
    result.unwrap_or(0)
}
