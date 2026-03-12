//! GLFW window provider -- wraps `GlfwPlatform` for the provider API.

use crate::libs::error::GoudResult;
use crate::libs::platform::glfw_platform::GlfwPlatform;
use crate::libs::platform::PlatformBackend;
use crate::libs::providers::window::WindowProvider;

/// GLFW-based window provider that wraps an existing [`GlfwPlatform`].
///
/// This provider is intentionally NOT `Send + Sync` because GLFW requires
/// all window calls on the main thread. It is stored directly in `GoudGame`
/// rather than in the thread-safe `ProviderRegistry`.
///
/// # Event Polling
///
/// The `WindowProvider::poll_events()` trait method only calls GLFW event
/// polling without dispatching to an input manager. For full input dispatch,
/// Layer 2 code (e.g., `GoudGame`) should call `platform_mut()` and
/// invoke `PlatformBackend::poll_events()` with an `InputManager`.
pub struct GlfwWindowProvider {
    platform: GlfwPlatform,
}

impl GlfwWindowProvider {
    /// Creates a new GLFW window provider wrapping the given platform.
    pub fn new(platform: GlfwPlatform) -> Self {
        Self { platform }
    }

    /// Returns a reference to the underlying platform.
    pub fn platform(&self) -> &GlfwPlatform {
        &self.platform
    }

    /// Returns a mutable reference to the underlying platform.
    ///
    /// Used by Layer 2 code to bridge platform event polling with input
    /// managers and other Layer 2 systems.
    pub fn platform_mut(&mut self) -> &mut GlfwPlatform {
        &mut self.platform
    }
}

impl WindowProvider for GlfwWindowProvider {
    fn name(&self) -> &str {
        "glfw"
    }

    fn init(&mut self) -> GoudResult<()> {
        // Platform is already initialized by construction.
        Ok(())
    }

    fn shutdown(&mut self) {
        // GlfwPlatform cleans up in its Drop impl.
    }

    fn should_close(&self) -> bool {
        self.platform.should_close()
    }

    fn set_should_close(&mut self, value: bool) {
        self.platform.set_should_close(value);
    }

    fn poll_events(&mut self) {
        // No-op: use poll_events_with_input() for full input dispatch.
        // We cannot call PlatformBackend::poll_events here because it
        // requires an InputManager reference that the trait does not
        // provide.
    }

    fn swap_buffers(&mut self) {
        self.platform.swap_buffers();
    }

    fn get_size(&self) -> (u32, u32) {
        self.platform.get_size()
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        self.platform.get_framebuffer_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GlfwWindowProvider requires a GLFW context and display server.
    // These tests can only run in an environment with a windowing system.
    // They are skipped here since we cannot instantiate GlfwPlatform without
    // a real display server. In integration tests or when running in a
    // windowed environment, you can test this provider directly.

    #[test]
    fn test_glfw_window_provider_has_platform_methods() {
        // Verify that the public methods for accessing the platform exist.
        // This is a compile-time check encoded as a runtime test.
        // If platform() and platform_mut() don't exist, this won't compile.

        // We cannot instantiate GlfwWindowProvider without GLFW,
        // so we just verify the type is well-formed by creating a reference.
        let _: fn(&GlfwWindowProvider) -> &GlfwPlatform = GlfwWindowProvider::platform;
        let _: fn(&mut GlfwWindowProvider) -> &mut GlfwPlatform = GlfwWindowProvider::platform_mut;
    }
}
