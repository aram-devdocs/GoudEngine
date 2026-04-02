//! winit implementation of the [`PlatformBackend`] trait.
//!
//! Provides cross-platform windowing using winit, supporting both desktop and
//! (future) web targets. Desktop mode uses [`pump_app_events`] for non-blocking
//! event polling compatible with the engine's frame-based game loop.
//!
//! # Requirements
//!
//! This module requires both `wgpu-backend` and `native` features. Key mapping
//! from winit is translated into the engine's platform-neutral input enums.
//!
//! [`pump_app_events`]: winit::platform::pump_events::EventLoopExtPumpEvents

use crate::core::input_manager::InputManager;
use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode as EngineKey, MouseButton as EngineMouseButton};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::platform::{PlatformBackend, WindowConfig};

use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowAttributes, WindowId};

/// Internal state separated from `EventLoop` to avoid borrow conflicts
/// when pumping events (handler borrows state while event loop is &mut self).
struct WinitState {
    window: Option<Arc<Window>>,
    should_close: bool,
    is_suspended: bool,
    width: u32,
    height: u32,
    last_frame_time: Instant,
    title: String,
    _vsync: bool,
    resizable: bool,
    fullscreen_mode: super::FullscreenMode,
}

/// winit-based platform backend for desktop (and future web) windowing.
///
/// The window is created lazily during the first event pump via the
/// [`ApplicationHandler::resumed`] callback, which is the canonical winit 0.30
/// pattern for cross-platform window creation.
///
/// # Usage with wgpu
///
/// ```rust,ignore
/// let platform = WinitPlatform::new(&config)?;
/// let backend = WgpuBackend::new(platform.window().clone())?;
/// ```
pub struct WinitPlatform {
    event_loop: EventLoop<()>,
    state: WinitState,
}

impl WinitPlatform {
    /// Creates a new winit platform with a window.
    ///
    /// Internally pumps one round of events to trigger the `resumed` callback
    /// which creates the window.
    pub fn new(config: &WindowConfig) -> GoudResult<Self> {
        let mut event_loop = EventLoop::new()
            .map_err(|e| GoudError::WindowCreationFailed(format!("EventLoop: {e}")))?;

        let mut state = WinitState {
            window: None,
            should_close: false,
            is_suspended: false,
            width: config.width,
            height: config.height,
            last_frame_time: Instant::now(),
            title: config.title.clone(),
            _vsync: config.vsync,
            resizable: config.resizable,
            fullscreen_mode: config.fullscreen_mode,
        };

        {
            let mut dummy_input = InputManager::new();
            let mut handler = WinitEventHandler {
                state: &mut state,
                input: &mut dummy_input,
            };
            let _ = event_loop.pump_app_events(Some(Duration::from_millis(100)), &mut handler);
        }

        if state.window.is_none() {
            return Err(GoudError::WindowCreationFailed(
                "winit window not created after initial pump".into(),
            ));
        }

        let mut platform = Self { event_loop, state };

        // Apply initial fullscreen mode from config.
        if config.fullscreen_mode != super::FullscreenMode::Windowed {
            platform.set_fullscreen(config.fullscreen_mode);
        }

        Ok(platform)
    }

    /// Returns the winit window handle for wgpu surface creation.
    pub fn window(&self) -> &Arc<Window> {
        self.state
            .window
            .as_ref()
            .expect("window must be created before calling window()")
    }
}

impl PlatformBackend for WinitPlatform {
    fn should_close(&self) -> bool {
        self.state.should_close
    }

    fn set_should_close(&mut self, should_close: bool) {
        self.state.should_close = should_close;
    }

    fn poll_events(&mut self, input: &mut InputManager) -> f32 {
        input.update();

        if self.state.is_suspended {
            return 0.0;
        }

        {
            let mut handler = WinitEventHandler {
                state: &mut self.state,
                input,
            };
            let _ = self
                .event_loop
                .pump_app_events(Some(Duration::ZERO), &mut handler);
        }

        let now = Instant::now();
        let delta = (now - self.state.last_frame_time).as_secs_f32();
        self.state.last_frame_time = now;
        delta
    }

    fn swap_buffers(&mut self) {
        // wgpu handles presentation in WgpuBackend::end_frame — no-op here.
    }

    fn get_size(&self) -> (u32, u32) {
        (self.state.width, self.state.height)
    }

