//! Lua Game Runner -- GoudEngine Lua Example
//!
//! A standalone binary that creates a native windowed context and runs
//! Lua game scripts. The Lua scripts define `on_init()`, `on_update(dt)`,
//! and `on_draw()` callbacks that are driven by the game loop here.
//!
//! # Running
//!
//! ```sh
//! cargo run -p lua-runner -- examples/lua/flappy_bird/main.lua
//! ```

use goud_engine::sdk::lua_runner::LuaGameRunner;

fn main() {
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/lua/flappy_bird/main.lua".to_string());

    if !std::path::Path::new(&script_path).exists() {
        eprintln!("Error: script not found: {script_path}");
        eprintln!("Run from the repository root:");
        eprintln!("  cargo run -p lua-runner -- examples/lua/flappy_bird/main.lua");
        std::process::exit(1);
    }

    // Derive the script directory for hot-reload watching.
    let script_dir = std::path::Path::new(&script_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    // Window dimensions: Flappy Bird default (288 x 512, matching cross-SDK parity).
    // These can be overridden by the Lua script calling goud_game.set_window_size().
    let mut runner =
        LuaGameRunner::create("Lua Game", 288, 512).expect("Failed to create Lua runner");

    // Load all .lua files in the script directory (constants first, then main).
    // This ensures constants.lua is available before main.lua references them.
    let mut lua_files: Vec<_> = std::fs::read_dir(script_dir)
        .expect("Failed to read script directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "lua")
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    // Sort so that supporting files (constants.lua, etc.) load before main.lua.
    lua_files.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // main.lua sorts last.
        let a_is_main = a_name == "main.lua";
        let b_is_main = b_name == "main.lua";
        match (a_is_main, b_is_main) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a_name.cmp(b_name),
        }
    });

    for path in &lua_files {
        let path_str = path.to_str().expect("non-UTF-8 path");
        runner
            .load_script(path_str)
            .unwrap_or_else(|e| panic!("Failed to load {path_str}: {e}"));
    }

    // Enable hot-reload for the script directory.
    if let Err(e) = runner.watch_dir(script_dir) {
        eprintln!("Warning: hot-reload watcher failed: {e}");
    }

    // Run the game loop.
    runner.run().expect("Game loop error");
}
