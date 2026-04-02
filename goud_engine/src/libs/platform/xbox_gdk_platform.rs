//! Xbox GDK platform backend (PoC).
//!
//! Provides a [`PlatformBackend`] implementation for the Xbox Game Development
//! Kit. On non-MSVC targets the constructor returns
//! [`GoudError::BackendNotSupported`] so the module can still be type-checked
//! during cross-platform development.

use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{GoudError, GoudResult};
use crate::core::input_manager::InputManager;

use super::{FullscreenMode, PlatformBackend, WindowConfig};
use crate::libs::graphics::backend::wgpu_backend::xbox_surface::XboxWindowHandle;

/// Xbox GDK platform backend.
///
/// Wraps an Xbox GDK `GameWindow` (HWND) and implements the engine's platform
/// abstraction. On non-MSVC hosts the constructor always fails with a
/// descriptive error.
pub struct XboxGdkPlatform {
    should_close: bool,
    width: u32,
    height: u32,
    last_frame: Instant,
    handle: Arc<XboxWindowHandle>,
}

impl XboxGdkPlatform {
    /// Creates the Xbox GDK platform.
    ///
    /// On MSVC targets this would call `CreateWindowExW` or
    /// `XGameWindowCreate` to obtain an HWND. On other targets this returns
    /// `BackendNotSupported`.
    pub fn new(config: &WindowConfig) -> GoudResult<Self> {
        Self::new_inner(config)
    }

    /// Returns the window handle wrapper for wgpu surface creation.
    pub fn window_handle(&self) -> Arc<XboxWindowHandle> {
        Arc::clone(&self.handle)
    }

    // ------------------------------------------------------------------
    // MSVC implementation (actual GDK path)
    // ------------------------------------------------------------------

    #[cfg(target_env = "msvc")]
    fn new_inner(config: &WindowConfig) -> GoudResult<Self> {
        // TODO(xbox-gdk): Replace with actual XGameWindowCreate / CreateWindowExW
        // call once the GDK SDK is available in the build environment.
        Err(GoudError::BackendNotSupported(
            "Xbox GDK platform requires the GDK SDK (not yet linked)".to_string(),
        ))
    }

    // ------------------------------------------------------------------
    // Non-MSVC stub (allows type-checking on macOS / Linux)
    // ------------------------------------------------------------------

    #[cfg(not(target_env = "msvc"))]
    fn new_inner(_config: &WindowConfig) -> GoudResult<Self> {
        Err(GoudError::BackendNotSupported(
            "Xbox GDK platform is only available on MSVC targets".to_string(),
        ))
    }
}

impl PlatformBackend for XboxGdkPlatform {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn set_should_close(&mut self, should_close: bool) {
        self.should_close = should_close;
    }

    fn poll_events(&mut self, _input: &mut InputManager) -> f32 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        dt
    }

    fn swap_buffers(&mut self) {
        // No-op: wgpu handles presentation via surface.present().
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn request_size(&mut self, width: u32, height: u32) -> bool {
        // Xbox apps are always fullscreen; resize is a no-op.
        self.width = width;
        self.height = height;
        true
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        // Xbox has no HiDPI scaling; logical == physical.
        (self.width, self.height)
    }

    fn set_fullscreen(&mut self, mode: FullscreenMode) -> bool {
        // Xbox is always fullscreen; accept borderless/exclusive, reject windowed.
        mode != FullscreenMode::Windowed
    }

    fn get_fullscreen(&self) -> FullscreenMode {
        FullscreenMode::Borderless
    }
}
