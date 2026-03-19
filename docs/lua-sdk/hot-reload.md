# Lua Hot Reload

Hot reload lets you edit `.lua` files and see changes reflected immediately in a running GoudEngine instance, without restarting.

## How It Works

1. The `LuaScriptWatcher` uses the OS file-system notification API (via the `notify` crate) to recursively monitor a directory for changes.
2. Each frame, `process_lua_hot_reload()` polls the watcher for modified `.lua` files.
3. Changed files are re-read from disk and re-executed in the existing Lua VM using `reload_script()`.
4. A 200ms debounce filter prevents duplicate reloads from rapid save events.

Because the script is re-executed in the same VM, any global state (variables, tables) is updated in place. Functions like `on_init`, `on_update`, and `on_draw` are replaced with their new definitions.

## Enabling Hot Reload

Hot reload is available when the engine is built with both `lua` and `native` features.

### GameConfig

The `lua_hot_reload` flag on `GameConfig` controls whether hot reload is active. It defaults to `true` in debug builds and `false` in release builds.

```rust
let config = GameConfig::default()
    .with_lua_hot_reload(true);
```

### Starting the Watcher

In your Rust host code, call `watch_lua_dir()` with the directory containing your Lua scripts:

```rust
let mut game = GoudGame::with_platform(config)?;
game.watch_lua_dir("scripts/")?;
```

### Polling Each Frame

Call `process_lua_hot_reload()` once per frame in your game loop:

```rust
loop {
    game.process_lua_hot_reload();
    // ... rest of frame
}
```

## What Gets Watched

- Only files with the `.lua` extension trigger reloads.
- The watcher is recursive: subdirectories are included.
- Hidden files (starting with `.`), temp files (`~`, `.tmp`, `.swp`) are ignored.

## Debounce Behavior

The watcher applies a 200ms debounce window per file. If the same file is modified multiple times within 200ms, only the last change triggers a reload. This prevents double-reloads from editors that perform write-then-rename save operations.

Stale debounce entries are cleaned up automatically when the internal map exceeds 500 entries.

## Development Workflow

A typical hot-reload workflow:

1. Start the engine with hot reload enabled and the watcher pointed at your scripts directory.
2. Run your initial script to set up game state.
3. Edit a `.lua` file in your editor and save.
4. The engine detects the change within ~200ms, re-reads the file, and re-executes it.
5. Updated function definitions take effect on the next frame.

### Tips

- **Keep state in globals.** Since reload re-executes the entire file, local variables are reset. Store persistent state in global tables so it survives reloads.
- **Use `on_init` carefully.** If `on_init` is re-defined on reload, it will not be called again automatically. Consider using a guard variable:

```lua
if not _initialized then
    _initialized = true
    -- one-time setup here
end
```

- **Error handling.** If a reloaded script has a syntax or runtime error, the error is logged but the engine continues running with the previous definitions intact.

## Limitations

- Hot reload only affects Lua scripts. Rust code changes still require a full rebuild.
- The Lua VM is not reset on reload. Stale global variables from previous script versions may persist.
- Asset handles (textures, fonts) loaded in a previous script execution remain valid across reloads.
