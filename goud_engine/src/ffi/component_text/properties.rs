//! # Text Property Methods
//!
//! Provides FFI functions for reading and writing text properties:
//! font size, color, alignment, max width, and line spacing.

use crate::core::types::FfiText;

// =============================================================================
// Font Size Methods
// =============================================================================

/// Sets the font size in pixels.
///
/// # Parameters
///
/// - `text`: Pointer to the text to modify
/// - `size`: Font size in pixels
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_set_font_size(text: *mut FfiText, size: f32) {
    if text.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).font_size = size;
}

/// Gets the font size in pixels.
///
/// # Parameters
///
/// - `text`: Pointer to the text to read
///
/// # Returns
///
/// The font size, or 0.0 if the pointer is null.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_font_size(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).font_size
}

// =============================================================================
// Color Methods
// =============================================================================

/// Sets the RGBA color of the text.
///
/// # Parameters
///
/// - `text`: Pointer to the text to modify
/// - `r`: Red component (0.0 - 1.0)
/// - `g`: Green component (0.0 - 1.0)
/// - `b`: Blue component (0.0 - 1.0)
/// - `a`: Alpha component (0.0 - 1.0)
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_set_color(text: *mut FfiText, r: f32, g: f32, b: f32, a: f32) {
    if text.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    let t = &mut *text;
    t.color_r = r;
    t.color_g = g;
    t.color_b = b;
    t.color_a = a;
}

/// Gets the red color component.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_color_r(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).color_r
}

/// Gets the green color component.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_color_g(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).color_g
}

/// Gets the blue color component.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_color_b(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).color_b
}

/// Gets the alpha color component.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_color_a(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).color_a
}

// =============================================================================
// Alignment Methods
// =============================================================================

/// Sets the horizontal text alignment.
///
/// # Parameters
///
/// - `text`: Pointer to the text to modify
/// - `alignment`: 0 = Left, 1 = Center, 2 = Right
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_set_alignment(text: *mut FfiText, alignment: u8) {
    if text.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).alignment = alignment;
}

/// Gets the horizontal text alignment.
///
/// # Returns
///
/// The alignment value (0 = Left, 1 = Center, 2 = Right), or 0 if null.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_alignment(text: *const FfiText) -> u8 {
    if text.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).alignment
}

// =============================================================================
// Max Width Methods
// =============================================================================

/// Sets the maximum width for word-wrapping.
///
/// # Parameters
///
/// - `text`: Pointer to the text to modify
/// - `width`: Maximum width in pixels
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_set_max_width(text: *mut FfiText, width: f32) {
    if text.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    let t = &mut *text;
    t.max_width = width;
    t.has_max_width = true;
}

/// Clears the maximum width, disabling word-wrapping.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_clear_max_width(text: *mut FfiText) {
    if text.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).has_max_width = false;
}

/// Gets the maximum width for word-wrapping.
///
/// # Returns
///
/// The max width value, or 0.0 if null.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_max_width(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).max_width
}

/// Returns whether the text has a max width set.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_has_max_width(text: *const FfiText) -> bool {
    if text.is_null() {
        return false;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).has_max_width
}

// =============================================================================
// Line Spacing Methods
// =============================================================================

/// Sets the line spacing multiplier.
///
/// # Parameters
///
/// - `text`: Pointer to the text to modify
/// - `spacing`: Line spacing multiplier (1.0 = default)
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_set_line_spacing(text: *mut FfiText, spacing: f32) {
    if text.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).line_spacing = spacing;
}

/// Gets the line spacing multiplier.
///
/// # Returns
///
/// The line spacing value, or 0.0 if null.
///
/// # Safety
///
/// - `text` must be a valid, non-null pointer
#[no_mangle]
pub unsafe extern "C" fn goud_text_get_line_spacing(text: *const FfiText) -> f32 {
    if text.is_null() {
        return 0.0;
    }
    // SAFETY: Caller guarantees `text` is valid and non-null; null check above.
    (*text).line_spacing
}
