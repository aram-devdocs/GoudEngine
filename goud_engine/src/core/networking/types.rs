//! Shared types for the session-level client-server networking module.

use std::collections::BTreeMap;

use crate::core::providers::network_types::{Channel, ConnectionId, DisconnectReason, HostConfig};

/// Session descriptor returned by discovery and used by host configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDescriptor {
    /// Stable session identifier.
    pub id: String,
    /// Human-readable session name.
    pub name: String,
    /// Join address (transport-dependent, e.g. `127.0.0.1:7000` or `ws://...`).
    pub address: String,
    /// Maximum expected clients for this session.
    pub max_clients: u32,
    /// Current connected client count.
    pub current_clients: u32,
    /// Additional metadata for listing UIs.
    pub metadata: BTreeMap<String, String>,
}

impl SessionDescriptor {
    /// Constructs a session descriptor with empty metadata.
    pub fn new(id: impl Into<String>, name: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            address: address.into(),
            max_clients: 0,
            current_clients: 0,
            metadata: BTreeMap::new(),
        }
    }
}

/// Logical channels used by the session protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionChannels {
    /// Control channel for join/leave and rejection responses.
    pub control: Channel,
    /// Command channel for client state-change requests.
    pub command: Channel,
    /// Authoritative state update channel from server to clients.
    pub state: Channel,
}

impl Default for SessionChannels {
    fn default() -> Self {
        Self {
            control: Channel(0),
            command: Channel(1),
            state: Channel(2),
        }
    }
}

/// Server bootstrap configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Transport host configuration.
    pub host: HostConfig,
    /// Session descriptor published to discovery providers.
    pub session: SessionDescriptor,
    /// Session protocol channels.
    pub channels: SessionChannels,
    /// Whether to publish this session to native LAN discovery.
    pub advertise_on_lan: bool,
    /// Whether accepted state commands should be rebroadcast automatically.
    pub auto_broadcast_commands: bool,
}

impl ServerConfig {
    /// Creates a server config with default session channels.
    pub fn new(host: HostConfig, session: SessionDescriptor) -> Self {
        Self {
            host,
            session,
            channels: SessionChannels::default(),
            advertise_on_lan: false,
            auto_broadcast_commands: true,
        }
    }
}

/// Server-side session events emitted by [`crate::core::networking::SessionServer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerEvent {
    /// A new client joined.
    ClientJoined {
        /// Connection identifier of the joined client.
        connection: ConnectionId,
    },
    /// A client left.
    ClientLeft {
        /// Connection identifier of the disconnected client.
        connection: ConnectionId,
        /// Transport-level reason for disconnection.
        reason: DisconnectReason,
    },
    /// A state-change command passed authority validation.
    CommandAccepted {
        /// Client connection that sent the command.
        connection: ConnectionId,
        /// Opaque command payload bytes.
        payload: Vec<u8>,
    },
    /// A command was rejected by authority validation.
    CommandRejected {
        /// Client connection that sent the rejected command.
        connection: ConnectionId,
        /// Original command payload.
        payload: Vec<u8>,
        /// Human-readable rejection reason.
        reason: String,
    },
    /// An authoritative state payload was broadcast.
    StateBroadcast {
        /// Monotonic authoritative update sequence.
        sequence: u64,
        /// Number of recipients at broadcast time.
        recipients: usize,
        /// Opaque authoritative state payload.
        payload: Vec<u8>,
    },
    /// A protocol payload could not be decoded or was invalid for context.
    ProtocolError {
        /// Connection associated with the protocol error.
        connection: ConnectionId,
        /// Human-readable reason.
        reason: String,
    },
}

/// Client-side session events emitted by [`crate::core::networking::SessionClient`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientEvent {
    /// Transport connection established; join handshake was sent.
    Connected {
        /// Connection identifier of the server connection.
        connection: ConnectionId,
    },
    /// Server accepted the join and supplied the current authoritative snapshot.
    Joined {
        /// Connection identifier of the server connection.
        connection: ConnectionId,
        /// Authoritative state snapshot at join time.
        snapshot: Vec<u8>,
    },
    /// Transport disconnection or server-initiated leave.
    Left {
        /// Connection identifier that was closed.
        connection: ConnectionId,
        /// Reason for leaving.
        reason: DisconnectReason,
    },
    /// Authoritative state update received from server.
    StateUpdated {
        /// Monotonic authoritative update sequence.
        sequence: u64,
        /// Opaque authoritative state payload.
        payload: Vec<u8>,
    },
    /// Server rejected a state-change command.
    ValidationRejected {
        /// Original command payload.
        payload: Vec<u8>,
        /// Server-provided rejection reason.
        reason: String,
    },
    /// A received protocol payload was invalid.
    ProtocolError {
        /// Human-readable reason.
        reason: String,
    },
}
