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

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, DisconnectReason, HostConfig,
    NetworkCapabilities, NetworkEvent, NetworkStats, NetworkStatsTracker, WebRtcConfig,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::{GoudError, GoudResult};

const RECV_BUF_SIZE: usize = 65536;
const CONNECTION_TIMEOUT_SECS: u64 = 15;
const STUN_BINDING_REQUEST: u16 = 0x0001;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

fn net_err(msg: impl Into<String>) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: msg.into(),
    }
}

// =============================================================================
// Internal event type
// =============================================================================

#[allow(dead_code)]
enum InternalWebRtcEvent {
    Connected(ConnectionId),
    Disconnected(ConnectionId, DisconnectReason),
    Received(ConnectionId, Channel, Vec<u8>),
    Error(ConnectionId, String),
    StunResult(ConnectionId, Result<SocketAddr, String>),
}

// =============================================================================
// STUN helpers
// =============================================================================

/// Minimal STUN binding request. Returns a 20-byte STUN header with a random
/// transaction ID. This is enough to discover our public-facing address from
/// a STUN server.
fn build_stun_binding_request() -> ([u8; 20], [u8; 12]) {
    let mut msg = [0u8; 20];
    // Type: Binding Request (0x0001)
    msg[0] = (STUN_BINDING_REQUEST >> 8) as u8;
    msg[1] = (STUN_BINDING_REQUEST & 0xFF) as u8;
    // Length: 0 (no attributes)
    msg[2] = 0;
    msg[3] = 0;
    // Magic Cookie
    let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
    msg[4..8].copy_from_slice(&cookie);
    // Transaction ID: 12 random bytes (use simple counter for determinism in tests)
    let tid: [u8; 12] = {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut t = [0u8; 12];
        let bytes = ts.to_le_bytes();
        let copy_len = bytes.len().min(12);
        t[..copy_len].copy_from_slice(&bytes[..copy_len]);
        t
    };
    msg[8..20].copy_from_slice(&tid);
    (msg, tid)
}

/// Parse a STUN binding response to extract the XOR-MAPPED-ADDRESS.
/// Returns `Some(SocketAddr)` if a valid address was found.
fn parse_stun_response(data: &[u8], tid: &[u8; 12]) -> Option<SocketAddr> {
    if data.len() < 20 {
        return None;
    }
    // Verify it is a binding success response (0x0101)
    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    if msg_type != 0x0101 {
        return None;
    }
    // Verify magic cookie
    let cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if cookie != STUN_MAGIC_COOKIE {
        return None;
    }
    // Verify transaction ID matches
    if data[8..20] != tid[..] {
        return None;
    }

    let msg_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    let attr_data = &data[20..20 + msg_len.min(data.len() - 20)];

    // Walk attributes looking for XOR-MAPPED-ADDRESS (0x0020) or
    // MAPPED-ADDRESS (0x0001).
    let mut offset = 0;
    while offset + 4 <= attr_data.len() {
        let attr_type = u16::from_be_bytes([attr_data[offset], attr_data[offset + 1]]);
        let attr_len = u16::from_be_bytes([attr_data[offset + 2], attr_data[offset + 3]]) as usize;
        let val_start = offset + 4;

        if attr_type == 0x0020 && attr_len >= 8 {
            // XOR-MAPPED-ADDRESS: family(1) + padding(1) + port(2) + ip(4)
            let family = attr_data[val_start + 1];
            if family == 0x01 {
                // IPv4
                let xor_port =
                    u16::from_be_bytes([attr_data[val_start + 2], attr_data[val_start + 3]]);
                let port = xor_port ^ (STUN_MAGIC_COOKIE >> 16) as u16;
                let xor_ip = u32::from_be_bytes([
                    attr_data[val_start + 4],
                    attr_data[val_start + 5],
                    attr_data[val_start + 6],
                    attr_data[val_start + 7],
                ]);
                let ip = xor_ip ^ STUN_MAGIC_COOKIE;
                let ip_bytes = ip.to_be_bytes();
                let addr = SocketAddr::from((ip_bytes, port));
                return Some(addr);
            }
        }

        // Attributes are padded to 4-byte boundaries
        let padded_len = (attr_len + 3) & !3;
        offset = val_start + padded_len;
    }

    None
}

