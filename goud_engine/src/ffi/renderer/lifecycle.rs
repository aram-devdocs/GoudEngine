//! # Renderer Lifecycle FFI
//!
//! Frame lifecycle management: begin/end frame, viewport, blending, depth testing.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::input::{goud_input_key_just_pressed, KEY_F6};
use crate::ffi::network::{
    network_overlay_handle_for_context, network_overlay_set_active_handle_override,
};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::{ClearOps, FrameOps, StateOps};
use crate::sdk::network_debug_overlay::NetworkOverlayState;

use super::draw::{render_network_debug_overlay, render_physics_debug_overlay};

// ============================================================================
// Renderer State
// ============================================================================

// Tracks whether we're currently in a rendering frame.
thread_local! {
    pub(super) static RENDER_ACTIVE: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

fn update_network_overlay_for_frame_end(
    context_id: GoudContextId,
    overlay: &mut NetworkOverlayState,
    toggle_requested: bool,
) {
    if toggle_requested {
        overlay.toggle_visibility();
        if !overlay.is_visible() {
            overlay.set_active_handle(None);
            let _ = network_overlay_set_active_handle_override(context_id, None);
            return;
        }
    }

    if overlay.is_visible() {
        overlay.set_active_handle(network_overlay_handle_for_context(context_id));
    }
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
        let begin_frame_start = std::time::Instant::now();
        if let Err(e) = state.backend_mut().begin_frame() {
            set_last_error(e);
            return false;
        }

        let (fb_width, fb_height) = state.get_framebuffer_size();
        state.backend_mut().set_viewport(0, 0, fb_width, fb_height);
        crate::libs::graphics::frame_timing::record_phase(
            "begin_frame",
            begin_frame_start.elapsed().as_micros() as u64,
        );

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
        let end_frame_start = std::time::Instant::now();
        update_network_overlay_for_frame_end(
            context_id,
            &mut state.network_overlay,
            goud_input_key_just_pressed(context_id, KEY_F6),
        );

        if let Err(e) = render_physics_debug_overlay(context_id, state) {
            set_last_error(e);
            return false;
        }

        if let Err(e) = render_network_debug_overlay(context_id, state) {
            set_last_error(e);
            return false;
        }

        if let Err(e) = state.backend_mut().end_frame() {
            set_last_error(e);
            return false;
        }
        crate::libs::graphics::frame_timing::record_phase(
            "end_frame",
            end_frame_start.elapsed().as_micros() as u64,
        );
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

// ============================================================================
// Coordinate Origin
// ============================================================================

/// Sets the coordinate origin for subsequent `DrawQuad` and `DrawSprite` calls.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `origin` - `0` = Center (default), `1` = TopLeft
///
/// # Returns
///
/// `true` on success, `false` if the origin value is invalid or the context is bad.
#[no_mangle]
pub extern "C" fn goud_renderer_set_coordinate_origin(
    context_id: GoudContextId,
    origin: u32,
) -> bool {
    use super::immediate::{set_coordinate_origin, CoordinateOrigin};

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    match CoordinateOrigin::from_u32(origin) {
        Some(o) => {
            set_coordinate_origin(context_id, o);
            true
        }
        None => {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    }
}

/// Returns the current coordinate origin setting for the given context.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `0` = Center, `1` = TopLeft.  Returns `0` (Center) for invalid contexts.
#[no_mangle]
pub extern "C" fn goud_renderer_get_coordinate_origin(context_id: GoudContextId) -> u32 {
    use super::immediate::{get_coordinate_origin, CoordinateOrigin};

    if context_id == GOUD_INVALID_CONTEXT_ID {
        return CoordinateOrigin::Center as u32;
    }

    get_coordinate_origin(context_id) as u32
}

#[cfg(test)]
mod tests {
    use super::update_network_overlay_for_frame_end;
    use crate::core::providers::impls::NullNetworkProvider;
    use crate::ffi::context::GoudContextId;
    use crate::ffi::network::{
        network_overlay_handle_for_context, network_overlay_set_active_handle_override,
        registry::{reset_registry_for_tests, with_registry, NetInstance},
    };
    use crate::sdk::network_debug_overlay::NetworkOverlayState;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    struct RegistryResetGuard {
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl RegistryResetGuard {
        fn new() -> Self {
            let guard = TEST_MUTEX.lock().expect("test mutex poisoned");
            reset_registry_for_tests();
            Self { _guard: guard }
        }
    }

    impl Drop for RegistryResetGuard {
        fn drop(&mut self) {
            reset_registry_for_tests();
        }
    }

    #[test]
    fn test_frame_end_toggle_syncs_visibility_and_overlay_handle() {
        let _registry = RegistryResetGuard::new();
        let context_id = GoudContextId::new(901, 1);
        let default_handle = with_registry(|reg| {
            let handle = reg.insert(NetInstance {
                provider: Box::new(NullNetworkProvider::new()),
                recv_queue: VecDeque::new(),
            });
            reg.set_default_handle_for_context(context_id, handle);
            Ok(handle)
        })
        .expect("failed to insert default handle");

        let mut overlay = NetworkOverlayState::default();
        update_network_overlay_for_frame_end(context_id, &mut overlay, true);

        assert!(overlay.is_visible());
        assert_eq!(overlay.active_handle(), Some(default_handle));

        update_network_overlay_for_frame_end(context_id, &mut overlay, true);

        assert!(!overlay.is_visible());
        assert_eq!(overlay.active_handle(), None);
        assert_eq!(
            network_overlay_handle_for_context(context_id),
            Some(default_handle)
        );
    }

    #[test]
    fn test_frame_end_respects_external_overlay_handle_updates_while_visible() {
        let _registry = RegistryResetGuard::new();
        let context_id = GoudContextId::new(902, 1);
        let (handle_a, handle_b) = with_registry(|reg| {
            let handle_a = reg.insert(NetInstance {
                provider: Box::new(NullNetworkProvider::new()),
                recv_queue: VecDeque::new(),
            });
            let handle_b = reg.insert(NetInstance {
                provider: Box::new(NullNetworkProvider::new()),
                recv_queue: VecDeque::new(),
            });
            reg.set_default_handle_for_context(context_id, handle_a);
            Ok((handle_a, handle_b))
        })
        .expect("failed to insert handles");

        let mut overlay = NetworkOverlayState::default();
        overlay.set_visible(true);
        overlay.set_active_handle(Some(handle_a));

        assert!(network_overlay_set_active_handle_override(
            context_id,
            Some(handle_b)
        ));
        update_network_overlay_for_frame_end(context_id, &mut overlay, false);

        assert_eq!(overlay.active_handle(), Some(handle_b));
        assert_eq!(
            network_overlay_handle_for_context(context_id),
            Some(handle_b)
        );
    }
}
