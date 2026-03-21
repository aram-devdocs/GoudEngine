//! FFI exports for the entity pool API.
//!
//! Provides C-compatible functions for creating, managing, and querying
//! entity pools. Pools are stored in a global registry keyed by `u32` handles,
//! supporting multiple concurrent pools with different capacities.

pub mod lifecycle;
pub mod operations;
pub mod queries;

pub use lifecycle::{goud_entity_pool_create, goud_entity_pool_destroy};
pub use operations::{
    goud_entity_pool_acquire, goud_entity_pool_acquire_batch, goud_entity_pool_release,
    goud_entity_pool_release_batch,
};
pub use queries::{goud_entity_pool_stats, FfiPoolStats};

/// Handle type for entity pools. Returned by `goud_entity_pool_create`.
pub type GoudPoolHandle = u32;

/// Sentinel value indicating an invalid or failed pool handle.
pub const GOUD_INVALID_POOL_HANDLE: u32 = u32::MAX;

// =============================================================================
// Global Pool Registry
// =============================================================================

pub(super) mod registry {
    use crate::core::pool::EntityPool;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    /// Registry mapping handles to entity pool instances.
    pub(super) struct PoolRegistry {
        pub(super) pools: HashMap<u32, EntityPool>,
        pub(super) next_handle: u32,
    }

    /// Returns the global pool registry (thread-safe).
    pub(super) fn get() -> &'static Mutex<PoolRegistry> {
        static REGISTRY: OnceLock<Mutex<PoolRegistry>> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Mutex::new(PoolRegistry {
                pools: HashMap::new(),
                next_handle: 0,
            })
        })
    }

    /// Allocates a unique handle that is not the sentinel and not already in use.
    ///
    /// Returns `Some(handle)` on success, `None` if the entire handle space is
    /// exhausted (2^32 - 1 live pools).
    pub(super) fn allocate_handle(reg: &mut PoolRegistry) -> Option<u32> {
        let start = reg.next_handle;
        let mut handle = start;
        loop {
            if handle != super::GOUD_INVALID_POOL_HANDLE && !reg.pools.contains_key(&handle) {
                reg.next_handle = handle.wrapping_add(1);
                return Some(handle);
            }
            handle = handle.wrapping_add(1);
            if handle == start {
                return None;
            }
        }
    }
}
