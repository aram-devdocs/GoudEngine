//! FFI-safe entity identifier type.

// =============================================================================
// Entity ID
// =============================================================================

/// FFI-safe entity identifier.
///
/// This is a raw u64 that packs entity index and generation.
/// It's a direct representation of `Entity::to_bits()`.
///
/// # FFI Safety
///
/// - `#[repr(transparent)]` ensures same layout as u64
/// - Can be passed by value on all platforms
/// - u64::MAX is the INVALID sentinel value
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GoudEntityId(pub u64);

impl GoudEntityId {
    /// Sentinel value for an invalid entity.
    pub const INVALID: Self = Self(u64::MAX);

    /// Creates a new entity ID from a u64 bit pattern.
    pub fn new(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the underlying u64 bit pattern.
    pub fn bits(self) -> u64 {
        self.0
    }

    /// Returns true if this is the invalid sentinel.
    pub fn is_invalid(self) -> bool {
        self.0 == u64::MAX
    }
}

impl Default for GoudEntityId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl From<u64> for GoudEntityId {
    fn from(bits: u64) -> Self {
        Self(bits)
    }
}

impl From<GoudEntityId> for u64 {
    fn from(id: GoudEntityId) -> u64 {
        id.0
    }
}

impl std::fmt::Display for GoudEntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "GoudEntityId(INVALID)")
        } else {
            write!(f, "GoudEntityId({})", self.0)
        }
    }
}
