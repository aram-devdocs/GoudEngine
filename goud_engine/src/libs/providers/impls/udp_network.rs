use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;

use super::udp_reliability::{
    PacketHeader, ReliabilityLayer, HEADER_SIZE, PACKET_CONNECT, PACKET_CONNECT_ACK, PACKET_DATA,
    PACKET_DISCONNECT, PACKET_HEARTBEAT,
};
use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkStats, NetworkStatsTracker,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::{GoudError, GoudResult};

const CONNECTION_TIMEOUT_SECS: u64 = 10;
const RECV_BUF_SIZE: usize = 65536;

fn net_err(msg: String) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: msg,
    }
}

#[derive(Debug)]
struct UdpConnection {
    id: ConnectionId,
    addr: SocketAddr,
    state: ConnectionState,
    reliability: ReliabilityLayer,
    metrics: NetworkStatsTracker,
    last_recv: Instant,
}

/// UDP transport provider. Channel(0) = reliable-ordered, Channel(1+) = unreliable.
pub struct UdpNetProvider {
    socket: Option<UdpSocket>,
    connections: HashMap<u64, UdpConnection>,
    addr_to_id: HashMap<SocketAddr, ConnectionId>,
    next_id: u64,
    capabilities: NetworkCapabilities,
    events: Vec<NetworkEvent>,
    stats: NetworkStatsTracker,
    is_host: bool,
    recv_buf: Box<[u8; RECV_BUF_SIZE]>,
}

impl UdpNetProvider {
    /// Create a new UDP network provider.
    pub fn new() -> Self {
        Self {
            socket: None,
            connections: HashMap::new(),
            addr_to_id: HashMap::new(),
            next_id: 1,
            capabilities: NetworkCapabilities {
                supports_hosting: true,
                max_connections: 32,
                max_channels: 2,
                max_message_size: (RECV_BUF_SIZE - HEADER_SIZE) as u32,
            },
            events: Vec::new(),
            stats: NetworkStatsTracker::new(),
            is_host: false,
            recv_buf: Box::new([0u8; RECV_BUF_SIZE]),
        }
    }

