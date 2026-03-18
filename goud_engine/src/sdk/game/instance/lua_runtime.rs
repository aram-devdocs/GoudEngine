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
    pub(super) fn new() -> Self {
        let runtime = Self {
            lua: mlua::Lua::new(),
            #[cfg(test)]
            drop_probe: None,
        };
        let _ = runtime.lua.globals();
        runtime
    }

    pub(super) fn is_ready(&self) -> bool {
        let _ = self.lua.globals();
        true
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
