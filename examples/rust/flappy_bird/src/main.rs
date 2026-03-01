//! Flappy Bird -- GoudEngine Rust Example
//!
//! A complete Flappy Bird clone demonstrating the GoudEngine from Rust.
//! Game constants and behavior match the C#, Python, and TypeScript
//! versions exactly for SDK parity validation.
//!
//! # Running
//!
//! ```sh
//! cargo run -p flappy-bird
//! ```
//!
//! # Controls
//!
//! - Space / Left Click -- Flap (jump)
//! - R                  -- Restart
//! - Escape             -- Quit

mod bird;
mod constants;
mod engine;
mod game_manager;
mod pipe;
mod score;

use constants::*;
use engine::Engine;
use game_manager::GameManager;

fn main() {
    // Window height includes the base strip below the playfield.
    let window_height = SCREEN_HEIGHT as u32 + BASE_HEIGHT as u32;
    let engine = Engine::new(SCREEN_WIDTH as u32, window_height, "Flappy Bird");

    // Assets live in the shared C# example directory.
    let asset_base = "examples/csharp/flappy_goud/assets";

    let mut manager = GameManager::new(&engine, asset_base);
    manager.start();

    engine.enable_blending();

    // -- Game loop ------------------------------------------------------------
    while !engine.should_close() {
        let dt = engine.poll_events();
        engine.begin_frame();
        engine.clear(0.4, 0.7, 0.9, 1.0); // sky blue

        if !manager.update(&engine, dt) {
            break; // Escape was pressed
        }
        manager.draw(&engine);

        engine.end_frame();
        engine.swap_buffers();
    }

    // Engine is dropped here, which destroys the window context.
}
