//! # FFI Window Management
//!
//! This module provides C-compatible functions for window creation, event handling,
//! and game loop management. It integrates the platform backend with the ECS
//! context system.
//!
//! ## Design
//!
//! Window operations are integrated into the context system. When you create a
//! windowed context, it includes:
//! - A [`GlfwPlatform`] backend (window + input polling)
//! - An InputManager (as an ECS resource)
//! - An OpenGL rendering backend
//!
//! The platform-specific code lives in [`GlfwPlatform`]; this module handles
//! FFI marshalling and context integration.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Create a windowed context
//! var contextId = goud_window_create(800, 600, "My Game");
//! if (contextId == GOUD_INVALID_CONTEXT_ID) {
//!     Console.WriteLine("Failed to create window");
//!     return;
//! }
//!
//! // Game loop
//! while (!goud_window_should_close(contextId)) {
//!     float deltaTime = goud_window_poll_events(contextId);
//!     
//!     // Update game logic...
//!     // Rendering will be handled by renderer FFI module
//!     
//!     goud_window_swap_buffers(contextId);
//! }
//!
//! goud_window_destroy(contextId);
//! ```

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::InputManager;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::libs::graphics::backend::RenderBackend;
use crate::libs::platform::glfw_platform::GlfwPlatform;
use crate::libs::platform::{PlatformBackend, WindowConfig};
use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_char;

// ============================================================================
// Window State
// ============================================================================

/// Window state attached to a context.
///
/// Composes a [`GlfwPlatform`] (windowing + input) with an [`OpenGLBackend`]
/// (rendering). The platform backend owns the window handle and event loop;
/// the render backend is stored alongside it.
pub struct WindowState {
    platform: GlfwPlatform,

    /// OpenGL rendering backend
    pub(crate) backend: OpenGLBackend,

    /// Delta time from last frame
    delta_time: f32,
}

impl WindowState {
    /// Returns true if the window should close.
    pub fn should_close(&self) -> bool {
        self.platform.should_close()
    }

    /// Sets whether the window should close.
    pub fn set_should_close(&mut self, should_close: bool) {
        self.platform.set_should_close(should_close);
    }

    /// Polls events, updates input state, and syncs the viewport on resize.
    pub fn poll_events(&mut self, input: &mut InputManager) -> f32 {
        let old_size = self.platform.get_size();
        self.delta_time = self.platform.poll_events(input);
        let new_size = self.platform.get_size();

        if old_size != new_size {
            self.backend.set_viewport(0, 0, new_size.0, new_size.1);
        }

        self.delta_time
    }

    /// Swaps the front and back buffers.
    pub fn swap_buffers(&mut self) {
        self.platform.swap_buffers();
    }

    /// Gets window size (logical).
    pub fn get_size(&self) -> (u32, u32) {
        self.platform.get_size()
    }

    /// Gets framebuffer size (physical - may differ on HiDPI/Retina displays).
    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        self.platform.get_framebuffer_size()
    }

    /// Gets a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut OpenGLBackend {
        &mut self.backend
    }

    /// Gets the delta time.
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
}

// ============================================================================
// Window State Storage (Thread-Local)
// ============================================================================

thread_local! {
    static WINDOW_STATES: RefCell<Vec<Option<WindowState>>> = const { RefCell::new(Vec::new()) };
}

fn set_window_state(context_id: GoudContextId, state: WindowState) -> Result<(), GoudError> {
    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;

        while states.len() <= index {
            states.push(None);
        }

        states[index] = Some(state);
        Ok(())
    })
}

fn remove_window_state(context_id: GoudContextId) {
    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if index < states.len() {
            states[index] = None;
        }
    });
}

/// Provides access to window state for a given context.
///
/// # Safety
///
/// The closure must not store references to the WindowState beyond the call.
pub fn with_window_state<F, R>(context_id: GoudContextId, f: F) -> Option<R>
where
    F: FnOnce(&mut WindowState) -> R,
{
    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        states.get_mut(index).and_then(|opt| opt.as_mut()).map(f)
    })
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Creates a new windowed context with OpenGL rendering.
///
/// This creates:
/// - A GLFW window with the specified dimensions and title
/// - An ECS World with InputManager resource
/// - An OpenGL 3.3 Core rendering backend
///
/// # Arguments
///
/// * `width` - Window width in pixels
/// * `height` - Window height in pixels  
/// * `title` - Window title as a null-terminated C string
///
/// # Returns
///
/// A context ID on success, or `GOUD_INVALID_CONTEXT_ID` on failure.
///
/// # Safety
///
/// The `title` pointer must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn goud_window_create(
    width: u32,
    height: u32,
    title: *const c_char,
) -> GoudContextId {
    // SAFETY: Caller guarantees `title` is a valid C string or null.
    let title_str = if title.is_null() {
        "GoudEngine"
    } else {
        match CStr::from_ptr(title).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Invalid UTF-8 in window title".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        }
    };

    let config = WindowConfig {
        width,
        height,
        title: title_str.to_string(),
        vsync: true,
        resizable: true,
    };

    let platform = match GlfwPlatform::new(&config) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    let mut backend = match OpenGLBackend::new() {
        Ok(b) => b,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    backend.set_viewport(0, 0, width, height);

    let context_id = {
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        match registry.create() {
            Ok(id) => id,
            Err(e) => {
                set_last_error(e);
                return GOUD_INVALID_CONTEXT_ID;
            }
        }
    };

    {
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        if let Some(context) = registry.get_mut(context_id) {
            context.world_mut().insert_resource(InputManager::new());
        }
    }

    let window_state = WindowState {
        platform,
        backend,
        delta_time: 0.0,
    };

    if let Err(e) = set_window_state(context_id, window_state) {
        if let Ok(mut registry) = get_context_registry().lock() {
            let _ = registry.destroy(context_id);
        }
        set_last_error(e);
        return GOUD_INVALID_CONTEXT_ID;
    }

    context_id
}

/// Destroys a windowed context and releases all resources.
///
/// This destroys the window, OpenGL context, and ECS world.
///
/// # Arguments
///
/// * `context_id` - The windowed context to destroy
///
/// # Returns
///
/// `true` on success, `false` on error.
#[no_mangle]
pub extern "C" fn goud_window_destroy(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    remove_window_state(context_id);

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return false;
        }
    };

    match registry.destroy(context_id) {
        Ok(()) => true,
        Err(e) => {
            set_last_error(e);
            false
        }
    }
}

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
