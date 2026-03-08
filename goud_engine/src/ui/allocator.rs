//! UI node ID allocation with generation counting and free-list recycling.
//!
//! Follows the same generational arena pattern as
//! [`EntityAllocator`](crate::ecs::entity::EntityAllocator).

use std::fmt;

use super::node_id::UiNodeId;

// =============================================================================
// UiNodeAllocator
// =============================================================================

/// Manages UI node ID allocation with generation counting and free-list recycling.
///
/// Provides O(1) allocation, deallocation, and liveness checks using the
/// generational index pattern.
pub struct UiNodeAllocator {
    /// Generation counter for each slot. Starts at 1 for new slots.
    generations: Vec<u32>,

    /// Stack of free slot indices available for reuse.
    free_list: Vec<u32>,
}

impl UiNodeAllocator {
    /// Creates a new, empty allocator.
    #[inline]
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Allocates a new UI node ID.
    ///
    /// Reuses a slot from the free list when available, otherwise grows.
    ///
    /// # Panics
    ///
    /// Panics if the number of slots exceeds `u32::MAX - 1`.
    pub fn allocate(&mut self) -> UiNodeId {
        if let Some(index) = self.free_list.pop() {
            let generation = self.generations[index as usize];
            UiNodeId::new(index, generation)
        } else {
            let index = self.generations.len();
            assert!(
                index < u32::MAX as usize,
                "UiNodeAllocator exceeded maximum capacity"
            );
            self.generations.push(1);
            UiNodeId::new(index as u32, 1)
        }
    }

    /// Deallocates a UI node ID, invalidating all existing references.
    ///
    /// Returns `true` if the node was alive and successfully deallocated.
    pub fn deallocate(&mut self, id: UiNodeId) -> bool {
        if id.is_invalid() {
            return false;
        }

        let index = id.index() as usize;

        if index >= self.generations.len() {
            return false;
        }

        if self.generations[index] != id.generation() {
            return false;
        }

        let new_gen = self.generations[index].wrapping_add(1);
        self.generations[index] = if new_gen == 0 { 1 } else { new_gen };
        self.free_list.push(id.index());

        true
    }

    /// Returns `true` if the node ID is currently alive.
    #[inline]
    pub fn is_alive(&self, id: UiNodeId) -> bool {
        if id.is_invalid() {
            return false;
        }
        let index = id.index() as usize;
        index < self.generations.len() && self.generations[index] == id.generation()
    }

    /// Returns the number of currently alive nodes.
    ///
    /// # Underflow Safety
    ///
    /// This subtraction cannot underflow due to the invariant that `free_list.len() <= generations.len()`.
    /// The free list only contains indices that were previously allocated from `generations`.
    #[inline]
    pub fn len(&self) -> usize {
        self.generations.len() - self.free_list.len()
    }

    /// Returns the total number of slots (alive + free).
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Returns `true` if no nodes are currently alive.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for UiNodeAllocator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for UiNodeAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiNodeAllocator")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("free_slots", &self.free_list.len())
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_returns_unique_ids() {
        let mut alloc = UiNodeAllocator::new();
        let a = alloc.allocate();
        let b = alloc.allocate();
        assert_ne!(a, b);
    }

    #[test]
    fn test_allocate_starts_at_generation_one() {
        let mut alloc = UiNodeAllocator::new();
        let id = alloc.allocate();
        assert_eq!(id.generation(), 1);
    }

    #[test]
    fn test_is_alive() {
        let mut alloc = UiNodeAllocator::new();
        let id = alloc.allocate();
        assert!(alloc.is_alive(id));
    }

    #[test]
    fn test_deallocate_makes_stale() {
        let mut alloc = UiNodeAllocator::new();
        let id = alloc.allocate();
        assert!(alloc.deallocate(id));
        assert!(!alloc.is_alive(id));
    }

    #[test]
    fn test_double_deallocate_returns_false() {
        let mut alloc = UiNodeAllocator::new();
        let id = alloc.allocate();
        assert!(alloc.deallocate(id));
        assert!(!alloc.deallocate(id));
    }

    #[test]
    fn test_generation_increments_on_reuse() {
        let mut alloc = UiNodeAllocator::new();
        let first = alloc.allocate();
        alloc.deallocate(first);
        let second = alloc.allocate();

        assert_eq!(first.index(), second.index());
        assert_eq!(second.generation(), first.generation() + 1);
        assert!(!alloc.is_alive(first));
        assert!(alloc.is_alive(second));
    }

    #[test]
    fn test_invalid_is_never_alive() {
        let alloc = UiNodeAllocator::new();
        assert!(!alloc.is_alive(UiNodeId::INVALID));
    }

    #[test]
    fn test_deallocate_invalid_returns_false() {
        let mut alloc = UiNodeAllocator::new();
        assert!(!alloc.deallocate(UiNodeId::INVALID));
    }

    #[test]
    fn test_len_and_capacity() {
        let mut alloc = UiNodeAllocator::new();
        assert_eq!(alloc.len(), 0);
        assert_eq!(alloc.capacity(), 0);
        assert!(alloc.is_empty());

        let a = alloc.allocate();
        let b = alloc.allocate();
        assert_eq!(alloc.len(), 2);
        assert_eq!(alloc.capacity(), 2);

        alloc.deallocate(a);
        assert_eq!(alloc.len(), 1);
        assert_eq!(alloc.capacity(), 2);
    }

    #[test]
    fn test_stale_id_not_alive() {
        let mut alloc = UiNodeAllocator::new();
        let old = alloc.allocate();
        alloc.deallocate(old);
        let _new = alloc.allocate();

        assert!(!alloc.is_alive(old));
    }
}
