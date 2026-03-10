//! Supporting types for parallel system execution.
//!
//! Contains configuration, batching, and statistics types used
//! by `ParallelSystemStage`.

use std::fmt;

use crate::ecs::system::SystemId;

/// A wrapper for raw pointers that implements Send + Sync.
///
/// # Safety
///
/// Only safe when the caller guarantees no conflicting concurrent access
/// and the pointer remains valid for the duration of use.
#[cfg(feature = "native")]
pub(super) struct UnsafePtr<T>(pub(super) *mut T);

#[cfg(feature = "native")]
impl<T> UnsafePtr<T> {
    #[inline]
    pub(super) fn get(&self) -> *mut T {
        self.0
    }
}

#[cfg(feature = "native")]
impl<T> Clone for UnsafePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(feature = "native")]
impl<T> Copy for UnsafePtr<T> {}

// SAFETY: UnsafePtr is only used where concurrent access is verified safe.
#[cfg(feature = "native")]
unsafe impl<T> Send for UnsafePtr<T> {}
#[cfg(feature = "native")]
unsafe impl<T> Sync for UnsafePtr<T> {}

/// Configuration for parallel system execution.
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    /// Maximum threads. 0 means use Rayon's default.
    pub max_threads: usize,
    /// Whether to auto-rebuild parallel groups when systems change.
    pub auto_rebuild: bool,
    /// Whether to respect ordering constraints (may reduce parallelism).
    pub respect_ordering: bool,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_threads: 0,
            auto_rebuild: true,
            respect_ordering: true,
        }
    }
}

impl ParallelExecutionConfig {
    /// Creates a config with the specified maximum threads.
    #[inline]
    pub fn with_max_threads(max_threads: usize) -> Self {
        Self {
            max_threads,
            ..Default::default()
        }
    }

    /// Creates a config that ignores ordering for maximum parallelism.
    #[inline]
    pub fn ignore_ordering() -> Self {
        Self {
            respect_ordering: false,
            ..Default::default()
        }
    }
}

/// A batched execution group of non-conflicting systems.
#[derive(Debug, Clone)]
pub struct ParallelBatch {
    /// System IDs in this batch.
    pub system_ids: Vec<SystemId>,
    /// Whether all systems in this batch are read-only.
    pub all_read_only: bool,
}

impl ParallelBatch {
    /// Creates a new empty batch.
    #[inline]
    pub fn new() -> Self {
        Self {
            system_ids: Vec::new(),
            all_read_only: true,
        }
    }

    /// Creates a batch with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            system_ids: Vec::with_capacity(capacity),
            all_read_only: true,
        }
    }

    /// Adds a system ID to the batch.
    #[inline]
    pub fn add(&mut self, id: SystemId, is_read_only: bool) {
        self.system_ids.push(id);
        if !is_read_only {
            self.all_read_only = false;
        }
    }

    /// Returns the number of systems in this batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.system_ids.len()
    }

    /// Returns true if the batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.system_ids.is_empty()
    }

    /// Returns true if the batch can run in parallel (more than 1 system).
    #[inline]
    pub fn can_parallelize(&self) -> bool {
        self.system_ids.len() > 1
    }
}

impl Default for ParallelBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about parallel execution performance.
#[derive(Clone, Default)]
pub struct ParallelExecutionStats {
    /// Number of batches executed.
    pub batch_count: usize,
    /// Total systems executed.
    pub system_count: usize,
    /// Systems that ran in parallel (batch size > 1).
    pub parallel_systems: usize,
    /// Systems that ran sequentially (batch size = 1).
    pub sequential_systems: usize,
    /// Maximum parallelism achieved (largest batch).
    pub max_parallelism: usize,
}

impl ParallelExecutionStats {
    /// Returns the parallelism ratio (0.0-1.0).
    #[inline]
    pub fn parallelism_ratio(&self) -> f32 {
        if self.system_count == 0 {
            return 0.0;
        }
        self.parallel_systems as f32 / self.system_count as f32
    }
}

impl fmt::Debug for ParallelExecutionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParallelExecutionStats")
            .field("batch_count", &self.batch_count)
            .field("system_count", &self.system_count)
            .field("parallel_systems", &self.parallel_systems)
            .field("sequential_systems", &self.sequential_systems)
            .field("max_parallelism", &self.max_parallelism)
            .field(
                "parallelism_ratio",
                &format!("{:.2}", self.parallelism_ratio()),
            )
            .finish()
    }
}
