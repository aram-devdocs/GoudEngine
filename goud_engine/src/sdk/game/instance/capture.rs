//! Deferred framebuffer capture coordination for the debugger.

#[cfg(feature = "native")]
use crate::core::debugger;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::RenderBackend;

use super::GoudGame;

impl GoudGame {
    /// Services a pending deferred capture request by performing the GL
    /// readback on the current (main) thread, then notifying the waiting
    /// IPC thread.
    #[cfg(feature = "native")]
    pub(crate) fn service_deferred_capture(&self) {
        let Some(ref deferred) = self.deferred_capture else {
            return;
        };
        let (lock, cvar) = &**deferred;
        let mut guard = match lock.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if !guard.requested {
            return;
        }
        let (w, h) = self.get_framebuffer_size();
        let result = self
            .render_backend
            .clone()
            .ok_or_else(|| "render backend not initialized".to_string())
            .and_then(|mut backend| backend.read_default_framebuffer_rgba8(w, h))
            .map(|rgba8| debugger::RawFramebufferReadbackV1 {
                width: w,
                height: h,
                rgba8,
            })
            .map_err(|e| format!("framebuffer readback failed: {e}"));
        guard.result = Some(result);
        cvar.notify_all();
    }
}
