//! # Lua Game Runner
//!
//! Public API for running Lua-scripted games with a native windowed context.
//!
//! This module creates a real FFI windowed context so that Lua scripts can call
//! the auto-generated tool and hand-written bridge functions (which require a
//! valid `GoudContextId` to look up the window state).
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::sdk::lua_runner::LuaGameRunner;
//!
//! let mut runner = LuaGameRunner::create("Flappy Bird", 288, 624)?;
//! runner.load_script("main.lua")?;
//! runner.run()?;
//! ```

use crate::core::context_id::GoudContextId;
use crate::core::error::{GoudError, GoudResult};
use crate::ffi::renderer::goud_renderer_enable_blending;
use crate::ffi::renderer::{goud_renderer_begin, goud_renderer_end};
use crate::ffi::window::{
    goud_window_clear, goud_window_destroy, goud_window_poll_events, goud_window_should_close,
    goud_window_swap_buffers,
};
use crate::sdk::game::instance::lua_runtime::LuaRuntime;

/// A runner for Lua-scripted games that creates a native windowed context.
pub struct LuaGameRunner {
    context_id: GoudContextId,
    lua: LuaRuntime,
    /// Directory watched for hot-reload, if active.
    #[cfg(feature = "native")]
    watcher: Option<super::game::instance::lua_hot_reload::LuaScriptWatcher>,
}

impl LuaGameRunner {
    /// Creates a new Lua game runner with a native window.
    ///
    /// The window is created through the FFI layer so that all Lua bridge
    /// functions receive a valid context ID.
    pub fn create(title: &str, width: u32, height: u32) -> GoudResult<Self> {
        crate::core::error::init_logger();

        let c_title = std::ffi::CString::new(title)
            .map_err(|e| GoudError::InitializationFailed(format!("invalid title: {e}")))?;
        // SAFETY: c_title is a valid null-terminated C string.
        let context_id =
            unsafe { crate::ffi::window::goud_window_create(width, height, c_title.as_ptr()) };
        if context_id.is_invalid() {
            return Err(GoudError::InitializationFailed(
                "goud_window_create returned invalid context".into(),
            ));
        }

        let lua = LuaRuntime::new(context_id.as_raw())?;

        // Enable alpha blending by default for sprite rendering.
        goud_renderer_enable_blending(context_id);

        Ok(Self {
            context_id,
            lua,
            #[cfg(feature = "native")]
            watcher: None,
        })
    }

    /// Loads and executes a Lua script file.
    pub fn load_script(&self, path: &str) -> GoudResult<()> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| GoudError::ScriptError(format!("failed to read {path}: {e}")))?;
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        self.lua.execute_script(&source, name)
    }

    /// Executes an inline Lua source string.
    pub fn execute(&self, source: &str, name: &str) -> GoudResult<()> {
        self.lua.execute_script(source, name)
    }

    /// Starts watching a directory for hot-reload of `.lua` files.
    #[cfg(feature = "native")]
    pub fn watch_dir(&mut self, path: impl AsRef<std::path::Path>) -> GoudResult<()> {
        let w = super::game::instance::lua_hot_reload::LuaScriptWatcher::new(path.as_ref())?;
        self.watcher = Some(w);
        Ok(())
    }

    /// Runs the game loop.
    ///
    /// The loop calls the following Lua globals if they exist:
    /// - `on_init()` -- called once before the first frame
    /// - `on_update(dt)` -- called each frame with the delta time
    /// - `on_draw()` -- called each frame after update
    pub fn run(&mut self) -> GoudResult<()> {
        let ctx = self.context_id;

        // Call on_init if defined.
        self.lua.call_global("on_init")?;

        while !goud_window_should_close(ctx) {
            // Hot-reload changed scripts.
            #[cfg(feature = "native")]
            self.process_hot_reload();

            let dt = goud_window_poll_events(ctx);

            goud_renderer_begin(ctx);
            goud_window_clear(ctx, 0.4, 0.7, 0.9, 1.0);

            self.lua.call_update(dt)?;
            self.lua.call_global("on_draw")?;

            goud_renderer_end(ctx);
            goud_window_swap_buffers(ctx);
        }

        Ok(())
    }

    /// Processes pending hot-reload events.
    #[cfg(feature = "native")]
    fn process_hot_reload(&mut self) {
        let changed = match self.watcher.as_mut() {
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
            if let Err(e) = self.lua.reload_script(&source, name) {
                log::error!("Lua hot-reload error for {:?}: {}", path, e);
            }
        }
    }
}

impl Drop for LuaGameRunner {
    fn drop(&mut self) {
        goud_window_destroy(self.context_id);
    }
}
