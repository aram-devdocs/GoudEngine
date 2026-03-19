//! Incoming packet handling, connection handshakes, timeouts, and retransmits.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::core::providers::network_types::{
    Channel, ConnectionState, DisconnectReason, NetworkEvent,
};
use crate::libs::error::GoudResult;

use super::framing::{decode_frame, encode_frame, WebRtcConnection, RELIABLE_HEADER_SIZE};
use super::{net_err, WebRtcNetProvider, CONNECTION_TIMEOUT_SECS};

impl WebRtcNetProvider {
    pub(super) fn send_raw(&mut self, addr: SocketAddr, data: &[u8]) -> GoudResult<()> {
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

    pub(super) fn handle_recv(&mut self, src: SocketAddr, data: &[u8]) {
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

    pub(super) fn check_timeouts(&mut self) {
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

    pub(super) fn process_retransmits(&mut self) {
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
