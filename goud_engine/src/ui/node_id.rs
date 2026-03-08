//! UI node identifier type.
//!
//! UI nodes use generational indices following the same pattern as
//! [`Entity`](crate::ecs::entity::Entity), but are a distinct type to prevent
//! accidental mixing of ECS entities and UI nodes.

use std::fmt;
use std::hash::{Hash, Hasher};

// =============================================================================
// UiNodeId
// =============================================================================

/// A lightweight identifier for a node in the UI tree.
///
/// Uses the same generational index pattern as [`Entity`](crate::ecs::entity::Entity)
/// but is intentionally a separate type -- UI nodes live in their own tree, not in
/// the ECS world.
///
/// # Memory Layout
///
/// ```text
/// UiNodeId (8 bytes total):
/// +----------------+----------------+
/// |  index (u32)   | generation(u32)|
/// +----------------+----------------+
/// ```
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UiNodeId {
    /// Slot index in the UI node allocator.
    index: u32,

    /// Generation counter for stale-reference detection.
    generation: u32,
}

impl UiNodeId {
    /// Sentinel value representing "no node".
    ///
    /// Uses `u32::MAX` for the index, which the allocator will never produce.
    pub const INVALID: UiNodeId = UiNodeId {
        index: u32::MAX,
        generation: 0,
    };

    /// Creates a new `UiNodeId` with the given index and generation.
    ///
    /// This is primarily used by [`UiNodeAllocator`](super::UiNodeAllocator).
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns the slot index.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation counter.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns `true` if this is the [`INVALID`](Self::INVALID) sentinel.
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        self.index == u32::MAX && self.generation == 0
    }
}

// =============================================================================
// Trait Implementations
// =============================================================================

impl Hash for UiNodeId {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bits = ((self.generation as u64) << 32) | (self.index as u64);
        bits.hash(state);
    }
}

impl fmt::Debug for UiNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UiNodeId({}:{})", self.index, self.generation)
    }
}

impl fmt::Display for UiNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UiNodeId({}:{})", self.index, self.generation)
    }
}

impl Default for UiNodeId {
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::entity::Entity;

    #[test]
    fn test_creation_and_accessors() {
        let id = UiNodeId::new(5, 3);
        assert_eq!(id.index(), 5);
        assert_eq!(id.generation(), 3);
    }

    #[test]
    fn test_equality() {
        let a = UiNodeId::new(1, 1);
        let b = UiNodeId::new(1, 1);
        let c = UiNodeId::new(1, 2);
        let d = UiNodeId::new(2, 1);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn test_invalid_sentinel() {
        let invalid = UiNodeId::INVALID;
        assert!(invalid.is_invalid());
        assert_eq!(invalid.index(), u32::MAX);
        assert_eq!(invalid.generation(), 0);

        let valid = UiNodeId::new(0, 1);
        assert!(!valid.is_invalid());
    }

    #[test]
    fn test_default_is_invalid() {
        let id = UiNodeId::default();
        assert!(id.is_invalid());
    }

    #[test]
    fn test_debug_format() {
        let id = UiNodeId::new(42, 7);
        assert_eq!(format!("{:?}", id), "UiNodeId(42:7)");
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashMap;

        let id = UiNodeId::new(10, 2);
        let mut map = HashMap::new();
        map.insert(id, "test");
        assert_eq!(map.get(&id), Some(&"test"));
    }

    #[test]
    fn test_not_type_compatible_with_entity() {
        // UiNodeId and Entity have the same layout but are distinct types.
        // This test verifies they cannot be accidentally substituted at the
        // type level (a compile-time guarantee). We confirm the sizes match
        // but the types are different by checking size_of and type_name.
        assert_eq!(
            std::mem::size_of::<UiNodeId>(),
            std::mem::size_of::<Entity>()
        );
        assert_ne!(
            std::any::type_name::<UiNodeId>(),
            std::any::type_name::<Entity>()
        );
    }

    #[test]
    fn test_copy_semantics() {
        let a = UiNodeId::new(1, 1);
        let b = a;
        assert_eq!(a, b);
    }
}
