//! Supporting types for the network provider subsystem.
//!
//! These types define the data structures used by `NetworkProvider` implementations
//! for connection management, event handling, and statistics reporting.

mod p2p_types;
mod stats_tracker;

pub use p2p_types::{P2pEvent, P2pMeshConfig, P2pTopology, PeerId};
pub(crate) use stats_tracker::NetworkStatsTracker;

// =============================================================================
// Connection Identifiers
// =============================================================================

/// Opaque transport-level connection identifier.
///
/// Assigned by the network provider when a connection is established.
/// Valid until the connection closes. Game-level peer identity (player IDs,
/// lobby slots) belongs above the provider boundary.
///
/// IDs are scoped to a single provider handle. If a later overlay or context
/// registry needs to map an engine context to a connection, it must first map
/// the context to the owning provider handle and only then use that
/// provider-local `ConnectionId`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct ConnectionId(pub u64);

/// Named channel for message routing.
///
/// Channels map to transport QoS settings (reliable/unreliable, ordered/unordered).
/// Channel 0 is always reliable-ordered. Higher channels are provider-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
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

/// Transport protocol discriminant used by native provider selection and FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum NetworkProtocol {
    /// UDP transport.
    #[default]
    Udp = 0,
    /// WebSocket transport.
    WebSocket = 1,
    /// TCP transport.
    Tcp = 2,
    /// WebRTC data channel transport.
    WebRTC = 3,
}

// =============================================================================
// WebRTC Configuration
// =============================================================================

/// TURN relay server configuration for NAT traversal fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnServer {
    /// TURN server URL (e.g., `"turn:relay.example.com:3478"`).
    pub url: String,
    /// Username for TURN authentication.
    pub username: String,
    /// Credential (password) for TURN authentication.
    pub credential: String,
}

/// WebRTC ICE server configuration for NAT traversal.
///
/// Contains STUN servers for reflexive candidate discovery and optional
/// TURN relay servers for fallback when direct connectivity fails.
#[derive(Debug, Clone, Default)]
pub struct WebRtcConfig {
    /// STUN server URLs (e.g., `["stun:stun.l.google.com:19302"]`).
    pub stun_servers: Vec<String>,
    /// TURN relay servers for NAT traversal fallback.
    pub turn_servers: Vec<TurnServer>,
}

/// Debug-only network simulation knobs applied per provider handle.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NetworkSimulationConfig {
    /// Artificial one-way latency applied before outbound send.
    pub one_way_latency_ms: u32,
    /// Additional random jitter range in milliseconds.
    pub jitter_ms: u32,
    /// Packet loss percentage in `[0.0, 100.0]`.
    pub packet_loss_percent: f32,
}

impl Default for NetworkSimulationConfig {
    fn default() -> Self {
        Self {
            one_way_latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 0.0,
        }
    }
}

impl NetworkSimulationConfig {
    /// Returns `true` when the config would alter transport behavior.
    pub fn is_enabled(self) -> bool {
        self.one_way_latency_ms > 0 || self.jitter_ms > 0 || self.packet_loss_percent > 0.0
    }

    /// Validates the config range constraints.
    pub fn validate(self) -> Result<(), String> {
        if !(0.0..=100.0).contains(&self.packet_loss_percent) {
            return Err("packet_loss_percent must be between 0.0 and 100.0".into());
        }
        Ok(())
    }
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
#[repr(C)]
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
    /// Most recent RTT sample in milliseconds.
    pub rtt_ms: f32,
    /// Send bandwidth over the rolling 1-second window.
    pub send_bandwidth_bytes_per_sec: f32,
    /// Receive bandwidth over the rolling 1-second window.
    pub receive_bandwidth_bytes_per_sec: f32,
    /// Packet loss percentage over the rolling 1-second window.
    pub packet_loss_percent: f32,
    /// Rolling RTT jitter in milliseconds.
    pub jitter_ms: f32,
}

/// Per-connection statistics.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ConnectionStats {
    /// Most recent RTT sample in milliseconds.
    pub rtt_ms: f32,
    /// Bytes sent on this connection.
    pub bytes_sent: u64,
    /// Bytes received on this connection.
    pub bytes_received: u64,
    /// Packets sent on this connection.
    pub packets_sent: u64,
    /// Packets received on this connection.
    pub packets_received: u64,
    /// Packets lost on this connection.
    pub packets_lost: u64,
    /// Send bandwidth over the rolling 1-second window.
    pub send_bandwidth_bytes_per_sec: f32,
    /// Receive bandwidth over the rolling 1-second window.
    pub receive_bandwidth_bytes_per_sec: f32,
    /// Packet loss percentage over the rolling 1-second window.
    pub packet_loss_percent: f32,
    /// Rolling RTT jitter in milliseconds.
    pub jitter_ms: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_simulation_config_validates_loss_range() {
        let invalid = NetworkSimulationConfig {
            one_way_latency_ms: 10,
            jitter_ms: 5,
            packet_loss_percent: 120.0,
        };

        assert!(invalid.validate().is_err());
        assert!(NetworkSimulationConfig::default().validate().is_ok());
    }
}
