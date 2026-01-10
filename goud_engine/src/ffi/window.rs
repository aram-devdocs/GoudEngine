//! # FFI Window Management
//!
//! This module provides C-compatible functions for window creation, event handling,
//! and game loop management. It integrates GLFW with the ECS context system.
//!
//! ## Design
//!
//! Window operations are integrated into the context system. When you create a
//! windowed context, it includes:
//! - A GLFW window
//! - An InputManager (as an ECS resource)
//! - An OpenGL rendering backend
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
use glfw::{Action, Context, Glfw, GlfwReceiver, PWindow, WindowEvent, WindowMode};
use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_char;

// ============================================================================
// Thread-Local GLFW Instance
// ============================================================================

// GLFW must be used from the main thread only. We use thread_local to ensure
// proper initialization and avoid Send/Sync issues.
thread_local! {
    static GLFW_INSTANCE: RefCell<Option<Glfw>> = const { RefCell::new(None) };
}

/// Initializes GLFW if not already initialized.
fn get_or_init_glfw() -> Result<Glfw, GoudError> {
    GLFW_INSTANCE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            let glfw = glfw::init(glfw::fail_on_errors)
                .map_err(|e| GoudError::InternalError(format!("Failed to initialize GLFW: {e}")))?;
            *borrow = Some(glfw);
        }
        borrow
            .clone()
            .ok_or_else(|| GoudError::InternalError("GLFW not initialized".to_string()))
    })
}

// ============================================================================
// Window State
// ============================================================================

/// Window state attached to a context.
pub struct WindowState {
    /// GLFW window handle
    window: PWindow,

    /// Event receiver for window events
    events: GlfwReceiver<(f64, WindowEvent)>,

    /// OpenGL rendering backend
    pub(crate) backend: OpenGLBackend,

    /// Delta time from last frame
    delta_time: f32,

    /// Timestamp of last frame
    last_frame_time: f64,

    /// Window width
    width: u32,

    /// Window height
    height: u32,
}

impl WindowState {
    /// Returns true if the window should close.
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    /// Polls events and updates input state.
    pub fn poll_events(&mut self, input: &mut InputManager, glfw: &mut Glfw) -> f32 {
        // Update input state for new frame
        input.update();

        glfw.poll_events();

        // Calculate delta time
        let current_time = glfw.get_time();
        self.delta_time = (current_time - self.last_frame_time) as f32;
        self.last_frame_time = current_time;

        // Process events
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(key, _scancode, action, _mods) => {
                    match action {
                        Action::Press => input.press_key(key),
                        Action::Release => input.release_key(key),
                        Action::Repeat => {} // Handled by key_pressed
                    }
                }
                WindowEvent::MouseButton(button, action, _mods) => match action {
                    Action::Press => input.press_mouse_button(button),
                    Action::Release => input.release_mouse_button(button),
                    Action::Repeat => {}
                },
                WindowEvent::CursorPos(x, y) => {
                    input.set_mouse_position(crate::core::math::Vec2::new(x as f32, y as f32));
                }
                WindowEvent::Scroll(x, y) => {
                    input.add_scroll_delta(crate::core::math::Vec2::new(x as f32, y as f32));
                }
                WindowEvent::Close => {
                    self.window.set_should_close(true);
                }
                WindowEvent::Size(w, h) => {
                    self.width = w as u32;
                    self.height = h as u32;
                    self.backend.set_viewport(0, 0, self.width, self.height);
                }
                _ => {}
            }
        }

        self.delta_time
    }

    /// Swaps the front and back buffers.
    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    /// Gets window size (logical).
    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Gets framebuffer size (physical - may differ on HiDPI/Retina displays).
    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        let (w, h) = self.window.get_framebuffer_size();
        (w as u32, h as u32)
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

// Window states are stored per-context in a thread-local container.
// This avoids Sync requirements and ensures GLFW is used from the correct thread.
thread_local! {
    static WINDOW_STATES: RefCell<Vec<Option<WindowState>>> = const { RefCell::new(Vec::new()) };
}

fn set_window_state(context_id: GoudContextId, state: WindowState) -> Result<(), GoudError> {
    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;

        // Ensure capacity
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
    // Parse title
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

    // Initialize GLFW
    let mut glfw = match get_or_init_glfw() {
        Ok(g) => g,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    // Configure GLFW for OpenGL 3.3 Core
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Create window
    let (mut window, events) =
        match glfw.create_window(width, height, title_str, WindowMode::Windowed) {
            Some(w) => w,
            None => {
                set_last_error(GoudError::WindowCreationFailed(
                    "Failed to create GLFW window".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

    // Make OpenGL context current
    window.make_current();

    // Enable vsync
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // Set up event polling
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_close_polling(true);
    window.set_size_polling(true);
    window.set_scroll_polling(true);

    // Load OpenGL function pointers
    gl::load_with(|s| window.get_proc_address(s));

    // Create OpenGL backend
    let mut backend = match OpenGLBackend::new() {
        Ok(b) => b,
        Err(e) => {
            set_last_error(e);
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    // Set initial viewport
    backend.set_viewport(0, 0, width, height);

    // Create context with ECS World
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

    // Insert InputManager as a resource in the World
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

    // Create window state
    let window_state = WindowState {
        window,
        events,
        backend,
        delta_time: 0.0,
        last_frame_time: glfw.get_time(),
        width,
        height,
    };

    // Store window state
    if let Err(e) = set_window_state(context_id, window_state) {
        // Clean up context on failure
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

    // Remove window state first (closes GLFW window)
    remove_window_state(context_id);

    // Destroy the context
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

    // Get GLFW instance
    let mut glfw = match get_or_init_glfw() {
        Ok(g) => g,
        Err(e) => {
            set_last_error(e);
            return 0.0;
        }
    };

    // We need to handle this carefully due to borrow checker constraints
    // First, get InputManager pointer (while holding registry lock)
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

        // Ensure InputManager exists
        if context.world().resource::<InputManager>().is_none() {
            context.world_mut().insert_resource(InputManager::new());
        }

        // Get a raw pointer to InputManager
        // SAFETY: The resource exists because we just inserted it if missing.
        // We're getting a pointer while holding the lock, but we'll release the lock
        // before using the pointer. This is safe because:
        // 1. The context won't be destroyed (we hold a valid context_id)
        // 2. We're the only ones holding the resource (single-threaded access)
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

    // Now poll events with window state
    // SAFETY: We have exclusive access to the InputManager and WindowState
    let delta_time = WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;

        match states.get_mut(index).and_then(|opt| opt.as_mut()) {
            Some(window_state) => {
                // SAFETY: We have exclusive access to InputManager
                let input = unsafe { &mut *input_ptr };
                window_state.poll_events(input, &mut glfw)
            }
            None => {
                set_last_error(GoudError::InvalidContext);
                0.0
            }
        }
    });

    delta_time
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
            state.window.set_should_close(should_close);
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
