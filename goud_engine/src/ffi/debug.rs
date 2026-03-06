//! # Debug Overlay FFI
//!
//! FFI functions for querying FPS statistics and controlling the debug overlay.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::sdk::debug_overlay::{FpsStats, OverlayCorner};

// ============================================================================
// FFI Functions
// ============================================================================

/// Retrieves the current FPS statistics from the debug overlay.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `out_stats` - Pointer to write the FPS stats into
///
/// # Returns
///
/// 0 on success, negative on error.
///
/// # Safety
///
/// `out_stats` must be a valid, non-null pointer to an `FpsStats` struct.
/// Caller owns the memory; this function only writes into it.
#[no_mangle]
pub unsafe extern "C" fn goud_debug_get_fps_stats(
    context_id: GoudContextId,
    out_stats: *mut FpsStats,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if out_stats.is_null() {
        set_last_error(GoudError::InternalError(
            "out_stats pointer is null".to_string(),
        ));
        return -2;
    }

    with_window_state(context_id, |state| {
        let stats = state.debug_overlay.stats();
        // SAFETY: Caller guarantees out_stats is a valid, aligned pointer.
        // We write a Copy type so no drop concerns.
        *out_stats = stats;
        0
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        -1
    })
}

/// Enables or disables the FPS overlay.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `enabled` - Whether to enable the overlay
///
/// # Returns
///
/// 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn goud_debug_set_fps_overlay_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_window_state(context_id, |state| {
        state.debug_overlay.set_enabled(enabled);
        0
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        -1
    })
}

/// Sets the FPS stats update interval in seconds.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `interval` - Update interval in seconds (e.g. 0.5 for twice per second)
///
/// # Returns
///
/// 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn goud_debug_set_fps_update_interval(
    context_id: GoudContextId,
    interval: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_window_state(context_id, |state| {
        state.debug_overlay.set_update_interval(interval);
        0
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        -1
    })
}

/// Sets the screen corner where the FPS overlay is displayed.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `corner` - Corner index: 0=TopLeft, 1=TopRight, 2=BottomLeft, 3=BottomRight
///
/// # Returns
///
/// 0 on success, negative on error (including invalid corner value).
#[no_mangle]
pub extern "C" fn goud_debug_set_fps_overlay_corner(
    context_id: GoudContextId,
    corner: i32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    let corner = match corner {
        0 => OverlayCorner::TopLeft,
        1 => OverlayCorner::TopRight,
        2 => OverlayCorner::BottomLeft,
        3 => OverlayCorner::BottomRight,
        _ => {
            set_last_error(GoudError::InternalError(format!(
                "Invalid corner value: {corner}. Expected 0-3."
            )));
            return -3;
        }
    };

    with_window_state(context_id, |state| {
        state.debug_overlay.set_corner(corner);
        0
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        -1
    })
}
