//! Deferred framebuffer capture coordination for the debugger.

#[cfg(feature = "native")]
use std::sync::{Arc, Condvar, Mutex};

#[cfg(feature = "native")]
use crate::core::debugger;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::RenderBackend;

use super::GoudGame;

/// Shared state for deferred framebuffer capture.
///
/// The IPC handler thread sets `requested = true` and waits on the condvar.
/// The main thread (in `swap_buffers`) checks `requested`, does the GL readback,
/// stores the result, and notifies the condvar.
#[cfg(feature = "native")]
pub(crate) struct DeferredCaptureState {
    pub(crate) requested: bool,
    pub(crate) result: Option<Result<debugger::RawFramebufferReadbackV1, String>>,
}

#[cfg(feature = "native")]
pub(crate) type DeferredCapture = Arc<(Mutex<DeferredCaptureState>, Condvar)>;

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
