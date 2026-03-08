//! # Factory Functions for Text Component
//!
//! Provides `goud_text_new` and `goud_text_default` for creating text
//! components with sensible defaults.

use crate::core::types::FfiText;

/// Creates a new text component with default settings.
///
/// The text will render with:
/// - Font size 16.0
/// - White color tint (1.0, 1.0, 1.0, 1.0)
/// - Left alignment
/// - No max width (no word-wrapping)
/// - Line spacing 1.0
///
/// # Parameters
///
/// - `font_handle`: The font asset handle (packed as u64)
///
/// # Returns
///
/// A new FfiText with default settings.
#[no_mangle]
pub extern "C" fn goud_text_new(font_handle: u64) -> FfiText {
    FfiText {
        font_handle,
        font_size: 16.0,
        color_r: 1.0,
        color_g: 1.0,
        color_b: 1.0,
        color_a: 1.0,
        alignment: 0, // Left
        max_width: 0.0,
        has_max_width: false,
        line_spacing: 1.0,
    }
}

/// Creates a default text component with an invalid font handle.
///
/// Primarily useful for deserialization or when the font handle
/// will be set later.
///
/// # Returns
///
/// A default FfiText with invalid font handle.
#[no_mangle]
pub extern "C" fn goud_text_default() -> FfiText {
    goud_text_new(u64::MAX) // Invalid handle
}
