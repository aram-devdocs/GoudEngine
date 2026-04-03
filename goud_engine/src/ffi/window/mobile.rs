//! # Mobile Responsive Scaling FFI
//!
//! FFI functions for querying DPI scale factor, safe area insets, and
//! logical/physical window sizes. These complement the existing window
//! property functions with a mobile-focused naming convention.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::state::WINDOW_STATES;

/// Gets the display scale factor (DPI ratio) for the window.
///
/// Returns 1.0 for standard density, 2.0 for Retina/xxhdpi, etc.
/// Returns 1.0 on error or invalid context.
#[no_mangle]
pub extern "C" fn goud_get_scale_factor(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 1.0;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        states
            .get(index)
            .and_then(|opt| opt.as_ref())
            .map(|state| state.platform.get_scale_factor())
            .unwrap_or(1.0)
    })
}

/// Gets the safe area insets for the current display.
///
/// Safe area insets describe regions obscured by hardware features (notch,
/// rounded corners) or system UI (status bar, home indicator). Values are
/// in logical points.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `top` - Pointer to store the top inset
/// * `bottom` - Pointer to store the bottom inset
/// * `left` - Pointer to store the left inset
/// * `right` - Pointer to store the right inset
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// All output pointers must be valid, non-null, and writable.
#[no_mangle]
pub unsafe extern "C" fn goud_get_safe_area_insets(
    context_id: GoudContextId,
    top: *mut f32,
    bottom: *mut f32,
    left: *mut f32,
    right: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if top.is_null() || bottom.is_null() || left.is_null() || right.is_null() {
        set_last_error(GoudError::InvalidState(
            "output pointer is null".to_string(),
        ));
        return false;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get(index) {
            let insets = state.platform.get_safe_area_insets();
            // SAFETY: Caller guarantees all pointers are valid and writable.
            *top = insets.top;
            *bottom = insets.bottom;
            *left = insets.left;
            *right = insets.right;
            true
        } else {
            set_last_error(GoudError::InvalidContext);
            false
        }
    })
}

/// Gets the logical window size.
///
/// This is an alias matching the mobile-focused naming convention. It
/// returns the same values as [`goud_window_get_size`](super::goud_window_get_size).
///
/// # Safety
///
/// `width` and `height` must be valid, non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_get_logical_size(
    context_id: GoudContextId,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    // SAFETY: The caller's contract (non-null, valid, aligned pointers) is
    // identical to `goud_window_get_size`'s own safety requirements, so
    // forwarding the raw pointers unchanged is sound. The delegated function
    // also performs its own null-pointer checks before any dereference.
    super::properties::goud_window_get_size(context_id, width, height)
}

/// Gets the physical framebuffer size.
///
/// This is an alias matching the mobile-focused naming convention. It
/// returns the same values as
/// [`goud_window_get_framebuffer_size`](super::goud_window_get_framebuffer_size).
///
/// # Safety
///
/// `width` and `height` must be valid, non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_get_framebuffer_size(
    context_id: GoudContextId,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    // SAFETY: The caller's contract (non-null, valid, aligned pointers) is
    // identical to `goud_window_get_framebuffer_size`'s own safety
    // requirements, so forwarding the raw pointers unchanged is sound. The
    // delegated function also performs its own null-pointer checks before any
    // dereference.
    super::properties::goud_window_get_framebuffer_size(context_id, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_area_insets_invalid_context_returns_false() {
        let mut top: f32 = 0.0;
        let mut bottom: f32 = 0.0;
        let mut left: f32 = 0.0;
        let mut right: f32 = 0.0;
        // SAFETY: All pointers are valid stack references.
        let result = unsafe {
            goud_get_safe_area_insets(
                GOUD_INVALID_CONTEXT_ID,
                &mut top,
                &mut bottom,
                &mut left,
                &mut right,
            )
        };
        assert!(!result);
    }

    #[test]
    fn safe_area_insets_null_top_returns_false() {
        let mut bottom: f32 = 0.0;
        let mut left: f32 = 0.0;
        let mut right: f32 = 0.0;
        // SAFETY: Passing null for top deliberately to test the null guard.
        let result = unsafe {
            goud_get_safe_area_insets(
                GOUD_INVALID_CONTEXT_ID,
                std::ptr::null_mut(),
                &mut bottom,
                &mut left,
                &mut right,
            )
        };
        assert!(!result);
    }

    #[test]
    fn safe_area_insets_null_bottom_returns_false() {
        let mut top: f32 = 0.0;
        let mut left: f32 = 0.0;
        let mut right: f32 = 0.0;
        // SAFETY: Passing null for bottom deliberately to test the null guard.
        let result = unsafe {
            goud_get_safe_area_insets(
                GOUD_INVALID_CONTEXT_ID,
                &mut top,
                std::ptr::null_mut(),
                &mut left,
                &mut right,
            )
        };
        assert!(!result);
    }

    #[test]
    fn safe_area_insets_null_left_returns_false() {
        let mut top: f32 = 0.0;
        let mut bottom: f32 = 0.0;
        let mut right: f32 = 0.0;
        // SAFETY: Passing null for left deliberately to test the null guard.
        let result = unsafe {
            goud_get_safe_area_insets(
                GOUD_INVALID_CONTEXT_ID,
                &mut top,
                &mut bottom,
                std::ptr::null_mut(),
                &mut right,
            )
        };
        assert!(!result);
    }

    #[test]
    fn safe_area_insets_null_right_returns_false() {
        let mut top: f32 = 0.0;
        let mut bottom: f32 = 0.0;
        let mut left: f32 = 0.0;
        // SAFETY: Passing null for right deliberately to test the null guard.
        let result = unsafe {
            goud_get_safe_area_insets(
                GOUD_INVALID_CONTEXT_ID,
                &mut top,
                &mut bottom,
                &mut left,
                std::ptr::null_mut(),
            )
        };
        assert!(!result);
    }

    #[test]
    fn safe_area_insets_all_null_returns_false() {
        // SAFETY: Passing null for all pointers to test the null guard.
        let result = unsafe {
            goud_get_safe_area_insets(
                GOUD_INVALID_CONTEXT_ID,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert!(!result);
    }

    #[test]
    fn scale_factor_invalid_context_returns_one() {
        assert_eq!(goud_get_scale_factor(GOUD_INVALID_CONTEXT_ID), 1.0);
    }
}
