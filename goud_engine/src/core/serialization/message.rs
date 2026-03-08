//! Network message envelope for full and delta payloads.
//!
//! Provides a simple framing format that wraps serialized data with a
//! message kind discriminator and a sequence number for ordering.

use crate::core::error::GoudError;

use super::binary;

/// Discriminator for the type of payload in a [`NetworkMessage`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessageKind {
    /// The payload contains a full state snapshot.
    Full = 0,
    /// The payload contains a delta update relative to a previous state.
    Delta = 1,
}

/// A network message envelope containing a typed payload.
///
/// The envelope carries metadata (kind, sequence number) alongside the raw
/// payload bytes. Use [`NetworkMessage::encode`] and [`NetworkMessage::decode`]
/// for wire-format conversion.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkMessage {
    /// Whether this message is a full snapshot or a delta update.
    pub kind: MessageKind,
    /// Monotonically increasing sequence number for ordering.
    pub sequence: u32,
    /// The serialized payload bytes.
    pub payload: Vec<u8>,
}

impl NetworkMessage {
    /// Creates a new network message.
    pub fn new(kind: MessageKind, sequence: u32, payload: Vec<u8>) -> Self {
        Self {
            kind,
            sequence,
            payload,
        }
    }

    /// Encodes this message into a binary wire format.
    ///
    /// # Errors
    ///
    /// Returns [`GoudError::InternalError`] if serialization fails.
    pub fn encode(&self) -> Result<Vec<u8>, GoudError> {
        binary::encode(self)
    }

    /// Decodes a network message from binary wire format.
    ///
    /// # Errors
    ///
    /// Returns [`GoudError::InternalError`] if the bytes are invalid.
    pub fn decode(bytes: &[u8]) -> Result<Self, GoudError> {
        binary::decode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_message_full_roundtrip() {
        // Arrange
        let msg = NetworkMessage::new(MessageKind::Full, 42, vec![1, 2, 3, 4]);

        // Act
        let bytes = msg.encode().unwrap();
        let decoded = NetworkMessage::decode(&bytes).unwrap();

        // Assert
        assert_eq!(decoded.kind, MessageKind::Full);
        assert_eq!(decoded.sequence, 42);
        assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_network_message_delta_roundtrip() {
        // Arrange
        let msg = NetworkMessage::new(MessageKind::Delta, 100, vec![0xAB, 0xCD]);

        // Act
        let bytes = msg.encode().unwrap();
        let decoded = NetworkMessage::decode(&bytes).unwrap();

        // Assert
        assert_eq!(decoded.kind, MessageKind::Delta);
        assert_eq!(decoded.sequence, 100);
        assert_eq!(decoded.payload, vec![0xAB, 0xCD]);
    }

    #[test]
    fn test_network_message_empty_payload() {
        // Arrange
        let msg = NetworkMessage::new(MessageKind::Full, 0, vec![]);

        // Act
        let bytes = msg.encode().unwrap();
        let decoded = NetworkMessage::decode(&bytes).unwrap();

        // Assert
        assert_eq!(decoded.payload, Vec::<u8>::new());
        assert_eq!(decoded.sequence, 0);
    }

    #[test]
    fn test_network_message_decode_invalid_bytes() {
        // Arrange
        let bad_bytes = vec![0xFF, 0xFE, 0x00];

        // Act
        let result = NetworkMessage::decode(&bad_bytes);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_message_kind_values() {
        // Assert discriminant values match spec
        assert_eq!(MessageKind::Full as u8, 0);
        assert_eq!(MessageKind::Delta as u8, 1);
    }
}
