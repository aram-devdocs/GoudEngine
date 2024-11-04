use platform::custom_errors;
use platform::graphics;
use platform::logger;

pub struct Game {
    pub window: graphics::window::Window,
}

impl Game {
    pub fn new(width: u32, height: u32, title: &str) -> Game {
        logger::init();
        Game {
            window: graphics::window::Window::new(width, height, title),
        }
    }

    pub fn run(&mut self) {
        self.window.init_gl();

        while !self.window.should_close() {
            self.window.update();
        }

        println!("Window closed!");
    }
}
