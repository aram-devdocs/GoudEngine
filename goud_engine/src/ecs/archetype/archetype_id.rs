//! [`ArchetypeId`] type — unique identifier for an archetype.

use std::fmt;

/// Unique identifier for an archetype.
///
/// Archetypes group entities that have the exact same set of components.
/// The `ArchetypeId` is used to efficiently look up and manage archetypes.
///
/// # Invariants
///
/// - `ArchetypeId(0)` is always the EMPTY archetype (no components)
/// - IDs are assigned sequentially as new archetypes are discovered
/// - IDs are stable within a single run but may differ between runs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    /// The empty archetype - contains entities with no components.
    ///
    /// This archetype always exists at index 0 in the archetype graph.
    /// Newly spawned entities without components start here.
    pub const EMPTY: Self = Self(0);

    /// Creates a new `ArchetypeId` with the given index.
    ///
    /// # Arguments
    ///
    /// * `id` - The numeric index for this archetype
    ///
    /// # Note
    ///
    /// This is typically called internally by the archetype graph.
    /// Users should not need to create archetype IDs manually.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the numeric index of this archetype.
    ///
    /// This can be used for indexing into archetype storage arrays.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.0
    }

    /// Returns whether this is the empty archetype (no components).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "ArchetypeId(EMPTY)")
        } else {
            write!(f, "ArchetypeId({})", self.0)
        }
    }
}

impl fmt::Display for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ArchetypeId {
    /// Returns the EMPTY archetype as the default.
    fn default() -> Self {
        Self::EMPTY
    }
}

impl From<u32> for ArchetypeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<ArchetypeId> for u32 {
    fn from(id: ArchetypeId) -> Self {
        id.0
    }
}

impl From<ArchetypeId> for usize {
    fn from(id: ArchetypeId) -> Self {
        id.0 as usize
    }
}
