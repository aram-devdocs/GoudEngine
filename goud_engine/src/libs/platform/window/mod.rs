mod input_handler;

use glfw::{Context, WindowEvent};
use glfw::{GlfwReceiver, Key};
use std::time::{Duration, Instant};

use input_handler::InputHandler;

use crate::types::MousePosition;

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
// window.rs
#[repr(C)]
pub struct Window {
    pub glfw: glfw::Glfw,
    pub window_handle: glfw::PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub input_handler: InputHandler,
    pub width: u32,
    pub height: u32,
    pub delta_time: f32, // Delta time in seconds
    frame_count: u32,
    elapsed_time: Duration,
    fps: u32,
    last_frame_time: Instant,
    target_fps: u32,
}

#[repr(C)]
pub struct WindowBuilder {
    pub width: u32,
    pub height: u32,
    pub title: *const std::ffi::c_char,
    pub target_fps: u32,
}

impl Window {
    pub fn new(data: WindowBuilder) -> Window {
        let WindowBuilder {
            width,
            height,
            title,
            target_fps,
        } = data;

        let title = unsafe {
            std::ffi::CStr::from_ptr(title)
                .to_string_lossy()
                .into_owned()
        };

        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
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
        window.set_size_polling(true);

        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);

        Window {
            glfw,
            window_handle: window,
            events,
            input_handler: InputHandler::new(),
            width,
            height,
            delta_time: 0.0, // Initialize delta time as 0
            frame_count: 0,
            elapsed_time: Duration::new(0, 0),
            fps: 0,
            last_frame_time: Instant::now(), // Set initial frame time
            target_fps,
        }
    }

    pub fn init_gl(&mut self) {
        self.window_handle.make_current();
        gl::load_with(|s| self.window_handle.get_proc_address(s) as *const _);

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    pub fn update(&mut self) {
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time); // Calculate frame time
        self.last_frame_time = current_time;

        // Update delta time in seconds
        self.delta_time = frame_time.as_secs_f32();

        // Handle frame-based processing
        self.glfw.poll_events();
        self.process_events();

        self.window_handle.swap_buffers();
        self.maintain_aspect_ratio();
        self.glfw.set_swap_interval(glfw::SwapInterval::Sync(0)); // Disable VSync

        // FPS counter
        self.frame_count += 1;
        self.elapsed_time += frame_time;
        if self.elapsed_time >= Duration::from_secs(1) {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.elapsed_time = Duration::new(0, 0);
            let version = env!("CARGO_PKG_VERSION");
            self.window_handle.set_title(&format!(
                "GoudEngine v{} | FPS: {} | Delta Time: {:.4}",
                version, self.fps, self.delta_time
            ));
        }

        let frame_duration = Duration::from_millis(1000 / self.target_fps as u64);

        let frame_end = Instant::now();
        while Instant::now().duration_since(frame_end) < frame_duration {}
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_handler.is_key_pressed(key)
    }

    pub fn is_mouse_button_pressed(&self, button: glfw::MouseButton) -> bool {
        self.input_handler.is_mouse_button_pressed(button)
    }

    pub fn get_mouse_position(&self) -> MousePosition {
        self.input_handler.get_mouse_position()
    }

    pub fn handle_gamepad_button(&mut self, gamepad_id: u32, button: u32, pressed: bool) {
        self.input_handler
            .handle_gamepad_button(gamepad_id, button, pressed);
    }

    pub fn is_gamepad_button_pressed(&self, gamepad_id: u32, button: u32) -> bool {
        self.input_handler
            .is_gamepad_button_pressed(gamepad_id, button)
    }

    // TODO: Is this going to add a lot of overhead?
    fn maintain_aspect_ratio(&mut self) {
        let (current_width, current_height) = self.window_handle.get_size();
        let aspect_ratio = self.width as f32 / self.height as f32;
        let new_width = (current_height as f32 * aspect_ratio).round() as i32;
        let new_height = (current_width as f32 / aspect_ratio).round() as i32;

        if new_width != current_width {
            self.window_handle.set_size(new_width, current_height);
        } else if new_height != current_height {
            self.window_handle.set_size(current_width, new_height);
        }
    }

    fn process_events(&mut self) {
        let events: Vec<_> = glfw::flush_messages(&self.events).collect();
        for (_, event) in events {
            self.input_handler.handle_event(&event);

            if let WindowEvent::Size(_width, _height) = event {
                self.maintain_aspect_ratio(); // Maintain aspect ratio on resize
            }
        }
    }

    pub fn terminate(&mut self) {
        self.window_handle.set_should_close(true);
    }
}
