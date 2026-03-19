//! Shared context ID type and invalid sentinel.
//!
//! `GoudContextId` is the opaque identifier returned to callers and used to
//! look up actual contexts in the registry. It uses generational indexing
//! to detect use-after-free bugs.

/// Opaque identifier for an engine context.
///
/// This ID is returned to callers and used to look up the actual context.
/// It uses generational indexing to detect use-after-free bugs.
///
/// # FFI Safety
///
/// - `#[repr(C)]` ensures predictable memory layout
/// - 64-bit value can be passed by value on all platforms
/// - Invalid ID (all bits 1) is distinguishable from any valid ID
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GoudContextId(u64);

impl GoudContextId {
    /// Creates a new context ID from index and generation.
    ///
    /// # Layout
    ///
    /// ```text
    /// | 32 bits: generation | 32 bits: index |
    /// ```
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        let packed = ((generation as u64) << 32) | (index as u64);
        Self(packed)
    }

    /// Returns the index component (lower 32 bits).
    pub(crate) fn index(self) -> u32 {
        self.0 as u32
    }

    /// Returns the generation component (upper 32 bits).
    pub(crate) fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Reconstructs a context ID from a raw `u64` value.
    ///
    /// This is the inverse of the implicit `#[repr(C)]` layout — used by
    /// in-process bindings (Lua) that store the context ID as a plain integer.
    #[allow(dead_code)]
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw `u64` representation of this context ID.
    #[allow(dead_code)]
    pub(crate) fn as_raw(self) -> u64 {
        self.0
    }

    /// Returns true if this is the invalid sentinel ID.
    pub fn is_invalid(self) -> bool {
        self.0 == u64::MAX
    }
}

impl Default for GoudContextId {
    fn default() -> Self {
        GOUD_INVALID_CONTEXT_ID
    }
}

impl std::fmt::Display for GoudContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "GoudContextId(INVALID)")
        } else {
            write!(f, "GoudContextId({}:{})", self.index(), self.generation())
        }
    }
}

/// Sentinel value representing an invalid context ID.
///
/// This is returned on failure and should be checked by callers before using
/// the ID.
pub const GOUD_INVALID_CONTEXT_ID: GoudContextId = GoudContextId(u64::MAX);
