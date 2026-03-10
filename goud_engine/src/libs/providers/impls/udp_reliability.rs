//! Reliability layer for UDP transport.
//!
//! Provides packet headers with sequence numbers, acknowledgment tracking,
//! and retransmission of reliable packets over unreliable UDP datagrams.

use std::time::{Duration, Instant};

// =============================================================================
// Packet Type Constants
// =============================================================================

/// Connection request from client to host.
pub const PACKET_CONNECT: u8 = 1;
/// Connection acknowledgment from host to client.
pub const PACKET_CONNECT_ACK: u8 = 2;
/// Application data payload.
pub const PACKET_DATA: u8 = 3;
/// Graceful disconnect notification.
pub const PACKET_DISCONNECT: u8 = 4;
/// Keep-alive heartbeat.
pub const PACKET_HEARTBEAT: u8 = 5;

/// Size of the encoded packet header in bytes.
pub const HEADER_SIZE: usize = 8;

// =============================================================================
// PacketHeader
// =============================================================================

/// Fixed-size 8-byte header prepended to every UDP packet.
///
/// Layout: seq(2) + ack(2) + ack_bitfield(2) + type(1) + channel(1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    /// Sender's monotonically increasing sequence number.
    pub sequence: u16,
    /// Latest remote sequence number acknowledged by sender.
    pub ack: u16,
    /// Bitmask: bit N set means `ack - N - 1` was also received.
    pub ack_bitfield: u16,
    /// Packet type (one of `PACKET_*` constants).
    pub packet_type: u8,
    /// Channel index (0 = reliable-ordered, 1+ = unreliable).
    pub channel: u8,
}

impl PacketHeader {
    /// Encode the header into an 8-byte array (big-endian).
    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..2].copy_from_slice(&self.sequence.to_be_bytes());
        buf[2..4].copy_from_slice(&self.ack.to_be_bytes());
        buf[4..6].copy_from_slice(&self.ack_bitfield.to_be_bytes());
        buf[6] = self.packet_type;
        buf[7] = self.channel;
        buf
    }

    /// Decode a header from a byte slice. Returns `None` if too short.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < HEADER_SIZE {
            return None;
        }
        Some(Self {
            sequence: u16::from_be_bytes([bytes[0], bytes[1]]),
            ack: u16::from_be_bytes([bytes[2], bytes[3]]),
            ack_bitfield: u16::from_be_bytes([bytes[4], bytes[5]]),
            packet_type: bytes[6],
            channel: bytes[7],
        })
    }
}

// =============================================================================
// ReliabilityLayer
// =============================================================================

/// Tracks a reliable packet awaiting acknowledgment.
#[derive(Debug, Clone)]
struct PendingPacket {
    sequence: u16,
    data: Vec<u8>,
    sent_at: Instant,
    retransmits: u32,
}

/// Per-connection reliability tracking.
///
/// Manages sequence numbering, ack generation, and retransmission of
/// reliable packets. Each `UdpConnection` owns one `ReliabilityLayer`.
#[derive(Debug)]
pub struct ReliabilityLayer {
    /// Next outgoing sequence number.
    local_sequence: u16,
    /// Highest remote sequence number received so far.
    remote_sequence: u16,
    /// Bitmask of received remote sequences relative to `remote_sequence`.
    received_bits: u16,
    /// Whether we have received at least one remote packet.
    has_remote: bool,
    /// Reliable packets awaiting acknowledgment.
    pending: Vec<PendingPacket>,
    /// RTT samples produced while processing acknowledgments.
    acked_rtt_samples_ms: Vec<f32>,
    /// Retransmission timeout duration.
    rto: Duration,
    /// Maximum retransmit attempts before considering the packet lost.
    max_retransmits: u32,
}

impl Default for ReliabilityLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReliabilityLayer {
    /// Create a new reliability layer with default settings.
    pub fn new() -> Self {
        Self {
            local_sequence: 0,
            remote_sequence: 0,
            received_bits: 0,
            has_remote: false,
            pending: Vec::new(),
            acked_rtt_samples_ms: Vec::new(),
            rto: Duration::from_millis(100),
            max_retransmits: 10,
        }
    }

    /// Create a reliability layer with custom retransmission settings.
    pub fn with_config(rto_ms: u64, max_retransmits: u32) -> Self {
        Self {
            rto: Duration::from_millis(rto_ms),
            max_retransmits,
            ..Self::new()
        }
    }

    /// Prepare a header for an outgoing packet, incrementing the local sequence.
    pub fn prepare_outgoing_header(&mut self, packet_type: u8, channel: u8) -> PacketHeader {
        let seq = self.local_sequence;
        self.local_sequence = self.local_sequence.wrapping_add(1);
        PacketHeader {
            sequence: seq,
            ack: self.remote_sequence,
            ack_bitfield: self.received_bits,
            packet_type,
            channel,
        }
    }

