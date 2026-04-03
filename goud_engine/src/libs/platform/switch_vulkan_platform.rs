//! Nintendo Switch Vulkan platform backend (PoC).
//!
//! Provides a [`PlatformBackend`] implementation for the Nintendo Switch using
//! Vulkan. On non-aarch64 targets the constructor returns
//! [`GoudError::BackendNotSupported`] so the module can still be type-checked
//! during cross-platform development.

use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{GoudError, GoudResult};
use crate::core::input_manager::InputManager;

use super::{FullscreenMode, PlatformBackend, WindowConfig};
use crate::libs::graphics::backend::wgpu_backend::switch_surface::SwitchWindowHandle;

/// Nintendo Switch Vulkan platform backend.
///
/// Wraps a Switch `nn::vi::Layer` handle and implements the engine's platform
/// abstraction. On non-aarch64 hosts the constructor always fails with a
/// descriptive error.
#[allow(dead_code)] // PoC stub: struct fields used once NintendoSDK is linked.
pub struct SwitchVulkanPlatform {
    should_close: bool,
    width: u32,
    height: u32,
    last_frame: Instant,
    handle: Arc<SwitchWindowHandle>,
}

impl SwitchVulkanPlatform {
    /// Creates the Nintendo Switch Vulkan platform.
    ///
    /// On aarch64 targets this would initialize the `nn::vi` display layer
    /// and extract the native window handle for Vulkan surface creation. On
    /// other targets this returns `BackendNotSupported`.
    pub fn new(config: &WindowConfig) -> GoudResult<Self> {
        Self::new_inner(config)
    }

    /// Returns the window handle wrapper for wgpu surface creation.
    #[allow(dead_code)] // Called from native_runtime once NintendoSDK is linked.
    pub fn window_handle(&self) -> Arc<SwitchWindowHandle> {
        Arc::clone(&self.handle)
    }

    // ------------------------------------------------------------------
    // aarch64 implementation (actual Switch path)
    // ------------------------------------------------------------------

    #[cfg(target_arch = "aarch64")]
    fn new_inner(_config: &WindowConfig) -> GoudResult<Self> {
        // TODO(switch-vulkan): Initialize nn::vi display, create layer,
        // extract native window handle for Vulkan surface creation.
        Err(GoudError::BackendNotSupported(
            "Nintendo Switch platform requires NintendoSDK (not yet linked)".to_string(),
        ))
    }

    // ------------------------------------------------------------------
    // Non-aarch64 stub (allows type-checking on x86_64 hosts)
    // ------------------------------------------------------------------

    #[cfg(not(target_arch = "aarch64"))]
    fn new_inner(_config: &WindowConfig) -> GoudResult<Self> {
        Err(GoudError::BackendNotSupported(
            "Nintendo Switch platform is only available on aarch64 targets".to_string(),
        ))
    }
}

impl PlatformBackend for SwitchVulkanPlatform {
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

    fn request_size(&mut self, _width: u32, _height: u32) -> bool {
        // Switch is always 1920x1080 docked; resize is a no-op.
        false
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        // Switch has no HiDPI scaling; logical == physical.
        (self.width, self.height)
    }

    fn set_fullscreen(&mut self, mode: FullscreenMode) -> bool {
        // Switch is always fullscreen; accept borderless/exclusive, reject windowed.
        mode != FullscreenMode::Windowed
    }

    fn get_fullscreen(&self) -> FullscreenMode {
        FullscreenMode::Exclusive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "aarch64"))]
    #[test]
    fn new_returns_backend_not_supported_on_non_aarch64() {
        let result = SwitchVulkanPlatform::new(&WindowConfig::default());
        let err = result.err().expect("should fail on non-aarch64");
        assert!(err.to_string().contains("aarch64"));
    }
}
