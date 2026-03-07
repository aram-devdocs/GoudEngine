//! Network provider trait definition.
//!
//! The `NetworkProvider` trait abstracts the transport backend, enabling
//! runtime selection between UDP, WebSocket, or null (no-op).

use super::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkStats,
};
use super::{Provider, ProviderLifecycle};
use crate::core::error::GoudResult;

/// Trait for network transport backends.
///
/// Abstracts connection lifecycle, message passing over typed channels,
/// and event polling. Uses raw bytes for data transfer; serialization
/// is the caller's responsibility.
///
/// The trait is object-safe and stored as `Option<Box<dyn NetworkProvider>>`
/// in `ProviderRegistry`.
pub trait NetworkProvider: Provider + ProviderLifecycle {
    /// Begin accepting inbound connections on the given config.
    ///
    /// Calling `host` on an already-hosting provider returns an error.
    fn host(&mut self, config: &HostConfig) -> GoudResult<()>;

    /// Open a connection to the given address.
    ///
    /// Returns a `ConnectionId` that is valid until the connection closes.
    /// The connection may not be fully established when this returns; poll
    /// `drain_events` for `NetworkEvent::Connected`.
    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId>;

    /// Close a specific connection.
    fn disconnect(&mut self, conn: ConnectionId) -> GoudResult<()>;

    /// Close all active connections.
    fn disconnect_all(&mut self) -> GoudResult<()>;

    /// Send raw bytes to one connection on the given channel.
    ///
    /// The provider does not inspect or frame the bytes. Serialization
    /// is the caller's responsibility.
    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()>;

    /// Send raw bytes to all active connections on the given channel.
    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()>;

    /// Return all buffered network events and clear the internal buffer.
    ///
    /// Must be called once per frame. Returns owned `Vec` to avoid holding
    /// a borrow on the provider while the caller processes events.
    fn drain_events(&mut self) -> Vec<NetworkEvent>;

    /// Return the current list of active connection IDs.
    fn connections(&self) -> Vec<ConnectionId>;

    /// Return the state of a specific connection.
    fn connection_state(&self, conn: ConnectionId) -> ConnectionState;

    /// Return this peer's own `ConnectionId`, if assigned.
    fn local_id(&self) -> Option<ConnectionId>;

    /// Return static capability flags for this provider.
    fn network_capabilities(&self) -> &NetworkCapabilities;

    /// Return aggregate network statistics.
    fn stats(&self) -> NetworkStats;

    /// Return per-connection statistics, or `None` if the ID is unknown.
    fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats>;
}
