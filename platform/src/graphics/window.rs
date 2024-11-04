// window.rs

use glfw::{Action, Context, Key, WindowEvent};
use std::sync::mpsc::Receiver;

mod input_handler;
use input_handler::{InputHandler, KeyInput as _KeyInput};

/// # Window
///
/// An abstraction layer for creating a GLFW window.
///
/// ## Example
/// ```
/// let mut window = Window::new(1280, 720, "Window Title");
/// window.init_gl();
///
/// while !window.should_close() {
///     window.update();
/// }
/// ```
pub struct Window {
    glfw: glfw::Glfw,
    pub window_handle: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    pub input_handler: InputHandler,
}

pub type KeyInput = _KeyInput;

pub struct WindowBuilder {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl Window {
    /// Create new window with settings
    pub fn new(data: WindowBuilder) -> Window {
        let WindowBuilder {
            width,
            height,
            title,
        } = data;

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        // Set OpenGL version (e.g., 3.3 Core)
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        // **Add this line to set forward compatibility**
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, events) = glfw
            .create_window(width, height, &title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window!");

        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);

        Window {
            glfw,
            window_handle: window,
            events,
            input_handler: InputHandler::new(),
        }
    }

    /// Load GL functions.
    pub fn init_gl(&mut self) {
        self.window_handle.make_current();
        gl::load_with(|s| self.window_handle.get_proc_address(s) as *const _);
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    /// Poll events and swap buffers.
    pub fn update(&mut self) {
        self.process_events();
        self.glfw.poll_events();
        self.window_handle.swap_buffers();

        for (_, event) in glfw::flush_messages(&self.events) {
            self.input_handler.handle_event(&event);
        }
    }

    pub fn close_window(&mut self) {
        self.window_handle.set_should_close(true);
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // Make sure the viewport matches the new window dimensions.
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window_handle.set_should_close(true)
                }
                _ => {}
            }
        }
    }
}