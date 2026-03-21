//! WebRTC data channel transport provider implementing `NetworkProvider`.
//!
//! This provider implements a WebRTC-like data channel abstraction for game
//! networking. It uses a signaling server (WebSocket or custom) for connection
//! establishment and the existing UDP reliability layer for actual data transfer.
//!
//! Channel 0 is reliable-ordered (uses the reliability layer with sequencing
//! and retransmission). Channel 1+ are unreliable (raw UDP datagrams).
//!
//! STUN/TURN configuration is stored for NAT traversal. On native targets
//! the provider performs STUN binding requests to discover the public-facing
//! address. TURN relay is used as a fallback when direct connectivity fails.
//!
//! # WASM
//!
//! WASM targets are currently stubbed. A future implementation will delegate
//! to the browser's built-in `RTCPeerConnection` API via `web-sys`.

pub(super) mod framing;
mod recv;
pub(super) mod stun;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, DisconnectReason, HostConfig,
    NetworkCapabilities, NetworkEvent, NetworkStats, NetworkStatsTracker, WebRtcConfig,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::{GoudError, GoudResult};

use self::framing::{encode_frame, WebRtcConnection, RELIABLE_HEADER_SIZE};

const RECV_BUF_SIZE: usize = 65536;
const CONNECTION_TIMEOUT_SECS: u64 = 15;

fn net_err(msg: impl Into<String>) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: msg.into(),
    }
}

// =============================================================================
// Internal event type
// =============================================================================

enum InternalWebRtcEvent {
    _Connected(ConnectionId),
    _Disconnected(ConnectionId, DisconnectReason),
    _Received(ConnectionId, Channel, Vec<u8>),
    _Error(ConnectionId, String),
    StunResult(ConnectionId, Result<SocketAddr, String>),
}

// =============================================================================
// WebRtcNetProvider
// =============================================================================

/// WebRTC data channel transport provider.
///
/// Uses UDP as the underlying datagram transport with an optional signaling
/// step for connection establishment. Channel 0 provides reliable-ordered
/// delivery with sequence tracking and retransmission. Channels 1+ are
/// unreliable datagrams.
///
/// STUN is used for NAT traversal (public address discovery). TURN relay
/// configuration is stored and available for fallback, though the actual
/// relay protocol requires a TURN-compatible server.
pub struct WebRtcNetProvider {
    config: WebRtcConfig,
    capabilities: NetworkCapabilities,
    socket: Option<UdpSocket>,
    connections: HashMap<u64, WebRtcConnection>,
    addr_to_id: HashMap<SocketAddr, ConnectionId>,
    next_id: Arc<AtomicU64>,
    event_tx: mpsc::Sender<InternalWebRtcEvent>,
    event_rx: Mutex<Option<mpsc::Receiver<InternalWebRtcEvent>>>,
    events: Vec<NetworkEvent>,
    stats: NetworkStatsTracker,
    running: Arc<AtomicBool>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    is_host: bool,
    local_addr: Option<SocketAddr>,
    /// Public-facing address discovered via STUN, if available.
    public_addr: Option<SocketAddr>,
    recv_buf: Box<[u8; RECV_BUF_SIZE]>,
}

// SAFETY: WebRtcNetProvider is only mutated through &mut self on the main
// thread. Non-Sync fields (mpsc::Receiver) are wrapped in Mutex.
unsafe impl Sync for WebRtcNetProvider {}

impl WebRtcNetProvider {
    /// Create a new WebRTC data channel provider with the given ICE config.
    pub fn new() -> Self {
        Self::with_config(WebRtcConfig::default())
    }

