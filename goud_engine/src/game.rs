// game.rs
mod platform;
pub use platform::graphics::gl_wrapper::{clear, Rectangle, Renderer, Renderer2D, Sprite, Texture};
use platform::logger;

pub use platform::graphics::cgmath;
pub use platform::graphics::window::Window;
pub use platform::graphics::window::WindowBuilder;

/// Single entry point for the game
#[repr(C)]
pub struct GameSdk {
    pub window: Window,
    pub renderer_2d: Option<Renderer2D>,
    pub elapsed_time: f32,
}

impl GameSdk {
    pub fn new(data: WindowBuilder) -> GameSdk {
        logger::init();
        let window = platform::graphics::window::Window::new(data);

        GameSdk {
            window,
            renderer_2d: None,
            elapsed_time: 0.0,
        }
    }

    pub extern "C" fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(&mut GameSdk),
    {
        self.window.init_gl();
        self.renderer_2d = Some(Renderer2D::new().expect("Failed to create Renderer2D"));
        init_callback(self);
    }

    pub extern "C" fn start<F>(&mut self, start_callback: F)
    where
        F: FnOnce(&mut GameSdk),
    {
        start_callback(self);
    }

    pub extern "C" fn update<F>(&mut self, update_callback: &F)
    where
        F: Fn(&mut GameSdk),
    {
        self.elapsed_time += 0.01;
        clear();

        if let Some(renderer) = &mut self.renderer_2d {
            renderer.render();
        }

        update_callback(self);
        self.window.update();
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }
}
