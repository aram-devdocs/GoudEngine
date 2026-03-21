//! Pre-allocated entity index pool with O(1) acquire/release.
//!
//! [`EntityPool`] manages a fixed-capacity set of slot indices backed by a
//! LIFO free-list. Acquiring and releasing slots involves no heap allocation
//! on the hot path -- only stack-local index manipulation.
//!
//! The pool does **not** interact with the ECS [`World`] directly. Slot
//! indices are mapped to real entity IDs (as `u64`) via
//! [`set_slot_entity`](EntityPool::set_slot_entity), which is intended to be
//! called by the integration layer after creating entities in the world.

use super::stats::PoolStats;

/// Internal bookkeeping for a single pool slot.
#[derive(Debug, Clone)]
struct PoolSlot {
    /// The entity ID stored in this slot (as u64 bits).
    entity_id: u64,
    /// Whether this slot is currently in use.
    active: bool,
}

/// A pre-allocated, free-list pool of entity slot indices.
///
/// The pool pre-allocates `capacity` slots at construction time. Each slot
/// can hold an entity ID (as `u64`) that is assigned later via
/// [`set_slot_entity`](Self::set_slot_entity). Acquire/release operations
/// return `(slot_index, entity_id)` pairs and run in O(1) with zero
/// allocation.
///
/// # Layer Note
///
/// This type lives in `core` (Layer 1) and therefore does NOT reference
/// any ECS types. Entity-aware convenience methods are provided by the
/// ECS integration layer in `ecs::world::pool_ops`.
///
/// # Example
///
/// ```
/// use goud_engine::core::pool::EntityPool;
///
/// let mut pool = EntityPool::new(4);
///
/// // Integration layer assigns entity IDs to slots.
/// for i in 0..4 {
///     pool.set_slot_entity(i, (i as u64 + 1) * 100);
/// }
///
/// let (slot, eid) = pool.acquire().unwrap();
/// assert!(pool.is_active(slot));
/// assert!(pool.release(slot));
/// assert!(!pool.is_active(slot));
/// ```
#[derive(Debug)]
pub struct EntityPool {
    /// Dense array of slot bookkeeping.
    slots: Vec<PoolSlot>,
    /// LIFO stack of available slot indices.
    free_indices: Vec<usize>,
    /// Diagnostic counters.
    stats: PoolStats,
}

