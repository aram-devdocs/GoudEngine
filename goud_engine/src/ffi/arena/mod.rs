//! FFI exports for the frame arena API.
//!
//! Provides C-compatible functions for resetting and querying a global
//! per-process frame arena. The arena is a bump allocator designed for
//! per-frame temporary allocations that are freed in bulk.

use crate::core::arena::FrameArena;
use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use std::sync::{Mutex, OnceLock};

/// Returns the global frame arena (one per process, thread-safe).
fn global_arena() -> &'static Mutex<FrameArena> {
    static ARENA: OnceLock<Mutex<FrameArena>> = OnceLock::new();
    ARENA.get_or_init(|| Mutex::new(FrameArena::new()))
}

/// FFI-safe arena statistics snapshot.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiArenaStats {
    /// Number of bytes currently allocated within the arena.
    pub bytes_allocated: u64,
    /// Total byte capacity of the arena's backing storage.
    pub bytes_capacity: u64,
    /// Number of times the arena has been reset since creation.
    pub reset_count: u64,
}

/// Resets the global frame arena, freeing all allocations at once.
///
/// This should be called once per frame, typically at the start of the
/// frame update loop.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_frame_arena_reset() -> i32 {
    let mut arena = match global_arena().lock() {
        Ok(a) => a,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock frame arena".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    arena.reset();
    0
}

/// Retrieves diagnostic statistics for the global frame arena.
///
/// # Arguments
///
/// * `out_stats` - Pointer to caller-allocated storage for one `FfiArenaStats`.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
///
/// # Safety
///
/// `out_stats` must point to writable storage for one [`FfiArenaStats`].
#[no_mangle]
pub unsafe extern "C" fn goud_frame_arena_stats(out_stats: *mut FfiArenaStats) -> i32 {
    if out_stats.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_stats pointer is null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let arena = match global_arena().lock() {
        Ok(a) => a,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock frame arena".to_string(),
            ));
            return ERR_INTERNAL_ERROR;
        }
    };

    let stats = arena.stats();
    // SAFETY: out_stats is non-null and points to writable storage for one FfiArenaStats.
    *out_stats = FfiArenaStats {
        bytes_allocated: stats.bytes_allocated as u64,
        bytes_capacity: stats.bytes_capacity as u64,
        reset_count: stats.reset_count,
    };
    0
}
