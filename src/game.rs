use crate::engine::Engine;

pub struct Game {
    engine: Engine,
}

impl Game {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
        }
    }

    pub fn start(&mut self) {
        // Initialize game-specific elements here
        println!("Starting game...");
        self.engine.run();
    }
}
