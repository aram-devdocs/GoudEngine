//! # Window State
//!
//! Defines [`WindowState`], which composes a
//! [`GlfwPlatform`](crate::libs::platform::glfw_platform::GlfwPlatform) and an
//! [`OpenGLBackend`] into a single per-context object, and the thread-local
//! storage helpers used to access it from FFI functions.

use crate::core::debugger::{self, RuntimeRouteId};
use crate::core::error::GoudError;
use crate::core::math::Vec2;
use crate::ecs::InputManager;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::libs::graphics::backend::{RenderBackend, StateOps};
use crate::libs::platform::glfw_platform::GlfwPlatform;
use crate::libs::platform::PlatformBackend;
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::network_debug_overlay::NetworkOverlayState;
use std::cell::RefCell;
use std::time::Instant;

#[cfg(feature = "native")]
use glfw::{Key, MouseButton};

// ============================================================================
// Window State
// ============================================================================

/// Window state attached to a context.
///
/// Composes a [`GlfwPlatform`](crate::libs::platform::glfw_platform::GlfwPlatform)
/// (windowing + input) with an [`OpenGLBackend`]
/// (rendering). The platform backend owns the window handle and event loop;
/// the render backend is stored alongside it.
pub struct WindowState {
    pub(crate) platform: GlfwPlatform,

    /// OpenGL rendering backend
    pub(crate) backend: OpenGLBackend,

    /// Whether the physics debug overlay should render for this context.
    pub(crate) physics_debug_enabled: bool,

    /// Route-independent physics debug setting from the window config.
    pub(crate) base_physics_debug_enabled: bool,

    /// Delta time from last frame
    delta_time: f32,

    /// Debug overlay for FPS stats tracking.
    pub(crate) debug_overlay: DebugOverlay,

    /// Runtime state for the network debug overlay in this context.
    pub(crate) network_overlay: NetworkOverlayState,

    /// Route registered with the debugger runtime for this window, if enabled.
    pub(crate) debugger_route: Option<RuntimeRouteId>,
}

impl WindowState {
    /// Creates a new [`WindowState`] from the given platform and backend.
    pub fn new(
        platform: GlfwPlatform,
        backend: OpenGLBackend,
        physics_debug_enabled: bool,
        debugger_route: Option<RuntimeRouteId>,
    ) -> Self {
        Self {
            platform,
            backend,
            physics_debug_enabled,
            base_physics_debug_enabled: physics_debug_enabled,
            delta_time: 0.0,
            debug_overlay: DebugOverlay::new(0.5),
            network_overlay: NetworkOverlayState::default(),
            debugger_route,
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
    pub fn poll_events(&mut self, context_id: GoudContextId, input: &mut InputManager) -> f32 {
        let old_size = self.platform.get_size();
        let started_at = Instant::now();
        let raw_delta = self.platform.poll_events(input);
        debugger::record_phase_duration("window_events", started_at.elapsed().as_micros() as u64);
        let new_size = self.platform.get_size();

        if old_size != new_size {
            self.backend.set_viewport(0, 0, new_size.0, new_size.1);
        }

        if let Some(route_id) = self.debugger_route.as_ref() {
            let frame_control =
                debugger::take_frame_control_for_route(route_id, raw_delta).unwrap_or_default();
            apply_synthetic_inputs(input, &frame_control.synthetic_inputs);
            self.physics_debug_enabled =
                self.base_physics_debug_enabled || frame_control.debug_draw_enabled;
            self.delta_time = frame_control.effective_delta_seconds;
            let (next_index, total_seconds) = debugger::snapshot_for_route(route_id)
                .map(|snapshot| {
                    (
                        snapshot.frame.index.saturating_add(1),
                        snapshot.frame.total_seconds + self.delta_time as f64,
                    )
                })
                .unwrap_or((1, self.delta_time as f64));
            debugger::begin_frame(route_id, next_index, self.delta_time, total_seconds);
        } else {
            self.delta_time = raw_delta;
        }
        self.debug_overlay.update(self.delta_time);
        let stats = self.debug_overlay.stats();
        let _ = debugger::update_fps_stats_for_context(
            context_id,
            stats.current_fps,
            stats.min_fps,
            stats.max_fps,
            stats.avg_fps,
            stats.frame_time_ms,
        );

        self.delta_time
    }

    /// Swaps the front and back buffers.
    pub fn swap_buffers(&mut self) {
        let started_at = Instant::now();
        self.platform.swap_buffers();
        debugger::record_phase_duration("frame_present", started_at.elapsed().as_micros() as u64);
        if let Some(route_id) = self.debugger_route.as_ref() {
            debugger::end_frame(route_id);
        }
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

#[cfg(feature = "native")]
fn apply_synthetic_inputs(input: &mut InputManager, events: &[debugger::SyntheticInputEventV1]) {
    for event in events {
        match (
            event.device.as_str(),
            event.action.as_str(),
            event.key.as_deref(),
            event.button.as_deref(),
        ) {
            ("keyboard", "press", Some(key), _) => {
                if let Some(key) = parse_key(key) {
                    input.press_key(key);
                }
            }
            ("keyboard", "release", Some(key), _) => {
                if let Some(key) = parse_key(key) {
                    input.release_key(key);
                }
            }
            ("mouse", "press", _, Some(button)) => {
                if let Some(button) = parse_mouse_button(button) {
                    input.press_mouse_button(button);
                }
            }
            ("mouse", "release", _, Some(button)) => {
                if let Some(button) = parse_mouse_button(button) {
                    input.release_mouse_button(button);
                }
            }
            ("mouse", "move", _, _) => {
                if let Some([x, y]) = event.position {
                    input.set_mouse_position(Vec2::new(x, y));
                }
            }
            ("mouse", "scroll", _, _) => {
                if let Some([x, y]) = event.delta {
                    input.add_scroll_delta(Vec2::new(x, y));
                }
            }
            _ => {}
        }
    }
}

#[cfg(feature = "native")]
fn parse_key(key: &str) -> Option<Key> {
    match key.to_ascii_lowercase().as_str() {
        "space" => Some(Key::Space),
        "enter" => Some(Key::Enter),
        "escape" => Some(Key::Escape),
        "tab" => Some(Key::Tab),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "up" => Some(Key::Up),
        "down" => Some(Key::Down),
        "a" => Some(Key::A),
        "d" => Some(Key::D),
        "s" => Some(Key::S),
        "w" => Some(Key::W),
        _ => None,
    }
}

#[cfg(feature = "native")]
fn parse_mouse_button(button: &str) -> Option<MouseButton> {
    match button.to_ascii_lowercase().as_str() {
        "left" => Some(MouseButton::Button1),
        "right" => Some(MouseButton::Button2),
        "middle" => Some(MouseButton::Button3),
        _ => None,
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
        let state = states.get_mut(index).and_then(|opt| opt.as_mut())?;
        let route_id = state.debugger_route.clone();
        Some(debugger::scoped_route(route_id, || f(state)))
    })
}

/// Reads the default framebuffer for one context as RGBA8 bytes.
pub fn read_default_framebuffer_rgba8_for_context(
    context_id: GoudContextId,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, GoudError> {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return Err(GoudError::InvalidContext);
    }

    with_window_state(context_id, |state| {
        state
            .backend_mut()
            .read_default_framebuffer_rgba8(width, height)
    })
    .ok_or(GoudError::InvalidContext)?
    .map_err(GoudError::InvalidState)
}