    /// Create a new WebRTC data channel provider with explicit STUN/TURN config.
    pub fn with_config(config: WebRtcConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            config,
            capabilities: NetworkCapabilities {
                supports_hosting: true,
                max_connections: 32,
                max_channels: 2,
                max_message_size: (RECV_BUF_SIZE - RELIABLE_HEADER_SIZE - 5) as u32,
            },
            socket: None,
            connections: HashMap::new(),
            addr_to_id: HashMap::new(),
            next_id: Arc::new(AtomicU64::new(1)),
            event_tx,
            event_rx: Mutex::new(Some(event_rx)),
            events: Vec::new(),
            stats: NetworkStatsTracker::new(),
            running: Arc::new(AtomicBool::new(true)),
            threads: Mutex::new(Vec::new()),
            is_host: false,
            local_addr: None,
            public_addr: None,
            recv_buf: Box::new([0u8; RECV_BUF_SIZE]),
        }
    }

    /// Returns the bound local address, if hosting or connected.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the public-facing address discovered via STUN, if available.
    pub fn public_addr(&self) -> Option<SocketAddr> {
        self.public_addr
    }

    /// Returns a reference to the current WebRTC configuration.
    pub fn webrtc_config(&self) -> &WebRtcConfig {
        &self.config
    }

    fn allocate_id(&self) -> ConnectionId {
        ConnectionId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    fn ensure_socket(&mut self) -> GoudResult<()> {
        if self.socket.is_some() {
            return Ok(());
        }
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| net_err(format!("Failed to bind ephemeral: {e}")))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| net_err(format!("Failed to set non-blocking: {e}")))?;
        self.local_addr = Some(
            socket
                .local_addr()
                .map_err(|e| net_err(format!("local_addr: {e}")))?,
        );
        self.socket = Some(socket);
        Ok(())
    }
}

impl Default for WebRtcNetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for WebRtcNetProvider {
    fn name(&self) -> &str {
        "webrtc"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for WebRtcNetProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        // Drain internal events from background threads (STUN results).
        {
            let guard = self.event_rx.lock().map_err(|e| net_err(e.to_string()))?;
            if let Some(ref rx) = *guard {
                while let Ok(evt) = rx.try_recv() {
                    match evt {
                        InternalWebRtcEvent::Connected(id) => {
                            if let Some(conn) = self.connections.get_mut(&id.0) {
                                conn.state = ConnectionState::Connected;
                            }
                            self.events.push(NetworkEvent::Connected { conn: id });
                        }
                        InternalWebRtcEvent::Disconnected(id, reason) => {
                            self.connections.remove(&id.0);
                            self.events
                                .push(NetworkEvent::Disconnected { conn: id, reason });
                        }
                        InternalWebRtcEvent::Received(id, channel, data) => {
                            self.stats.record_received_packet(data.len());
                            if let Some(conn) = self.connections.get_mut(&id.0) {
                                conn.stats.record_received_packet(data.len());
                            }
                            self.events.push(NetworkEvent::Received {
                                conn: id,
                                channel,
                                data,
                            });
                        }
                        InternalWebRtcEvent::Error(id, message) => {
                            if let Some(conn) = self.connections.get_mut(&id.0) {
                                conn.state = ConnectionState::Error;
                            }
                            self.events.push(NetworkEvent::Error { conn: id, message });
                        }
                        InternalWebRtcEvent::StunResult(_id, result) => match result {
                            Ok(addr) => {
                                self.public_addr = Some(addr);
                            }
                            Err(_msg) => {
                                // STUN failure is non-fatal; log and continue.
                            }
                        },
                    }
                }
            }
        }

        // Poll the UDP socket for incoming datagrams.
        let mut incoming: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
        if let Some(ref socket) = self.socket {
            loop {
                match socket.recv_from(self.recv_buf.as_mut()) {
                    Ok((len, src)) => incoming.push((src, self.recv_buf[..len].to_vec())),
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
        }

        for (src, data) in incoming {
            self.handle_recv(src, &data);
        }

        self.process_retransmits();
        self.check_timeouts();
        Ok(())
    }

    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        // Send disconnect to all peers.
        let addrs: Vec<SocketAddr> = self.connections.values().map(|c| c.addr).collect();
        for addr in addrs {
            if let Some(ref socket) = self.socket {
                let _ = socket.send_to(b"WRTD", addr);
            }
        }

        if let Ok(mut guard) = self.event_rx.lock() {
            guard.take();
        }
        if let Ok(mut guard) = self.threads.lock() {
            for handle in std::mem::take(&mut *guard) {
                let _ = handle.join();
            }
        }
        self.connections.clear();
        self.addr_to_id.clear();
        self.socket = None;
    }
}

impl NetworkProvider for WebRtcNetProvider {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        if self.socket.is_some() {
            return Err(net_err("Already hosting or connected"));
        }

        let addr = format!("{}:{}", config.bind_address, config.port);
        let socket = UdpSocket::bind(&addr).map_err(|e| net_err(format!("bind {addr}: {e}")))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| net_err(format!("set non-blocking: {e}")))?;
        self.local_addr = Some(socket.local_addr().map_err(|e| net_err(e.to_string()))?);
        self.socket = Some(socket);
        self.is_host = true;

        // Start STUN discovery in the background if configured.
        self.start_stun_discovery();