    fn request_size(&mut self, width: u32, height: u32) -> bool {
        let Some(window) = self.state.window.as_ref() else {
            return false;
        };

        self.state.width = width;
        self.state.height = height;
        let _ = window.request_inner_size(LogicalSize::new(width as f64, height as f64));
        true
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        self.state
            .window
            .as_ref()
            .map(|w| {
                let size = w.inner_size();
                let scale = w.scale_factor().max(1.0);
                let expected_width = (self.state.width as f64 * scale).round() as u32;
                let expected_height = (self.state.height as f64 * scale).round() as u32;
                (
                    size.width.max(expected_width.max(1)),
                    size.height.max(expected_height.max(1)),
                )
            })
            .unwrap_or((self.state.width, self.state.height))
    }

    fn set_fullscreen(&mut self, mode: super::FullscreenMode) -> bool {
        let Some(window) = self.state.window.as_ref() else {
            return false;
        };
        let winit_mode = match mode {
            super::FullscreenMode::Windowed => None,
            super::FullscreenMode::Borderless => Some(winit::window::Fullscreen::Borderless(None)),
            super::FullscreenMode::Exclusive => {
                let monitor = match window.current_monitor() {
                    Some(m) => m,
                    None => return false,
                };
                let video_mode = match monitor.video_modes().next() {
                    Some(v) => v,
                    None => return false,
                };
                Some(winit::window::Fullscreen::Exclusive(video_mode))
            }
        };
        window.set_fullscreen(winit_mode);
        self.state.fullscreen_mode = mode;
        true
    }

    fn get_fullscreen(&self) -> super::FullscreenMode {
        self.state.fullscreen_mode
    }

    fn is_suspended(&self) -> bool {
        self.state.is_suspended
    }
}

// =============================================================================
// ApplicationHandler — bridges winit events to InputManager
// =============================================================================

struct WinitEventHandler<'a> {
    state: &'a mut WinitState,
    input: &'a mut InputManager,
}

impl ApplicationHandler for WinitEventHandler<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.window.is_some() {
            // Re-entering foreground on mobile -- clear suspended flag
            self.state.is_suspended = false;
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(&self.state.title)
            .with_inner_size(LogicalSize::new(self.state.width, self.state.height))
            .with_resizable(self.state.resizable);

