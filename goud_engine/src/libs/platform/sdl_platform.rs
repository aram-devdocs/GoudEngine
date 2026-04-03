//! SDL2 platform backend.
//!
//! Provides a [`PlatformBackend`] implementation using SDL2 for windowing and
//! input. On non-desktop targets the constructor returns
//! [`GoudError::BackendNotSupported`] so the module can still be type-checked.

use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{GoudError, GoudResult};
use crate::core::input_manager::InputManager;
use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode, MouseButton as EngineMouseButton};

use super::{FullscreenMode, PlatformBackend, WindowConfig};
use crate::libs::graphics::backend::wgpu_backend::sdl_surface::SdlWindowHandle;

/// SDL2 platform backend.
///
/// Wraps an SDL2 window and implements the engine's platform abstraction.
/// Uses SDL2's `raw-window-handle` support to provide wgpu-compatible
/// window handles for Vulkan surface creation.
pub struct SdlPlatform {
    should_close: bool,
    width: u32,
    height: u32,
    last_frame: Instant,
    handle: Arc<SdlWindowHandle>,
    /// Retained to keep the SDL2 context alive; dropped on shutdown.
    #[allow(dead_code)]
    sdl_context: sdl2::Sdl,
    /// Retained to keep the SDL2 video subsystem alive.
    #[allow(dead_code)]
    video: sdl2::VideoSubsystem,
    /// The SDL2 window driving the event pump.
    sdl_window: sdl2::video::Window,
    /// SDL2 event pump for polling OS events.
    event_pump: sdl2::EventPump,
}

impl SdlPlatform {
    /// Creates the SDL2 platform.
    ///
    /// Initialises SDL2, creates a window with the requested configuration,
    /// and extracts raw window handles for wgpu surface creation.
    pub fn new(config: &WindowConfig) -> GoudResult<Self> {
        Self::new_inner(config)
    }

    /// Returns the window handle wrapper for wgpu surface creation.
    pub fn window_handle(&self) -> Arc<SdlWindowHandle> {
        Arc::clone(&self.handle)
    }

    // ------------------------------------------------------------------
    // Desktop implementation (actual SDL2 path)
    // ------------------------------------------------------------------

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn new_inner(config: &WindowConfig) -> GoudResult<Self> {
        let sdl_context = sdl2::init()
            .map_err(|e| GoudError::InitializationFailed(format!("SDL2 init failed: {e}")))?;
        let video = sdl_context
            .video()
            .map_err(|e| GoudError::InitializationFailed(format!("SDL2 video init failed: {e}")))?;

        let sdl_window = video
            .window(&config.title, config.width, config.height)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| {
                GoudError::InitializationFailed(format!("SDL2 window creation failed: {e}"))
            })?;

        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| GoudError::InitializationFailed(format!("SDL2 event pump failed: {e}")))?;

        // Extract raw window handle via SDL2's raw-window-handle 0.6 support.
        let handle = SdlWindowHandle::from_sdl_window(&sdl_window)?;

        let (w, h) = sdl_window.size();

        Ok(Self {
            should_close: false,
            width: w,
            height: h,
            last_frame: Instant::now(),
            handle: Arc::new(handle),
            sdl_context,
            video,
            sdl_window,
            event_pump,
        })
    }

    // ------------------------------------------------------------------
    // Non-desktop stub (allows type-checking on other targets)
    // ------------------------------------------------------------------

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn new_inner(_config: &WindowConfig) -> GoudResult<Self> {
        Err(GoudError::BackendNotSupported(
            "SDL2 windowing is only available on desktop targets".to_string(),
        ))
    }
}

impl PlatformBackend for SdlPlatform {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn set_should_close(&mut self, should_close: bool) {
        self.should_close = should_close;
    }

    fn poll_events(&mut self, input: &mut InputManager) -> f32 {
        input.update();

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    self.should_close = true;
                }
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    self.width = w as u32;
                    self.height = h as u32;
                }
                sdl2::event::Event::KeyDown {
                    scancode: Some(sc),
                    repeat: false,
                    ..
                } => {
                    if let Some(key) = map_sdl_scancode(sc) {
                        input.press_key(key);
                    }
                }
                sdl2::event::Event::KeyUp {
                    scancode: Some(sc), ..
                } => {
                    if let Some(key) = map_sdl_scancode(sc) {
                        input.release_key(key);
                    }
                }
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    input.set_mouse_position(Vec2::new(x as f32, y as f32));
                }
                sdl2::event::Event::MouseButtonDown { mouse_btn, .. } => {
                    if let Some(btn) = map_sdl_mouse_button(mouse_btn) {
                        input.press_mouse_button(btn);
                    }
                }
                sdl2::event::Event::MouseButtonUp { mouse_btn, .. } => {
                    if let Some(btn) = map_sdl_mouse_button(mouse_btn) {
                        input.release_mouse_button(btn);
                    }
                }
                sdl2::event::Event::MouseWheel { x, y, .. } => {
                    input.add_scroll_delta(Vec2::new(x as f32, y as f32));
                }
                _ => {}
            }
        }

        dt
    }

    fn swap_buffers(&mut self) {
        // No-op: wgpu handles presentation via surface.present().
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn request_size(&mut self, width: u32, height: u32) -> bool {
        self.sdl_window
            .set_size(width, height)
            .map(|()| {
                self.width = width;
                self.height = height;
            })
            .is_ok()
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        self.sdl_window.drawable_size()
    }

    fn set_fullscreen(&mut self, mode: FullscreenMode) -> bool {
        let sdl_mode = match mode {
            FullscreenMode::Windowed => sdl2::video::FullscreenType::Off,
            FullscreenMode::Borderless => sdl2::video::FullscreenType::Desktop,
            FullscreenMode::Exclusive => sdl2::video::FullscreenType::True,
        };
        self.sdl_window.set_fullscreen(sdl_mode).is_ok()
    }

    fn get_fullscreen(&self) -> FullscreenMode {
        match self.sdl_window.fullscreen_state() {
            sdl2::video::FullscreenType::Off => FullscreenMode::Windowed,
            sdl2::video::FullscreenType::Desktop => FullscreenMode::Borderless,
            sdl2::video::FullscreenType::True => FullscreenMode::Exclusive,
        }
    }
}

