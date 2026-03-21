//! Bump-allocated frame arena.
//!
//! [`FrameArena`] wraps [`bumpalo::Bump`] to provide extremely fast
//! allocation for data whose lifetime is bounded by a single frame (or any
//! other well-defined scope). All allocations are freed in bulk when
//! [`reset`](FrameArena::reset) is called, avoiding per-object deallocation
//! overhead.

use bumpalo::Bump;

use super::stats::ArenaStats;

/// A bump allocator designed for per-frame temporary allocations.
///
/// Allocations are O(1) pointer bumps. Calling [`reset`](Self::reset) frees
/// every allocation at once and increments the internal reset counter.
///
/// # Example
///
/// ```
/// use goud_engine::core::arena::FrameArena;
///
/// let mut arena = FrameArena::new();
/// let val = arena.alloc(42u32);
/// assert_eq!(*val, 42);
///
/// arena.reset(); // All allocations freed.
/// assert_eq!(arena.stats().reset_count, 1);
/// ```
pub struct FrameArena {
    /// Underlying bump allocator.
    bump: Bump,
    /// Number of resets performed since creation.
    reset_count: u64,
}

impl FrameArena {
    /// Create a new frame arena with default initial capacity.
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            reset_count: 0,
        }
    }

    /// Create a new frame arena pre-allocated with at least `bytes` of capacity.
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(bytes),
            reset_count: 0,
        }
    }

    /// Allocate a value in the arena, returning a mutable reference.
    ///
    /// The returned reference is valid until the next call to [`reset`](Self::reset).
    #[inline]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.bump.alloc(val)
    }

    /// Allocate a copy of a slice in the arena.
    ///
    /// The returned slice is valid until the next call to [`reset`](Self::reset).
    #[inline]
    pub fn alloc_slice_copy<T: Copy>(&self, src: &[T]) -> &mut [T] {
        self.bump.alloc_slice_copy(src)
    }

    /// Reset the arena, freeing all allocations at once.
    ///
    /// This is an O(n) operation over the number of backing chunks, but
    /// individual allocations are not individually freed.
    pub fn reset(&mut self) {
        self.bump.reset();
        self.reset_count += 1;
    }

    /// Get a snapshot of arena statistics.
    pub fn stats(&self) -> ArenaStats {
        ArenaStats {
            bytes_allocated: self.bump.allocated_bytes(),
            bytes_capacity: self.bump.allocated_bytes(), // Bump exposes allocated_bytes
            reset_count: self.reset_count,
        }
    }
}

impl Default for FrameArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_value() {
        let arena = FrameArena::new();
        let val = arena.alloc(42u64);
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_alloc_slice_copy() {
        let arena = FrameArena::new();
        let src = [1u32, 2, 3, 4, 5];
        let slice = arena.alloc_slice_copy(&src);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_alloc_multiple() {
        let arena = FrameArena::new();
        let a = arena.alloc(1u32);
        let b = arena.alloc(2u32);
        let c = arena.alloc(3u32);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    fn test_reset() {
        let mut arena = FrameArena::new();
        arena.alloc(42u64);
        arena.alloc(99u64);
        arena.reset();
        assert_eq!(arena.stats().reset_count, 1);
    }

    #[test]
    fn test_stats_after_alloc() {
        let arena = FrameArena::new();
        let stats_before = arena.stats();
        arena.alloc([0u8; 128]);
        let stats_after = arena.stats();
        assert!(stats_after.bytes_allocated >= stats_before.bytes_allocated + 128);
    }

    #[test]
    fn test_with_capacity() {
        let arena = FrameArena::with_capacity(4096);
        // After creation with capacity, no user bytes are allocated yet.
        let stats = arena.stats();
        assert_eq!(stats.reset_count, 0);
    }

    #[test]
    fn test_multiple_resets() {
        let mut arena = FrameArena::new();
        for _ in 0..5 {
            arena.alloc(123u32);
            arena.reset();
        }
        assert_eq!(arena.stats().reset_count, 5);
    }

    #[test]
    fn test_default() {
        let arena = FrameArena::default();
        assert_eq!(arena.stats().reset_count, 0);
    }
}
