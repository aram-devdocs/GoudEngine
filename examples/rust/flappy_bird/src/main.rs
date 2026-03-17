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
mod game_manager;
mod pipe;
mod score;

use constants::*;
use game_manager::GameManager;
use goudengine::{GameConfig, GoudGame, RenderBackendKind, WindowBackendKind};

fn configure_native_backends(config: GameConfig) -> GameConfig {
    match std::env::var("GOUD_NATIVE_BACKEND").ok().as_deref() {
        Some("legacy") => config
            .with_window_backend(WindowBackendKind::GlfwLegacy)
            .with_render_backend(RenderBackendKind::OpenGlLegacy),
        _ => config,
    }
}

fn main() {
    // This binary must be run from the repository root so that relative asset
    // paths (e.g. "examples/csharp/flappy_goud/assets/...") resolve correctly:
    //   cargo run -p flappy-bird
    if !std::path::Path::new("examples/rust/flappy_bird/assets").exists()
        && !std::path::Path::new("examples/csharp/flappy_goud/assets").exists()
    {
        eprintln!("Error: assets directory not found. Run from the repository root:");
        eprintln!("  cargo run -p flappy-bird");
        std::process::exit(1);
    }

    // Window height includes the base strip below the playfield.
    let window_height = SCREEN_HEIGHT as u32 + BASE_HEIGHT as u32;
    let config = configure_native_backends(GameConfig::new(
        "Flappy Bird",
        SCREEN_WIDTH as u32,
        window_height,
    ));
    let mut game = GoudGame::with_platform(config).expect("Failed to create game");

    // Assets live in the shared C# example directory.
    let asset_base = "examples/csharp/flappy_goud/assets";

    let mut manager = GameManager::new(&mut game, asset_base);
    manager.start();

    game.enable_blending();

    // -- Game loop ------------------------------------------------------------
    while !game.should_close() {
        let dt = game.poll_events().unwrap_or(0.016);
        game.begin_render();
        game.clear(0.4, 0.7, 0.9, 1.0); // sky blue

        if !manager.update(&game, dt) {
            break; // Escape was pressed
        }
        manager.draw(&mut game);

        game.end_render();
        game.swap_buffers().expect("swap_buffers failed");
    }

    // GoudGame is dropped here, which destroys the window context.
}
