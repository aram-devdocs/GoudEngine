// lib.rs

use platform::graphics::gl_wrapper::{clear, Renderer, Renderer2D};
use platform::logger;

pub use platform::graphics::gl_wrapper::{Rectangle, Sprite, Texture};
pub use platform::graphics::window::{KeyInput, WindowBuilder};

pub use platform::graphics::cgmath;
// pub type WindowBuilder = _WindowBuilder;
// pub type KeyInput = _KeyInput;

/// Single entry point for the game
pub struct Game {
    pub window: platform::graphics::window::Window,
    pub renderer_2d: Option<Renderer2D>,
    // pub renderer_3d: Option<Renderer3D>, // If you implement Renderer3D
    pub elapsed_time: f32,
}

impl Game {
    /// Creates a new game instance.
    pub fn new(data: WindowBuilder) -> Game {
        logger::init();
        let window = platform::graphics::window::Window::new(data);

        Game {
            window,
            renderer_2d: None,
            // renderer_3d: None,
            elapsed_time: 0.0,
        }
    }

    /// Initializes the game.
    pub fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(&mut Game),
    {
        self.window.init_gl();

        // Create renderer
        self.renderer_2d = Some(Renderer2D::new().expect("Failed to create Renderer2D"));
        // If you implement Renderer3D, initialize it here

        init_callback(self);
    }

    /// Runs the game loop.
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
        if let Some(renderer) = &mut self.renderer_2d {
            renderer.render();
        }

        // Execute custom update logic
        update_callback(self);

        // Update window
        self.window.update();
    }
}