        match event_loop.create_window(attrs) {
            Ok(window) => {
                self.state.window = Some(Arc::new(window));
            }
            Err(e) => {
                log::error!("Failed to create winit window: {e}");
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state.is_suspended = true;
        self.input.clear();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.state.should_close = true;
            }
            WindowEvent::Resized(size) => {
                if let Some(window) = self.state.window.as_ref() {
                    let logical = size.to_logical::<f64>(window.scale_factor());
                    self.state.width = logical.width.round().max(1.0) as u32;
                    self.state.height = logical.height.round().max(1.0) as u32;
                } else {
                    self.state.width = size.width;
                    self.state.height = size.height;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    if let Some(engine_key) = map_keycode(keycode) {
                        match event.state {
                            ElementState::Pressed => self.input.press_key(engine_key),
                            ElementState::Released => self.input.release_key(engine_key),
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(engine_btn) = map_mouse_button(button) {
                    match state {
                        ElementState::Pressed => self.input.press_mouse_button(engine_btn),
                        ElementState::Released => self.input.release_mouse_button(engine_btn),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self
                    .state
                    .window
                    .as_ref()
                    .map(|w| w.scale_factor())
                    .unwrap_or(1.0);
                self.input.set_mouse_position(Vec2::new(
                    (position.x / scale) as f32,
                    (position.y / scale) as f32,
                ));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.input.add_scroll_delta(Vec2::new(dx, dy));
            }
            WindowEvent::Touch(touch) => {
                let scale = self
                    .state
                    .window
                    .as_ref()
                    .map(|w| w.scale_factor())
                    .unwrap_or(1.0);
                let position = Vec2::new(
                    (touch.location.x / scale) as f32,
                    (touch.location.y / scale) as f32,
                );
                match touch.phase {
                    winit::event::TouchPhase::Started => {
                        self.input.touch_start(touch.id, position);
                    }
                    winit::event::TouchPhase::Moved => {
                        self.input.touch_move(touch.id, position);
                    }
                    winit::event::TouchPhase::Ended => {
                        self.input.touch_end(touch.id);
                    }
                    winit::event::TouchPhase::Cancelled => {
                        self.input.touch_cancel(touch.id);
                    }
                }
            }
            _ => {}
        }
    }
}

// =============================================================================
// Key and mouse button mapping: winit → engine-neutral input types
// =============================================================================

fn map_mouse_button(button: winit::event::MouseButton) -> Option<EngineMouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(EngineMouseButton::Left),
        winit::event::MouseButton::Right => Some(EngineMouseButton::Right),
        winit::event::MouseButton::Middle => Some(EngineMouseButton::Middle),
        winit::event::MouseButton::Back => Some(EngineMouseButton::Button4),
        winit::event::MouseButton::Forward => Some(EngineMouseButton::Button5),
        _ => None,
    }
}

fn map_keycode(key: KeyCode) -> Option<EngineKey> {
    Some(match key {
        KeyCode::Space => EngineKey::Space,
        KeyCode::Quote => EngineKey::Apostrophe,
        KeyCode::Comma => EngineKey::Comma,
        KeyCode::Minus => EngineKey::Minus,
        KeyCode::Period => EngineKey::Period,
        KeyCode::Slash => EngineKey::Slash,
        KeyCode::Digit0 => EngineKey::Num0,
        KeyCode::Digit1 => EngineKey::Num1,
        KeyCode::Digit2 => EngineKey::Num2,
        KeyCode::Digit3 => EngineKey::Num3,
        KeyCode::Digit4 => EngineKey::Num4,
        KeyCode::Digit5 => EngineKey::Num5,
        KeyCode::Digit6 => EngineKey::Num6,
        KeyCode::Digit7 => EngineKey::Num7,
        KeyCode::Digit8 => EngineKey::Num8,
        KeyCode::Digit9 => EngineKey::Num9,
        KeyCode::Semicolon => EngineKey::Semicolon,
        KeyCode::Equal => EngineKey::Equal,
        KeyCode::KeyA => EngineKey::A,
        KeyCode::KeyB => EngineKey::B,
        KeyCode::KeyC => EngineKey::C,
        KeyCode::KeyD => EngineKey::D,
        KeyCode::KeyE => EngineKey::E,
        KeyCode::KeyF => EngineKey::F,
        KeyCode::KeyG => EngineKey::G,
        KeyCode::KeyH => EngineKey::H,
        KeyCode::KeyI => EngineKey::I,
        KeyCode::KeyJ => EngineKey::J,
        KeyCode::KeyK => EngineKey::K,
        KeyCode::KeyL => EngineKey::L,
        KeyCode::KeyM => EngineKey::M,
        KeyCode::KeyN => EngineKey::N,
        KeyCode::KeyO => EngineKey::O,
        KeyCode::KeyP => EngineKey::P,
        KeyCode::KeyQ => EngineKey::Q,
        KeyCode::KeyR => EngineKey::R,
        KeyCode::KeyS => EngineKey::S,
        KeyCode::KeyT => EngineKey::T,
        KeyCode::KeyU => EngineKey::U,
        KeyCode::KeyV => EngineKey::V,
        KeyCode::KeyW => EngineKey::W,
        KeyCode::KeyX => EngineKey::X,
        KeyCode::KeyY => EngineKey::Y,
        KeyCode::KeyZ => EngineKey::Z,
        KeyCode::Escape => EngineKey::Escape,
        KeyCode::Enter => EngineKey::Enter,
        KeyCode::NumpadEnter => EngineKey::KpEnter,
        KeyCode::Tab => EngineKey::Tab,
        KeyCode::Backspace => EngineKey::Backspace,
        KeyCode::Insert => EngineKey::Insert,
        KeyCode::Delete => EngineKey::Delete,
        KeyCode::ArrowRight => EngineKey::Right,
        KeyCode::ArrowLeft => EngineKey::Left,
        KeyCode::ArrowDown => EngineKey::Down,
        KeyCode::ArrowUp => EngineKey::Up,
        KeyCode::PageUp => EngineKey::PageUp,
        KeyCode::PageDown => EngineKey::PageDown,
        KeyCode::Home => EngineKey::Home,
        KeyCode::End => EngineKey::End,
        KeyCode::F1 => EngineKey::F1,
        KeyCode::F2 => EngineKey::F2,
        KeyCode::F3 => EngineKey::F3,
        KeyCode::F4 => EngineKey::F4,
        KeyCode::F5 => EngineKey::F5,
        KeyCode::F6 => EngineKey::F6,
        KeyCode::F7 => EngineKey::F7,
        KeyCode::F8 => EngineKey::F8,
        KeyCode::F9 => EngineKey::F9,
        KeyCode::F10 => EngineKey::F10,
        KeyCode::F11 => EngineKey::F11,
        KeyCode::F12 => EngineKey::F12,
        KeyCode::ShiftLeft => EngineKey::LeftShift,
        KeyCode::ControlLeft => EngineKey::LeftControl,
        KeyCode::AltLeft => EngineKey::LeftAlt,
        KeyCode::SuperLeft => EngineKey::LeftSuper,
        KeyCode::ShiftRight => EngineKey::RightShift,
        KeyCode::ControlRight => EngineKey::RightControl,
        KeyCode::AltRight => EngineKey::RightAlt,
        KeyCode::SuperRight => EngineKey::RightSuper,
        _ => return None,
    })
}
