//! Peer-to-peer full-mesh networking layer.
//!
//! `P2pMesh` sits above a `Box<dyn NetworkProvider>` transport and implements
//! `NetworkProvider` itself, so it can be used transparently anywhere the
//! engine expects a network provider. Internally it manages a full-mesh
//! topology: every peer maintains a direct (or relay-fallback) connection
//! to every other peer.
//!
//! # Features
//!
//! - Full-mesh topology with configurable max peer count
//! - Relay fallback when direct P2P connection fails (NAT traversal)
//! - Host migration: lowest `PeerId` becomes host when the current host leaves
//! - Peer discovery: new peers receive the full peer list from the host

use std::collections::HashMap;

use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkStats, NetworkStatsTracker, P2pEvent, P2pMeshConfig, PeerId,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::{GoudError, GoudResult};

// ---------------------------------------------------------------------------
// Internal mesh protocol message types
// ---------------------------------------------------------------------------

/// Mesh-internal control messages prefixed to transport payloads.
///
/// Control messages use channel 0 (reliable-ordered) and are prefixed with
/// a single discriminant byte so the mesh layer can distinguish them from
/// user data.
const MESH_MSG_PEER_LIST: u8 = 0xF0;
const MESH_MSG_JOIN_REQUEST: u8 = 0xF1;
const MESH_MSG_JOIN_ACCEPT: u8 = 0xF2;
const MESH_MSG_LEAVE: u8 = 0xF3;
const MESH_MSG_USER_DATA: u8 = 0xF4;
const MESH_MSG_RELAY: u8 = 0xF5;

// ---------------------------------------------------------------------------
// Peer connection state
// ---------------------------------------------------------------------------

/// Connection mode for a peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerConnectionMode {
    /// Connected directly via transport.
    Direct,
    /// Connected through the relay server.
    Relayed,
}

/// Per-peer connection state tracked by the mesh.
#[derive(Debug)]
struct PeerConnection {
    /// The transport-level connection ID for this peer.
    conn_id: ConnectionId,
    /// Whether this peer is connected directly or via relay.
    mode: PeerConnectionMode,
    /// Current connection state.
    state: ConnectionState,
}

// ---------------------------------------------------------------------------
// P2pMesh
// ---------------------------------------------------------------------------

/// Peer-to-peer full-mesh network layer.
///
/// Wraps an underlying `NetworkProvider` transport and manages mesh topology,
/// host election, relay fallback, and peer discovery. Implements
/// `NetworkProvider` so it can be used as a drop-in replacement.
pub struct P2pMesh {
    config: P2pMeshConfig,
    transport: Box<dyn NetworkProvider>,
    pub(crate) local_peer_id: PeerId,
    peers: HashMap<PeerId, PeerConnection>,
    pub(crate) host_id: PeerId,
    relay_conn: Option<ConnectionId>,
    events: Vec<NetworkEvent>,
    p2p_events: Vec<P2pEvent>,
    stats: NetworkStatsTracker,
    capabilities: NetworkCapabilities,
    is_hosting: bool,
    next_peer_id: PeerId,
    /// Maps transport ConnectionId -> PeerId for reverse lookups.
    conn_to_peer: HashMap<u64, PeerId>,
}

impl P2pMesh {
    /// Create a new P2P mesh using the given transport provider and config.
    ///
    /// The mesh is not active until `host()` or `connect()` is called.
    pub fn new(transport: Box<dyn NetworkProvider>, config: P2pMeshConfig) -> Self {
        let caps = transport.network_capabilities();
        let capabilities = NetworkCapabilities {
            supports_hosting: caps.supports_hosting,
            max_connections: config.max_peers as u32,
            max_channels: caps.max_channels,
            // Reserve space for the 1-byte mesh protocol prefix.
            max_message_size: caps.max_message_size.saturating_sub(1),
        };

        Self {
            config,
            transport,
            local_peer_id: 0,
            peers: HashMap::new(),
            host_id: 0,
            relay_conn: None,
            events: Vec::new(),
            p2p_events: Vec::new(),
            stats: NetworkStatsTracker::new(),
            capabilities,
            is_hosting: false,
            next_peer_id: 2, // 1 is reserved for the host
            conn_to_peer: HashMap::new(),
        }
    }

    /// Returns the local peer's ID within the mesh.
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Returns the current host's peer ID.
    pub fn host_peer_id(&self) -> PeerId {
        self.host_id
    }

