//! SDL2 window handle wrapper for wgpu surface creation.
//!
//! Provides [`SdlWindowHandle`] which implements the `raw-window-handle`
//! traits required by wgpu to create a Vulkan surface from an SDL2 window.
//! Uses SDL2's built-in `raw-window-handle` 0.6 support to extract handles.

use crate::core::error::{GoudError, GoudResult};

/// Wrapper providing `raw-window-handle` traits for an SDL2 window.
///
/// Captures the raw window and display handles from an SDL2 window at
/// construction time and presents them via the `raw-window-handle` traits
/// that wgpu requires for surface creation.
pub struct SdlWindowHandle {
    raw_window: wgpu::rwh::RawWindowHandle,
    raw_display: wgpu::rwh::RawDisplayHandle,
}

impl SdlWindowHandle {
    /// Extracts raw window handles from an SDL2 window.
    ///
    /// Uses SDL2's `raw-window-handle` 0.6 support to obtain platform-native
    /// handles (X11, Wayland, Cocoa, Win32) for wgpu surface creation.
    pub fn from_sdl_window(window: &sdl2::video::Window) -> GoudResult<Self> {
        use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

        let window_handle = window.window_handle().map_err(|e| {
            GoudError::InitializationFailed(format!("Failed to get SDL2 window handle: {e}"))
        })?;
        let display_handle = window.display_handle().map_err(|e| {
            GoudError::InitializationFailed(format!("Failed to get SDL2 display handle: {e}"))
        })?;

        Ok(Self {
            raw_window: window_handle.as_raw(),
            raw_display: display_handle.as_raw(),
        })
    }

    /// Creates a dummy handle for type-checking on non-desktop platforms.
    ///
    /// The returned handle must never be used to create an actual surface.
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    pub fn dummy() -> Self {
        Self {
            raw_window: wgpu::rwh::RawWindowHandle::Web(wgpu::rwh::WebWindowHandle::new(0)),
            raw_display: wgpu::rwh::RawDisplayHandle::Web(wgpu::rwh::WebDisplayHandle::new()),
        }
    }
}

// SAFETY: The raw window and display handles are owned by the SDL2 window
// which is kept alive for the duration of the game in SdlPlatform. The handles
// are only used to create a wgpu surface during initialization, after which
// wgpu owns the surface independently.
unsafe impl Send for SdlWindowHandle {}
// SAFETY: Same reasoning as Send -- the raw handles are opaque values and wgpu
// only reads them during surface creation on the main thread.
unsafe impl Sync for SdlWindowHandle {}

impl wgpu::rwh::HasWindowHandle for SdlWindowHandle {
    fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
        // SAFETY: The raw handle was obtained from a valid SDL2 window and
        // outlives the returned `WindowHandle` via the borrow of `self`.
        Ok(unsafe { wgpu::rwh::WindowHandle::borrow_raw(self.raw_window) })
    }
}

impl wgpu::rwh::HasDisplayHandle for SdlWindowHandle {
    fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
        // SAFETY: The raw handle was obtained from a valid SDL2 display and
        // outlives the returned `DisplayHandle` via the borrow of `self`.
        Ok(unsafe { wgpu::rwh::DisplayHandle::borrow_raw(self.raw_display) })
    }
}

#[cfg(test)]
mod tests {
    /// SDL2 requires a display server; skip in headless CI.
    #[ignore]
    #[test]
    fn sdl_handle_from_window_succeeds() {
        let sdl = sdl2::init().expect("SDL2 init");
        let video = sdl.video().expect("SDL2 video");
        let window = video.window("test", 64, 64).build().expect("window");
        let handle = super::SdlWindowHandle::from_sdl_window(&window);
        assert!(handle.is_ok());
    }
}
