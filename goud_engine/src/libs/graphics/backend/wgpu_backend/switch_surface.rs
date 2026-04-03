//! Nintendo Switch window handle wrapper for wgpu surface creation.
//!
//! Provides [`SwitchWindowHandle`] which implements the `raw-window-handle`
//! traits required by wgpu. This is a pure PoC stub -- no real Switch handles
//! are available without the NintendoSDK, so all trait methods return
//! `HandleError::Unavailable`.

/// Wrapper providing `raw-window-handle` traits for a Nintendo Switch window.
///
/// On the actual Switch hardware, this would wrap an `nn::vi::Layer` native
/// window handle. For the PoC, all methods return `HandleError::Unavailable`.
pub struct SwitchWindowHandle {
    #[allow(dead_code)] // Will hold nn::vi native window once SDK is available.
    native_window: *mut std::ffi::c_void,
}

impl SwitchWindowHandle {
    /// Creates a dummy handle for type-checking on all platforms.
    ///
    /// The returned handle must never be used to create an actual surface.
    pub fn dummy() -> Self {
        Self {
            native_window: std::ptr::null_mut(),
        }
    }
}

// SAFETY: The native window handle would be owned by the nn::vi runtime
// which lives for the duration of the game. The handle is only used to
// create a wgpu surface during initialization.
unsafe impl Send for SwitchWindowHandle {}
// SAFETY: Same reasoning as Send -- the native window handle is an opaque
// pointer and wgpu only reads it during surface creation.
unsafe impl Sync for SwitchWindowHandle {}

impl wgpu::rwh::HasWindowHandle for SwitchWindowHandle {
    fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
        // PoC stub: no real Switch handle format available without NintendoSDK.
        Err(wgpu::rwh::HandleError::Unavailable)
    }
}

impl wgpu::rwh::HasDisplayHandle for SwitchWindowHandle {
    fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
        // PoC stub: no real Switch display handle available without NintendoSDK.
        Err(wgpu::rwh::HandleError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_handle_returns_unavailable() {
        use wgpu::rwh::HasWindowHandle;
        let handle = SwitchWindowHandle::dummy();
        assert!(handle.window_handle().is_err());
    }
}
