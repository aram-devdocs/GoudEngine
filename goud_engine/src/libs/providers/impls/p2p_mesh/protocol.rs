//! Mesh protocol message handling for the P2P mesh layer.

use crate::core::providers::network_types::{Channel, ConnectionId, NetworkEvent, P2pEvent};
use crate::libs::error::GoudResult;

use super::{P2pMesh, PeerConnectionMode};

// ---------------------------------------------------------------------------
// Internal mesh protocol message types
// ---------------------------------------------------------------------------

/// Mesh-internal control messages prefixed to transport payloads.
///
/// Control messages use channel 0 (reliable-ordered) and are prefixed with
/// a single discriminant byte so the mesh layer can distinguish them from
/// user data.
pub(super) const MESH_MSG_PEER_LIST: u8 = 0xF0;
pub(super) const MESH_MSG_JOIN_REQUEST: u8 = 0xF1;
pub(super) const MESH_MSG_JOIN_ACCEPT: u8 = 0xF2;
pub(super) const MESH_MSG_LEAVE: u8 = 0xF3;
pub(super) const MESH_MSG_USER_DATA: u8 = 0xF4;
pub(super) const MESH_MSG_RELAY: u8 = 0xF5;

impl P2pMesh {
    /// Process transport-level events and translate them into mesh events.
    pub(super) fn process_transport_events(&mut self) {
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

    /// Encode a peer list as bytes: [PEER_LIST_MSG][count:u32][peer_id:u64]...[self_id:u64]
    pub(super) fn encode_peer_list(&self) -> Vec<u8> {
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
    pub(super) fn send_join_accept(
        &mut self,
        conn_id: ConnectionId,
        assigned_id: super::PeerId,
    ) -> GoudResult<()> {
        let mut payload = Vec::with_capacity(1 + 8 + 8);
        payload.push(MESH_MSG_JOIN_ACCEPT);
        payload.extend_from_slice(&assigned_id.to_le_bytes());
        payload.extend_from_slice(&self.host_id.to_le_bytes());
        self.transport.send(conn_id, Channel(0), &payload)
    }

    /// Send the current peer list to a newly joined peer.
    pub(super) fn send_peer_list(&mut self, conn_id: ConnectionId) -> GoudResult<()> {
        let peer_list = self.encode_peer_list();
        self.transport.send(conn_id, Channel(0), &peer_list)
    }

    pub(super) fn send_via_relay(
        &mut self,
        target_peer: super::PeerId,
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
}
