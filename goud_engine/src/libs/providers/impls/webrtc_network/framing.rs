//! Frame encoding/decoding and reliability for WebRTC data channels.

use std::collections::HashMap;
use std::time::Instant;

use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, NetworkStatsTracker,
};

/// Frame layout for WebRTC data channel messages:
///   [4 bytes: frame length (BE)] [1 byte: channel] [payload]
///
/// Reliable channel (0) additionally tracks sequence numbers using
/// a simple header prepended to the payload.
///
/// Reliable sub-frame:
///   [4 bytes: sequence (BE)] [4 bytes: ack (BE)] [4 bytes: ack_bitfield (BE)] [payload]
pub(super) const RELIABLE_HEADER_SIZE: usize = 12;

pub(super) fn encode_frame(channel: Channel, data: &[u8]) -> Vec<u8> {
    let frame_len = (data.len() + 1) as u32;
    let mut frame = Vec::with_capacity(4 + frame_len as usize);
    frame.extend_from_slice(&frame_len.to_be_bytes());
    frame.push(channel.0);
    frame.extend_from_slice(data);
    frame
}

pub(super) fn decode_frame(data: &[u8]) -> Option<(Channel, Vec<u8>)> {
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

pub(super) struct WebRtcConnection {
    pub(super) id: ConnectionId,
    pub(super) addr: std::net::SocketAddr,
    pub(super) state: ConnectionState,
    pub(super) stats: NetworkStatsTracker,
    pub(super) last_recv: Instant,
    /// Outgoing sequence number for reliable channel.
    pub(super) send_seq: u32,
    /// Highest received sequence number for reliable channel.
    pub(super) recv_seq: u32,
    /// Ack bitfield for reliable channel.
    pub(super) ack_bitfield: u32,
    /// Buffered reliable payloads awaiting ack, keyed by sequence.
    pub(super) reliable_buffer: HashMap<u32, (Vec<u8>, Instant)>,
}

impl WebRtcConnection {
    pub(super) fn new(id: ConnectionId, addr: std::net::SocketAddr) -> Self {
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

    pub(super) fn next_seq(&mut self) -> u32 {
        self.send_seq = self.send_seq.wrapping_add(1);
        self.send_seq
    }

    pub(super) fn process_ack(&mut self, ack: u32, ack_bits: u32) {
        self.reliable_buffer.remove(&ack);
        for i in 0..32u32 {
            if ack_bits & (1 << i) != 0 {
                self.reliable_buffer.remove(&ack.wrapping_sub(i + 1));
            }
        }
    }

    pub(super) fn record_incoming_seq(&mut self, seq: u32) -> bool {
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