    /// Process an incoming packet header: update remote tracking and ack pending packets.
    ///
    /// Returns `true` if this is a new (non-duplicate) packet.
    pub fn process_incoming_header(&mut self, header: &PacketHeader) -> bool {
        let seq = header.sequence;

        // Process the ack information in the header to mark our pending packets.
        self.mark_acked(header.ack, header.ack_bitfield);

        if !self.has_remote {
            // First packet received.
            self.remote_sequence = seq;
            self.received_bits = 0;
            self.has_remote = true;
            return true;
        }

        let diff = sequence_diff(seq, self.remote_sequence);

        if diff > 0 {
            // Newer sequence: shift the bitmap and set the new head.
            if diff <= 16 {
                self.received_bits = self.received_bits.checked_shl(diff as u32).unwrap_or(0);
                // The old remote_sequence is now at bit (diff - 1).
                self.received_bits |= 1 << (diff - 1);
            } else {
                // Gap too large; reset bitmap.
                self.received_bits = 0;
            }
            self.remote_sequence = seq;
            true
        } else if diff < 0 {
            // Older sequence: check if it fits in the bitmap.
            let age = (-diff) as u16;
            if age <= 16 {
                let bit = 1u16 << (age - 1);
                if self.received_bits & bit != 0 {
                    return false; // Duplicate.
                }
                self.received_bits |= bit;
                true
            } else {
                false // Too old.
            }
        } else {
            false // Exact duplicate of remote_sequence.
        }
    }

    /// Mark pending reliable packets as acknowledged based on remote ack info.
    pub fn mark_acked(&mut self, ack: u16, ack_bitfield: u16) {
        let now = Instant::now();
        let mut acked_samples = Vec::new();
        self.pending.retain(|p| {
            if p.sequence == ack {
                acked_samples.push(now.duration_since(p.sent_at).as_secs_f32() * 1000.0);
                return false; // Acked directly.
            }
            let diff = sequence_diff(ack, p.sequence);
            if diff > 0 && diff <= 16 {
                let bit = 1u16 << (diff - 1);
                if ack_bitfield & bit != 0 {
                    acked_samples.push(now.duration_since(p.sent_at).as_secs_f32() * 1000.0);
                    return false; // Acked via bitfield.
                }
            }
            true // Not yet acked.
        });
        self.acked_rtt_samples_ms.extend(acked_samples);
    }

    /// Queue a reliable packet for retransmission tracking.
    pub fn queue_reliable(&mut self, sequence: u16, data: Vec<u8>) {
        self.pending.push(PendingPacket {
            sequence,
            data,
            sent_at: Instant::now(),
            retransmits: 0,
        });
    }

    /// Check for packets needing retransmission.
    ///
    /// Returns a list of `(sequence, data)` pairs that should be resent,
    /// and the count of packets that exceeded `max_retransmits` (lost).
    pub fn check_retransmits(&mut self) -> (Vec<(u16, Vec<u8>)>, u32) {
        let now = Instant::now();
        let mut resend = Vec::new();
        let mut lost_count = 0u32;

        self.pending.retain_mut(|p| {
            if now.duration_since(p.sent_at) >= self.rto {
                if p.retransmits >= self.max_retransmits {
                    lost_count += 1;
                    return false; // Drop as lost.
                }
                p.retransmits += 1;
                p.sent_at = now;
                resend.push((p.sequence, p.data.clone()));
            }
            true
        });

        (resend, lost_count)
    }

    /// Number of pending reliable packets awaiting acknowledgment.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Drains RTT samples gathered while processing acknowledgments.
    pub fn drain_acked_rtt_samples_ms(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.acked_rtt_samples_ms)
    }
}

