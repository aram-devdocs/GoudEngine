use crate::core::error::{GoudError, GoudResult};

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::Arc;

pub(super) struct LuaRuntime {
    lua: mlua::Lua,
    #[cfg(test)]
    drop_probe: Option<Arc<AtomicUsize>>,
}

impl LuaRuntime {
    pub(super) fn new(ctx_id: u64) -> GoudResult<Self> {
        let lua = mlua::Lua::new();
        super::lua_bindings::register_lua_bindings(&lua, ctx_id);
        let runtime = Self {
            lua,
            #[cfg(test)]
            drop_probe: None,
        };
        runtime
            .lua
            .load("return true")
            .eval::<bool>()
            .map_err(|error| {
                GoudError::InitializationFailed(format!(
                    "embedded Lua VM bootstrap failed: {error}"
                ))
            })?;
        Ok(runtime)
    }

    /// Executes a Lua script in this runtime.
    pub(super) fn execute_script(&self, source: &str, name: &str) -> GoudResult<()> {
        self.lua
            .load(source)
            .set_name(name)
            .exec()
            .map_err(|e| GoudError::ScriptError(format!("{e}")))
    }

    #[cfg(test)]
    pub(super) fn is_ready(&self) -> bool {
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
