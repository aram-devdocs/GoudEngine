// game.rs
mod platform;
pub use platform::graphics::gl_wrapper::{clear, Rectangle, Renderer, Renderer2D, Sprite, Texture};
use platform::logger;

pub use platform::graphics::cgmath;
// pub use platform::graphics::gl_wrapper::Sprite;
pub use platform::graphics::window::Window;
pub use platform::graphics::window::WindowBuilder;

// pub use platform::graphics::cgmath;

/// Single entry point for the game
#[repr(C)]
pub struct GameSdk {
    pub window: Window,
    pub renderer_2d: Option<Renderer2D>,
    // pub renderer_3d: Option<Renderer3D>, // If you implement Renderer3D
    pub elapsed_time: f32,
}

impl GameSdk {
    /// Creates a new game instance.
    pub fn new(data: WindowBuilder) -> GameSdk {
        logger::init();
        let window = platform::graphics::window::Window::new(data);

        GameSdk {
            window,
            renderer_2d: None,
            // renderer_3d: None,
            elapsed_time: 0.0,
        }
    }

    /// Initializes the game.
    pub extern "C" fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(&mut GameSdk),
    {
        self.window.init_gl();

        // Create renderer
        self.renderer_2d = Some(Renderer2D::new().expect("Failed to create Renderer2D"));
        // If you implement Renderer3D, initialize it here

        init_callback(self);
    }

    /// Runs the game loop.
    pub extern "C" fn run<F>(&mut self, update_callback: F)
    where
        F: Fn(&mut GameSdk),
    {
        while !self.window.should_close() {
            self.update(&update_callback);
        }

        println!("Window closed!");
    }

    /// Updates the game.
    pub extern "C" fn update<F>(&mut self, update_callback: &F)
    where
        F: Fn(&mut GameSdk),
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
