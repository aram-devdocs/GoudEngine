//! `SystemId` type and unique ID generation.

use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic counter for generating unique SystemIds.
static NEXT_SYSTEM_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a system instance.
///
/// `SystemId` provides a way to identify and reference systems after they've
/// been registered with a scheduler. Each system instance gets a unique ID
/// when registered.
///
/// # Thread Safety
///
/// SystemId is Copy and can be shared freely between threads.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::system::SystemId;
///
/// // Create a new unique ID
/// let id1 = SystemId::new();
/// let id2 = SystemId::new();
///
/// assert_ne!(id1, id2);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(u64);

impl SystemId {
    /// An invalid/placeholder system ID.
    ///
    /// Used as a default value before a system is registered.
    pub const INVALID: SystemId = SystemId(0);

    /// Creates a new unique SystemId.
    ///
    /// Each call returns a different ID, guaranteed to be unique
    /// within the process lifetime.
    #[inline]
    pub fn new() -> Self {
        SystemId(NEXT_SYSTEM_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates a SystemId from a raw value.
    ///
    /// # Safety Note
    ///
    /// This should only be used for deserialization or testing.
    /// For normal use, prefer [`SystemId::new()`].
    #[inline]
    pub const fn from_raw(id: u64) -> Self {
        SystemId(id)
    }

    /// Returns the raw ID value.
    #[inline]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Returns true if this is the INVALID system ID.
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if this is a valid (non-INVALID) system ID.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl Default for SystemId {
    fn default() -> Self {
        SystemId::INVALID
    }
}

impl std::fmt::Debug for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "SystemId(INVALID)")
        } else {
            write!(f, "SystemId({})", self.0)
        }
    }
}

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "INVALID")
        } else {
            write!(f, "{}", self.0)
        }
    }
}
