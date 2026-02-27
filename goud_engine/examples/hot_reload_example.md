# Hot Reloading Example

This document demonstrates how to use the hot reloading system in GoudEngine.

## Overview

Hot reloading allows you to modify asset files (textures, shaders, audio, etc.) during development and see the changes immediately without restarting your game.

## Basic Usage

```rust
use goud_engine::assets::{Asset, AssetServer, HotReloadWatcher};

// Define your asset type
#[derive(Debug, Clone)]
struct GameTexture {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Asset for GameTexture {
    fn asset_type_name() -> &'static str {
        "GameTexture"
    }
}

fn main() {
    // Create asset server
    let mut server = AssetServer::new();

    // Create hot reload watcher
    let mut watcher = server.create_hot_reload_watcher().unwrap();

    // Start watching the asset directory
    watcher.watch("assets").unwrap();

    // Load an asset
    let texture_handle = server.load::<GameTexture>("textures/player.png");

    // Game loop
    loop {
        // Process hot reload events (reloads changed assets)
        let reloaded_count = watcher.process_events(&mut server);
        if reloaded_count > 0 {
            println!("Reloaded {} assets", reloaded_count);
        }

        // Use your assets
        if let Some(texture) = server.get(&texture_handle) {
            // Render texture...
        }

        // ... rest of game loop
    }
}
```

## Advanced Configuration

You can customize the hot reloading behavior:

```rust
use goud_engine::assets::{AssetServer, HotReloadConfig};
use std::time::Duration;

let mut server = AssetServer::new();

// Configure hot reloading
let config = HotReloadConfig::new()
    .with_enabled(true)                           // Enable hot reloading
    .with_debounce(Duration::from_millis(200))    // 200ms debounce delay
    .watch_extension("png")                       // Only watch PNG files
    .watch_extension("jpg")                       // And JPG files
    .watch_extension("json")                      // And JSON files
    .with_ignore_hidden(true)                     // Ignore .hidden files
    .with_ignore_temp(true);                      // Ignore temp files (~, .tmp, etc.)

let mut watcher = server.create_hot_reload_watcher_with_config(config).unwrap();
watcher.watch("assets").unwrap();
```

## Production vs Development

Hot reloading is automatically enabled in debug builds and disabled in release builds:

```rust
use goud_engine::assets::HotReloadConfig;

// Default behavior
let config = HotReloadConfig::new();
// config.enabled == true in debug builds
// config.enabled == false in release builds

// Override for specific builds
let config = HotReloadConfig::new().with_enabled(false); // Always disabled
```

## File System Events

The hot reload system detects these types of changes:

- **Modified**: File content changed (most common)
- **Created**: New file added
- **Deleted**: File removed
- **Renamed**: File moved or renamed

## Debouncing

File systems often emit multiple events for a single change (e.g., editors saving files in stages). The debounce delay groups rapid changes together to avoid redundant reloads:

```rust
use std::time::Duration;

// Short debounce (more responsive, may trigger multiple reloads)
let config = HotReloadConfig::new()
    .with_debounce(Duration::from_millis(50));

// Long debounce (fewer reloads, less responsive)
let config = HotReloadConfig::new()
    .with_debounce(Duration::from_millis(500));

// Default is 100ms (good balance)
```

## Extension Filtering

Only watch specific file types to reduce unnecessary events:

```rust
let config = HotReloadConfig::new()
    .watch_extension("png")
    .watch_extension("jpg")
    .watch_extension("shader")
    .watch_extension("json");
// Now only these file types trigger reloads
```

## Ignoring Files

Prevent certain files from triggering reloads:

```rust
let config = HotReloadConfig::new()
    .with_ignore_hidden(true)    // Ignore .git, .DS_Store, etc.
    .with_ignore_temp(true);     // Ignore file~, *.tmp, *.swp, *.bak

// These patterns are checked:
// - Hidden: file name starts with '.'
// - Temp: file name ends with '~', '.tmp', '.swp', or '.bak'
```

## Integration with ECS

Use hot reloading with the ECS resource system:

```rust
use goud_engine::assets::{AssetServer, HotReloadWatcher};
use goud_engine::ecs::World;

fn hot_reload_system(
    mut server: ResMut<AssetServer>,
    mut watcher: ResMut<HotReloadWatcher>,
) {
    let count = watcher.process_events(&mut server);
    if count > 0 {
        log::info!("Hot reloaded {} assets", count);
    }
}

// In your game setup:
fn main() {
    let mut world = World::new();

    let server = AssetServer::new();
    let watcher = server.create_hot_reload_watcher().unwrap();

    world.insert_resource(server);
    world.insert_resource(watcher);

    // Add hot reload system to your schedule
    // schedule.add_system(hot_reload_system);
}
```

## Performance Considerations

- Hot reloading has minimal overhead when no files change
- File watching uses platform-specific APIs (inotify on Linux, FSEvents on macOS, etc.)
- The watcher runs on a background thread and uses channels for event passing
- Debouncing prevents performance spikes from rapid file changes
- Disable hot reloading in production builds to eliminate all overhead

## Troubleshooting

### Reloads not triggering

1. Check that `config.enabled` is true
2. Verify the watcher is watching the correct directory
3. Check extension filters (empty = watch all)
4. Ensure files aren't being filtered by hidden/temp settings

### Multiple reloads for single change

- Increase the debounce duration
- Some editors save files multiple times (auto-save, final save, etc.)

### Performance issues

- Use extension filters to reduce watched files
- Increase debounce duration
- Watch specific subdirectories instead of entire asset tree
- Disable recursive watching if not needed

## Example: Complete Game Loop

```rust
use goud_engine::assets::{Asset, AssetServer, HotReloadConfig, HotReloadWatcher};
use std::time::Duration;

fn main() {
    // Setup
    let mut server = AssetServer::new();
    let config = HotReloadConfig::new()
        .with_debounce(Duration::from_millis(100))
        .watch_extension("png");

    let mut watcher = server.create_hot_reload_watcher_with_config(config).unwrap();
    watcher.watch("assets").unwrap();

    // Load initial assets
    let player_texture = server.load::<Texture>("sprites/player.png");
    let enemy_texture = server.load::<Texture>("sprites/enemy.png");

    // Game loop
    loop {
        // Hot reload (at start of frame)
        let reloaded = watcher.process_events(&mut server);
        if reloaded > 0 {
            println!("[Hot Reload] Reloaded {} assets", reloaded);
        }

        // Update game state
        update_game_logic();

        // Render (uses latest asset versions)
        if let Some(texture) = server.get(&player_texture) {
            render_sprite(texture);
        }
        if let Some(texture) = server.get(&enemy_texture) {
            render_sprite(texture);
        }

        // ... rest of rendering
    }
}
```

## Future Enhancements

The current implementation provides the foundation for hot reloading. Future improvements may include:

- Automatic asset reload (currently detects changes but doesn't trigger reload)
- Hot reload events for game code to respond to changes
- Selective reload (reload only specific asset types)
- Asset dependency tracking (reload dependencies when parent changes)
- Hot reload statistics and debugging UI