        Ok(())
    }

    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
        self.ensure_socket()?;

        let remote: SocketAddr = addr
            .parse()
            .map_err(|e| net_err(format!("Invalid address '{addr}': {e}")))?;

        let id = self.allocate_id();
        let conn = WebRtcConnection::new(id, remote);
        self.addr_to_id.insert(remote, id);
        self.connections.insert(id.0, conn);

        // Send connect handshake.
        if let Some(ref socket) = self.socket {
            let _ = socket.send_to(b"WRTC", remote);
            self.stats.record_sent_packet(4);
        }

        // Start STUN discovery if configured and not already done.
        if self.public_addr.is_none() {
            self.start_stun_discovery();
        }

        Ok(id)
    }

    fn disconnect(&mut self, conn_id: ConnectionId) -> GoudResult<()> {
        let conn = self
            .connections
            .remove(&conn_id.0)
            .ok_or_else(|| net_err(format!("Unknown connection {:?}", conn_id)))?;
        self.addr_to_id.remove(&conn.addr);

        // Send disconnect packet.
        if let Some(ref socket) = self.socket {
            let _ = socket.send_to(b"WRTD", conn.addr);
        }

        self.events.push(NetworkEvent::Disconnected {
            conn: conn_id,
            reason: DisconnectReason::LocalClose,
        });
        Ok(())
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        let ids: Vec<ConnectionId> = self.connections.keys().map(|k| ConnectionId(*k)).collect();
        for id in ids {
            let _ = self.disconnect(id);
        }
        Ok(())
    }

    fn send(&mut self, conn_id: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let conn = self
            .connections
            .get_mut(&conn_id.0)
            .ok_or_else(|| net_err(format!("Unknown connection {:?}", conn_id)))?;
        if conn.state != ConnectionState::Connected {
            return Err(net_err("Connection not established"));
        }

        let addr = conn.addr;

        if channel.0 == 0 {
            // Reliable channel: prepend sequence header.
            let seq = conn.next_seq();
            let mut reliable_data = Vec::with_capacity(RELIABLE_HEADER_SIZE + data.len());
            reliable_data.extend_from_slice(&seq.to_be_bytes());
            reliable_data.extend_from_slice(&conn.recv_seq.to_be_bytes());
            reliable_data.extend_from_slice(&conn.ack_bitfield.to_be_bytes());
            reliable_data.extend_from_slice(data);

            // Buffer for retransmission.
            conn.reliable_buffer
                .insert(seq, (data.to_vec(), Instant::now()));

            let frame = encode_frame(channel, &reliable_data);
            conn.stats.record_sent_packet(frame.len());
            self.send_raw(addr, &frame)?;
        } else {
            // Unreliable channel: send directly.
            let frame = encode_frame(channel, data);
            conn.stats.record_sent_packet(frame.len());
            self.send_raw(addr, &frame)?;
        }

        Ok(())
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let ids: Vec<ConnectionId> = self
            .connections
            .values()
            .filter(|c| c.state == ConnectionState::Connected)
            .map(|c| c.id)
            .collect();
        for id in ids {
            let _ = self.send(id, channel, data);
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        std::mem::take(&mut self.events)
    }

    fn connections(&self) -> Vec<ConnectionId> {
        self.connections.keys().map(|k| ConnectionId(*k)).collect()
    }

    fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
        self.connections
            .get(&conn.0)
            .map(|c| c.state)
            .unwrap_or(ConnectionState::Disconnected)
    }

    fn local_id(&self) -> Option<ConnectionId> {
        None
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        &self.capabilities
    }

    fn stats(&self) -> NetworkStats {
        self.stats.snapshot_network()
    }

    fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats> {
        self.connections
            .get(&conn.0)
            .map(|c| c.stats.snapshot_connection())
    }

    fn network_diagnostics(&self) -> NetworkDiagnosticsV1 {
        let s = self.stats();
        NetworkDiagnosticsV1 {
            bytes_sent: s.bytes_sent,
            bytes_received: s.bytes_received,
            packets_sent: s.packets_sent,
            packets_received: s.packets_received,
            rtt_ms: s.rtt_ms,
            active_connections: self.connections().len() as u32,
        }
    }
}

impl std::fmt::Debug for WebRtcNetProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRtcNetProvider")
            .field("is_host", &self.is_host)
            .field("connections", &self.connections.len())
            .field("has_socket", &self.socket.is_some())
            .field("public_addr", &self.public_addr)
            .finish()
    }
}

#[cfg(test)]
#[path = "../webrtc_network_tests/mod.rs"]
mod tests;
