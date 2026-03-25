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

#[cfg(feature = "legacy-glfw-opengl")]
pub mod glfw_platform;
#[cfg(feature = "native")]
pub mod native_runtime;
#[cfg(all(
    feature = "wgpu-backend",
    feature = "native",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
pub mod winit_platform;

#[cfg(feature = "native")]
use crate::core::input_manager::InputManager;

/// Fullscreen mode for the native window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FullscreenMode {
    /// Standard windowed mode.
    #[default]
    Windowed = 0,
    /// Borderless fullscreen (covers the whole screen without exclusive access).
    Borderless = 1,
    /// Exclusive fullscreen (takes over the monitor with a video mode change).
    Exclusive = 2,
}

impl FullscreenMode {
    /// Converts an FFI/backend code into a fullscreen mode.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Windowed),
            1 => Some(Self::Borderless),
            2 => Some(Self::Exclusive),
            _ => None,
        }
    }
}

/// Native rendering backend selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum RenderBackendKind {
    /// Cross-platform wgpu backend.
    #[default]
    Wgpu = 0,
    /// Legacy OpenGL backend.
    OpenGlLegacy = 1,
    /// Auto-detect the best available backend at runtime.
    Auto = 2,
}

impl RenderBackendKind {
    /// Converts an FFI/backend code into a render backend.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Wgpu),
            1 => Some(Self::OpenGlLegacy),
            2 => Some(Self::Auto),
            _ => None,
        }
    }
}

/// Native window backend selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum WindowBackendKind {
    /// winit native windowing path.
    #[default]
    Winit = 0,
    /// Legacy GLFW windowing path.
    GlfwLegacy = 1,
}

impl WindowBackendKind {
    /// Converts an FFI/backend code into a window backend.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Winit),
            1 => Some(Self::GlfwLegacy),
            _ => None,
        }
    }
}

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

    /// Requested MSAA sample count for native window creation.
    pub msaa_samples: u32,

    /// Requested fullscreen mode for window creation.
    pub fullscreen_mode: FullscreenMode,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "GoudEngine".to_string(),
            vsync: true,
            resizable: true,
            msaa_samples: 1,
            fullscreen_mode: FullscreenMode::Windowed,
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

    /// Requests a logical window resize.
    ///
    /// The resize may apply asynchronously after the next event pump.
    fn request_size(&mut self, width: u32, height: u32) -> bool;

    /// Returns the physical framebuffer size `(width, height)` in pixels.
    ///
    /// This may differ from [`get_size`](PlatformBackend::get_size) on
    /// HiDPI/Retina displays where the framebuffer resolution is higher
    /// than the logical window size.
    fn get_framebuffer_size(&self) -> (u32, u32);

    /// Sets the fullscreen mode on the window.
    ///
    /// Returns `true` if the mode was applied, `false` if unsupported.
    fn set_fullscreen(&mut self, _mode: FullscreenMode) -> bool {
        false
    }

    /// Returns the current fullscreen mode.
    fn get_fullscreen(&self) -> FullscreenMode {
        FullscreenMode::Windowed
    }
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
        assert_eq!(config.msaa_samples, 1);
        assert_eq!(config.fullscreen_mode, FullscreenMode::Windowed);
    }

    #[test]
    fn window_config_clone() {
        let config = WindowConfig {
            width: 1920,
            height: 1080,
            title: "Test".to_string(),
            vsync: false,
            resizable: false,
            msaa_samples: 4,
            fullscreen_mode: FullscreenMode::Borderless,
        };
        let cloned = config.clone();
        assert_eq!(config.width, cloned.width);
        assert_eq!(config.height, cloned.height);
        assert_eq!(config.title, cloned.title);
        assert_eq!(config.vsync, cloned.vsync);
        assert_eq!(config.resizable, cloned.resizable);
        assert_eq!(config.msaa_samples, cloned.msaa_samples);
        assert_eq!(config.fullscreen_mode, cloned.fullscreen_mode);
    }

    #[test]
    fn fullscreen_mode_from_u32_round_trips() {
        assert_eq!(FullscreenMode::from_u32(0), Some(FullscreenMode::Windowed));
        assert_eq!(
            FullscreenMode::from_u32(1),
            Some(FullscreenMode::Borderless)
        );
        assert_eq!(FullscreenMode::from_u32(2), Some(FullscreenMode::Exclusive));
        assert_eq!(FullscreenMode::from_u32(3), None);
        assert_eq!(FullscreenMode::from_u32(99), None);
    }

    #[test]
    fn fullscreen_mode_default_is_windowed() {
        assert_eq!(FullscreenMode::default(), FullscreenMode::Windowed);
    }

    #[test]
    fn render_backend_kind_auto_variant() {
        assert_eq!(
            RenderBackendKind::from_u32(2),
            Some(RenderBackendKind::Auto)
        );
    }
}
