//! # FFI Functions for Text Component
//!
//! This module provides C-compatible functions for manipulating Text components.
//! These functions allow language bindings to perform text operations without
//! duplicating logic across SDKs.
//!
//! ## Module Layout
//!
//! - `factory`    -- `goud_text_new`, `goud_text_default`
//! - `properties` -- font size, color, alignment, max width, line spacing

pub mod factory;
pub mod properties;

// Re-export the FfiText type from core for backward compatibility
pub use crate::core::types::FfiText;

// Re-export all public FFI functions
pub use factory::{goud_text_default, goud_text_new};

pub use properties::{
    goud_text_clear_max_width, goud_text_get_alignment, goud_text_get_color_a,
    goud_text_get_color_b, goud_text_get_color_g, goud_text_get_color_r, goud_text_get_font_size,
    goud_text_get_line_spacing, goud_text_get_max_width, goud_text_has_max_width,
    goud_text_set_alignment, goud_text_set_color, goud_text_set_font_size,
    goud_text_set_line_spacing, goud_text_set_max_width,
};
