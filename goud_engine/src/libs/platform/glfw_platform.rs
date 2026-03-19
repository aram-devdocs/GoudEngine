//! GLFW implementation of the [`PlatformBackend`] trait.
//!
//! Provides desktop window management using GLFW with OpenGL 3.3 Core context.
//! This module handles GLFW initialization, window creation, event polling,
//! and input dispatch to the [`InputManager`].
//!
//! # Thread Safety
//!
//! GLFW must be used from the main thread only. The [`GlfwPlatform`] is
//! neither `Send` nor `Sync` by design.

use crate::core::input_manager::InputManager;
use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode, MouseButton as EngineMouseButton};
use crate::libs::error::{GoudError, GoudResult};
use crate::libs::platform::{PlatformBackend, WindowConfig};

use glfw::{Action, Context, Glfw, GlfwReceiver, PWindow, WindowEvent, WindowMode};
use std::cell::RefCell;

thread_local! {
    static GLFW_INSTANCE: RefCell<Option<Glfw>> = const { RefCell::new(None) };
}

fn get_or_init_glfw() -> GoudResult<Glfw> {
    GLFW_INSTANCE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            let glfw = glfw::init(glfw::fail_on_errors).map_err(|e| {
                GoudError::InternalError(format!("Failed to initialize GLFW: {}", e))
            })?;
            *borrow = Some(glfw);
        }
        borrow
            .clone()
            .ok_or_else(|| GoudError::InternalError("GLFW not initialized".to_string()))
    })
}

/// GLFW-based platform backend for desktop window management.
///
/// Wraps a GLFW window, event receiver, and timing state. After construction
/// the OpenGL context is current and GL function pointers are loaded, so the
/// caller can immediately create an [`OpenGLBackend`](crate::libs::graphics::backend::opengl::OpenGLBackend).
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine::libs::platform::{WindowConfig, PlatformBackend};
/// use goud_engine::libs::platform::glfw_platform::GlfwPlatform;
///
/// let config = WindowConfig { width: 800, height: 600, ..Default::default() };
/// let mut platform = GlfwPlatform::new(&config)?;
///
/// while !platform.should_close() {
///     let dt = platform.poll_events(&mut input_manager);
///     // ... render ...
///     platform.swap_buffers();
/// }
/// ```
pub struct GlfwPlatform {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    last_frame_time: f64,
    width: u32,
    height: u32,
    fullscreen_mode: super::FullscreenMode,
    /// Saved window position/size before entering fullscreen, for restore.
    pre_fullscreen_rect: Option<(i32, i32, u32, u32)>,
}

/// Creates a GLFW platform with an OpenGL 3.3 Core window.
///
/// After this returns successfully:
/// - The OpenGL context is current on the calling thread
/// - GL function pointers are loaded via `gl::load_with`
/// - VSync is set according to `config.vsync`
/// - Key, mouse, cursor, scroll, close, and resize polling are enabled
impl GlfwPlatform {
    /// Creates a new GLFW platform backend with a window.
    pub fn new(config: &WindowConfig) -> GoudResult<Self> {
        let mut glfw = get_or_init_glfw()?;

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::Samples(if config.msaa_samples > 1 {
            Some(config.msaa_samples)
        } else {
            None
        }));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, events) = glfw
            .create_window(
                config.width,
                config.height,
                &config.title,
                WindowMode::Windowed,
            )
            .ok_or_else(|| {
                GoudError::WindowCreationFailed("Failed to create GLFW window".to_string())
            })?;

        window.make_current();

        if config.vsync {
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        }

        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_close_polling(true);
        window.set_size_polling(true);
        window.set_scroll_polling(true);

        gl::load_with(|s| window.get_proc_address(s));

        let last_frame_time = glfw.get_time();

        let mut platform = Self {
            glfw,
            window,
            events,
            last_frame_time,
            width: config.width,
            height: config.height,
            fullscreen_mode: super::FullscreenMode::Windowed,
            pre_fullscreen_rect: None,
        };

        // Apply initial fullscreen mode from config.
        if config.fullscreen_mode != super::FullscreenMode::Windowed {
            platform.set_fullscreen(config.fullscreen_mode);
        }

        Ok(platform)
    }
}

