//! # Renderer Lifecycle FFI
//!
//! Frame lifecycle management: begin/end frame, viewport, blending, depth testing.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::{ClearOps, FrameOps, StateOps};

// ============================================================================
// Renderer State
// ============================================================================

// Tracks whether we're currently in a rendering frame.
thread_local! {
    pub(super) static RENDER_ACTIVE: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Begins a new rendering frame.
///
/// This must be called before any drawing operations and before `goud_renderer_end`.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_begin(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Mark rendering as active
    RENDER_ACTIVE.with(|cell| {
        *cell.borrow_mut() = true;
    });

    // Begin frame on the backend and set viewport to framebuffer size
    with_window_state(context_id, |state| {
        if let Err(e) = state.backend_mut().begin_frame() {
            set_last_error(e);
            return false;
        }

        // Set viewport to framebuffer size (handles HiDPI/Retina displays)
        let (fb_width, fb_height) = state.get_framebuffer_size();
        // SAFETY: gl::Viewport is always safe to call with valid integer dimensions.
        unsafe {
            gl::Viewport(0, 0, fb_width as i32, fb_height as i32);
        }

        true
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        false
    })
}

/// Ends the current rendering frame.
///
/// This must be called after all drawing operations and before `goud_window_swap_buffers`.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_renderer_end(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Mark rendering as inactive
    RENDER_ACTIVE.with(|cell| {
        *cell.borrow_mut() = false;
    });

    // End frame on the backend
    with_window_state(context_id, |state| {
        if let Err(e) = state.backend_mut().end_frame() {
            set_last_error(e);
            return false;
        }
        true
    })
    .unwrap_or_else(|| {
        set_last_error(GoudError::InvalidContext);
        false
    })
}

/// Sets the viewport for rendering.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `x` - Viewport X position
/// * `y` - Viewport Y position
/// * `width` - Viewport width
/// * `height` - Viewport height
#[no_mangle]
pub extern "C" fn goud_renderer_set_viewport(
    context_id: GoudContextId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().set_viewport(x, y, width, height);
    });
}

/// Enables alpha blending for transparent sprites.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_enable_blending(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().enable_blending();
    });
}

/// Disables alpha blending.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_disable_blending(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().disable_blending();
    });
}

/// Enables depth testing.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_enable_depth_test(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().enable_depth_test();
    });
}

/// Disables depth testing.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_disable_depth_test(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().disable_depth_test();
    });
}

/// Clears the depth buffer.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_renderer_clear_depth(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return;
    }

    with_window_state(context_id, |state| {
        state.backend_mut().clear_depth();
    });
}
