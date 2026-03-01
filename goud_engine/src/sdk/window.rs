//! # SDK Window Management API
//!
//! Provides methods on [`GoudGame`](super::GoudGame) for window lifecycle
//! management: polling events, swapping buffers, querying window state, and
//! clearing the screen. Also provides static lifecycle functions on [`Window`]
//! for creating and destroying windowed contexts from FFI.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop platform with GLFW).
//! Window methods are only available when GoudGame has been initialized with
//! a platform backend (i.e., when running on desktop, not in headless/test mode).
//!
//! # Example
//!
//! ```rust,ignore
//! use goud_engine::sdk::{GoudGame, GameConfig};
//!
//! let mut game = GoudGame::new(GameConfig::default()).unwrap();
//!
//! // Main game loop
//! while !game.should_close() {
//!     let dt = game.poll_events();
//!     // ... update game logic with dt ...
//!     // ... render ...
//!     game.swap_buffers();
//! }
//! ```

use super::GoudGame;
use crate::core::error::{set_last_error, GoudError, GoudResult};
use crate::core::context_registry::{
    get_context_registry, GoudContext, GoudContextId, GOUD_INVALID_CONTEXT_ID,
};
use crate::libs::graphics::backend::opengl::OpenGLBackend;
use crate::libs::graphics::backend::RenderBackend;
use super::GameConfig;
use std::ffi::CStr;

// =============================================================================
// Window Lifecycle (static functions for FFI context creation/destruction)
// =============================================================================

/// Zero-sized type that hosts window lifecycle functions.
///
/// All methods are static (no `self` receiver) and generate FFI wrappers
/// via the `#[goud_api]` proc-macro.
pub struct Window;

#[goud_engine_macros::goud_api(module = "window", feature = "native")]
impl Window {
    /// Creates a new windowed context with OpenGL rendering.
    ///
    /// This creates:
    /// - A `GoudGame` with a GLFW platform backend (window + input)
    /// - An OpenGL 3.3 Core rendering backend (stored in GoudGame)
    ///
    /// Returns a context ID on success, or `GOUD_INVALID_CONTEXT_ID` on
    /// failure.
    ///
    /// # Safety
    ///
    /// The `title` pointer must be a valid null-terminated C string or null.
    pub fn create(width: u32, height: u32, title: &str) -> GoudContextId {
        let game_config = GameConfig {
            title: title.to_string(),
            width,
            height,
            vsync: true,
            resizable: true,
            ..GameConfig::default()
        };

        let mut game = match GoudGame::with_platform(game_config) {
            Ok(g) => g,
            Err(e) => {
                set_last_error(e);
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        // Create the OpenGL backend for rendering.
        let mut backend = match OpenGLBackend::new() {
            Ok(b) => b,
            Err(e) => {
                set_last_error(e);
                return GOUD_INVALID_CONTEXT_ID;
            }
        };
        backend.set_viewport(0, 0, width, height);
        game.render_backend = Some(backend);

        // Allocate a context slot, then replace with a GoudGame-backed context.
        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        let context_id = match registry.create() {
            Ok(id) => id,
            Err(e) => {
                set_last_error(e);
                return GOUD_INVALID_CONTEXT_ID;
            }
        };

        let generation = context_id.generation();
        let real_context = GoudContext::with_game(game, generation);
        registry.replace(context_id, real_context);

        context_id
    }

    /// Destroys a windowed context and releases all resources.
    ///
    /// This destroys the window, OpenGL context, and ECS world.
    ///
    /// Returns `true` on success, `false` on error.
    pub fn destroy(context_id: GoudContextId) -> bool {
        if context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return false;
        }

        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return false;
            }
        };

        match registry.destroy(context_id) {
            Ok(()) => true,
            Err(e) => {
                set_last_error(e);
                false
            }
        }
    }
}

// =============================================================================
// Window Instance Methods (on GoudGame)
// =============================================================================

#[goud_engine_macros::goud_api(module = "window", feature = "native")]
impl GoudGame {
    /// Returns `true` if the window has been requested to close.
    ///
    /// This checks whether the user clicked the close button, pressed Alt+F4,
    /// or called [`set_should_close`](Self::set_should_close).
    ///
    /// Returns `false` if no platform backend is initialized (headless mode).
    #[inline]
    pub fn should_close(&self) -> bool {
        match &self.platform {
            Some(platform) => platform.should_close(),
            None => false,
        }
    }

