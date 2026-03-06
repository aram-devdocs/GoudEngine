//! winit implementation of the [`PlatformBackend`] trait.
//!
//! Provides cross-platform windowing using winit, supporting both desktop and
//! (future) web targets. Desktop mode uses [`pump_app_events`] for non-blocking
//! event polling compatible with the engine's frame-based game loop.
//!
//! # Requirements
//!
//! This module requires both `wgpu-backend` and `native` features because the
//! [`InputManager`] currently depends on `glfw::Key` and `glfw::MouseButton`
//! types. Key mapping from winit to GLFW types is handled internally.
//!
//! [`pump_app_events`]: winit::platform::pump_events::EventLoopExtPumpEvents

use crate::core::error::{GoudError, GoudResult};
use crate::core::math::Vec2;
use crate::ecs::InputManager;
use crate::libs::platform::{PlatformBackend, WindowConfig};

use glfw::{Key, MouseButton};
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
    width: u32,
    height: u32,
    last_frame_time: Instant,
    title: String,
    _vsync: bool,
    resizable: bool,
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
            width: config.width,
            height: config.height,
            last_frame_time: Instant::now(),
            title: config.title.clone(),
            _vsync: config.vsync,
            resizable: config.resizable,
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

        Ok(Self { event_loop, state })
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

    fn get_framebuffer_size(&self) -> (u32, u32) {
        self.state
            .window
            .as_ref()
            .map(|w| {
                let size = w.inner_size();
                (size.width, size.height)
            })
            .unwrap_or((self.state.width, self.state.height))
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
                self.state.width = size.width;
                self.state.height = size.height;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    if let Some(glfw_key) = map_keycode(keycode) {
                        match event.state {
                            ElementState::Pressed => self.input.press_key(glfw_key),
                            ElementState::Released => self.input.release_key(glfw_key),
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(glfw_btn) = map_mouse_button(button) {
                    match state {
                        ElementState::Pressed => self.input.press_mouse_button(glfw_btn),
                        ElementState::Released => self.input.release_mouse_button(glfw_btn),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_mouse_position(Vec2::new(position.x as f32, position.y as f32));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.input.add_scroll_delta(Vec2::new(dx, dy));
            }
            _ => {}
        }
    }
}

// =============================================================================
// Key and mouse button mapping: winit → GLFW
// =============================================================================

fn map_mouse_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Button1),
        winit::event::MouseButton::Right => Some(MouseButton::Button2),
        winit::event::MouseButton::Middle => Some(MouseButton::Button3),
        winit::event::MouseButton::Back => Some(MouseButton::Button4),
        winit::event::MouseButton::Forward => Some(MouseButton::Button5),
        _ => None,
    }
}

fn map_keycode(key: KeyCode) -> Option<Key> {
    Some(match key {
        KeyCode::Space => Key::Space,
        KeyCode::Quote => Key::Apostrophe,
        KeyCode::Comma => Key::Comma,
        KeyCode::Minus => Key::Minus,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
        KeyCode::Digit0 => Key::Num0,
        KeyCode::Digit1 => Key::Num1,
        KeyCode::Digit2 => Key::Num2,
        KeyCode::Digit3 => Key::Num3,
        KeyCode::Digit4 => Key::Num4,
        KeyCode::Digit5 => Key::Num5,
        KeyCode::Digit6 => Key::Num6,
        KeyCode::Digit7 => Key::Num7,
        KeyCode::Digit8 => Key::Num8,
        KeyCode::Digit9 => Key::Num9,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Equal => Key::Equal,
        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,
        KeyCode::BracketLeft => Key::LeftBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::BracketRight => Key::RightBracket,
        KeyCode::Backquote => Key::GraveAccent,
        KeyCode::Escape => Key::Escape,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::CapsLock => Key::CapsLock,
        KeyCode::ScrollLock => Key::ScrollLock,
        KeyCode::NumLock => Key::NumLock,
        KeyCode::PrintScreen => Key::PrintScreen,
        KeyCode::Pause => Key::Pause,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::Numpad0 => Key::Kp0,
        KeyCode::Numpad1 => Key::Kp1,
        KeyCode::Numpad2 => Key::Kp2,
        KeyCode::Numpad3 => Key::Kp3,
        KeyCode::Numpad4 => Key::Kp4,
        KeyCode::Numpad5 => Key::Kp5,
        KeyCode::Numpad6 => Key::Kp6,
        KeyCode::Numpad7 => Key::Kp7,
        KeyCode::Numpad8 => Key::Kp8,
        KeyCode::Numpad9 => Key::Kp9,
        KeyCode::NumpadDecimal => Key::KpDecimal,
        KeyCode::NumpadDivide => Key::KpDivide,
        KeyCode::NumpadMultiply => Key::KpMultiply,
        KeyCode::NumpadSubtract => Key::KpSubtract,
        KeyCode::NumpadAdd => Key::KpAdd,
        KeyCode::NumpadEnter => Key::KpEnter,
        KeyCode::NumpadEqual => Key::KpEqual,
        KeyCode::ShiftLeft => Key::LeftShift,
        KeyCode::ControlLeft => Key::LeftControl,
        KeyCode::AltLeft => Key::LeftAlt,
        KeyCode::SuperLeft => Key::LeftSuper,
        KeyCode::ShiftRight => Key::RightShift,
        KeyCode::ControlRight => Key::RightControl,
        KeyCode::AltRight => Key::RightAlt,
        KeyCode::SuperRight => Key::RightSuper,
        _ => return None,
    })
}
