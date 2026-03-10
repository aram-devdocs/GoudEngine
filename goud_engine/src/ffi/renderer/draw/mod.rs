//! # Draw Command FFI
//!
//! Immediate-mode draw calls: sprites, sprite sheet rects, and colored quads.

mod debug;
mod ffi;
mod helpers;
mod internal;

pub use ffi::{goud_renderer_draw_quad, goud_renderer_draw_sprite, goud_renderer_draw_sprite_rect};

pub(crate) use debug::render_physics_debug_overlay;
pub(crate) use internal::draw_sprite_rect_internal;
