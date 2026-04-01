//! FFI exports for the AABB-based spatial hash API.
//!
//! Provides C-compatible functions for creating, managing, and querying
//! spatial hashes. Unlike `spatial_grid` (point-based), this module wraps
//! the broad-phase `SpatialHash` which stores axis-aligned bounding boxes
//! and supports AABB/circle range queries.
//!
//! Hashes are stored in a global registry keyed by `u32` handles,
//! supporting multiple concurrent hashes with different cell sizes.

pub mod lifecycle;
pub mod operations;
pub mod queries;

pub use lifecycle::{
    goud_spatial_hash_clear, goud_spatial_hash_create, goud_spatial_hash_create_with_capacity,
    goud_spatial_hash_destroy,
};
pub use operations::{
    goud_spatial_hash_insert, goud_spatial_hash_remove, goud_spatial_hash_update,
};
pub use queries::{
    goud_spatial_hash_entity_count, goud_spatial_hash_query_range, goud_spatial_hash_query_rect,
};

#[cfg(test)]
mod tests;

/// Handle type for spatial hashes. Returned by `goud_spatial_hash_create`.
pub type GoudSpatialHashHandle = u32;

/// Sentinel value indicating an invalid or failed spatial hash handle.
pub const GOUD_INVALID_SPATIAL_HASH_HANDLE: u32 = u32::MAX;

// =============================================================================
// Global Spatial Hash Registry
// =============================================================================

pub(super) mod registry {
    use crate::ecs::broad_phase::SpatialHash;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    /// Registry mapping handles to spatial hash instances.
    pub(super) struct SpatialHashRegistry {
        pub(super) hashes: HashMap<u32, SpatialHash>,
        pub(super) next_handle: u32,
    }

    /// Returns the global spatial hash registry (thread-safe).
    pub(super) fn get() -> &'static Mutex<SpatialHashRegistry> {
        static REGISTRY: OnceLock<Mutex<SpatialHashRegistry>> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Mutex::new(SpatialHashRegistry {
                hashes: HashMap::new(),
                next_handle: 0,
            })
        })
    }

    /// Allocates a unique handle that is not the sentinel and not already in use.
    ///
    /// Returns `Some(handle)` on success, `None` if the entire handle space is
    /// exhausted (2^32 - 1 live hashes).
    pub(super) fn allocate_handle(reg: &mut SpatialHashRegistry) -> Option<u32> {
        let start = reg.next_handle;
        let mut handle = start;
        loop {
            if handle != super::GOUD_INVALID_SPATIAL_HASH_HANDLE
                && !reg.hashes.contains_key(&handle)
            {
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