    /// Signals the window to close (or not) after the current frame.
    ///
    /// # Errors
    ///
    /// Returns an error if no platform backend is initialized.
    pub fn set_should_close(&mut self, should_close: bool) -> GoudResult<()> {
        match &mut self.platform {
            Some(platform) => {
                platform.set_should_close(should_close);
                Ok(())
            }
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Polls platform events and advances input state for the new frame.
    ///
    /// This processes all pending window/input events, syncs the OpenGL
    /// viewport on window resize, and returns the delta time (seconds since
    /// last call). Must be called once per frame before querying input.
    ///
    /// # Errors
    ///
    /// Returns an error if no platform backend is initialized.
    pub fn poll_events(&mut self) -> GoudResult<f32> {
        let (dt, fb_size) = match &mut self.platform {
            Some(platform) => {
                let dt = platform.poll_events(&mut self.input_manager);
                let fb_size = platform.get_framebuffer_size();
                (dt, fb_size)
            }
            None => return Err(GoudError::NotInitialized),
        };

        // Sync the render backend viewport to the current framebuffer size.
        if let Some(backend) = &mut self.render_backend {
            backend.set_viewport(0, 0, fb_size.0, fb_size.1);
        }

        self.context.update(dt);
        Ok(dt)
    }

    /// Presents the rendered frame by swapping front and back buffers.
    ///
    /// Call this at the end of each frame after all rendering is complete.
    ///
    /// # Errors
    ///
    /// Returns an error if no platform backend is initialized.
    pub fn swap_buffers(&mut self) -> GoudResult<()> {
        match &mut self.platform {
            Some(platform) => {
                platform.swap_buffers();
                Ok(())
            }
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Clears the window with the specified color.
    ///
    /// Sets the clear color and clears the color buffer using the
    /// render backend.
    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        if let Some(backend) = &mut self.render_backend {
            backend.set_clear_color(r, g, b, a);
            backend.clear_color();
        }
    }

    /// Returns the physical window size `(width, height)` in pixels.
    ///
    /// If no platform backend is initialized, returns the configured size
    /// from [`GameConfig`](super::GameConfig).
    #[inline]
    #[goud_api(name = "get_size")]
    pub fn get_window_size(&self) -> (u32, u32) {
        match &self.platform {
            Some(platform) => platform.get_size(),
            None => (self.config.width, self.config.height),
        }
    }

    /// Returns `true` if a platform backend is initialized.
    ///
    /// When `false`, window/rendering methods will return errors or
    /// fall back to headless behavior.
    #[inline]
    #[goud_api(skip)]
    pub fn has_platform(&self) -> bool {
        self.platform.is_some()
    }

    /// Returns the delta time (seconds since last `poll_events` call).
    ///
    /// This reads from the internal `GameContext` which is updated by
    /// [`poll_events`](Self::poll_events). Returns `0.0` before the first
    /// poll.
    #[inline]
    pub fn get_delta_time(&self) -> f32 {
        self.context.delta_time()
    }

    /// Returns the physical framebuffer size `(width, height)` in pixels.
    ///
    /// On HiDPI/Retina displays, this may differ from the logical window
    /// size returned by [`get_window_size`](Self::get_window_size).
    /// Renderers should use this for `gl::Viewport`.
    ///
    /// If no platform backend is initialized, returns the configured size.
    #[inline]
    #[goud_api(skip)]
    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        match &self.platform {
            Some(platform) => platform.get_framebuffer_size(),
            None => (self.config.width, self.config.height),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_close_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        // No platform => should_close returns false
        assert!(!game.should_close());
    }

    #[test]
    fn test_set_should_close_headless_returns_error() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let result = game.set_should_close(true);
        assert!(result.is_err());
    }

    #[test]
    fn test_poll_events_headless_returns_error() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let result = game.poll_events();
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_buffers_headless_returns_error() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let result = game.swap_buffers();
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_headless_no_panic() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        // No backend => clear is a no-op, should not panic
        game.clear(0.0, 0.0, 0.0, 1.0);
    }

    #[test]
    fn test_get_window_size_headless() {
        let game = GoudGame::new(GameConfig::new("Test", 1280, 720)).unwrap();
        // No platform => falls back to config size
        assert_eq!(game.get_window_size(), (1280, 720));
    }

    #[test]
    fn test_has_platform_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.has_platform());
    }

    #[test]
    fn test_get_delta_time_initial() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!((game.get_delta_time() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_get_framebuffer_size_headless() {
        let game = GoudGame::new(GameConfig::new("Test", 1920, 1080)).unwrap();
        // No platform => falls back to config size
        assert_eq!(game.get_framebuffer_size(), (1920, 1080));
    }

    #[test]
    fn test_window_destroy_invalid_returns_false() {
        assert!(!Window::destroy(GOUD_INVALID_CONTEXT_ID));
    }
}