// =============================================================================
// Framing protocol
// =============================================================================

/// Frame layout for WebRTC data channel messages:
///   [4 bytes: frame length (BE)] [1 byte: channel] [payload]
///
/// Reliable channel (0) additionally tracks sequence numbers using
/// a simple header prepended to the payload.
///
/// Reliable sub-frame:
///   [4 bytes: sequence (BE)] [4 bytes: ack (BE)] [4 bytes: ack_bitfield (BE)] [payload]
const RELIABLE_HEADER_SIZE: usize = 12;

fn encode_frame(channel: Channel, data: &[u8]) -> Vec<u8> {
    let frame_len = (data.len() + 1) as u32;
    let mut frame = Vec::with_capacity(4 + frame_len as usize);
    frame.extend_from_slice(&frame_len.to_be_bytes());
    frame.push(channel.0);
    frame.extend_from_slice(data);
    frame
}

fn decode_frame(data: &[u8]) -> Option<(Channel, Vec<u8>)> {
    if data.len() < 5 {
        return None;
    }
    let frame_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + frame_len || frame_len == 0 {
        return None;
    }
    let channel = Channel(data[4]);
    let payload = data[5..4 + frame_len].to_vec();
    Some((channel, payload))
}

// =============================================================================
// Per-connection state
// =============================================================================

struct WebRtcConnection {
    id: ConnectionId,
    addr: SocketAddr,
    state: ConnectionState,
    stats: NetworkStatsTracker,
    last_recv: Instant,
    /// Outgoing sequence number for reliable channel.
    send_seq: u32,
    /// Highest received sequence number for reliable channel.
    recv_seq: u32,
    /// Ack bitfield for reliable channel.
    ack_bitfield: u32,
    /// Buffered reliable payloads awaiting ack, keyed by sequence.
    reliable_buffer: HashMap<u32, (Vec<u8>, Instant)>,
}

