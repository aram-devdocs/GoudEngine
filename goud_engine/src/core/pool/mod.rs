//! Object pooling for entity indices.
//!
//! Provides a pre-allocated free-list pool that manages slot indices for
//! entity recycling. The pool itself does not interact with the ECS `World`
//! directly -- that responsibility belongs to the integration layer.
//!
//! - [`EntityPool`]: O(1) acquire/release with zero allocation on the hot path.
//! - [`PoolStats`]: Diagnostic counters for monitoring pool utilisation.

pub mod entity_pool;
pub mod stats;

pub use entity_pool::EntityPool;
pub use stats::PoolStats;
