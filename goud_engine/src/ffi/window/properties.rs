//! # Window Properties FFI
//!
//! FFI functions for querying and mutating window properties during the game
//! loop: polling events, swapping buffers, reading/writing window size, delta
//! time, close flag, and clearing the color buffer.

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::backend::ClearOps;

use super::state::WINDOW_STATES;

/// Checks if the window should close.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// `true` if the window should close (e.g., user clicked X), `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_window_should_close(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return true;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        states
            .get(index)
            .and_then(|opt| opt.as_ref())
            .map(|state| state.should_close())
            .unwrap_or(true)
    })
}

/// Polls window events and updates input state.
///
/// This should be called once per frame at the beginning of the game loop.
/// It updates the InputManager resource with current key/mouse states.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// The delta time since the last frame in seconds, or 0.0 on error.
#[no_mangle]
pub extern "C" fn goud_window_poll_events(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0.0;
    }

    let input_ptr: Option<*mut InputManager> = {
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return 0.0;
            }
        };

        let context = match registry.get_mut(context_id) {
            Some(c) => c,
            None => {
                set_last_error(GoudError::InvalidContext);
                return 0.0;
            }
        };

        if context.world().resource::<InputManager>().is_none() {
            context.world_mut().insert_resource(InputManager::new());
        }

        // SAFETY: The resource exists because we just inserted it if missing.
        // The pointer is obtained while holding the lock and used below with
        // exclusive access guaranteed by single-threaded window state access.
        context
            .world_mut()
            .resource_mut::<InputManager>()
            .map(|r| r.into_inner() as *mut InputManager)
    };

    let input_ptr = match input_ptr {
        Some(p) => p,
        None => {
            set_last_error(GoudError::InternalError(
                "Failed to get InputManager".to_string(),
            ));
            return 0.0;
        }
    };

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;

        match states.get_mut(index).and_then(|opt| opt.as_mut()) {
            Some(window_state) => {
                // SAFETY: We have exclusive access to InputManager via the raw
                // pointer obtained above. No other code accesses it concurrently
                // because GLFW and this module are single-threaded.
                let input = unsafe { &mut *input_ptr };
                window_state.poll_events(input)
            }
            None => {
                set_last_error(GoudError::InvalidContext);
                0.0
            }
        }
    })
}

/// Swaps the front and back buffers, presenting the rendered frame.
///
/// Call this at the end of your frame after all rendering is complete.
///
/// # Arguments
///
/// * `context_id` - The windowed context
#[no_mangle]
pub extern "C" fn goud_window_swap_buffers(context_id: GoudContextId) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            state.swap_buffers();
        }
    });
}

/// Gets the window size.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `out_width` - Pointer to store the width
/// * `out_height` - Pointer to store the height
///
/// # Returns
///
/// `true` on success, `false` on error.
///
/// # Safety
///
/// `out_width` and `out_height` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn goud_window_get_size(
    context_id: GoudContextId,
    out_width: *mut u32,
    out_height: *mut u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID || out_width.is_null() || out_height.is_null() {
        return false;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get(index) {
            let (w, h) = state.get_size();
            // SAFETY: Caller guarantees pointers are valid.
            *out_width = w;
            *out_height = h;
            true
        } else {
            false
        }
    })
}

/// Sets whether the window should close.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `should_close` - `true` to request close, `false` to cancel
#[no_mangle]
pub extern "C" fn goud_window_set_should_close(context_id: GoudContextId, should_close: bool) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            state.set_should_close(should_close);
        }
    });
}

/// Gets the delta time from the last frame.
///
/// # Arguments
///
/// * `context_id` - The windowed context
///
/// # Returns
///
/// Delta time in seconds, or 0.0 on error.
#[no_mangle]
pub extern "C" fn goud_window_get_delta_time(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return 0.0;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        states
            .get(index)
            .and_then(|opt| opt.as_ref())
            .map(|state| state.delta_time())
            .unwrap_or(0.0)
    })
}

/// Clears the window with the specified color.
///
/// # Arguments
///
/// * `context_id` - The windowed context
/// * `r` - Red component (0.0 - 1.0)
/// * `g` - Green component (0.0 - 1.0)
/// * `b` - Blue component (0.0 - 1.0)
/// * `a` - Alpha component (0.0 - 1.0)
#[no_mangle]
pub extern "C" fn goud_window_clear(context_id: GoudContextId, r: f32, g: f32, b: f32, a: f32) {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            state.backend.set_clear_color(r, g, b, a);
            state.backend.clear_color();
        }
    });
}
