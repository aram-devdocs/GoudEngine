//! UI system FFI exports.
//!
//! Provides C-compatible functions for creating and managing UI trees.
//! The UI system is independent of the ECS world -- nodes live in their
//! own [`UiManager`](crate::ui::UiManager).
//!
//! # ID Packing
//!
//! [`UiNodeId`](crate::ui::UiNodeId) is packed into a `u64` for the FFI
//! boundary:
//!
//! ```text
//! +----------------------------------+----------------------------------+
//! |  generation (upper 32 bits)      |  index (lower 32 bits)           |
//! +----------------------------------+----------------------------------+
//! ```
//!
//! The sentinel value `u64::MAX` represents "no node" / invalid.

pub mod manager;
pub mod node;

use crate::ui::UiNodeId;

/// Sentinel `u64` returned when a node operation fails or no node exists.
pub const INVALID_NODE_U64: u64 = u64::MAX;

/// Packs a [`UiNodeId`] into a `u64`.
///
/// Layout: `index` in the lower 32 bits, `generation` in the upper 32 bits.
#[inline]
fn pack_node_id(id: UiNodeId) -> u64 {
    (id.index() as u64) | ((id.generation() as u64) << 32)
}

/// Unpacks a `u64` into a [`UiNodeId`].
///
/// Inverse of [`pack_node_id`].
#[inline]
fn unpack_node_id(packed: u64) -> UiNodeId {
    let index = packed as u32;
    let generation = (packed >> 32) as u32;
    UiNodeId::new(index, generation)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_roundtrip() {
        let id = UiNodeId::new(42, 7);
        let packed = pack_node_id(id);
        let unpacked = unpack_node_id(packed);
        assert_eq!(unpacked.index(), 42);
        assert_eq!(unpacked.generation(), 7);
    }

    #[test]
    fn test_pack_layout() {
        let id = UiNodeId::new(0xDEAD, 0xBEEF);
        let packed = pack_node_id(id);
        assert_eq!(packed & 0xFFFF_FFFF, 0xDEAD);
        assert_eq!(packed >> 32, 0xBEEF);
    }

    #[test]
    fn test_pack_zero() {
        let id = UiNodeId::new(0, 0);
        assert_eq!(pack_node_id(id), 0);
    }

    #[test]
    fn test_invalid_sentinel_is_distinct() {
        let max_id = UiNodeId::new(u32::MAX, u32::MAX);
        assert_eq!(pack_node_id(max_id), INVALID_NODE_U64);
    }
}
