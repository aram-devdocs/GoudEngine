//! # Window State
//!
//! Defines [`WindowState`], which composes the selected native platform and
//! render backend into a single per-context object, and the thread-local
//! storage helpers used to access it from FFI functions.

use crate::core::debugger::{self, DeferredCapture, RuntimeRouteId};
use crate::core::error::GoudError;
use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode as Key, MouseButton};
use crate::ecs::InputManager;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::backend::native_backend::SharedNativeRenderBackend;
use crate::libs::graphics::backend::RenderBackend;
use crate::libs::platform::PlatformBackend;
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::network_debug_overlay::NetworkOverlayState;
use std::cell::RefCell;
use std::time::Instant;

// ============================================================================
// Window State
// ============================================================================

/// Window state attached to a context.
///
/// Composes a platform backend (windowing + input) with a native render
/// backend (presentation + GPU resources). The platform backend owns the
/// window handle and event loop; the render backend is stored alongside it.
pub struct WindowState {
    pub(crate) platform: Box<dyn PlatformBackend>,

    /// Active native rendering backend
    pub(crate) backend: SharedNativeRenderBackend,

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

    /// Deferred capture coordination between the IPC thread and the main thread.
    pub(crate) deferred_capture: Option<DeferredCapture>,

    /// Tracks the previous suspended state to detect transitions.
    was_suspended: bool,

    /// Fixed timestep step size in seconds (0.0 = disabled).
    pub(crate) fixed_timestep: f32,

    /// Accumulated time waiting to be consumed by fixed steps.
    pub(crate) accumulator: f32,

    /// Maximum fixed steps allowed per frame.
    pub(crate) max_fixed_steps: u32,

    /// Number of fixed steps consumed this frame.
    pub(crate) fixed_steps_this_frame: u32,

    /// Interpolation alpha for render smoothing (0.0 to 1.0).
    pub(crate) interpolation_alpha: f32,
}

impl WindowState {
    /// Creates a new [`WindowState`] from the given platform and backend.
    pub(crate) fn new(
        platform: Box<dyn PlatformBackend>,
        backend: SharedNativeRenderBackend,
        physics_debug_enabled: bool,
        debugger_route: Option<RuntimeRouteId>,
        deferred_capture: Option<DeferredCapture>,
    ) -> Self {
        Self {
            platform,
            backend,
            physics_debug_enabled,
            base_physics_debug_enabled: physics_debug_enabled,
            delta_time: 0.0,
            was_suspended: false,
            debug_overlay: DebugOverlay::new(0.5),
            network_overlay: NetworkOverlayState::default(),
            debugger_route,
            deferred_capture,
            fixed_timestep: 0.0,
            accumulator: 0.0,
            max_fixed_steps: 8,
            fixed_steps_this_frame: 0,
            interpolation_alpha: 0.0,
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
        let started_at = Instant::now();
        let raw_delta = self.platform.poll_events(input);
        debugger::record_phase_duration("window_events", started_at.elapsed().as_micros() as u64);

        // Detect suspend/resume transitions and manage GPU surface lifecycle.
        let is_suspended = self.platform.is_suspended();
        if is_suspended && !self.was_suspended {
            // Entering suspended state — drop the GPU surface.
            self.backend.drop_surface();
            log::info!("App suspended — GPU surface dropped");
        } else if !is_suspended && self.was_suspended {
            // Resuming from suspended state — recreate the GPU surface.
            if let Err(e) = self.backend.recreate_surface() {
                log::error!("Failed to recreate GPU surface on resume: {e}");
            } else {
                log::info!("App resumed — GPU surface recreated");
            }
        }
        self.was_suspended = is_suspended;

        if is_suspended {
            self.delta_time = 0.0;
            return 0.0;
        }

        let framebuffer_size = self.platform.get_framebuffer_size();
        self.backend
            .resize_surface(framebuffer_size.0, framebuffer_size.1);

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

    /// Services a pending deferred capture request by performing framebuffer
    /// readback on the current main thread, then notifying the waiting IPC thread.
    fn service_deferred_capture(&mut self) {
        let Some(ref deferred) = self.deferred_capture else {
            return;
        };
        let (lock, cvar) = &**deferred;
        let mut guard = match lock.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if !guard.requested {
            return;
        }
        let (w, h) = self.get_framebuffer_size();
        let result = self
            .backend
            .read_default_framebuffer_rgba8(w, h)
            .map(|rgba8| debugger::RawFramebufferReadbackV1 {
                width: w,
                height: h,
                rgba8,
            })
            .map_err(|e| format!("framebuffer readback failed: {e}"));
        guard.result = Some(result);
        cvar.notify_all();
    }

    /// Swaps the front and back buffers.
    pub fn swap_buffers(&mut self) {
        self.service_deferred_capture();
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
    pub fn backend_mut(&mut self) -> &mut SharedNativeRenderBackend {
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
                } else {
                    log::warn!("Ignoring unsupported debugger synthetic key '{key}'");
                }
            }
            ("keyboard", "release", Some(key), _) => {
                if let Some(key) = parse_key(key) {
                    input.release_key(key);
                } else {
                    log::warn!("Ignoring unsupported debugger synthetic key '{key}'");
                }
            }
            ("mouse", "press", _, Some(button)) => {
                if let Some(button) = parse_mouse_button(button) {
                    input.press_mouse_button(button);
                } else {
                    log::warn!("Ignoring unsupported debugger mouse button '{button}'");
                }
            }
            ("mouse", "release", _, Some(button)) => {
                if let Some(button) = parse_mouse_button(button) {
                    input.release_mouse_button(button);
                } else {
                    log::warn!("Ignoring unsupported debugger mouse button '{button}'");
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
    Key::from_debugger_name(key)
}

#[cfg(feature = "native")]
fn parse_mouse_button(button: &str) -> Option<MouseButton> {
    MouseButton::from_debugger_name(button)
}

// ============================================================================
// Window State Storage (Thread-Local)
// ============================================================================

thread_local! {
    pub(super) static WINDOW_STATES: RefCell<Vec<Option<WindowState>>> = const { RefCell::new(Vec::new()) };
}

/// Stores the given [`WindowState`] for the specified context.
pub fn set_window_state(
    context_id: GoudContextId,
    mut state: WindowState,
) -> Result<(), GoudError> {
    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;

        while states.len() <= index {
            states.push(None);
        }

        if let Some(previous) = states[index].take() {
            if let Some(route_id) = previous.debugger_route.as_ref() {
                debugger::unregister_capture_hook_for_route(route_id);
            }
        }

        let deferred_capture = if let Some(ref route_id) = state.debugger_route {
            let deferred = debugger::new_deferred_capture();
            debugger::register_deferred_capture_hook_for_route(route_id.clone(), deferred.clone());
            Some(deferred)
        } else {
            None
        };
        state.deferred_capture = deferred_capture;

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
            if let Some(previous) = states[index].take() {
                if let Some(route_id) = previous.debugger_route.as_ref() {
                    debugger::unregister_capture_hook_for_route(route_id);
                }
            }
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