impl EntityPool {
    /// Create a new pool with the given capacity.
    ///
    /// All slots start as available (inactive). Entity IDs default to `0`
    /// and should be set via [`set_slot_entity`](Self::set_slot_entity)
    /// before first use.
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity)
            .map(|_| PoolSlot {
                entity_id: 0,
                active: false,
            })
            .collect();

        // Build the free list in reverse so that index 0 is popped first.
        let free_indices: Vec<usize> = (0..capacity).rev().collect();

        let stats = PoolStats {
            capacity,
            active: 0,
            available: capacity,
            high_water_mark: 0,
            total_acquires: 0,
            total_releases: 0,
        };

        Self {
            slots,
            free_indices,
            stats,
        }
    }

    /// Create a pool from a vector of entity IDs (as `u64` bits).
    ///
    /// All entities start in the available (free) state.
    pub fn from_entity_ids(entity_ids: Vec<u64>) -> Self {
        let capacity = entity_ids.len();
        let slots = entity_ids
            .into_iter()
            .map(|id| PoolSlot {
                entity_id: id,
                active: false,
            })
            .collect();

        let free_indices: Vec<usize> = (0..capacity).rev().collect();

        let stats = PoolStats {
            capacity,
            active: 0,
            available: capacity,
            high_water_mark: 0,
            total_acquires: 0,
            total_releases: 0,
        };

        Self {
            slots,
            free_indices,
            stats,
        }
    }

    /// Assign an entity ID to a specific slot index.
    ///
    /// This is called by the integration layer after creating entities in
    /// the world. `index` must be less than the pool capacity.
    ///
    /// # Panics
    ///
    /// Panics if `index >= capacity`.
    pub fn set_slot_entity(&mut self, index: usize, entity_id: u64) {
        assert!(
            index < self.slots.len(),
            "slot index {index} out of range (capacity {})",
            self.slots.len()
        );
        self.slots[index].entity_id = entity_id;
    }

    /// Get the entity ID for a specific slot index.
    ///
    /// Returns `None` if the index is out of range.
    #[inline]
    pub fn slot_entity_id(&self, slot_index: usize) -> Option<u64> {
        self.slots.get(slot_index).map(|s| s.entity_id)
    }

    /// Acquire an entity from the pool.
    ///
    /// Returns `Some((slot_index, entity_id))` if a slot is available,
    /// or `None` if the pool is exhausted.
    #[inline]
    pub fn acquire(&mut self) -> Option<(usize, u64)> {
        let slot_index = self.free_indices.pop()?;
        let slot = &mut self.slots[slot_index];
        slot.active = true;

        self.stats.active += 1;
        self.stats.available -= 1;
        self.stats.total_acquires += 1;
        if self.stats.active > self.stats.high_water_mark {
            self.stats.high_water_mark = self.stats.active;
        }

        Some((slot_index, slot.entity_id))
    }

    /// Release an entity back to the pool by slot index.
    ///
    /// Returns `true` if the slot was active and is now released, or `false`
    /// if the index is out of range or the slot was already inactive (double
    /// release).
    #[inline]
    pub fn release(&mut self, slot_index: usize) -> bool {
        if slot_index >= self.slots.len() {
            return false;
        }
        let slot = &mut self.slots[slot_index];
        if !slot.active {
            return false;
        }
        slot.active = false;
        self.free_indices.push(slot_index);

        self.stats.active -= 1;
        self.stats.available += 1;
        self.stats.total_releases += 1;

        true
    }

    /// Release an entity back to the pool by its entity ID (u64).
    ///
    /// Searches for the entity in the pool and releases the corresponding
    /// slot. Returns `true` if found and released, `false` otherwise.
    pub fn release_by_id(&mut self, entity_id: u64) -> bool {
        let slot_index = match self
            .slots
            .iter()
            .position(|s| s.entity_id == entity_id && s.active)
        {
            Some(idx) => idx,
            None => return false,
        };

        self.slots[slot_index].active = false;
        self.free_indices.push(slot_index);

        self.stats.active -= 1;
        self.stats.available += 1;
        self.stats.total_releases += 1;

        true
    }

    /// Acquire up to `count` entities in a single call.
    ///
    /// Returns a vector of `(slot_index, entity_id)` pairs. The returned
    /// vector may contain fewer than `count` entries if the pool does not
    /// have enough available slots.
    pub fn acquire_batch(&mut self, count: usize) -> Vec<(usize, u64)> {
        let actual = count.min(self.free_indices.len());
        let mut result = Vec::with_capacity(actual);
        for _ in 0..actual {
            if let Some(pair) = self.acquire() {
                result.push(pair);
            }
        }
        result
    }

    /// Release multiple slots in a single call.
    ///
    /// Returns the number of slots that were successfully released.
    /// Slots that are out of range or already inactive are silently skipped.
    pub fn release_batch(&mut self, slot_indices: &[usize]) -> usize {
        let mut count = 0;
        for &idx in slot_indices {
            if self.release(idx) {
                count += 1;
            }
        }
        count
    }

    /// Get a reference to the pool statistics.
    #[inline]
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Check whether the given slot index is within range and currently active.
    #[inline]
    pub fn is_active(&self, slot_index: usize) -> bool {
        slot_index < self.slots.len() && self.slots[slot_index].active
    }

    /// Total capacity of the pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Returns all entity IDs managed by this pool.
    pub fn all_entity_ids(&self) -> Vec<u64> {
        self.slots.iter().map(|s| s.entity_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = EntityPool::new(10);
        assert_eq!(pool.capacity(), 10);
        assert_eq!(pool.stats().active, 0);
        assert_eq!(pool.stats().available, 10);
        assert_eq!(pool.stats().high_water_mark, 0);
    }

    #[test]
    fn test_acquire_and_release() {
        let mut pool = EntityPool::new(4);
        for i in 0..4 {
            pool.set_slot_entity(i, (i as u64 + 1) * 10);
        }

        let (slot, eid) = pool.acquire().unwrap();
        assert_eq!(slot, 0);
        assert_eq!(eid, 10);
        assert!(pool.is_active(slot));
        assert_eq!(pool.stats().active, 1);
        assert_eq!(pool.stats().available, 3);

        assert!(pool.release(slot));
        assert!(!pool.is_active(slot));
        assert_eq!(pool.stats().active, 0);
        assert_eq!(pool.stats().available, 4);
    }

    #[test]
    fn test_acquire_exhaustion() {
        let mut pool = EntityPool::new(2);
        assert!(pool.acquire().is_some());
        assert!(pool.acquire().is_some());
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn test_double_release_returns_false() {
        let mut pool = EntityPool::new(2);
        let (slot, _) = pool.acquire().unwrap();
        assert!(pool.release(slot));
        assert!(!pool.release(slot));
    }

    #[test]
    fn test_release_out_of_range() {
        let mut pool = EntityPool::new(2);
        assert!(!pool.release(999));
    }

    #[test]
    fn test_batch_acquire() {
        let mut pool = EntityPool::new(5);
        for i in 0..5 {
            pool.set_slot_entity(i, i as u64);
        }

        let batch = pool.acquire_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(pool.stats().active, 3);
        assert_eq!(pool.stats().available, 2);
    }

    #[test]
    fn test_batch_acquire_partial() {
        let mut pool = EntityPool::new(2);
        let batch = pool.acquire_batch(5);
        assert_eq!(batch.len(), 2);
        assert_eq!(pool.stats().active, 2);
        assert_eq!(pool.stats().available, 0);
    }

    #[test]
    fn test_batch_release() {
        let mut pool = EntityPool::new(4);
        let batch = pool.acquire_batch(4);
        let indices: Vec<usize> = batch.iter().map(|(idx, _)| *idx).collect();

        let released = pool.release_batch(&indices);
        assert_eq!(released, 4);
        assert_eq!(pool.stats().active, 0);
        assert_eq!(pool.stats().available, 4);
    }

    #[test]
    fn test_batch_release_with_invalid() {
        let mut pool = EntityPool::new(2);
        let (slot, _) = pool.acquire().unwrap();
        let released = pool.release_batch(&[slot, 999, 1]);
        assert_eq!(released, 1);
    }

    #[test]
    fn test_high_water_mark() {
        let mut pool = EntityPool::new(4);
        pool.acquire();
        pool.acquire();
        pool.acquire();
        let (slot, _) = pool.acquire().unwrap();
        pool.release(slot);
        assert_eq!(pool.stats().high_water_mark, 4);
        assert_eq!(pool.stats().active, 3);
    }

    #[test]
    fn test_stats_counters() {
        let mut pool = EntityPool::new(3);
        pool.acquire();
        pool.acquire();
        let (slot, _) = pool.acquire().unwrap();
        pool.release(slot);
        pool.acquire();

        assert_eq!(pool.stats().total_acquires, 4);
        assert_eq!(pool.stats().total_releases, 1);
    }

    #[test]
    fn test_zero_capacity_pool() {
        let mut pool = EntityPool::new(0);
        assert_eq!(pool.capacity(), 0);
        assert!(pool.acquire().is_none());
        assert_eq!(pool.acquire_batch(5).len(), 0);
    }

    #[test]
    fn test_is_active_out_of_range() {
        let pool = EntityPool::new(2);
        assert!(!pool.is_active(100));
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn test_set_slot_entity_out_of_range() {
        let mut pool = EntityPool::new(2);
        pool.set_slot_entity(5, 42);
    }

    #[test]
    fn test_acquire_returns_slots_in_order() {
        let mut pool = EntityPool::new(3);
        for i in 0..3 {
            pool.set_slot_entity(i, i as u64 * 100);
        }
        let (s0, _) = pool.acquire().unwrap();
        let (s1, _) = pool.acquire().unwrap();
        let (s2, _) = pool.acquire().unwrap();
        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }

    #[test]
    fn test_release_reuses_slot_lifo() {
        let mut pool = EntityPool::new(3);
        for i in 0..3 {
            pool.set_slot_entity(i, i as u64);
        }
        let (s0, _) = pool.acquire().unwrap();
        let (s1, _) = pool.acquire().unwrap();
        pool.release(s0);
        pool.release(s1);
        // LIFO: s1 should be returned first.
        let (next, _) = pool.acquire().unwrap();
        assert_eq!(next, s1);
    }

    #[test]
    fn test_from_entity_ids() {
        let ids = vec![100u64, 200, 300];
        let mut pool = EntityPool::from_entity_ids(ids);
        assert_eq!(pool.capacity(), 3);
        assert_eq!(pool.stats().available, 3);

        let (slot, eid) = pool.acquire().unwrap();
        assert_eq!(slot, 0);
        assert_eq!(eid, 100);

        assert!(pool.release_by_id(100));
        assert_eq!(pool.stats().available, 3);
    }

    #[test]
    fn test_all_entity_ids() {
        let ids = vec![10u64, 20];
        let pool = EntityPool::from_entity_ids(ids);
        let all = pool.all_entity_ids();
        assert_eq!(all, vec![10, 20]);
    }

    #[test]
    fn test_slot_entity_id() {
        let mut pool = EntityPool::new(2);
        pool.set_slot_entity(0, 42);
        assert_eq!(pool.slot_entity_id(0), Some(42));
        assert_eq!(pool.slot_entity_id(99), None);
    }

    #[test]
    fn test_debug() {
        let pool = EntityPool::new(2);
        let debug_str = format!("{:?}", pool);
        assert!(debug_str.contains("EntityPool"));
    }
}