/// Maps an SDL2 scancode to the engine's platform-neutral [`KeyCode`].
///
/// The engine's `KeyCode` values follow the GLFW convention. SDL2 scancodes
/// differ, so we map the most common keys. Returns `None` for unmapped codes.
fn map_sdl_scancode(sc: sdl2::keyboard::Scancode) -> Option<KeyCode> {
    use sdl2::keyboard::Scancode;
    match sc {
        Scancode::Space => Some(KeyCode::Space),
        Scancode::A => Some(KeyCode::A),
        Scancode::B => Some(KeyCode::B),
        Scancode::C => Some(KeyCode::C),
        Scancode::D => Some(KeyCode::D),
        Scancode::E => Some(KeyCode::E),
        Scancode::F => Some(KeyCode::F),
        Scancode::G => Some(KeyCode::G),
        Scancode::H => Some(KeyCode::H),
        Scancode::I => Some(KeyCode::I),
        Scancode::J => Some(KeyCode::J),
        Scancode::K => Some(KeyCode::K),
        Scancode::L => Some(KeyCode::L),
        Scancode::M => Some(KeyCode::M),
        Scancode::N => Some(KeyCode::N),
        Scancode::O => Some(KeyCode::O),
        Scancode::P => Some(KeyCode::P),
        Scancode::Q => Some(KeyCode::Q),
        Scancode::R => Some(KeyCode::R),
        Scancode::S => Some(KeyCode::S),
        Scancode::T => Some(KeyCode::T),
        Scancode::U => Some(KeyCode::U),
        Scancode::V => Some(KeyCode::V),
        Scancode::W => Some(KeyCode::W),
        Scancode::X => Some(KeyCode::X),
        Scancode::Y => Some(KeyCode::Y),
        Scancode::Z => Some(KeyCode::Z),
        Scancode::Num0 => Some(KeyCode::Num0),
        Scancode::Num1 => Some(KeyCode::Num1),
        Scancode::Num2 => Some(KeyCode::Num2),
        Scancode::Num3 => Some(KeyCode::Num3),
        Scancode::Num4 => Some(KeyCode::Num4),
        Scancode::Num5 => Some(KeyCode::Num5),
        Scancode::Num6 => Some(KeyCode::Num6),
        Scancode::Num7 => Some(KeyCode::Num7),
        Scancode::Num8 => Some(KeyCode::Num8),
        Scancode::Num9 => Some(KeyCode::Num9),
        Scancode::Escape => Some(KeyCode::Escape),
        Scancode::Return => Some(KeyCode::Enter),
        Scancode::Tab => Some(KeyCode::Tab),
        Scancode::Backspace => Some(KeyCode::Backspace),
        Scancode::Right => Some(KeyCode::Right),
        Scancode::Left => Some(KeyCode::Left),
        Scancode::Down => Some(KeyCode::Down),
        Scancode::Up => Some(KeyCode::Up),
        Scancode::LShift => Some(KeyCode::LeftShift),
        Scancode::RShift => Some(KeyCode::RightShift),
        Scancode::LCtrl => Some(KeyCode::LeftControl),
        Scancode::RCtrl => Some(KeyCode::RightControl),
        Scancode::LAlt => Some(KeyCode::LeftAlt),
        Scancode::RAlt => Some(KeyCode::RightAlt),
        _ => None,
    }
}

/// Maps an SDL2 mouse button to the engine's platform-neutral [`EngineMouseButton`].
fn map_sdl_mouse_button(btn: sdl2::mouse::MouseButton) -> Option<EngineMouseButton> {
    match btn {
        sdl2::mouse::MouseButton::Left => Some(EngineMouseButton::Left),
        sdl2::mouse::MouseButton::Right => Some(EngineMouseButton::Right),
        sdl2::mouse::MouseButton::Middle => Some(EngineMouseButton::Middle),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_sdl_scancode_covers_wasd() {
        use sdl2::keyboard::Scancode;
        assert_eq!(map_sdl_scancode(Scancode::W), Some(KeyCode::W));
        assert_eq!(map_sdl_scancode(Scancode::A), Some(KeyCode::A));
        assert_eq!(map_sdl_scancode(Scancode::S), Some(KeyCode::S));
        assert_eq!(map_sdl_scancode(Scancode::D), Some(KeyCode::D));
    }

    #[test]
    fn map_sdl_mouse_button_covers_basics() {
        assert_eq!(
            map_sdl_mouse_button(sdl2::mouse::MouseButton::Left),
            Some(EngineMouseButton::Left)
        );
        assert_eq!(
            map_sdl_mouse_button(sdl2::mouse::MouseButton::Right),
            Some(EngineMouseButton::Right)
        );
    }

    /// SDL2 requires a display server; skip in headless CI.
    #[ignore]
    #[test]
    fn sdl_platform_creates_and_reports_size() {
        let config = WindowConfig {
            width: 320,
            height: 240,
            title: "sdl-test".to_string(),
            ..WindowConfig::default()
        };
        let platform = SdlPlatform::new(&config).expect("SDL2 should init");
        let (w, h) = platform.get_size();
        assert_eq!((w, h), (320, 240));
    }
}
