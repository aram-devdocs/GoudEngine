//! Platform abstraction layer.
//!
//! This module provides the [`PlatformBackend`] trait for windowing and input,
//! enabling different platform implementations (GLFW, winit, SDL2) to be
//! swapped without changing higher-level engine code.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────┐
//! │   PlatformBackend (trait)    │
//! ├──────────────────────────────┤
//! │ GlfwPlatform  │ WinitPlatform│  ← concrete implementations
//! │  (desktop)    │  (web+desk)  │
//! └──────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use goud_engine::libs::platform::{PlatformBackend, WindowConfig};
//! use goud_engine::libs::platform::glfw_platform::GlfwPlatform;
//!
//! let config = WindowConfig {
//!     width: 800,
//!     height: 600,
//!     title: "My Game".to_string(),
//!     ..Default::default()
//! };
//! let mut platform = GlfwPlatform::new(&config)?;
//! ```

#[cfg(feature = "native")]
pub mod glfw_platform;
#[cfg(all(feature = "wgpu-backend", feature = "native"))]
pub mod winit_platform;

#[cfg(feature = "native")]
use crate::ecs::InputManager;

/// Configuration for creating a platform window.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window width in pixels.
    pub width: u32,

    /// Window height in pixels.
    pub height: u32,

    /// Window title displayed in the title bar.
    pub title: String,

    /// Enable vertical sync to prevent screen tearing.
    pub vsync: bool,

    /// Allow the user to resize the window.
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "GoudEngine".to_string(),
            vsync: true,
            resizable: true,
        }
    }
}

/// Platform backend abstraction for window and input management.
///
/// Implementations handle platform-specific window lifecycle, event polling,
/// input dispatch, and buffer presentation. This trait enables the engine to
/// support multiple windowing systems through a unified interface.
///
/// # Lifecycle
///
/// 1. Create the backend via its constructor (e.g., `GlfwPlatform::new(config)`)
/// 2. Each frame: call [`poll_events`](PlatformBackend::poll_events) → render → call [`swap_buffers`](PlatformBackend::swap_buffers)
/// 3. Check [`should_close`](PlatformBackend::should_close) to determine when to exit
///
/// # Thread Safety
///
/// Most windowing APIs require main-thread access. Implementations are NOT
/// required to be `Send` or `Sync`.
#[cfg(feature = "native")]
pub trait PlatformBackend {
    /// Returns `true` if the window has been requested to close.
    fn should_close(&self) -> bool;

    /// Sets whether the window should close.
    fn set_should_close(&mut self, should_close: bool);

    /// Polls platform events and feeds input events to the [`InputManager`].
    ///
    /// This advances the input state for the new frame, processes all pending
    /// platform events, and calculates the time elapsed since the last call.
    ///
    /// Returns delta time in seconds since the last call.
    fn poll_events(&mut self, input: &mut InputManager) -> f32;

    /// Presents the rendered frame by swapping front and back buffers.
    fn swap_buffers(&mut self);

    /// Returns the logical window size `(width, height)` in screen coordinates.
    fn get_size(&self) -> (u32, u32);

    /// Returns the physical framebuffer size `(width, height)` in pixels.
    ///
    /// This may differ from [`get_size`](PlatformBackend::get_size) on
    /// HiDPI/Retina displays where the framebuffer resolution is higher
    /// than the logical window size.
    fn get_framebuffer_size(&self) -> (u32, u32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_config_default_values() {
        let config = WindowConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.title, "GoudEngine");
        assert!(config.vsync);
        assert!(config.resizable);
    }

    #[test]
    fn window_config_clone() {
        let config = WindowConfig {
            width: 1920,
            height: 1080,
            title: "Test".to_string(),
            vsync: false,
            resizable: false,
        };
        let cloned = config.clone();
        assert_eq!(config.width, cloned.width);
        assert_eq!(config.height, cloned.height);
        assert_eq!(config.title, cloned.title);
        assert_eq!(config.vsync, cloned.vsync);
        assert_eq!(config.resizable, cloned.resizable);
    }
}