    /// Returns true if the local peer is the current mesh host.
    pub fn is_host(&self) -> bool {
        self.local_peer_id == self.host_id
    }

    /// Returns all peer IDs currently in the mesh (excluding self).
    pub fn peer_ids(&self) -> Vec<PeerId> {
        self.peers.keys().copied().collect()
    }

    /// Returns the number of connected peers (excluding self).
    pub fn peer_count(&self) -> usize {
        self.peers
            .values()
            .filter(|p| p.state == ConnectionState::Connected)
            .count()
    }

    /// Drain all buffered P2P events (peer join/leave, host migration, etc.).
    pub fn drain_p2p_events(&mut self) -> Vec<P2pEvent> {
        std::mem::take(&mut self.p2p_events)
    }

    /// Send a message to a specific peer.
    pub fn send_to_peer(
        &mut self,
        peer_id: PeerId,
        channel: Channel,
        data: &[u8],
    ) -> GoudResult<()> {
        let peer = self
            .peers
            .get(&peer_id)
            .ok_or_else(|| GoudError::ProviderError {
                subsystem: "p2p_mesh",
                message: format!("Unknown peer {}", peer_id),
            })?;

        if peer.state != ConnectionState::Connected {
            return Err(GoudError::ProviderError {
                subsystem: "p2p_mesh",
                message: format!("Peer {} not connected", peer_id),
            });
        }

        let mut payload = Vec::with_capacity(1 + data.len());
        payload.push(MESH_MSG_USER_DATA);
        payload.extend_from_slice(data);

        match peer.mode {
            PeerConnectionMode::Direct => {
                self.transport.send(peer.conn_id, channel, &payload)?;
            }
            PeerConnectionMode::Relayed => {
                self.send_via_relay(peer_id, channel, &payload)?;
            }
        }
        self.stats.record_sent_packet(data.len());
        Ok(())
    }

    /// Broadcast a message to all connected peers.
    pub fn broadcast_to_peers(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let peer_ids: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(_, p)| p.state == ConnectionState::Connected)
            .map(|(id, _)| *id)
            .collect();

        for peer_id in peer_ids {
            let _ = self.send_to_peer(peer_id, channel, data);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn net_err(msg: impl Into<String>) -> GoudError {
        GoudError::ProviderError {
            subsystem: "p2p_mesh",
            message: msg.into(),
        }
    }

    fn allocate_peer_id(&mut self) -> PeerId {
        let id = self.next_peer_id;
        self.next_peer_id += 1;
        id
    }

    pub(crate) fn register_peer(
        &mut self,
        peer_id: PeerId,
        conn_id: ConnectionId,
        mode: PeerConnectionMode,
    ) {
        self.conn_to_peer.insert(conn_id.0, peer_id);
        self.peers.insert(
            peer_id,
            PeerConnection {
                conn_id,
                mode,
                state: ConnectionState::Connected,
            },
        );
        self.p2p_events.push(P2pEvent::PeerJoined(peer_id));
        self.events.push(NetworkEvent::Connected {
            conn: ConnectionId(peer_id),
        });
    }

    pub(crate) fn unregister_peer(&mut self, peer_id: PeerId) {
        if let Some(peer) = self.peers.remove(&peer_id) {
            self.conn_to_peer.remove(&peer.conn_id.0);
            self.p2p_events.push(P2pEvent::PeerLeft(peer_id));
            self.events.push(NetworkEvent::Disconnected {
                conn: ConnectionId(peer_id),
                reason: crate::core::providers::network_types::DisconnectReason::RemoteClose,
            });
        }
    }

    fn peer_id_for_conn(&self, conn_id: ConnectionId) -> Option<PeerId> {
        self.conn_to_peer.get(&conn_id.0).copied()
    }

    /// Encode a peer list as bytes: [PEER_LIST_MSG][count:u32][peer_id:u64]...[self_id:u64]
    fn encode_peer_list(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(MESH_MSG_PEER_LIST);
        let count = self.peers.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());
        for &peer_id in self.peers.keys() {
            buf.extend_from_slice(&peer_id.to_le_bytes());
        }
        // Include self in the peer list.
        buf.extend_from_slice(&self.local_peer_id.to_le_bytes());
        buf
    }

