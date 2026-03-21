//! # Text Batch FFI
//!
//! Provides `goud_renderer_draw_text_batch` for drawing many text labels in a
//! single batched call.  Each command reuses the glyph atlas cached by
//! `(font_handle, size)`, so repeated labels with the same font+size avoid
//! redundant atlas rebuilds.

use std::os::raw::c_char;

use crate::core::debugger;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::GoudContextId;
use crate::ffi::window::with_window_state;
use crate::rendering::text::TextLayoutConfig;

use super::draw_impl::draw_text_internal;
use super::parse::{parse_text_alignment, parse_text_direction, read_utf8_cstr};

// ============================================================================
// FfiTextCmd -- the struct callers fill in from C# / Python / TypeScript
// ============================================================================

/// A single text command for batch rendering.
///
/// All position values are in screen-space pixels.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiTextCmd {
    /// Font handle from `goud_font_load`.
    pub font_handle: u64,
    /// Null-terminated UTF-8 text string.
    pub text: *const c_char,
    /// X position in screen-space pixels.
    pub x: f32,
    /// Y position in screen-space pixels.
    pub y: f32,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text alignment: 0=Left, 1=Center, 2=Right.
    pub alignment: u8,
    /// Text direction: 0=Auto, 1=LTR, 2=RTL.
    pub direction: u8,
    /// Alignment padding.
    pub _pad0: u16,
    /// Maximum line width (0 = no wrap).
    pub max_width: f32,
    /// Line spacing multiplier (default 1.0).
    pub line_spacing: f32,
    /// Red color component.
    pub r: f32,
    /// Green color component.
    pub g: f32,
    /// Blue color component.
    pub b: f32,
    /// Alpha (opacity, 1.0 = fully opaque).
    pub a: f32,
}

// ============================================================================
// Public FFI entry point
// ============================================================================

/// Draws a batch of text labels. Each command specifies font, position, color,
/// and layout parameters. Returns the number of successfully drawn items
/// (0 on error).
///
/// # Safety
///
/// `cmds` must point to `count` valid `FfiTextCmd` values for the call duration.
/// Each `FfiTextCmd::text` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_draw_text_batch(
    context_id: GoudContextId,
    cmds: *const FfiTextCmd,
    count: u32,
) -> u32 {
    // --- null / zero guards ---------------------------------------------------
    if cmds.is_null() {
        set_last_error(GoudError::InvalidState("cmds pointer is null".into()));
        return 0;
    }
    if count == 0 {
        return 0;
    }

    // SAFETY: caller guarantees `cmds` points to `count` valid FfiTextCmds.
    let cmds_slice = std::slice::from_raw_parts(cmds, count as usize);

    let mut drawn = 0u32;

    for cmd in cmds_slice {
        // Skip commands with null text pointers
        if cmd.text.is_null() {
            continue;
        }

        // Parse text string
        let text_str = match read_utf8_cstr(cmd.text) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if text_str.is_empty() {
            drawn += 1;
            continue;
        }

        // Validate font_size and line_spacing
        if cmd.font_size <= 0.0 || cmd.line_spacing <= 0.0 {
            continue;
        }

        // Parse alignment and direction
        let text_alignment = match parse_text_alignment(cmd.alignment) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let text_direction = match parse_text_direction(cmd.direction) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let config = TextLayoutConfig {
            max_width: if cmd.max_width > 0.0 {
                Some(cmd.max_width)
            } else {
                None
            },
            line_spacing: cmd.line_spacing,
            alignment: text_alignment,
        };

        let draw_result = with_window_state(context_id, |window_state| {
            draw_text_internal(
                window_state,
                context_id,
                cmd.font_handle,
                &text_str,
                cmd.x,
                cmd.y,
                cmd.font_size,
                config,
                text_direction,
                cmd.r,
                cmd.g,
                cmd.b,
                cmd.a,
            )
        });

        match draw_result {
            Some(Ok(())) => {
                drawn += 1;
            }
            _ => continue,
        }
    }

    if drawn > 0 {
        let _ = debugger::update_render_stats_for_context(context_id, drawn, drawn * 2, drawn, 1);
    }

    drawn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_text_cmd_size_and_alignment() {
        // u64 (8) + ptr (8) + 5×f32 (20) + u8+u8+u16 (4) + 6×f32 (24) = 64
        // On 64-bit: pointer is 8 bytes, so layout is:
        // font_handle: 8, text: 8, x/y/font_size: 12, alignment+direction+pad: 4,
        // max_width+line_spacing: 8, r/g/b/a: 16 = 56
        assert_eq!(
            std::mem::size_of::<FfiTextCmd>(),
            56,
            "FfiTextCmd should be 56 bytes"
        );
        assert_eq!(
            std::mem::align_of::<FfiTextCmd>(),
            8,
            "FfiTextCmd should be 8-byte aligned"
        );
    }

    #[test]
    fn test_batch_null_cmds_returns_zero() {
        // SAFETY: passing null pointer deliberately to test the guard
        let result = unsafe {
            goud_renderer_draw_text_batch(GoudContextId::from_raw(0), std::ptr::null(), 10)
        };
        assert_eq!(result, 0);
    }

    #[test]
    fn test_batch_zero_count_returns_zero() {
        let cmd = FfiTextCmd {
            font_handle: 0,
            text: std::ptr::null(),
            x: 0.0,
            y: 0.0,
            font_size: 16.0,
            alignment: 0,
            direction: 0,
            _pad0: 0,
            max_width: 0.0,
            line_spacing: 1.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        // SAFETY: valid pointer, count=0
        let result = unsafe {
            goud_renderer_draw_text_batch(GoudContextId::from_raw(0), &cmd as *const FfiTextCmd, 0)
        };
        assert_eq!(result, 0);
    }
}
