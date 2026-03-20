//! # FFI Renderer Module
//!
//! This module provides C-compatible functions for rendering operations.
//! It integrates with the window FFI to provide basic 2D rendering capabilities.
//!
//! ## Design
//!
//! The renderer FFI provides two modes of operation:
//!
//! 1. **Immediate mode**: Draw individual sprites/quads with explicit parameters
//! 2. **ECS mode**: Automatically render all entities with Sprite + Transform2D components
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // In game loop
//! while (!goud_window_should_close(contextId)) {
//!     float deltaTime = goud_window_poll_events(contextId);
//!
//!     // Clear screen
//!     goud_window_clear(contextId, 0.1f, 0.1f, 0.2f, 1.0f);
//!
//!     // Begin rendering
//!     goud_renderer_begin(contextId);
//!
//!     // Draw sprites (immediate mode)
//!     goud_renderer_draw_quad(contextId, x, y, width, height, r, g, b, a);
//!
//!     // Or draw all ECS sprites
//!     goud_renderer_draw_ecs_sprites(contextId);
//!
//!     // End rendering
//!     goud_renderer_end(contextId);
//!
//!     goud_window_swap_buffers(contextId);
//! }
//! ```
//!
//! ## Submodules
//!
//! - `lifecycle` — Frame begin/end, viewport, blending, depth testing
//! - `texture` — Texture loading and destruction
//! - `handles` — Opaque handle types and rendering statistics
//! - `immediate` — Immediate-mode rendering state and shader setup
//! - `draw` — Draw call FFI functions (sprites, quads)

mod draw;
mod handles;
mod immediate;
mod lifecycle;
mod text;
mod texture;

// Re-export all public items to preserve the original flat public API.

pub use draw::{
    goud_renderer_draw_quad, goud_renderer_draw_sprite, goud_renderer_draw_sprite_rect,
};

#[allow(deprecated)]
pub use text::{
    goud_draw_text, goud_font_destroy, goud_font_load, goud_renderer_draw_text, GoudFontHandle,
    GOUD_INVALID_FONT,
};

pub use handles::{
    goud_renderer_get_stats, GoudBufferHandle, GoudRenderStats, GoudShaderHandle,
    GOUD_INVALID_BUFFER, GOUD_INVALID_SHADER,
};

pub use lifecycle::{
    goud_renderer_begin, goud_renderer_clear_depth, goud_renderer_disable_blending,
    goud_renderer_disable_depth_test, goud_renderer_enable_blending,
    goud_renderer_enable_depth_test, goud_renderer_end, goud_renderer_get_coordinate_origin,
    goud_renderer_set_coordinate_origin, goud_renderer_set_viewport,
};

pub use texture::{
    goud_texture_destroy, goud_texture_load, GoudTextureHandle, GOUD_INVALID_TEXTURE,
};

pub(crate) use text::cleanup_text_state;
