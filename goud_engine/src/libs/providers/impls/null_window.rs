//! Null window provider -- silent no-op for headless testing.

use crate::libs::error::GoudResult;
use crate::libs::providers::window::WindowProvider;

/// A window provider that does nothing. Used for headless testing and as
/// a default when no windowing system is available.
pub struct NullWindowProvider {
    should_close: bool,
}

impl NullWindowProvider {
    /// Create a new null window provider.
    pub fn new() -> Self {
        Self {
            should_close: false,
        }
    }
}

impl Default for NullWindowProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowProvider for NullWindowProvider {
    fn name(&self) -> &str {
        "null"
    }

    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}

    fn should_close(&self) -> bool {
        self.should_close
    }

    fn set_should_close(&mut self, value: bool) {
        self.should_close = value;
    }

    fn poll_events(&mut self) {}

    fn swap_buffers(&mut self) {}

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_framebuffer_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_window_construction() {
        let provider = NullWindowProvider::new();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_window_default() {
        let provider = NullWindowProvider::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_window_init_shutdown() {
        let mut provider = NullWindowProvider::new();
        assert!(provider.init().is_ok());
        provider.shutdown();
    }

    #[test]
    fn test_null_window_should_close() {
        let mut provider = NullWindowProvider::new();
        assert!(!provider.should_close());
        provider.set_should_close(true);
        assert!(provider.should_close());
    }

    #[test]
    fn test_null_window_size() {
        let provider = NullWindowProvider::new();
        assert_eq!(provider.get_size(), (0, 0));
        assert_eq!(provider.get_framebuffer_size(), (0, 0));
    }

    #[test]
    fn test_null_window_poll_and_swap() {
        let mut provider = NullWindowProvider::new();
        provider.poll_events();
        provider.swap_buffers();
        // Verify that no-op operations don't change state
        assert!(!provider.should_close());
    }
}
