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
mod provider_factory;
pub(crate) mod registry;
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
pub use stats::{goud_network_get_stats, goud_network_get_stats_v2, FfiNetworkStats};

#[cfg(test)]
use controls::simulation_controls_unavailable;

#[cfg(test)]
mod tests;
