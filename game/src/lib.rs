// lib.rs

use platform::graphics;
use platform::graphics::gl_wrapper::{clear, Renderer};
use platform::graphics::window::KeyInput as _KeyInput;
use platform::graphics::window::WindowBuilder as _WindowBuilder;
use platform::logger;

// Expose the types from the platform module
pub type WindowBuilder = _WindowBuilder;
pub type KeyInput = _KeyInput;

/// Single entry point for the game
pub struct Game {
    pub window: graphics::window::Window,
    pub renderer: Option<Renderer>,
    pub elapsed_time: f32,
}

impl Game {
    pub fn new(data: WindowBuilder) -> Game {
        logger::init();
        let window = graphics::window::Window::new(data);

        Game {
            window,
            renderer: None,
            elapsed_time: 0.0,
        }
    }

    pub fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(&mut Game),
    {
        self.window.init_gl();
        // Create renderer
        self.renderer = Some(Renderer::new());
        init_callback(self);
    }

    pub fn run<F>(&mut self, update_callback: F)
    where
        F: Fn(&mut Game),
    {
        while !self.window.should_close() {
            self.update(&update_callback);
        }

        println!("Window closed!");
    }

    fn update<F>(&mut self, update_callback: &F)
    where
        F: Fn(&mut Game),
    {
        // Update elapsed time
        self.elapsed_time += 0.01;

        // Clear the screen
        clear();

        // Render all sprites via renderer
        if let Some(renderer) = &mut self.renderer {
            renderer.render();
        }

        // Execute custom update logic
        update_callback(self);

        // Update window
        self.window.update();
    }
}
