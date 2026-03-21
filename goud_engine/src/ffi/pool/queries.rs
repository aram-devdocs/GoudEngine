//! Entity pool query FFI functions.

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};

use super::registry;
use super::GOUD_INVALID_POOL_HANDLE;

/// FFI-safe pool statistics snapshot.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiPoolStats {
    /// Total number of slots in the pool.
    pub capacity: u32,
    /// Number of slots currently acquired (in use).
    pub active: u32,
    /// Number of slots currently available for acquisition.
    pub available: u32,
    /// Peak number of simultaneously active slots since pool creation.
    pub high_water_mark: u32,
    /// Cumulative number of successful acquire operations.
    pub total_acquires: u64,
    /// Cumulative number of successful release operations.
    pub total_releases: u64,
}

/// Retrieves diagnostic statistics for an entity pool.
///
/// # Arguments
///
/// * `handle` - The pool handle.
/// * `out_stats` - Pointer to caller-allocated storage for one `FfiPoolStats`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
///
/// # Safety
///
/// `out_stats` must point to writable storage for one [`FfiPoolStats`].
#[no_mangle]
pub unsafe extern "C" fn goud_entity_pool_stats(
    handle: u32,
    out_stats: *mut FfiPoolStats,
) -> i32 {
    if handle == GOUD_INVALID_POOL_HANDLE {
        set_last_error(GoudError::InvalidState(
            "invalid pool handle".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    if out_stats.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_stats pointer is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let reg = match registry::get().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock pool registry".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    match reg.pools.get(&handle) {
        Some(pool) => {
            let stats = pool.stats();
            // SAFETY: out_stats is non-null and points to writable storage for one FfiPoolStats.
            *out_stats = FfiPoolStats {
                capacity: stats.capacity as u32,
                active: stats.active as u32,
                available: stats.available as u32,
                high_water_mark: stats.high_water_mark as u32,
                total_acquires: stats.total_acquires,
                total_releases: stats.total_releases,
            };
            0
        }
        None => {
            set_last_error(GoudError::InvalidState(
                "pool handle not found".to_string(),
            ));
            GoudError::InvalidState(String::new()).error_code()
        }
    }
}
