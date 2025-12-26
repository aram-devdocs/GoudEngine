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

        // Frame rate limiting with proper sleep instead of busy-wait
        let frame_duration = Duration::from_millis(1000 / self.target_fps as u64);
        let elapsed = current_time.elapsed();
        if let Some(remaining) = frame_duration.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Create a test WindowBuilder
    fn create_test_window_builder() -> WindowBuilder {
        let title = CString::new("Test Window").unwrap();
        WindowBuilder {
            width: 800,
            height: 600,
            title: title.into_raw(),
            target_fps: 60,
        }
    }

    #[test]
    #[ignore] // This test requires a window context, which may not be available in CI
    fn test_window_creation() {
        let builder = create_test_window_builder();
        let window = Window::new(builder);

        assert_eq!(window.width, 800);
        assert_eq!(window.height, 600);
        assert_eq!(window.target_fps, 60);
        assert_eq!(window.frame_count, 0);
        assert_eq!(window.fps, 0);
        assert_eq!(window.delta_time, 0.0);
    }

    #[test]
    #[ignore] // This test requires a window context
    fn test_window_should_close() {
        let builder = create_test_window_builder();
        let mut window = Window::new(builder);

        assert_eq!(window.should_close(), false);
        window.terminate();
        assert_eq!(window.should_close(), true);
    }

    #[test]
    #[ignore] // This test requires a window context
    fn test_window_input_delegation() {
        let builder = create_test_window_builder();
        let window = Window::new(builder);

        // Test input handler delegation
        assert_eq!(window.is_key_pressed(Key::A), false);
        assert_eq!(
            window.is_mouse_button_pressed(glfw::MouseButton::Button1),
            false
        );

        let mouse_pos = window.get_mouse_position();
        assert_eq!(mouse_pos.x, 0.0);
        assert_eq!(mouse_pos.y, 0.0);
    }

    #[test]
    #[ignore] // This test requires a window context
    fn test_window_gamepad_delegation() {
        let builder = create_test_window_builder();
        let mut window = Window::new(builder);

        assert_eq!(window.is_gamepad_button_pressed(0, 1), false);

        window.handle_gamepad_button(0, 1, true);
        assert_eq!(window.is_gamepad_button_pressed(0, 1), true);

        window.handle_gamepad_button(0, 1, false);
        assert_eq!(window.is_gamepad_button_pressed(0, 1), false);
    }

    #[test]
    #[ignore] // This test requires a window context
    fn test_aspect_ratio_maintenance() {
        let builder = create_test_window_builder();
        let mut window = Window::new(builder);

        // Initial aspect ratio is 800 / 600 = 4 / 3
        let initial_aspect_ratio = window.width as f32 / window.height as f32;

        // Call maintain_aspect_ratio (note: this is difficult to test properly
        // without actually resizing the window, which we can't reliably do in a test)
        window.maintain_aspect_ratio();

        // After maintenance, the aspect ratio should still be the same
        let (width, height) = window.window_handle.get_size();
        let new_aspect_ratio = width as f32 / height as f32;

        // Allow for floating point imprecision
        assert!((initial_aspect_ratio - new_aspect_ratio).abs() < 0.001);
    }
}

// Integration-style test module
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::ffi::CString;
    use std::time::Duration;

    #[test]
    #[ignore] // Requires a window context
    fn test_window_lifecycle() {
        // Create a test window
        let title = CString::new("Integration Test Window").unwrap();
        let builder = WindowBuilder {
            width: 800,
            height: 600,
            title: title.as_ptr(),
            target_fps: 60,
        };

        let mut window = Window::new(builder);
        window.init_gl();

        // Verify initial state
        assert_eq!(window.width, 800);
        assert_eq!(window.height, 600);
        assert!(!window.should_close());

        // Test input handling
        assert!(!window.is_key_pressed(Key::A));
        assert!(!window.is_mouse_button_pressed(glfw::MouseButton::Button1));

        // Run a few update cycles
        for _ in 0..3 {
            window.update();
            std::thread::sleep(Duration::from_millis(16));
        }

        // Verify delta time is updated
        assert!(window.delta_time > 0.0);

        // Clean up
        window.terminate();
        assert!(window.should_close());
    }

    #[test]
    #[ignore] // Requires a window context
    fn test_input_handling() {
        // Create a test window
        let title = CString::new("Input Test Window").unwrap();
        let builder = WindowBuilder {
            width: 640,
            height: 480,
            title: title.as_ptr(),
            target_fps: 60,
        };

        let mut window = Window::new(builder);

        // Test gamepad input delegation
        window.handle_gamepad_button(0, 1, true);
        assert!(window.is_gamepad_button_pressed(0, 1));

        window.handle_gamepad_button(0, 1, false);
        assert!(!window.is_gamepad_button_pressed(0, 1));

        // Clean up
        window.terminate();
    }

    #[test]
    #[ignore] // Requires a window context
    fn test_fps_management() {
        // Create a window with specific target FPS
        let title = CString::new("FPS Test Window").unwrap();
        let builder = WindowBuilder {
            width: 640,
            height: 480,
            title: title.as_ptr(),
            target_fps: 30, // Lower FPS for testing
        };

        let mut window = Window::new(builder);
        window.init_gl();

        // Run several update cycles
        let start = std::time::Instant::now();

        for _ in 0..10 {
            window.update();
        }

        let elapsed = start.elapsed();

        // With target_fps of 30, 10 frames should take at least 10/30 seconds
        // We're checking that FPS limiting is working
        let min_expected_duration = Duration::from_secs_f32(10.0 / 30.0);

        // Allow some margin for measurement error
        let min_threshold = min_expected_duration.as_secs_f32() * 0.8;

        assert!(
            elapsed.as_secs_f32() >= min_threshold,
            "FPS limiting not working, elapsed time was {:?}",
            elapsed
        );

        window.terminate();
    }
}
