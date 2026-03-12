//! Supporting types for the network provider subsystem.
//!
//! These types define the data structures used by `NetworkProvider` implementations
//! for connection management, event handling, and statistics reporting.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

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

// This helper is consumed only by feature-gated native transport backends.
#[derive(Debug, Clone, Copy)]
struct TimedValue {
    at: Instant,
    value: u64,
}

/// Rolling statistics accumulator shared by network providers.
#[derive(Debug, Clone)]
pub(crate) struct NetworkStatsTracker {
    total_bytes_sent: u64,
    total_bytes_received: u64,
    total_packets_sent: u64,
    total_packets_received: u64,
    total_packets_lost: u64,
    send_window: VecDeque<TimedValue>,
    recv_window: VecDeque<TimedValue>,
    sent_packet_window: VecDeque<TimedValue>,
    lost_packet_window: VecDeque<TimedValue>,
    rtt_samples_ms: VecDeque<f32>,
    latest_rtt_ms: f32,
}

impl Default for NetworkStatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkStatsTracker {
    const BANDWIDTH_WINDOW: Duration = Duration::from_secs(1);
    const RTT_SAMPLE_CAPACITY: usize = 32;

    pub(crate) fn new() -> Self {
        Self {
            total_bytes_sent: 0,
            total_bytes_received: 0,
            total_packets_sent: 0,
            total_packets_received: 0,
            total_packets_lost: 0,
            send_window: VecDeque::new(),
            recv_window: VecDeque::new(),
            sent_packet_window: VecDeque::new(),
            lost_packet_window: VecDeque::new(),
            rtt_samples_ms: VecDeque::with_capacity(Self::RTT_SAMPLE_CAPACITY),
            latest_rtt_ms: 0.0,
        }
    }

    pub(crate) fn record_sent_packet(&mut self, bytes: usize) {
        let now = Instant::now();
        self.total_bytes_sent += bytes as u64;
        self.total_packets_sent += 1;
        self.send_window.push_back(TimedValue {
            at: now,
            value: bytes as u64,
        });
        self.sent_packet_window
            .push_back(TimedValue { at: now, value: 1 });
        self.prune(now);
    }

    pub(crate) fn record_received_packet(&mut self, bytes: usize) {
        let now = Instant::now();
        self.total_bytes_received += bytes as u64;
        self.total_packets_received += 1;
        self.recv_window.push_back(TimedValue {
            at: now,
            value: bytes as u64,
        });
        self.prune(now);
    }

    pub(crate) fn record_packets_lost(&mut self, count: u64) {
        if count == 0 {
            return;
        }
        let now = Instant::now();
        self.total_packets_lost += count;
        self.lost_packet_window.push_back(TimedValue {
            at: now,
            value: count,
        });
        self.prune(now);
    }

    pub(crate) fn record_rtt_sample(&mut self, rtt_ms: f32) {
        self.latest_rtt_ms = rtt_ms.max(0.0);
        if self.rtt_samples_ms.len() >= Self::RTT_SAMPLE_CAPACITY {
            self.rtt_samples_ms.pop_front();
        }
        self.rtt_samples_ms.push_back(self.latest_rtt_ms);
    }

    pub(crate) fn snapshot_network(&self) -> NetworkStats {
        let now = Instant::now();
        NetworkStats {
            bytes_sent: self.total_bytes_sent,
            bytes_received: self.total_bytes_received,
            packets_sent: self.total_packets_sent,
            packets_received: self.total_packets_received,
            packets_lost: self.total_packets_lost,
            rtt_ms: self.latest_rtt_ms,
            send_bandwidth_bytes_per_sec: self.window_sum(&self.send_window, now) as f32,
            receive_bandwidth_bytes_per_sec: self.window_sum(&self.recv_window, now) as f32,
            packet_loss_percent: self.packet_loss_percent(now),
            jitter_ms: self.jitter_ms(),
        }
    }

    pub(crate) fn snapshot_connection(&self) -> ConnectionStats {
        let now = Instant::now();
        ConnectionStats {
            rtt_ms: self.latest_rtt_ms,
            bytes_sent: self.total_bytes_sent,
            bytes_received: self.total_bytes_received,
            packets_sent: self.total_packets_sent,
            packets_received: self.total_packets_received,
            packets_lost: self.total_packets_lost,
            send_bandwidth_bytes_per_sec: self.window_sum(&self.send_window, now) as f32,
            receive_bandwidth_bytes_per_sec: self.window_sum(&self.recv_window, now) as f32,
            packet_loss_percent: self.packet_loss_percent(now),
            jitter_ms: self.jitter_ms(),
        }
    }

    fn prune(&mut self, now: Instant) {
        Self::prune_window(&mut self.send_window, now);
        Self::prune_window(&mut self.recv_window, now);
        Self::prune_window(&mut self.sent_packet_window, now);
        Self::prune_window(&mut self.lost_packet_window, now);
    }

    fn prune_window(window: &mut VecDeque<TimedValue>, now: Instant) {
        while let Some(front) = window.front() {
            if now.duration_since(front.at) <= Self::BANDWIDTH_WINDOW {
                break;
            }
            window.pop_front();
        }
    }

    fn window_sum(&self, window: &VecDeque<TimedValue>, now: Instant) -> u64 {
        window
            .iter()
            .filter(|sample| now.duration_since(sample.at) <= Self::BANDWIDTH_WINDOW)
            .map(|sample| sample.value)
            .sum()
    }

    fn packet_loss_percent(&self, now: Instant) -> f32 {
        let sent = self.window_sum(&self.sent_packet_window, now);
        let lost = self.window_sum(&self.lost_packet_window, now);
        if sent + lost == 0 {
            return 0.0;
        }
        (lost as f32 / (sent + lost) as f32) * 100.0
    }

    fn jitter_ms(&self) -> f32 {
        if self.rtt_samples_ms.len() < 2 {
            return 0.0;
        }

        let mean = self.rtt_samples_ms.iter().sum::<f32>() / self.rtt_samples_ms.len() as f32;
        let variance = self
            .rtt_samples_ms
            .iter()
            .map(|sample| {
                let delta = sample - mean;
                delta * delta
            })
            .sum::<f32>()
            / self.rtt_samples_ms.len() as f32;

        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_stats_tracker_computes_bandwidth_and_loss_windows() {
        let mut tracker = NetworkStatsTracker::new();
        tracker.record_sent_packet(128);
        tracker.record_sent_packet(64);
        tracker.record_received_packet(256);
        tracker.record_packets_lost(1);

        let snapshot = tracker.snapshot_network();
        assert_eq!(snapshot.bytes_sent, 192);
        assert_eq!(snapshot.bytes_received, 256);
        assert_eq!(snapshot.packets_sent, 2);
        assert_eq!(snapshot.packets_received, 1);
        assert_eq!(snapshot.packets_lost, 1);
        assert_eq!(snapshot.send_bandwidth_bytes_per_sec, 192.0);
        assert_eq!(snapshot.receive_bandwidth_bytes_per_sec, 256.0);
        assert!((snapshot.packet_loss_percent - (100.0 / 3.0)).abs() < 0.001);
    }

    #[test]
    fn test_network_stats_tracker_computes_jitter_from_rtt_samples() {
        let mut tracker = NetworkStatsTracker::new();
        tracker.record_rtt_sample(10.0);
        tracker.record_rtt_sample(20.0);
        tracker.record_rtt_sample(30.0);

        let snapshot = tracker.snapshot_connection();
        assert_eq!(snapshot.rtt_ms, 30.0);
        assert!((snapshot.jitter_ms - 8.164966).abs() < 0.001);
    }

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