    fn allocate_id(&mut self) -> ConnectionId {
        let id = ConnectionId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    fn send_raw(&mut self, addr: SocketAddr, data: &[u8]) -> GoudResult<()> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| net_err("Socket not bound".into()))?;
        socket
            .send_to(data, addr)
            .map_err(|e| net_err(format!("send_to failed: {}", e)))?;
        self.stats.record_sent_packet(data.len());
        Ok(())
    }

    fn record_rtt_samples(provider_metrics: &mut NetworkStatsTracker, conn: &mut UdpConnection) {
        for sample in conn.reliability.drain_acked_rtt_samples_ms() {
            provider_metrics.record_rtt_sample(sample);
            conn.metrics.record_rtt_sample(sample);
        }
    }

    fn handle_recv(&mut self, src: SocketAddr, data: &[u8]) {
        let Some(header) = PacketHeader::decode(data) else {
            return;
        };
        self.stats.record_received_packet(data.len());

        match header.packet_type {
            PACKET_CONNECT => self.handle_connect(src, &header, data.len()),
            PACKET_CONNECT_ACK => self.handle_connect_ack(src, &header, data.len()),
            PACKET_DATA => self.handle_data(src, &header, &data[HEADER_SIZE..], data.len()),
            PACKET_DISCONNECT => self.handle_disconnect_packet(src),
            PACKET_HEARTBEAT => self.handle_heartbeat(src, &header, data.len()),
            _ => {}
        }
    }

    fn handle_connect(&mut self, src: SocketAddr, header: &PacketHeader, packet_len: usize) {
        if !self.is_host {
            return;
        }
        let id = if let Some(existing) = self.addr_to_id.get(&src) {
            *existing
        } else {
            if self.connections.len() >= self.capabilities.max_connections as usize {
                return;
            }
            let id = self.allocate_id();
            let mut conn = UdpConnection {
                id,
                addr: src,
                state: ConnectionState::Connected,
                reliability: ReliabilityLayer::new(),
                metrics: NetworkStatsTracker::new(),
                last_recv: Instant::now(),
            };
            conn.metrics.record_received_packet(packet_len);
            conn.reliability.process_incoming_header(header);
            Self::record_rtt_samples(&mut self.stats, &mut conn);
            self.addr_to_id.insert(src, id);
            self.connections.insert(id.0, conn);
            self.events.push(NetworkEvent::Connected { conn: id });
            id
        };

        if let Some(conn) = self.connections.get_mut(&id.0) {
            let ack_header = conn
                .reliability
                .prepare_outgoing_header(PACKET_CONNECT_ACK, 0);
            let bytes = ack_header.encode();
            if let Some(ref socket) = self.socket {
                let _ = socket.send_to(&bytes, src);
                self.stats.record_sent_packet(bytes.len());
                conn.metrics.record_sent_packet(bytes.len());
            }
        }
    }

    fn handle_connect_ack(&mut self, src: SocketAddr, header: &PacketHeader, packet_len: usize) {
        if let Some(&id) = self.addr_to_id.get(&src) {
            if let Some(conn) = self.connections.get_mut(&id.0) {
                conn.metrics.record_received_packet(packet_len);
                if conn.state == ConnectionState::Connecting {
                    conn.state = ConnectionState::Connected;
                    conn.reliability.process_incoming_header(header);
                    Self::record_rtt_samples(&mut self.stats, conn);
                    conn.last_recv = Instant::now();
                    self.events.push(NetworkEvent::Connected { conn: id });
                }
            }
        }
    }

    fn handle_data(
        &mut self,
        src: SocketAddr,
        header: &PacketHeader,
        payload: &[u8],
        packet_len: usize,
    ) {
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
        conn.metrics.record_received_packet(packet_len);
        let is_new = conn.reliability.process_incoming_header(header);
        Self::record_rtt_samples(&mut self.stats, conn);

        if is_new && !payload.is_empty() {
            self.events.push(NetworkEvent::Received {
                conn: id,
                channel: Channel(header.channel),
                data: payload.to_vec(),
            });
        }
    }

    fn handle_disconnect_packet(&mut self, src: SocketAddr) {
        if let Some(id) = self.addr_to_id.remove(&src) {
            if let Some(mut conn) = self.connections.remove(&id.0) {
                conn.state = ConnectionState::Disconnected;
                self.events.push(NetworkEvent::Disconnected {
                    conn: id,
                    reason: crate::core::providers::network_types::DisconnectReason::RemoteClose,
                });
            }
        }
    }

    fn handle_heartbeat(&mut self, src: SocketAddr, header: &PacketHeader, packet_len: usize) {
        if let Some(&id) = self.addr_to_id.get(&src) {
            if let Some(conn) = self.connections.get_mut(&id.0) {
                conn.last_recv = Instant::now();
                conn.metrics.record_received_packet(packet_len);
                conn.reliability.process_incoming_header(header);
                Self::record_rtt_samples(&mut self.stats, conn);
            }
        }
    }
    fn check_timeouts(&mut self) {
        let now = Instant::now();
        let timeout = std::time::Duration::from_secs(CONNECTION_TIMEOUT_SECS);
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
                    reason: crate::core::providers::network_types::DisconnectReason::Timeout,
                });
            }
        }
    }
    fn process_retransmits(&mut self) {
        let mut to_send: Vec<(ConnectionId, SocketAddr, Vec<u8>)> = Vec::new();
        let mut total_lost = 0u64;
        for conn in self.connections.values_mut() {
            let (resend_list, lost) = conn.reliability.check_retransmits();
            if lost > 0 {
                total_lost += lost as u64;
                conn.metrics.record_packets_lost(lost as u64);
            }

            for (_seq, data) in resend_list {
                let header = conn.reliability.prepare_outgoing_header(PACKET_DATA, 0);
                let mut packet = Vec::with_capacity(HEADER_SIZE + data.len());
                packet.extend_from_slice(&header.encode());
                packet.extend_from_slice(&data);
                to_send.push((conn.id, conn.addr, packet));
            }
        }
        if total_lost > 0 {
            self.stats.record_packets_lost(total_lost);
        }
        for (conn_id, addr, packet) in to_send {
            if let Some(conn) = self.connections.get_mut(&conn_id.0) {
                conn.metrics.record_sent_packet(packet.len());
            }
            if let Some(ref socket) = self.socket {
                let _ = socket.send_to(&packet, addr);
                self.stats.record_sent_packet(packet.len());
            }
        }
    }
}

#[rustfmt::skip]
impl Default for UdpNetProvider {
    fn default() -> Self { Self::new() }
}
#[rustfmt::skip]
impl Provider for UdpNetProvider {
    fn name(&self) -> &str { "udp" }
    fn version(&self) -> &str { "0.1.0" }
    fn capabilities(&self) -> Box<dyn std::any::Any> { Box::new(self.capabilities.clone()) }
}

