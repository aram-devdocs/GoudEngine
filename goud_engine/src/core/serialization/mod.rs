//! Serialization framework for binary encoding, delta compression, and network messages.
//!
//! This module provides:
//! - **Binary encoding**: Compact binary serialization via bincode
//! - **Delta encoding**: Bitmask-based delta compression for network-efficient updates
//! - **Network messages**: Envelope format for full and delta network messages

pub mod binary;
pub mod delta;
pub mod message;

#[cfg(test)]
mod tests;

pub use binary::{decode, encode};
pub use delta::{DeltaEncode, DeltaPayload};
pub use message::{MessageKind, NetworkMessage};
