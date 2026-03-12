//! # Debug Overlay FFI
//!
//! FFI functions for querying FPS statistics and controlling the debug overlay.

use crate::core::error::{
    is_diagnostic_enabled, last_error_backtrace, set_diagnostic_enabled, set_last_error, GoudError,
};
use crate::core::debugger;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::sdk::debug_overlay::{FpsStats, OverlayCorner};

mod debugger_runtime;

pub use debugger_runtime::{GoudMemoryCategoryStats, GoudMemorySummary};
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
        let stats = debugger::fps_stats_for_context(context_id)
            .map(|stats| FpsStats {
                current_fps: stats[0],
                min_fps: stats[1],
                max_fps: stats[2],
                avg_fps: stats[3],
                frame_time_ms: stats[4],
            })
            .unwrap_or_else(|| state.debug_overlay.stats());
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
pub extern "C" fn goud_debug_set_fps_overlay_corner(context_id: GoudContextId, corner: i32) -> i32 {
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

// ============================================================================
// Diagnostic Mode FFI Functions
// ============================================================================

/// Enables or disables diagnostic mode.
///
/// When enabled in debug builds, backtraces are captured on every error
/// and logged at `debug!` level.
///
/// # Arguments
///
/// * `enabled` - Whether to enable diagnostic mode
#[no_mangle]
pub extern "C" fn goud_diagnostic_set_enabled(enabled: bool) {
    set_diagnostic_enabled(enabled);
}

/// Returns whether diagnostic mode is currently enabled.
///
/// # Returns
///
/// `true` if diagnostic mode is enabled, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_diagnostic_is_enabled() -> bool {
    is_diagnostic_enabled()
}

/// Writes the last captured diagnostic backtrace into a caller-provided buffer.
///
/// Follows the same buffer protocol as `goud_last_error_message`:
/// - If `buf` is null or `buf_len` is 0, returns the negative required size
///   (including null terminator), e.g. `-256` means 256 bytes are needed.
/// - Otherwise copies the backtrace string into `buf`, null-terminates it,
///   and returns the number of bytes written (excluding null terminator).
/// - Returns 0 if no backtrace is available.
///
/// **Note:** Backtraces are stored in thread-local storage. This function
/// must be called from the same thread that triggered the error; calling
/// from a different thread will always return 0 (no backtrace).
///
/// # Arguments
///
/// * `buf` - Pointer to a caller-owned buffer, or null to query the required size.
///   Caller allocates and frees this buffer.
/// * `buf_len` - Length of the buffer in bytes
///
/// # Returns
///
/// Bytes written (positive), negative required size, or 0 if no backtrace.
///
/// # Safety
///
/// `buf` must point to a valid buffer of at least `buf_len` bytes, or be null.
#[no_mangle]
pub unsafe extern "C" fn goud_diagnostic_last_backtrace(buf: *mut u8, buf_len: usize) -> i32 {
    if buf.is_null() || buf_len == 0 {
        return match last_error_backtrace() {
            Some(bt) => i32::try_from(bt.len().saturating_add(1))
                .map(|n| -n)
                .unwrap_or(i32::MIN),
            None => 0,
        };
    }

    match last_error_backtrace() {
        Some(bt) => {
            let bytes = bt.as_bytes();
            let copy_len = bytes.len().min(buf_len - 1);
            // SAFETY: buf is valid for buf_len bytes per caller contract, copy_len < buf_len
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
            *buf.add(copy_len) = 0; // null terminator
            copy_len as i32
        }
        None => 0,
    }
}