impl ProviderLifecycle for UdpNetProvider {
    #[rustfmt::skip]
    fn init(&mut self) -> GoudResult<()> { Ok(()) }
    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        let mut incoming: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
        {
            let Some(socket) = self.socket.as_ref() else {
                return Ok(());
            };
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
        let addrs: Vec<SocketAddr> = self.connections.values().map(|c| c.addr).collect();
        for addr in addrs {
            let header = PacketHeader {
                sequence: 0,
                ack: 0,
                ack_bitfield: 0,
                packet_type: PACKET_DISCONNECT,
                channel: 0,
            };
            if let Some(ref socket) = self.socket {
                let _ = socket.send_to(&header.encode(), addr);
            }
        }
        self.connections.clear();
        self.addr_to_id.clear();
        self.socket = None;
    }
}
impl NetworkProvider for UdpNetProvider {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        if self.socket.is_some() {
            return Err(net_err("Already hosting or connected".into()));
        }
        let addr = format!("{}:{}", config.bind_address, config.port);
        let socket = UdpSocket::bind(&addr)
            .map_err(|e| net_err(format!("Failed to bind {}: {}", addr, e)))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| net_err(format!("Failed to set non-blocking: {}", e)))?;
        self.socket = Some(socket);
        self.is_host = true;
        Ok(())
    }
    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
        if self.socket.is_none() {
            let socket = UdpSocket::bind("0.0.0.0:0")
                .map_err(|e| net_err(format!("Failed to bind ephemeral: {}", e)))?;
            socket
                .set_nonblocking(true)
                .map_err(|e| net_err(format!("Failed to set non-blocking: {}", e)))?;
            self.socket = Some(socket);
        }
        let remote: SocketAddr = addr
            .parse()
            .map_err(|e| net_err(format!("Invalid address '{}': {}", addr, e)))?;
        let id = self.allocate_id();
        let mut conn = UdpConnection {
            id,
            addr: remote,
            state: ConnectionState::Connecting,
            reliability: ReliabilityLayer::new(),
            metrics: NetworkStatsTracker::new(),
            last_recv: Instant::now(),
        };
        let header = conn.reliability.prepare_outgoing_header(PACKET_CONNECT, 0);
        let bytes = header.encode();
        self.addr_to_id.insert(remote, id);
        self.connections.insert(id.0, conn);
        if let Some(conn) = self.connections.get_mut(&id.0) {
            conn.metrics.record_sent_packet(bytes.len());
        }
        self.send_raw(remote, &bytes)?;
        Ok(id)
    }
    fn disconnect(&mut self, conn_id: ConnectionId) -> GoudResult<()> {
        let conn = self
            .connections
            .remove(&conn_id.0)
            .ok_or(net_err(format!("Unknown connection {:?}", conn_id)))?;
        self.addr_to_id.remove(&conn.addr);
        let header = PacketHeader {
            sequence: 0,
            ack: 0,
            ack_bitfield: 0,
            packet_type: PACKET_DISCONNECT,
            channel: 0,
        };
        if let Some(ref socket) = self.socket {
            let _ = socket.send_to(&header.encode(), conn.addr);
        }
        self.events.push(NetworkEvent::Disconnected {
            conn: conn_id,
            reason: crate::core::providers::network_types::DisconnectReason::LocalClose,
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
            .ok_or(net_err(format!("Unknown connection {:?}", conn_id)))?;
        if conn.state != ConnectionState::Connected {
            return Err(net_err("Connection not established".into()));
        }
        let header = conn
            .reliability
            .prepare_outgoing_header(PACKET_DATA, channel.0);
        let seq = header.sequence;
        let mut packet = Vec::with_capacity(HEADER_SIZE + data.len());
        packet.extend_from_slice(&header.encode());
        packet.extend_from_slice(data);
        if channel.0 == 0 {
            conn.reliability.queue_reliable(seq, data.to_vec());
        }
        let addr = conn.addr;
        conn.metrics.record_sent_packet(packet.len());
        self.send_raw(addr, &packet)
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
            .map(|c| c.metrics.snapshot_connection())
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
impl std::fmt::Debug for UdpNetProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UdpNetProvider")
            .field("is_host", &self.is_host)
            .field("connections", &self.connections.len())
            .field("has_socket", &self.socket.is_some())
            .finish()
    }
}
#[cfg(test)]
#[path = "udp_network_tests.rs"]
mod tests;
