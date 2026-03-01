//! Thin safe wrapper around the GoudEngine FFI functions.
//!
//! Provides an idiomatic Rust interface to the engine's rendering,
//! input, and window management. Once the Rust SDK exposes these
//! capabilities directly, this wrapper can be replaced by
//! `goud_engine::sdk::*` imports.

use std::ffi::CString;

use goud_engine::ffi::input;
use goud_engine::ffi::renderer;
use goud_engine::ffi::window;
use goud_engine::ffi::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// Safe handle to a windowed engine context.
///
/// Creates a GLFW window with an OpenGL backend on construction
/// and destroys it on drop.
pub struct Engine {
    ctx: GoudContextId,
}

impl Engine {
    /// Creates a new window with the given dimensions and title.
    ///
    /// # Panics
    ///
    /// Panics if the window could not be created (e.g. no display).
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let c_title = CString::new(title).expect("title contained a NUL byte");
        // SAFETY: `c_title` is a valid null-terminated C string.
        let ctx = unsafe { window::goud_window_create(width, height, c_title.as_ptr()) };
        assert_ne!(
            ctx, GOUD_INVALID_CONTEXT_ID,
            "Failed to create engine window"
        );
        Self { ctx }
    }

    // -- Window ---------------------------------------------------------------

    pub fn should_close(&self) -> bool {
        window::goud_window_should_close(self.ctx)
    }

    /// Polls platform events and returns delta time in seconds.
    pub fn poll_events(&self) -> f32 {
        window::goud_window_poll_events(self.ctx)
    }

    pub fn swap_buffers(&self) {
        window::goud_window_swap_buffers(self.ctx);
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        window::goud_window_clear(self.ctx, r, g, b, a);
    }

    // -- Rendering ------------------------------------------------------------

    pub fn begin_frame(&self) -> bool {
        renderer::goud_renderer_begin(self.ctx)
    }

    pub fn end_frame(&self) -> bool {
        renderer::goud_renderer_end(self.ctx)
    }

    pub fn enable_blending(&self) {
        renderer::goud_renderer_enable_blending(self.ctx);
    }

    /// Loads a texture from disk and returns its handle.
    pub fn load_texture(&self, path: &str) -> u64 {
        let c_path = CString::new(path).expect("path contained a NUL byte");
        // SAFETY: `c_path` is a valid null-terminated C string.
        unsafe { renderer::goud_texture_load(self.ctx, c_path.as_ptr()) }
    }

    /// Draws a textured sprite at `(x, y)` with the given size and rotation.
    /// Color tint is white (1,1,1,1).
    pub fn draw_sprite(
        &self,
        texture: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
    ) {
        renderer::goud_renderer_draw_sprite(
            self.ctx, texture, x, y, width, height, rotation, 1.0, 1.0, 1.0, 1.0,
        );
    }

    // -- Input ----------------------------------------------------------------

    pub fn is_key_pressed(&self, key: i32) -> bool {
        input::goud_input_key_pressed(self.ctx, key)
    }

    pub fn is_mouse_button_pressed(&self, button: i32) -> bool {
        input::goud_input_mouse_button_pressed(self.ctx, button)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        window::goud_window_destroy(self.ctx);
    }
}

// -- Key / mouse-button constants re-exported for convenience -----------------

pub const KEY_SPACE: i32 = input::KEY_SPACE;
pub const KEY_ESCAPE: i32 = input::KEY_ESCAPE;
pub const KEY_R: i32 = input::KEY_R;
pub const MOUSE_LEFT: i32 = input::MOUSE_BUTTON_LEFT;
