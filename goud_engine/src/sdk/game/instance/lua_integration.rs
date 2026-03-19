//! Lua integration methods for [`GoudGame`].
//!
//! Extracted from the main `mod.rs` to keep individual files under the
//! 500-line limit.  These methods delegate to the embedded
//! [`LuaRuntime`](super::lua_runtime::LuaRuntime) and the optional
//! hot-reload watcher.

use crate::core::error::GoudResult;

use super::GoudGame;

impl GoudGame {
    /// Executes a Lua script in the embedded runtime.
    ///
    /// # Errors
    ///
    /// Returns a `GoudError` if the script has syntax or runtime errors.
    pub fn execute_lua(&self, source: &str, name: &str) -> GoudResult<()> {
        self.lua_runtime.execute_script(source, name)
    }

    /// Calls a Lua global function by name, if it exists.
    ///
    /// If the global is not defined this is a no-op and returns `Ok(())`.
    pub fn call_lua_global(&self, name: &str) -> GoudResult<()> {
        self.lua_runtime.call_global(name)
    }

    /// Calls `on_update(dt)` if defined in the Lua environment.
    pub fn call_lua_update(&self, dt: f32) -> GoudResult<()> {
        self.lua_runtime.call_update(dt)
    }

    /// Checks if a global Lua function exists.
    pub fn has_lua_global(&self, name: &str) -> bool {
        self.lua_runtime.has_global(name)
    }

    /// Starts watching a directory for `.lua` file changes.
    ///
    /// Changed scripts will be automatically re-executed when
    /// [`process_lua_hot_reload`](Self::process_lua_hot_reload) is called
    /// each frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the file-system watcher cannot be initialised.
    #[cfg(feature = "native")]
    pub fn watch_lua_dir(&mut self, path: impl AsRef<std::path::Path>) -> GoudResult<()> {
        let watcher = super::lua_hot_reload::LuaScriptWatcher::new(path.as_ref())?;
        self.lua_watcher = Some(watcher);
        Ok(())
    }

    /// Polls the Lua hot-reload watcher and re-executes any changed scripts.
    ///
    /// Call this once per frame (e.g., at the start of the update loop).
    /// If no watcher is active this is a no-op.
    #[cfg(feature = "native")]
    pub fn process_lua_hot_reload(&mut self) {
        let changed = match self.lua_watcher.as_mut() {
            Some(w) => w.poll_changes(),
            None => return,
        };

        for path in changed {
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to read changed Lua file {:?}: {}", path, e);
                    continue;
                }
            };
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unknown>");
            if let Err(e) = self.lua_runtime.reload_script(&source, name) {
                log::error!("Lua hot-reload error for {:?}: {}", path, e);
            }
        }
    }
}
