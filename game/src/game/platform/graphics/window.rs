// window.rs

use glfw::Key;
use glfw::{Action, Context, WindowEvent};
use std::{collections::btree_map::Keys, sync::mpsc::Receiver};
mod input_handler;
use input_handler::InputHandler;

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
///

#[repr(C)]
pub struct Window {
    pub glfw: glfw::Glfw,
    pub window_handle: Box<glfw::Window>,
    pub events: Receiver<(f64, WindowEvent)>,
    pub input_handler: InputHandler,
}

#[repr(C)]
pub struct WindowBuilder {
    pub width: u32,
    pub height: u32,
    pub title: *const std::ffi::c_char,
}

impl Window {
    pub fn new(data: WindowBuilder) -> Window {
        let WindowBuilder {
            width,
            height,
            title,
        } = data;

        let title = unsafe {
            std::ffi::CStr::from_ptr(title)
                .to_string_lossy()
                .into_owned()
        };

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut window, events) = glfw
            .create_window(width, height, &title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window!");

        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);

        Window {
            glfw,
            window_handle: Box::new(window),
            events,
            input_handler: InputHandler::new(),
        }
    }

    pub fn init_gl(&mut self) {
        self.window_handle.make_current();
        gl::load_with(|s| self.window_handle.get_proc_address(s) as *const _);
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    pub fn update(&mut self) {
        self.process_events();
        self.glfw.poll_events();
        self.window_handle.swap_buffers();
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_handler.is_key_pressed(key)
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            self.input_handler.handle_event(&event);
            if let WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                self.window_handle.set_should_close(true);
            }
        }
    }
}