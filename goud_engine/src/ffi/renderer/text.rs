//! # Renderer Text FFI
//!
//! Font lifecycle and immediate-mode text rendering APIs.

use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_char;

use crate::core::error::{set_last_error, GoudError};
use crate::core::handle::{Handle, HandleMap};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::backend::TextureOps;
use crate::rendering::text::{GlyphAtlas, TextLayoutConfig};

mod draw_impl;
mod parse;

#[cfg(test)]
mod tests;

use draw_impl::draw_text_internal;
use parse::{parse_text_alignment, parse_text_direction, read_utf8_cstr};

/// Opaque font handle for native FFI text rendering.
pub type GoudFontHandle = u64;

/// Invalid font handle constant.
pub const GOUD_INVALID_FONT: GoudFontHandle = u64::MAX;

struct FontMarker;

struct LoadedFont {
    font: fontdue::Font,
    font_bytes: Vec<u8>,
    atlases: HashMap<u32, GlyphAtlas>,
}

struct ContextFontState {
    fonts: HandleMap<FontMarker, LoadedFont>,
}

impl ContextFontState {
    fn new() -> Self {
        Self {
            fonts: HandleMap::new(),
        }
    }
}

thread_local! {
    static FONT_STATES: RefCell<HashMap<(u32, u32), ContextFontState>> = RefCell::new(HashMap::new());
}

/// Loads a font from disk and returns an opaque handle.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_font_load(
    context_id: GoudContextId,
    path: *const c_char,
) -> GoudFontHandle {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_FONT;
    }

    if path.is_null() {
        set_last_error(GoudError::InvalidState("path pointer is null".to_string()));
        return GOUD_INVALID_FONT;
    }

    if with_window_state(context_id, |_| ()).is_none() {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_FONT;
    }

    let path_str = match read_utf8_cstr(path) {
        Ok(value) => value,
        Err(err) => {
            set_last_error(err);
            return GOUD_INVALID_FONT;
        }
    };

    let font_bytes = match std::fs::read(&path_str) {
        Ok(bytes) => bytes,
        Err(err) => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "failed to read font '{}': {}",
                path_str, err
            )));
            return GOUD_INVALID_FONT;
        }
    };

    let parsed_font =
        match fontdue::Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default()) {
            Ok(font) => font,
            Err(err) => {
                set_last_error(GoudError::ResourceInvalidFormat(format!(
                    "failed to parse font '{}': {}",
                    path_str, err
                )));
                return GOUD_INVALID_FONT;
            }
        };

    let context_key = (context_id.index(), context_id.generation());
    FONT_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let state = states
            .entry(context_key)
            .or_insert_with(ContextFontState::new);
        let handle = state.fonts.insert(LoadedFont {
            font: parsed_font,
            font_bytes,
            atlases: HashMap::new(),
        });
        handle.to_u64()
    })
}

/// Destroys a loaded font and frees associated GPU atlases.
#[no_mangle]
pub extern "C" fn goud_font_destroy(
    context_id: GoudContextId,
    font_handle: GoudFontHandle,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if font_handle == GOUD_INVALID_FONT {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    let handle = Handle::<FontMarker>::from_u64(font_handle);
    if !handle.is_valid() {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    let context_key = (context_id.index(), context_id.generation());
    let result = with_window_state(context_id, |window_state| {
        FONT_STATES.with(|cell| {
            let mut states = cell.borrow_mut();
            let Some(state) = states.get_mut(&context_key) else {
                set_last_error(GoudError::InvalidHandle);
                return false;
            };

            let Some(mut loaded_font) = state.fonts.remove(handle) else {
                set_last_error(GoudError::InvalidHandle);
                return false;
            };

            for atlas in loaded_font.atlases.values_mut() {
                if let Some(texture) = atlas.take_gpu_texture() {
                    window_state.backend_mut().destroy_texture(texture);
                }
            }

            if state.fonts.is_empty() {
                states.remove(&context_key);
            }

            true
        })
    });

    match result {
        Some(ok) => ok,
        None => {
            set_last_error(GoudError::InvalidContext);
            false
        }
    }
}

/// Draws UTF-8 text in immediate mode.
///
/// `alignment`: 0 = Left, 1 = Center, 2 = Right
/// `direction`: 0 = Auto, 1 = LTR, 2 = RTL
///
/// # Safety
///
/// `text` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer_draw_text(
    context_id: GoudContextId,
    font_handle: GoudFontHandle,
    text: *const c_char,
    x: f32,
    y: f32,
    font_size: f32,
    alignment: u8,
    max_width: f32,
    line_spacing: f32,
    direction: u8,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if font_handle == GOUD_INVALID_FONT {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    if text.is_null() {
        set_last_error(GoudError::InvalidState("text pointer is null".to_string()));
        return false;
    }

    if font_size <= 0.0 {
        set_last_error(GoudError::InvalidState(
            "font_size must be greater than 0".to_string(),
        ));
        return false;
    }

    if line_spacing <= 0.0 {
        set_last_error(GoudError::InvalidState(
            "line_spacing must be greater than 0".to_string(),
        ));
        return false;
    }

    let text_str = match read_utf8_cstr(text) {
        Ok(value) => value,
        Err(err) => {
            set_last_error(err);
            return false;
        }
    };

    if text_str.is_empty() {
        return true;
    }

    let text_alignment = match parse_text_alignment(alignment) {
        Ok(value) => value,
        Err(err) => {
            set_last_error(err);
            return false;
        }
    };

    let text_direction = match parse_text_direction(direction) {
        Ok(value) => value,
        Err(err) => {
            set_last_error(err);
            return false;
        }
    };

    let config = TextLayoutConfig {
        max_width: if max_width > 0.0 {
            Some(max_width)
        } else {
            None
        },
        line_spacing,
        alignment: text_alignment,
    };

    let draw_result = with_window_state(context_id, |window_state| {
        draw_text_internal(
            window_state,
            context_id,
            font_handle,
            &text_str,
            x,
            y,
            font_size,
            config,
            text_direction,
            r,
            g,
            b,
            a,
        )
    });

    match draw_result {
        Some(Ok(())) => true,
        Some(Err(err)) => {
            set_last_error(err);
            false
        }
        None => {
            set_last_error(GoudError::InvalidContext);
            false
        }
    }
}

/// Alias export for SDK compatibility.
///
/// # Safety
///
/// `text` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn goud_draw_text(
    context_id: GoudContextId,
    font_handle: GoudFontHandle,
    text: *const c_char,
    x: f32,
    y: f32,
    font_size: f32,
    alignment: u8,
    max_width: f32,
    line_spacing: f32,
    direction: u8,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> bool {
    // SAFETY: this alias forwards the same ABI contract to goud_renderer_draw_text.
    unsafe {
        goud_renderer_draw_text(
            context_id,
            font_handle,
            text,
            x,
            y,
            font_size,
            alignment,
            max_width,
            line_spacing,
            direction,
            r,
            g,
            b,
            a,
        )
    }
}

/// Releases all font state for a destroyed context.
pub(crate) fn cleanup_text_state(context_id: GoudContextId) {
    let context_key = (context_id.index(), context_id.generation());
    let _ = with_window_state(context_id, |window_state| {
        FONT_STATES.with(|cell| {
            let mut states = cell.borrow_mut();
            if let Some(mut state) = states.remove(&context_key) {
                for loaded_font in state.fonts.values_mut() {
                    for atlas in loaded_font.atlases.values_mut() {
                        if let Some(texture) = atlas.take_gpu_texture() {
                            window_state.backend_mut().destroy_texture(texture);
                        }
                    }
                }
            }
        });
    });
}
