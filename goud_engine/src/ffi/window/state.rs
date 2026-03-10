//! # Window State
//!
//! Defines [`WindowState`], which composes a [`GlfwPlatform`] and an
//! [`OpenGLBackend`] into a single per-context object, and the thread-local
//! storage helpers used to access it from FFI functions.

use crate::core::error::GoudError;
use crate::ecs::InputManager;
use crate::ffi::context::GoudContextId;
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::libs::graphics::backend::StateOps;
use crate::libs::platform::glfw_platform::GlfwPlatform;
use crate::libs::platform::PlatformBackend;
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::network_debug_overlay::NetworkOverlayState;
use std::cell::RefCell;

// ============================================================================
// Window State
// ============================================================================

/// Window state attached to a context.
///
/// Composes a [`GlfwPlatform`] (windowing + input) with an [`OpenGLBackend`]
/// (rendering). The platform backend owns the window handle and event loop;
/// the render backend is stored alongside it.
pub struct WindowState {
    pub(crate) platform: GlfwPlatform,

    /// OpenGL rendering backend
    pub(crate) backend: OpenGLBackend,

    /// Whether the physics debug overlay should render for this context.
    pub(crate) physics_debug_enabled: bool,

    /// Delta time from last frame
    delta_time: f32,

    /// Debug overlay for FPS stats tracking.
    pub(crate) debug_overlay: DebugOverlay,

    /// Runtime state for the network debug overlay in this context.
    pub(crate) network_overlay: NetworkOverlayState,
}

impl WindowState {
    /// Creates a new [`WindowState`] from the given platform and backend.
    pub fn new(
        platform: GlfwPlatform,
        backend: OpenGLBackend,
        physics_debug_enabled: bool,
    ) -> Self {
        Self {
            platform,
            backend,
            physics_debug_enabled,
            delta_time: 0.0,
            debug_overlay: DebugOverlay::new(0.5),
            network_overlay: NetworkOverlayState::default(),
        }
    }

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

        self.debug_overlay.update(self.delta_time);

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
    pub(super) static WINDOW_STATES: RefCell<Vec<Option<WindowState>>> = const { RefCell::new(Vec::new()) };
}

/// Stores the given [`WindowState`] for the specified context.
pub fn set_window_state(context_id: GoudContextId, state: WindowState) -> Result<(), GoudError> {
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

/// Removes the [`WindowState`] associated with the specified context.
pub fn remove_window_state(context_id: GoudContextId) {
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
