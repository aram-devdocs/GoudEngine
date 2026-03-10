//! Supporting types for the network provider subsystem.
//!
//! These types define the data structures used by `NetworkProvider` implementations
//! for connection management, event handling, and statistics reporting.

// =============================================================================
// Connection Identifiers
// =============================================================================

/// Opaque transport-level connection identifier.
///
/// Assigned by the network provider when a connection is established.
/// Valid until the connection closes. Game-level peer identity (player IDs,
/// lobby slots) belongs above the provider boundary.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ConnectionId(pub u64);

/// Named channel for message routing.
///
/// Channels map to transport QoS settings (reliable/unreliable, ordered/unordered).
/// Channel 0 is always reliable-ordered. Higher channels are provider-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel(pub u8);

// =============================================================================
// Connection State
// =============================================================================

/// Lifecycle state of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected.
    Disconnected,
    /// Handshake in progress.
    Connecting,
    /// Fully established and ready for data.
    Connected,
    /// Graceful shutdown in progress.
    Disconnecting,
    /// An error occurred on the connection.
    Error,
}

/// Why a connection closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    /// The local peer initiated the close.
    LocalClose,
    /// The remote peer initiated the close.
    RemoteClose,
    /// The connection timed out.
    Timeout,
    /// An error caused the disconnection.
    Error(String),
}

// =============================================================================
// Events
// =============================================================================

/// An event produced by the network provider during `drain_events`.
///
/// Events are buffered internally by the provider and returned as an owned
/// `Vec` to avoid holding a borrow on the provider while the caller processes
/// events.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A connection was fully established.
    Connected {
        /// The connection that was established.
        conn: ConnectionId,
    },
    /// A connection was closed.
    Disconnected {
        /// The connection that was closed.
        conn: ConnectionId,
        /// The reason for disconnection.
        reason: DisconnectReason,
    },
    /// Data was received on a connection.
    Received {
        /// The connection the data arrived on.
        conn: ConnectionId,
        /// The channel the data was sent on.
        channel: Channel,
        /// The raw payload bytes.
        data: Vec<u8>,
    },
    /// An error occurred on a connection.
    Error {
        /// The connection that encountered the error.
        conn: ConnectionId,
        /// A human-readable error description.
        message: String,
    },
}

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for hosting (accepting inbound connections).
#[derive(Debug, Clone)]
pub struct HostConfig {
    /// Address to bind the listener on.
    pub bind_address: String,
    /// Port to listen on.
    pub port: u16,
    /// Maximum number of simultaneous connections to accept.
    pub max_connections: u32,
    /// Optional path to TLS certificate file.
    pub tls_cert_path: Option<String>,
    /// Optional path to TLS private key file.
    pub tls_key_path: Option<String>,
}

// =============================================================================
// Capabilities and Statistics
// =============================================================================

/// Static capability flags for a network provider.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct NetworkCapabilities {
    /// Whether this provider can act as a host (accept connections).
    pub supports_hosting: bool,
    /// Maximum number of simultaneous connections.
    pub max_connections: u32,
    /// Maximum number of channels supported.
    pub max_channels: u8,
    /// Maximum size of a single message in bytes.
    pub max_message_size: u32,
}

/// Aggregate network statistics for the provider.
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// Total bytes sent across all connections.
    pub bytes_sent: u64,
    /// Total bytes received across all connections.
    pub bytes_received: u64,
    /// Total packets sent.
    pub packets_sent: u64,
    /// Total packets received.
    pub packets_received: u64,
    /// Total packets lost.
    pub packets_lost: u64,
}

/// Per-connection statistics.
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Round-trip time in milliseconds.
    pub round_trip_ms: f32,
    /// Bytes sent on this connection.
    pub bytes_sent: u64,
    /// Bytes received on this connection.
    pub bytes_received: u64,
    /// Packets lost on this connection.
    pub packets_lost: u64,
}
