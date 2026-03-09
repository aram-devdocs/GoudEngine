//! Session protocol message framing.
//!
//! Payloads are intentionally opaque bytes at this layer.

use serde::{Deserialize, Serialize};

use crate::core::error::{GoudError, GoudResult};
use crate::core::serialization::binary;

/// Session protocol version used by this module.
pub const PROTOCOL_VERSION: u8 = 1;

/// Wire-level message exchanged between session clients and servers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolMessage {
    /// Client requests to join the session after transport connection.
    JoinRequest,
    /// Server accepts join and sends current authoritative snapshot.
    JoinAccepted {
        /// Opaque snapshot bytes.
        snapshot: Vec<u8>,
    },
    /// Client state-change command to be authority validated server-side.
    StateCommand {
        /// Opaque command bytes.
        payload: Vec<u8>,
    },
    /// Server-authoritative state update broadcast.
    StateUpdate {
        /// Monotonic authoritative sequence.
        sequence: u64,
        /// Opaque state bytes.
        payload: Vec<u8>,
    },
    /// Server authority rejected a command.
    ValidationRejected {
        /// Human-readable rejection reason.
        reason: String,
        /// Original opaque command bytes.
        payload: Vec<u8>,
    },
    /// Graceful leave notice.
    LeaveNotice {
        /// Human-readable leave reason.
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProtocolEnvelope {
    version: u8,
    message: ProtocolMessage,
}

/// Encodes a protocol message to bytes.
pub fn encode_message(message: &ProtocolMessage) -> GoudResult<Vec<u8>> {
    let envelope = ProtocolEnvelope {
        version: PROTOCOL_VERSION,
        message: message.clone(),
    };
    binary::encode(&envelope)
}

/// Decodes bytes into a protocol message.
pub fn decode_message(bytes: &[u8]) -> GoudResult<ProtocolMessage> {
    let envelope: ProtocolEnvelope = binary::decode(bytes)?;
    if envelope.version != PROTOCOL_VERSION {
        return Err(GoudError::ProviderError {
            subsystem: "network",
            message: format!(
                "Unsupported session protocol version {} (expected {})",
                envelope.version, PROTOCOL_VERSION
            ),
        });
    }
    Ok(envelope.message)
}
