use crate::core::error::{GoudError, GoudResult};

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::Arc;

pub(crate) struct LuaRuntime {
    lua: mlua::Lua,
    #[cfg(test)]
    drop_probe: Option<Arc<AtomicUsize>>,
}

impl LuaRuntime {
    pub(crate) fn new(ctx_id: u64) -> GoudResult<Self> {
        let lua = mlua::Lua::new();

        // Bootstrap: verify the VM works.
        lua.load("return true").eval::<bool>().map_err(|error| {
            GoudError::InitializationFailed(format!("embedded Lua VM bootstrap failed: {error}"))
        })?;

        // Register generated type/enum/tool bindings.
        super::lua_bindings::register_lua_bindings(&lua, ctx_id).map_err(|error| {
            GoudError::InitializationFailed(format!("Lua binding registration failed: {error}"))
        })?;

        // Register hand-written bridge functions for string-param FFI methods.
        #[cfg(feature = "native")]
        super::lua_bridge::register_lua_bridge(&lua, ctx_id).map_err(|error| {
            GoudError::InitializationFailed(format!("Lua bridge registration failed: {error}"))
        })?;

        Ok(Self {
            lua,
            #[cfg(test)]
            drop_probe: None,
        })
    }

    /// Executes a Lua script in this runtime.
    pub(crate) fn execute_script(&self, source: &str, name: &str) -> GoudResult<()> {
        self.lua
            .load(source)
            .set_name(name)
            .exec()
            .map_err(|e| GoudError::ScriptError(format!("{e}")))
    }

    /// Calls a Lua global function by name, if it exists.
    ///
    /// If the global is not defined this is a no-op and returns `Ok(())`.
    pub(crate) fn call_global(&self, name: &str) -> GoudResult<()> {
        let globals = self.lua.globals();
        let func: mlua::Value = globals
            .get(name)
            .map_err(|e| GoudError::ScriptError(format!("{e}")))?;
        if let mlua::Value::Function(f) = func {
            f.call::<()>(())
                .map_err(|e| GoudError::ScriptError(format!("{e}")))?;
        }
        Ok(())
    }

    /// Calls `on_update(dt)` if defined in the Lua environment.
    pub(crate) fn call_update(&self, dt: f32) -> GoudResult<()> {
        let globals = self.lua.globals();
        let func: mlua::Value = globals
            .get("on_update")
            .map_err(|e| GoudError::ScriptError(format!("{e}")))?;
        if let mlua::Value::Function(f) = func {
            f.call::<()>(dt as f64)
                .map_err(|e| GoudError::ScriptError(format!("{e}")))?;
        }
        Ok(())
    }

    /// Checks if a global Lua function exists.
    pub(crate) fn has_global(&self, name: &str) -> bool {
        self.lua
            .globals()
            .get::<mlua::Value>(name)
            .map(|v| matches!(v, mlua::Value::Function(_)))
            .unwrap_or(false)
    }

    /// Re-executes a Lua script after a hot-reload change.
    ///
    /// This is functionally identical to [`execute_script`](Self::execute_script)
    /// but emits a log message so reload events are observable.
    pub(crate) fn reload_script(&self, source: &str, name: &str) -> GoudResult<()> {
        log::info!("Hot-reloading Lua script: {}", name);
        self.execute_script(source, name)
    }

    #[cfg(test)]
    pub(crate) fn is_ready(&self) -> bool {
        self.lua.load("return true").eval::<bool>().is_ok()
    }

    #[cfg(test)]
    pub(crate) fn set_drop_probe(&mut self, probe: Arc<AtomicUsize>) {
        self.drop_probe = Some(probe);
    }
}

impl Drop for LuaRuntime {
    fn drop(&mut self) {
        #[cfg(test)]
        if let Some(probe) = &self.drop_probe {
            probe.fetch_add(1, Ordering::SeqCst);
        }
    }
}
