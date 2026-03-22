//! Arena statistics tracking.
//!
//! [`ArenaStats`] captures diagnostic counters that describe the current state
//! and historical usage of a [`super::FrameArena`].

/// Diagnostic counters for a frame arena.
///
/// Obtain a snapshot via [`FrameArena::stats`](super::FrameArena::stats).
#[derive(Debug, Clone, Default)]
pub struct ArenaStats {
    /// Number of bytes currently allocated (in use) within the arena.
    pub bytes_allocated: usize,
    /// Total byte capacity of the arena's backing storage.
    pub bytes_capacity: usize,
    /// Number of times the arena has been reset since creation.
    pub reset_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_stats_default() {
        let stats = ArenaStats::default();
        assert_eq!(stats.bytes_allocated, 0);
        assert_eq!(stats.bytes_capacity, 0);
        assert_eq!(stats.reset_count, 0);
    }

    #[test]
    fn test_arena_stats_clone() {
        let stats = ArenaStats {
            bytes_allocated: 1024,
            bytes_capacity: 4096,
            reset_count: 5,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.bytes_allocated, 1024);
        assert_eq!(cloned.bytes_capacity, 4096);
        assert_eq!(cloned.reset_count, 5);
    }

    #[test]
    fn test_arena_stats_debug() {
        let stats = ArenaStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("ArenaStats"));
    }
}
