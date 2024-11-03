use platform::{create_event_loop, App, EventLoop_};


pub struct Game {
    pub app: App,
    pub create_event_loop: fn() -> EventLoop_,
}

impl Game {
    pub fn new() -> Self {
        Self {
            app: App::default(),
            create_event_loop,
        }
    }
}
