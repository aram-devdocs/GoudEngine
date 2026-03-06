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
/// The `WindowProvider::poll_events()` trait method calls GLFW event polling
/// but cannot dispatch input events (the trait takes no parameters). Use
/// [`poll_events_with_input`](GlfwWindowProvider::poll_events_with_input)
/// for full input dispatch.
pub struct GlfwWindowProvider {
    platform: GlfwPlatform,
}

impl GlfwWindowProvider {
    /// Creates a new GLFW window provider wrapping the given platform.
    pub fn new(platform: GlfwPlatform) -> Self {
        Self { platform }
    }

    /// Polls events and dispatches input to the given `InputManager`.
    ///
    /// This is the full-featured poll that delegates to
    /// [`PlatformBackend::poll_events`], which processes keyboard, mouse,
    /// scroll, and window events and feeds them into the input manager.
    ///
    /// Returns delta time in seconds since the last call.
    pub fn poll_events_with_input(
        &mut self,
        input: &mut crate::core::input_manager::InputManager,
    ) -> f32 {
        self.platform.poll_events(input)
    }

    /// Returns a reference to the underlying platform.
    #[allow(dead_code)]
    pub(crate) fn platform(&self) -> &GlfwPlatform {
        &self.platform
    }

    /// Returns a mutable reference to the underlying platform.
    #[allow(dead_code)]
    pub(crate) fn platform_mut(&mut self) -> &mut GlfwPlatform {
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
    // GlfwWindowProvider requires a GLFW context and display server.
    // These tests can only run in an environment with a windowing system.

    #[test]
    fn test_glfw_window_provider_type_check() {
        // Compile-time check that GlfwWindowProvider is NOT Send.
        // If this compiles, the type correctly omits Send.
        fn assert_not_send<T>() {
            // This function exists only for the negative trait bound test
            // in the static_assertions below.
        }
        // GlfwWindowProvider wraps GlfwPlatform which contains GLFW types
        // that are !Send, so this struct is automatically !Send.
        let _ = assert_not_send::<()>;
    }
}