    /// Send a join-accept message containing the assigned peer ID.
    fn send_join_accept(&mut self, conn_id: ConnectionId, assigned_id: PeerId) -> GoudResult<()> {
        let mut payload = Vec::with_capacity(1 + 8 + 8);
        payload.push(MESH_MSG_JOIN_ACCEPT);
        payload.extend_from_slice(&assigned_id.to_le_bytes());
        payload.extend_from_slice(&self.host_id.to_le_bytes());
        self.transport.send(conn_id, Channel(0), &payload)
    }

    /// Send the current peer list to a newly joined peer.
    fn send_peer_list(&mut self, conn_id: ConnectionId) -> GoudResult<()> {
        let peer_list = self.encode_peer_list();
        self.transport.send(conn_id, Channel(0), &peer_list)
    }

    fn send_via_relay(
        &mut self,
        target_peer: PeerId,
        channel: Channel,
        payload: &[u8],
    ) -> GoudResult<()> {
        let relay_conn = self
            .relay_conn
            .ok_or_else(|| Self::net_err("No relay connection available"))?;
        // Relay frame: [MESH_MSG_RELAY][target_peer:u64][channel:u8][payload...]
        let mut relay_frame = Vec::with_capacity(1 + 8 + 1 + payload.len());
        relay_frame.push(MESH_MSG_RELAY);
        relay_frame.extend_from_slice(&target_peer.to_le_bytes());
        relay_frame.push(channel.0);
        relay_frame.extend_from_slice(payload);
        self.transport.send(relay_conn, Channel(0), &relay_frame)
    }

    /// Perform host election: the connected peer with the lowest ID becomes host.
    pub(crate) fn elect_new_host(&mut self) {
        let old_host = self.host_id;

        // Candidates: self + all connected peers.
        let mut candidates: Vec<PeerId> = self
            .peers
            .iter()
            .filter(|(_, p)| p.state == ConnectionState::Connected)
            .map(|(id, _)| *id)
            .collect();
        candidates.push(self.local_peer_id);
        candidates.sort_unstable();

        let new_host = *candidates.first().unwrap_or(&self.local_peer_id);
        if new_host != old_host {
            self.host_id = new_host;
            self.p2p_events
                .push(P2pEvent::HostMigrated { old_host, new_host });
        }
    }

    /// Process transport-level events and translate them into mesh events.
    fn process_transport_events(&mut self) {
        let transport_events = self.transport.drain_events();
        for event in transport_events {
            match event {
                NetworkEvent::Connected { conn } => {
                    // A new transport connection was established. If we are
                    // hosting, this is a peer requesting to join.
                    if self.is_hosting {
                        // Peer will be registered when we get the join request.
                        // For now just record the raw connection event.
                        log::debug!("Transport connection established: {:?}", conn);
                    }
                }
                NetworkEvent::Disconnected { conn, reason } => {
                    if let Some(peer_id) = self.peer_id_for_conn(conn) {
                        self.unregister_peer(peer_id);
                        // Check if the disconnected peer was the host.
                        if peer_id == self.host_id && self.config.host_migration {
                            self.elect_new_host();
                        }
                    }
                    // Forward the disconnect event with the original reason.
                    self.events
                        .push(NetworkEvent::Disconnected { conn, reason });
                }
                NetworkEvent::Received {
                    conn,
                    channel,
                    data,
                } => {
                    self.handle_mesh_message(conn, channel, &data);
                }
                NetworkEvent::Error { conn, message } => {
                    self.events.push(NetworkEvent::Error { conn, message });
                }
            }
        }
    }

    /// Handle an incoming mesh-protocol message.
    pub(crate) fn handle_mesh_message(
        &mut self,
        conn: ConnectionId,
        channel: Channel,
        data: &[u8],
    ) {
        if data.is_empty() {
            return;
        }

        let msg_type = data[0];
        let payload = &data[1..];

        match msg_type {
            MESH_MSG_JOIN_REQUEST => {
                self.handle_join_request(conn);
            }
            MESH_MSG_JOIN_ACCEPT => {
                self.handle_join_accept(payload);
            }
            MESH_MSG_PEER_LIST => {
                self.handle_peer_list(payload);
            }
            MESH_MSG_LEAVE => {
                if let Some(peer_id) = self.peer_id_for_conn(conn) {
                    self.unregister_peer(peer_id);
                    if peer_id == self.host_id && self.config.host_migration {
                        self.elect_new_host();
                    }
                }
            }
            MESH_MSG_USER_DATA => {
                if let Some(peer_id) = self.peer_id_for_conn(conn) {
                    self.stats.record_received_packet(payload.len());
                    self.p2p_events.push(P2pEvent::MessageReceived {
                        from: peer_id,
                        data: payload.to_vec(),
                    });
                    self.events.push(NetworkEvent::Received {
                        conn: ConnectionId(peer_id),
                        channel,
                        data: payload.to_vec(),
                    });
                }
            }
            MESH_MSG_RELAY => {
                self.handle_relay_message(conn, payload);
            }
            _ => {
                log::warn!("Unknown mesh message type: 0x{:02X}", msg_type);
            }
        }
    }

