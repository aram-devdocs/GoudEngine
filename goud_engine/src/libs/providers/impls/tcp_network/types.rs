//! Internal types for the TCP transport provider.

use std::sync::mpsc;
use std::time::Instant;

use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, DisconnectReason, NetworkStatsTracker,
};

use super::ReconnectConfig;

pub(crate) enum InternalTcpEvent {
    /// A connection was established. Carries the IO generation.
    Connected(ConnectionId, u64),
    /// A connection was closed. Carries the IO generation.
    Disconnected(ConnectionId, DisconnectReason, u64),
    /// Data received (no generation needed; stale data is harmless).
    Received(ConnectionId, Channel, Vec<u8>),
    /// Error on a connection.
    Error(ConnectionId, String),
    /// The write channel for a connection is ready. Carries the IO generation.
    WriteTxReady(ConnectionId, mpsc::Sender<Vec<u8>>, u64),
}

pub(crate) struct TcpConnection {
    pub(crate) id: ConnectionId,
    pub(crate) state: ConnectionState,
    pub(crate) stats: NetworkStatsTracker,
    /// The remote address this connection was established to (client-side only).
    pub(crate) remote_addr: Option<String>,
    /// Reconnect tracking state.
    pub(crate) reconnect: ReconnectConfig,
    /// Number of reconnect attempts made so far.
    pub(crate) reconnect_attempts: u32,
    /// When the last reconnect attempt was started.
    pub(crate) last_reconnect_at: Option<Instant>,
    /// Generation counter to disambiguate events from old vs. new IO threads.
    pub(crate) generation: u64,
}
