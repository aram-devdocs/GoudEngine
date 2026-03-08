//! Reference counting for asset handles.
//!
//! Provides external (non-RAII) reference counting so that `AssetHandle<A>`
//! can remain `Copy`. The count is stored separately in [`RefCountMap`] and
//! managed explicitly via `retain` / `release` calls.

use std::collections::HashMap;

// =============================================================================
// RefCountMap
// =============================================================================

/// External reference-count map keyed by `(index, generation)`.
///
/// # Design
///
/// `AssetHandle<A>` is `Copy` -- it has no `Drop` impl. Reference counting
/// is therefore explicit: callers increment with [`Self::increment`] and
/// decrement with [`Self::decrement`]. When the count reaches zero the
/// asset becomes eligible for deferred unloading.
///
/// # Example
///
/// ```ignore
/// use goud_engine::assets::storage::ref_count::RefCountMap;
///
/// let mut rc = RefCountMap::new();
/// rc.insert(0, 1); // handle (index=0, gen=1) starts at ref count 1
/// assert_eq!(rc.get(0, 1), 1);
///
/// rc.increment(0, 1);
/// assert_eq!(rc.get(0, 1), 2);
///
/// let new = rc.decrement(0, 1);
/// assert_eq!(new, Some(1));
/// ```
pub struct RefCountMap {
    counts: HashMap<(u32, u32), u32>,
}

impl RefCountMap {
    /// Creates an empty ref-count map.
    #[inline]
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Inserts a new entry with ref count 1.
    #[inline]
    pub fn insert(&mut self, index: u32, generation: u32) {
        self.counts.insert((index, generation), 1);
    }

    /// Increments the ref count, returning the new value.
    ///
    /// Returns `None` if the key does not exist.
    pub fn increment(&mut self, index: u32, generation: u32) -> Option<u32> {
        let count = self.counts.get_mut(&(index, generation))?;
        *count = count.saturating_add(1);
        Some(*count)
    }

    /// Decrements the ref count, returning the new value.
    ///
    /// Returns `None` if the key does not exist. The count saturates at zero.
    pub fn decrement(&mut self, index: u32, generation: u32) -> Option<u32> {
        let count = self.counts.get_mut(&(index, generation))?;
        *count = count.saturating_sub(1);
        Some(*count)
    }

    /// Returns the current ref count, or 0 if not tracked.
    #[inline]
    pub fn get(&self, index: u32, generation: u32) -> u32 {
        self.counts.get(&(index, generation)).copied().unwrap_or(0)
    }

    /// Removes the entry for this handle.
    #[inline]
    pub fn remove(&mut self, index: u32, generation: u32) {
        self.counts.remove(&(index, generation));
    }

    /// Clears all entries.
    #[inline]
    pub fn clear(&mut self) {
        self.counts.clear();
    }
}

impl Default for RefCountMap {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_sets_count_to_one() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        assert_eq!(rc.get(0, 1), 1);
    }

    #[test]
    fn test_get_returns_zero_for_unknown() {
        let rc = RefCountMap::new();
        assert_eq!(rc.get(99, 99), 0);
    }

    #[test]
    fn test_increment() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        assert_eq!(rc.increment(0, 1), Some(2));
        assert_eq!(rc.increment(0, 1), Some(3));
    }

    #[test]
    fn test_increment_unknown_returns_none() {
        let mut rc = RefCountMap::new();
        assert_eq!(rc.increment(0, 1), None);
    }

    #[test]
    fn test_decrement() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        rc.increment(0, 1);
        assert_eq!(rc.decrement(0, 1), Some(1));
        assert_eq!(rc.decrement(0, 1), Some(0));
    }

    #[test]
    fn test_decrement_saturates_at_zero() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        assert_eq!(rc.decrement(0, 1), Some(0));
        assert_eq!(rc.decrement(0, 1), Some(0));
    }

    #[test]
    fn test_decrement_unknown_returns_none() {
        let mut rc = RefCountMap::new();
        assert_eq!(rc.decrement(0, 1), None);
    }

    #[test]
    fn test_remove() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        rc.remove(0, 1);
        assert_eq!(rc.get(0, 1), 0);
    }

    #[test]
    fn test_clear() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        rc.insert(1, 1);
        rc.clear();
        assert_eq!(rc.get(0, 1), 0);
        assert_eq!(rc.get(1, 1), 0);
    }

    #[test]
    fn test_different_generations_independent() {
        let mut rc = RefCountMap::new();
        rc.insert(0, 1);
        rc.insert(0, 2);
        rc.increment(0, 1);
        assert_eq!(rc.get(0, 1), 2);
        assert_eq!(rc.get(0, 2), 1);
    }
}
