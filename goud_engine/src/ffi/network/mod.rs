//! Networking FFI functions.
//!
//! Provides C-compatible functions for creating and managing network
//! connections using UDP, WebSocket, or TCP transports. Network provider
//! instances are stored in a global registry keyed by opaque handles.
//!
//! Split across submodules:
//! - `lifecycle`: host, connect, disconnect, send, receive, poll
//! - `stats`: FFI-safe aggregate stats types and query functions
//! - `controls`: overlay-handle selection, peer count, simulation controls

#[cfg(test)]
use crate::core::error::ERR_INVALID_STATE;
#[cfg(test)]
use crate::ffi::context::GoudContextId;

mod controls;
mod lifecycle;
mod overlay;
mod p2p;
mod provider_factory;
pub(crate) mod registry;
mod rollback;
mod rpc;
mod stats;

#[allow(unused_imports)]
pub(crate) use overlay::NetworkOverlaySnapshot;
pub(crate) use overlay::{
    network_overlay_handle_for_context, network_overlay_set_active_handle_override,
    network_overlay_snapshot_for_context,
};

pub use controls::{
    goud_network_clear_overlay_handle, goud_network_clear_simulation, goud_network_peer_count,
    goud_network_set_overlay_handle, goud_network_set_simulation,
};
pub use lifecycle::{
    goud_network_connect, goud_network_connect_with_peer, goud_network_disconnect,
    goud_network_host, goud_network_poll, goud_network_receive, goud_network_send,
};
pub use p2p::{
    goud_p2p_create_mesh, goud_p2p_get_host, goud_p2p_get_peers, goud_p2p_join_mesh,
    goud_p2p_leave_mesh, FfiP2pMeshConfig,
};
pub use rollback::{
    goud_rollback_advance_frame, goud_rollback_check_desync, goud_rollback_confirmed_frame,
    goud_rollback_create, goud_rollback_current_frame, goud_rollback_destroy,
    goud_rollback_receive_remote_input, goud_rollback_resimulate, goud_rollback_should_rollback,
};
pub use rpc::{
    goud_rpc_call, goud_rpc_create, goud_rpc_destroy, goud_rpc_drain_one, goud_rpc_poll,
    goud_rpc_process_incoming, goud_rpc_receive_response, goud_rpc_register,
};
pub use stats::{goud_network_get_stats, goud_network_get_stats_v2, FfiNetworkStats};

#[cfg(test)]
use controls::simulation_controls_unavailable;

#[cfg(test)]
mod tests;