/// Compute signed difference between two sequence numbers with wraparound.
///
/// Returns positive if `a` is ahead of `b`, negative if behind.
fn sequence_diff(a: u16, b: u16) -> i32 {
    let diff = a.wrapping_sub(b) as i16;
    diff as i32
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_encode_decode_roundtrip() {
        let header = PacketHeader {
            sequence: 42,
            ack: 100,
            ack_bitfield: 0b1010_1010_1010_1010,
            packet_type: PACKET_DATA,
            channel: 0,
        };
        let bytes = header.encode();
        assert_eq!(bytes.len(), HEADER_SIZE);
        let decoded = PacketHeader::decode(&bytes).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_header_decode_too_short() {
        assert!(PacketHeader::decode(&[0u8; 7]).is_none());
        assert!(PacketHeader::decode(&[]).is_none());
    }

    #[test]
    fn test_header_all_packet_types() {
        for ptype in [
            PACKET_CONNECT,
            PACKET_CONNECT_ACK,
            PACKET_DATA,
            PACKET_DISCONNECT,
            PACKET_HEARTBEAT,
        ] {
            let h = PacketHeader {
                sequence: 0,
                ack: 0,
                ack_bitfield: 0,
                packet_type: ptype,
                channel: 1,
            };
            let decoded = PacketHeader::decode(&h.encode()).unwrap();
            assert_eq!(decoded.packet_type, ptype);
        }
    }

    #[test]
    fn test_sequence_diff_basic() {
        assert_eq!(sequence_diff(10, 5), 5);
        assert_eq!(sequence_diff(5, 10), -5);
        assert_eq!(sequence_diff(5, 5), 0);
    }

    #[test]
    fn test_sequence_diff_wraparound() {
        // 0 is 1 ahead of 65535 with wraparound
        assert_eq!(sequence_diff(0, u16::MAX), 1);
        // 65535 is 1 behind 0 with wraparound
        assert_eq!(sequence_diff(u16::MAX, 0), -1);
        // 2 is 5 ahead of 65533
        assert_eq!(sequence_diff(2, 65533), 5);
    }

    #[test]
    fn test_reliability_layer_outgoing_sequence_increments() {
        let mut layer = ReliabilityLayer::new();
        let h1 = layer.prepare_outgoing_header(PACKET_DATA, 0);
        assert_eq!(h1.sequence, 0);
        let h2 = layer.prepare_outgoing_header(PACKET_DATA, 0);
        assert_eq!(h2.sequence, 1);
    }

    #[test]
    fn test_reliability_layer_incoming_updates_remote() {
        let mut layer = ReliabilityLayer::new();
        let header = PacketHeader {
            sequence: 10,
            ack: 0,
            ack_bitfield: 0,
            packet_type: PACKET_DATA,
            channel: 0,
        };
        assert!(layer.process_incoming_header(&header));
        let out = layer.prepare_outgoing_header(PACKET_DATA, 0);
        assert_eq!(out.ack, 10);
    }

    #[test]
    fn test_reliability_layer_duplicate_detection() {
        let mut layer = ReliabilityLayer::new();
        let header = PacketHeader {
            sequence: 5,
            ack: 0,
            ack_bitfield: 0,
            packet_type: PACKET_DATA,
            channel: 0,
        };
        assert!(layer.process_incoming_header(&header));
        assert!(!layer.process_incoming_header(&header)); // Duplicate
    }

    #[test]
    fn test_reliability_layer_ack_bitfield_tracking() {
        let mut layer = ReliabilityLayer::new();

        // Receive packets 0, 1, 2, 3 in order
        for seq in 0..4u16 {
            let h = PacketHeader {
                sequence: seq,
                ack: 0,
                ack_bitfield: 0,
                packet_type: PACKET_DATA,
                channel: 0,
            };
            assert!(layer.process_incoming_header(&h));
        }

        // After receiving 0,1,2,3: remote_sequence=3, bits should have 2,1,0
        let out = layer.prepare_outgoing_header(PACKET_DATA, 0);
        assert_eq!(out.ack, 3);
        // bit 0 (seq 2), bit 1 (seq 1), bit 2 (seq 0) should be set
        assert!(out.ack_bitfield & 0b001 != 0); // seq 2
        assert!(out.ack_bitfield & 0b010 != 0); // seq 1
        assert!(out.ack_bitfield & 0b100 != 0); // seq 0
    }

    #[test]
    fn test_reliability_mark_acked_removes_pending() {
        let mut layer = ReliabilityLayer::new();
        layer.queue_reliable(5, vec![1, 2, 3]);
        layer.queue_reliable(6, vec![4, 5, 6]);
        assert_eq!(layer.pending_count(), 2);

        // Ack sequence 5 directly
        layer.mark_acked(5, 0);
        assert_eq!(layer.pending_count(), 1);

        // Ack sequence 6 via bitfield (ack=7, bit 0 set means seq 6)
        layer.mark_acked(7, 0b01);
        assert_eq!(layer.pending_count(), 0);
    }

    #[test]
    fn test_reliability_retransmission() {
        let mut layer = ReliabilityLayer::with_config(0, 10); // 0ms RTO for testing
        layer.queue_reliable(0, vec![42]);
        assert_eq!(layer.pending_count(), 1);

        let (resend, lost) = layer.check_retransmits();
        assert_eq!(lost, 0);
        assert_eq!(resend.len(), 1);
        assert_eq!(resend[0], (0, vec![42]));
        // Still pending after retransmit
        assert_eq!(layer.pending_count(), 1);
    }

    #[test]
    fn test_reliability_max_retransmits_drops_packet() {
        let mut layer = ReliabilityLayer::with_config(0, 2);
        layer.queue_reliable(0, vec![42]);

        // Exhaust retransmits
        for _ in 0..2 {
            let _ = layer.check_retransmits();
        }
        // Third check should mark as lost
        let (_, lost) = layer.check_retransmits();
        assert_eq!(lost, 1);
        assert_eq!(layer.pending_count(), 0);
    }

    #[test]
    fn test_reliability_out_of_order_incoming() {
        let mut layer = ReliabilityLayer::new();

        // Receive 5, then 3 (out of order)
        let h5 = PacketHeader {
            sequence: 5,
            ack: 0,
            ack_bitfield: 0,
            packet_type: PACKET_DATA,
            channel: 0,
        };
        let h3 = PacketHeader {
            sequence: 3,
            ack: 0,
            ack_bitfield: 0,
            packet_type: PACKET_DATA,
            channel: 0,
        };

        assert!(layer.process_incoming_header(&h5));
        assert!(layer.process_incoming_header(&h3));
        // Duplicate of 3 should be rejected
        assert!(!layer.process_incoming_header(&h3));
    }
}
