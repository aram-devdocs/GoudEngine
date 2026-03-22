//! Pool statistics tracking.
//!
//! [`PoolStats`] captures diagnostic counters that describe the current state
//! and historical usage of an [`super::EntityPool`].

/// Diagnostic counters for an entity pool.
///
/// All fields are public so callers can inspect them directly, but the
/// canonical way to obtain a `PoolStats` snapshot is via
/// [`EntityPool::stats`](super::EntityPool::stats).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PoolStats {
    /// Total number of slots in the pool.
    pub capacity: usize,
    /// Number of slots currently acquired (in use).
    pub active: usize,
    /// Number of slots currently available for acquisition.
    pub available: usize,
    /// Peak number of simultaneously active slots since pool creation.
    pub high_water_mark: usize,
    /// Cumulative number of successful acquire operations.
    pub total_acquires: u64,
    /// Cumulative number of successful release operations.
    pub total_releases: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_stats_default() {
        let stats = PoolStats::default();
        assert_eq!(stats.capacity, 0);
        assert_eq!(stats.active, 0);
        assert_eq!(stats.available, 0);
        assert_eq!(stats.high_water_mark, 0);
        assert_eq!(stats.total_acquires, 0);
        assert_eq!(stats.total_releases, 0);
    }

    #[test]
    fn test_pool_stats_clone() {
        let stats = PoolStats {
            capacity: 100,
            active: 42,
            available: 58,
            high_water_mark: 50,
            total_acquires: 200,
            total_releases: 158,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.capacity, 100);
        assert_eq!(cloned.active, 42);
        assert_eq!(cloned.available, 58);
        assert_eq!(cloned.high_water_mark, 50);
        assert_eq!(cloned.total_acquires, 200);
        assert_eq!(cloned.total_releases, 158);
    }

    #[test]
    fn test_pool_stats_debug() {
        let stats = PoolStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("PoolStats"));
    }
}