impl PlatformBackend for GlfwPlatform {
    fn should_close(&self) -> bool {
        self.window.should_close()
    }

    fn set_should_close(&mut self, should_close: bool) {
        self.window.set_should_close(should_close);
    }

    fn poll_events(&mut self, input: &mut InputManager) -> f32 {
        input.update();
        self.glfw.poll_events();

        let current_time = self.glfw.get_time();
        let delta_time = (current_time - self.last_frame_time) as f32;
        self.last_frame_time = current_time;

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(key, _scancode, action, _mods) => match action {
                    Action::Press => {
                        if let Some(key) = KeyCode::from_u32(key as i32 as u32) {
                            input.press_key(key);
                        }
                    }
                    Action::Release => {
                        if let Some(key) = KeyCode::from_u32(key as i32 as u32) {
                            input.release_key(key);
                        }
                    }
                    Action::Repeat => {}
                },
                WindowEvent::MouseButton(button, action, _mods) => match action {
                    Action::Press => {
                        if let Some(button) = EngineMouseButton::from_u32(button as u32) {
                            input.press_mouse_button(button);
                        }
                    }
                    Action::Release => {
                        if let Some(button) = EngineMouseButton::from_u32(button as u32) {
                            input.release_mouse_button(button);
                        }
                    }
                    Action::Repeat => {}
                },
                WindowEvent::CursorPos(x, y) => {
                    input.set_mouse_position(Vec2::new(x as f32, y as f32));
                }
                WindowEvent::Scroll(x, y) => {
                    input.add_scroll_delta(Vec2::new(x as f32, y as f32));
                }
                WindowEvent::Close => {
                    self.window.set_should_close(true);
                }
                WindowEvent::Size(w, h) => {
                    self.width = w as u32;
                    self.height = h as u32;
                }
                _ => {}
            }
        }

        delta_time
    }

    fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn request_size(&mut self, width: u32, height: u32) -> bool {
        self.width = width;
        self.height = height;
        self.window.set_size(width as i32, height as i32);
        true
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        let (w, h) = self.window.get_framebuffer_size();
        (w as u32, h as u32)
    }

    fn set_fullscreen(&mut self, mode: super::FullscreenMode) -> bool {
        match mode {
            super::FullscreenMode::Windowed => {
                if let Some((x, y, w, h)) = self.pre_fullscreen_rect.take() {
                    self.window
                        .set_monitor(WindowMode::Windowed, x, y, w, h, None);
                    self.width = w;
                    self.height = h;
                }
                self.fullscreen_mode = super::FullscreenMode::Windowed;
                true
            }
            super::FullscreenMode::Borderless | super::FullscreenMode::Exclusive => {
                // Save current window position and size for later restore.
                let (x, y) = self.window.get_pos();
                let (w, h) = self.window.get_size();
                self.pre_fullscreen_rect = Some((x, y, w as u32, h as u32));

                self.glfw.with_primary_monitor(|_, monitor| {
                    if let Some(monitor) = monitor {
                        if let Some(vid_mode) = monitor.get_video_mode() {
                            self.window.set_monitor(
                                WindowMode::FullScreen(monitor),
                                0,
                                0,
                                vid_mode.width,
                                vid_mode.height,
                                Some(vid_mode.refresh_rate),
                            );
                            self.width = vid_mode.width;
                            self.height = vid_mode.height;
                        }
                    }
                });
                self.fullscreen_mode = mode;
                true
            }
        }
    }

    fn get_fullscreen(&self) -> super::FullscreenMode {
        self.fullscreen_mode
    }
}
