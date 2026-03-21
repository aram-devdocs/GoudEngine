//! FFI exports for the spatial grid API.
//!
//! Provides C-compatible functions for creating, managing, and querying
//! spatial grids. Grids are stored in a global registry keyed by `u32` handles,
//! supporting multiple concurrent grids with different cell sizes.

pub mod lifecycle;
pub mod operations;
pub mod queries;

pub use lifecycle::{
    goud_spatial_grid_clear, goud_spatial_grid_create, goud_spatial_grid_create_with_capacity,
    goud_spatial_grid_destroy,
};
pub use operations::{
    goud_spatial_grid_insert, goud_spatial_grid_remove, goud_spatial_grid_update,
};
pub use queries::{goud_spatial_grid_entity_count, goud_spatial_grid_query_radius};

/// Handle type for spatial grids. Returned by `goud_spatial_grid_create`.
pub type GoudSpatialGridHandle = u32;

/// Sentinel value indicating an invalid or failed spatial grid handle.
pub const GOUD_INVALID_SPATIAL_GRID_HANDLE: u32 = u32::MAX;

// =============================================================================
// Global Spatial Grid Registry
// =============================================================================

pub(super) mod registry {
    use crate::ecs::spatial_grid::SpatialGrid;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    /// Registry mapping handles to spatial grid instances.
    pub(super) struct SpatialGridRegistry {
        pub(super) grids: HashMap<u32, SpatialGrid>,
        pub(super) next_handle: u32,
    }

    /// Returns the global spatial grid registry (thread-safe).
    pub(super) fn get() -> &'static Mutex<SpatialGridRegistry> {
        static REGISTRY: OnceLock<Mutex<SpatialGridRegistry>> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Mutex::new(SpatialGridRegistry {
                grids: HashMap::new(),
                next_handle: 0,
            })
        })
    }

    /// Allocates a unique handle that is not the sentinel and not already in use.
    ///
    /// Returns `Some(handle)` on success, `None` if the entire handle space is
    /// exhausted (2^32 - 1 live grids).
    pub(super) fn allocate_handle(reg: &mut SpatialGridRegistry) -> Option<u32> {
        let start = reg.next_handle;
        let mut handle = start;
        loop {
            if handle != super::GOUD_INVALID_SPATIAL_GRID_HANDLE && !reg.grids.contains_key(&handle)
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