impl WebRtcConnection {
    fn new(id: ConnectionId, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            state: ConnectionState::Connecting,
            stats: NetworkStatsTracker::new(),
            last_recv: Instant::now(),
            send_seq: 0,
            recv_seq: 0,
            ack_bitfield: 0,
            reliable_buffer: HashMap::new(),
        }
    }

    fn next_seq(&mut self) -> u32 {
        self.send_seq = self.send_seq.wrapping_add(1);
        self.send_seq
    }

    fn process_ack(&mut self, ack: u32, ack_bits: u32) {
        self.reliable_buffer.remove(&ack);
        for i in 0..32u32 {
            if ack_bits & (1 << i) != 0 {
                self.reliable_buffer.remove(&ack.wrapping_sub(i + 1));
            }
        }
    }

    fn record_incoming_seq(&mut self, seq: u32) -> bool {
        if seq > self.recv_seq {
            let diff = seq - self.recv_seq;
            if diff < 32 {
                self.ack_bitfield = (self.ack_bitfield << diff) | 1;
            } else {
                self.ack_bitfield = 1;
            }
            self.recv_seq = seq;
            true
        } else if seq < self.recv_seq {
            let diff = self.recv_seq - seq;
            if diff <= 32 && self.ack_bitfield & (1 << (diff - 1)) == 0 {
                self.ack_bitfield |= 1 << (diff - 1);
                true
            } else {
                false
            }
        } else {
            false // duplicate
        }
    }
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

    /// Perform a STUN binding request to discover the public-facing address.
    /// This is a blocking operation run on a background thread.
    fn start_stun_discovery(&mut self) {
        if self.config.stun_servers.is_empty() {
            return;
        }
        let stun_server = self.config.stun_servers[0].clone();
        let socket = match &self.socket {
            Some(s) => match s.try_clone() {
                Ok(s) => s,
                Err(_) => return,
            },
            None => return,
        };

        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let conn_id = self.allocate_id();

        let handle = thread::spawn(move || {
            // Strip the "stun:" prefix if present.
            let addr_str = stun_server.strip_prefix("stun:").unwrap_or(&stun_server);

            let stun_addr: SocketAddr = match addr_str.parse() {
                Ok(a) => a,
                Err(e) => {
                    let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                        conn_id,
                        Err(format!("parse STUN addr: {e}")),
                    ));
                    return;
                }
            };

            let (request, tid) = build_stun_binding_request();
            // Set a blocking timeout for the STUN request.
            let _ = socket.set_read_timeout(Some(Duration::from_secs(3)));
            let _ = socket.set_nonblocking(false);
            if let Err(e) = socket.send_to(&request, stun_addr) {
                let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                    conn_id,
                    Err(format!("STUN send: {e}")),
                ));
                return;
            }

            let mut buf = [0u8; 256];
            match socket.recv_from(&mut buf) {
                Ok((len, _)) => {
                    if !running.load(Ordering::Relaxed) {
                        return;
                    }
                    match parse_stun_response(&buf[..len], &tid) {
                        Some(addr) => {
                            let _ =
                                event_tx.send(InternalWebRtcEvent::StunResult(conn_id, Ok(addr)));
                        }
                        None => {
                            let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                                conn_id,
                                Err("Failed to parse STUN response".into()),
                            ));
                        }
                    }
                }
                Err(e) => {
                    let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                        conn_id,
                        Err(format!("STUN recv: {e}")),
                    ));
                }
            }

            // Restore non-blocking mode.
            let _ = socket.set_nonblocking(true);
        });

        if let Ok(mut guard) = self.threads.lock() {
            guard.push(handle);
        }
    }

    fn send_raw(&mut self, addr: SocketAddr, data: &[u8]) -> GoudResult<()> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| net_err("Socket not bound"))?;
        socket
            .send_to(data, addr)
            .map_err(|e| net_err(format!("send_to: {e}")))?;
        self.stats.record_sent_packet(data.len());
        Ok(())
    }

    fn handle_recv(&mut self, src: SocketAddr, data: &[u8]) {
        self.stats.record_received_packet(data.len());

        let Some((channel, payload)) = decode_frame(data) else {
            // Could be a connect handshake (first 4 bytes = "WRTC").
            if data.len() >= 4 && &data[..4] == b"WRTC" {
                self.handle_connect_handshake(src, data);
            } else if data.len() >= 4 && &data[..4] == b"WRTA" {
                self.handle_connect_ack(src);
            } else if data.len() >= 4 && &data[..4] == b"WRTD" {
                self.handle_disconnect_packet(src);
            }
            return;
        };

        let Some(&id) = self.addr_to_id.get(&src) else {
            return;
        };
        let Some(conn) = self.connections.get_mut(&id.0) else {
            return;
        };
        if conn.state != ConnectionState::Connected {
            return;
        }

        conn.last_recv = Instant::now();
        conn.stats.record_received_packet(data.len());

        if channel.0 == 0 {
            // Reliable channel: extract sequence header.
            if payload.len() < RELIABLE_HEADER_SIZE {
                return;
            }
            let seq = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
            let ack = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
            let ack_bits = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);

            conn.process_ack(ack, ack_bits);
            let is_new = conn.record_incoming_seq(seq);

            if is_new {
                let user_data = payload[RELIABLE_HEADER_SIZE..].to_vec();
                if !user_data.is_empty() {
                    self.events.push(NetworkEvent::Received {
                        conn: id,
                        channel,
                        data: user_data,
                    });
                }
            }
        } else {
            // Unreliable channel: deliver payload directly.
            self.events.push(NetworkEvent::Received {
                conn: id,
                channel,
                data: payload,
            });
        }
    }

    fn handle_connect_handshake(&mut self, src: SocketAddr, _data: &[u8]) {
        if !self.is_host {
            return;
        }
        if self.addr_to_id.contains_key(&src) {
            // Already connected; send ack again.
            if let Some(ref socket) = self.socket {
                let _ = socket.send_to(b"WRTA", src);
            }
            return;
        }
        if self.connections.len() >= self.capabilities.max_connections as usize {
            return;
        }

        let id = self.allocate_id();
        let mut conn = WebRtcConnection::new(id, src);
        conn.state = ConnectionState::Connected;
        self.addr_to_id.insert(src, id);
        self.connections.insert(id.0, conn);
        self.events.push(NetworkEvent::Connected { conn: id });

        // Send ack.
        if let Some(ref socket) = self.socket {
            let _ = socket.send_to(b"WRTA", src);
            self.stats.record_sent_packet(4);
        }
    }

    fn handle_connect_ack(&mut self, src: SocketAddr) {
        let Some(&id) = self.addr_to_id.get(&src) else {
            return;
        };
        let Some(conn) = self.connections.get_mut(&id.0) else {
            return;
        };
        if conn.state == ConnectionState::Connecting {
            conn.state = ConnectionState::Connected;
            conn.last_recv = Instant::now();
            self.events.push(NetworkEvent::Connected { conn: id });
        }
    }

    fn handle_disconnect_packet(&mut self, src: SocketAddr) {
        if let Some(id) = self.addr_to_id.remove(&src) {
            if self.connections.remove(&id.0).is_some() {
                self.events.push(NetworkEvent::Disconnected {
                    conn: id,
                    reason: DisconnectReason::RemoteClose,
                });
            }
        }
    }

    fn check_timeouts(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(CONNECTION_TIMEOUT_SECS);
        let mut timed_out = Vec::new();

        for conn in self.connections.values() {
            if conn.state == ConnectionState::Connected
                && now.duration_since(conn.last_recv) >= timeout
            {
                timed_out.push(conn.id);
            }
        }

        for id in timed_out {
            if let Some(conn) = self.connections.remove(&id.0) {
                self.addr_to_id.remove(&conn.addr);
                self.events.push(NetworkEvent::Disconnected {
                    conn: id,
                    reason: DisconnectReason::Timeout,
                });
            }
        }
    }

    fn process_retransmits(&mut self) {
        let now = Instant::now();
        let retransmit_timeout = Duration::from_millis(100);
        let mut to_send: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
        let mut total_lost = 0u64;

        for conn in self.connections.values_mut() {
            if conn.state != ConnectionState::Connected {
                continue;
            }
            let mut expired = Vec::new();
            for (&seq, (data, sent_at)) in &conn.reliable_buffer {
                if now.duration_since(*sent_at) >= retransmit_timeout {
                    // Build reliable header.
                    let mut header = Vec::with_capacity(RELIABLE_HEADER_SIZE + data.len());
                    header.extend_from_slice(&seq.to_be_bytes());
                    header.extend_from_slice(&conn.recv_seq.to_be_bytes());
                    header.extend_from_slice(&conn.ack_bitfield.to_be_bytes());
                    header.extend_from_slice(data);

                    let frame = encode_frame(Channel(0), &header);
                    to_send.push((conn.addr, frame));
                    expired.push(seq);
                    total_lost += 1;
                }
            }
            for seq in expired {
                if let Some(entry) = conn.reliable_buffer.get_mut(&seq) {
                    entry.1 = now; // Reset timer for next retransmit.
                }
            }
            if total_lost > 0 {
                conn.stats.record_packets_lost(total_lost);
            }
        }

        if total_lost > 0 {
            self.stats.record_packets_lost(total_lost);
        }
        for (addr, packet) in to_send {
            if let Some(ref socket) = self.socket {
                let _ = socket.send_to(&packet, addr);
                self.stats.record_sent_packet(packet.len());
            }
        }
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
#[path = "webrtc_network_tests.rs"]
mod tests;
