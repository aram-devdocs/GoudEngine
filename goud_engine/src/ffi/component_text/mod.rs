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

pub(crate) mod factory;
pub(crate) mod properties;

// Re-export the FfiText type from core for backward compatibility
pub use crate::core::types::FfiText;

use crate::core::types::TextAlignment;
use crate::ecs::components::text::Text;

/// Converts a [`Text`] component into its FFI-safe representation.
///
/// This impl lives in the FFI layer (Layer 5) because it bridges a
/// Layer-3 type ([`Text`]) and a Layer-1 type ([`FfiText`]).
impl From<&Text> for FfiText {
    fn from(text: &Text) -> Self {
        Self {
            font_handle: text.font_handle.to_u64(),
            font_size: text.font_size,
            color_r: text.color.r,
            color_g: text.color.g,
            color_b: text.color.b,
            color_a: text.color.a,
            alignment: match text.alignment {
                TextAlignment::Left => 0,
                TextAlignment::Center => 1,
                TextAlignment::Right => 2,
            },
            max_width: text.max_width.unwrap_or(0.0),
            has_max_width: text.max_width.is_some(),
            line_spacing: text.line_spacing,
        }
    }
}

// Re-export all public FFI functions
pub use factory::{goud_text_default, goud_text_new};

pub use properties::{
    goud_text_clear_max_width, goud_text_get_alignment, goud_text_get_color_a,
    goud_text_get_color_b, goud_text_get_color_g, goud_text_get_color_r, goud_text_get_font_size,
    goud_text_get_line_spacing, goud_text_get_max_width, goud_text_has_max_width,
    goud_text_set_alignment, goud_text_set_color, goud_text_set_font_size,
    goud_text_set_line_spacing, goud_text_set_max_width,
};