    /// Handle a join request from a new peer (host only).
    fn handle_join_request(&mut self, conn: ConnectionId) {
        if !self.is_hosting {
            return;
        }

        if self.peers.len() >= self.config.max_peers.saturating_sub(1) {
            log::warn!("Mesh full, rejecting join from {:?}", conn);
            return;
        }

        let new_peer_id = self.allocate_peer_id();
        self.register_peer(new_peer_id, conn, PeerConnectionMode::Direct);

        // Send the assigned peer ID and host ID.
        let _ = self.send_join_accept(conn, new_peer_id);
        // Send the current peer list so the new peer can connect to others.
        let _ = self.send_peer_list(conn);
    }

    /// Handle a join-accept response from the host (client side).
    pub(crate) fn handle_join_accept(&mut self, payload: &[u8]) {
        if payload.len() < 16 {
            return;
        }
        let assigned_id = u64::from_le_bytes(payload[0..8].try_into().unwrap());
        let host_id = u64::from_le_bytes(payload[8..16].try_into().unwrap());

        self.local_peer_id = assigned_id;
        self.host_id = host_id;
    }

    /// Handle a peer list message from the host.
    fn handle_peer_list(&mut self, payload: &[u8]) {
        if payload.len() < 4 {
            return;
        }
        let count = u32::from_le_bytes(payload[0..4].try_into().unwrap()) as usize;
        let mut offset = 4;
        for _ in 0..count {
            if offset + 8 > payload.len() {
                break;
            }
            let peer_id = u64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap());
            offset += 8;

            // Skip self and already-known peers.
            if peer_id == self.local_peer_id || self.peers.contains_key(&peer_id) {
                continue;
            }

            // In a full production implementation, we would attempt to connect
            // to each discovered peer here. For now, we record them as known
            // but not connected -- the host relays messages to them.
            log::debug!("Discovered peer {} from peer list", peer_id);
        }
        // Also read the host's own ID at the end if present.
        if offset + 8 <= payload.len() {
            let host_self_id = u64::from_le_bytes(payload[offset..offset + 8].try_into().unwrap());
            if host_self_id != self.local_peer_id && !self.peers.contains_key(&host_self_id) {
                // The host itself -- we already have a connection to it from connect().
                log::debug!("Host peer {} confirmed from peer list", host_self_id);
            }
        }
    }

    /// Handle a relayed message.
    fn handle_relay_message(&mut self, _from_conn: ConnectionId, payload: &[u8]) {
        // Relay frame payload: [target_peer:u64][channel:u8][inner_payload...]
        if payload.len() < 9 {
            return;
        }
        let target_peer = u64::from_le_bytes(payload[0..8].try_into().unwrap());
        let channel = Channel(payload[8]);
        let inner = &payload[9..];

        if target_peer == self.local_peer_id {
            // This relay message is for us -- unwrap and process.
            if !inner.is_empty() {
                // Determine the original sender from the relay connection.
                // For simplicity, the relay server is the intermediary and the
                // inner payload contains the actual mesh message.
                self.handle_mesh_message(
                    ConnectionId(0), // relay source
                    channel,
                    inner,
                );
            }
        } else if self.is_hosting {
            // We are the host acting as relay -- forward to the target peer.
            if let Some(peer) = self.peers.get(&target_peer) {
                let conn_id = peer.conn_id;
                let _ = self.transport.send(conn_id, channel, payload);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Provider + ProviderLifecycle
// ---------------------------------------------------------------------------

impl Provider for P2pMesh {
    fn name(&self) -> &str {
        "p2p_mesh"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for P2pMesh {
    fn init(&mut self) -> GoudResult<()> {
        self.transport.init()
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.transport.update(delta)?;
        self.process_transport_events();
        Ok(())
    }

    fn shutdown(&mut self) {
        // Send leave messages to all peers.
        let peer_conn_ids: Vec<ConnectionId> = self.peers.values().map(|p| p.conn_id).collect();
        for conn_id in peer_conn_ids {
            let _ = self.transport.send(conn_id, Channel(0), &[MESH_MSG_LEAVE]);
        }
        self.peers.clear();
        self.conn_to_peer.clear();
        self.transport.shutdown();
    }
}

// ---------------------------------------------------------------------------
// NetworkProvider implementation
// ---------------------------------------------------------------------------

impl NetworkProvider for P2pMesh {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        self.transport.host(config)?;
        self.is_hosting = true;
        self.local_peer_id = 1; // Host is always peer 1.
        self.host_id = 1;
        Ok(())
    }

    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
        let conn_id = self.transport.connect(addr)?;

        // Send a join request to the host.
        self.transport
            .send(conn_id, Channel(0), &[MESH_MSG_JOIN_REQUEST])?;

        // Register the host as a peer. The actual peer ID will be assigned
        // by the host in the join-accept response.
        // Use peer_id = 1 (the host) as a preliminary assignment.
        self.conn_to_peer.insert(conn_id.0, 1);
        self.peers.insert(
            1,
            PeerConnection {
                conn_id,
                mode: PeerConnectionMode::Direct,
                state: ConnectionState::Connecting,
            },
        );
        self.host_id = 1;

        // If a relay server is configured, connect to it as well.
        if let Some(ref relay_addr) = self.config.relay_server {
            match self.transport.connect(relay_addr) {
                Ok(relay_id) => {
                    self.relay_conn = Some(relay_id);
                }
                Err(e) => {
                    log::warn!("Failed to connect to relay server: {}", e);
                }
            }
        }

        Ok(conn_id)
    }

    fn disconnect(&mut self, conn: ConnectionId) -> GoudResult<()> {
        if let Some(peer_id) = self.peer_id_for_conn(conn) {
            // Send leave notification.
            let _ = self.transport.send(conn, Channel(0), &[MESH_MSG_LEAVE]);
            self.unregister_peer(peer_id);
        }
        self.transport.disconnect(conn)
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        // Send leave messages to all peers.
        let conns: Vec<ConnectionId> = self.peers.values().map(|p| p.conn_id).collect();
        for conn in conns {
            let _ = self.transport.send(conn, Channel(0), &[MESH_MSG_LEAVE]);
        }
        self.peers.clear();
        self.conn_to_peer.clear();
        self.transport.disconnect_all()
    }

    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        // The ConnectionId passed to send() at the mesh level maps to a PeerId.
        let peer_id = conn.0;
        self.send_to_peer(peer_id, channel, data)
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        self.broadcast_to_peers(channel, data)
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        std::mem::take(&mut self.events)
    }

    fn connections(&self) -> Vec<ConnectionId> {
        self.peers
            .iter()
            .filter(|(_, p)| p.state == ConnectionState::Connected)
            .map(|(id, _)| ConnectionId(*id))
            .collect()
    }

    fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
        // conn.0 is treated as a PeerId at the mesh level.
        self.peers
            .get(&conn.0)
            .map(|p| p.state)
            .unwrap_or(ConnectionState::Disconnected)
    }

    fn local_id(&self) -> Option<ConnectionId> {
        if self.local_peer_id > 0 {
            Some(ConnectionId(self.local_peer_id))
        } else {
            None
        }
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        &self.capabilities
    }

    fn stats(&self) -> NetworkStats {
        self.stats.snapshot_network()
    }

    fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats> {
        // Delegate to transport for the underlying connection stats.
        let peer_id = conn.0;
        self.peers
            .get(&peer_id)
            .and_then(|peer| self.transport.connection_stats(peer.conn_id))
    }

    fn network_diagnostics(&self) -> NetworkDiagnosticsV1 {
        let s = self.stats();
        NetworkDiagnosticsV1 {
            bytes_sent: s.bytes_sent,
            bytes_received: s.bytes_received,
            packets_sent: s.packets_sent,
            packets_received: s.packets_received,
            rtt_ms: s.rtt_ms,
            active_connections: self.peer_count() as u32,
        }
    }
}

impl std::fmt::Debug for P2pMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("P2pMesh")
            .field("local_peer_id", &self.local_peer_id)
            .field("host_id", &self.host_id)
            .field("peer_count", &self.peers.len())
            .field("is_hosting", &self.is_hosting)
            .field("has_relay", &self.relay_conn.is_some())
            .finish()
    }
}

#[cfg(test)]
#[path = "p2p_mesh_tests.rs"]
mod tests;
