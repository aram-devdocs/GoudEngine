use std::ffi::CStr;
use std::os::raw::c_char;

use crate::core::error::GoudError;
use crate::core::types::TextAlignment;
use crate::rendering::text::TextDirection;

pub(super) fn read_utf8_cstr(ptr: *const c_char) -> Result<String, GoudError> {
    // SAFETY: caller guarantees `ptr` is non-null and points to a valid C string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str()
        .map(str::to_string)
        .map_err(|_| GoudError::InvalidState("string is not valid UTF-8".to_string()))
}

pub(super) fn parse_text_alignment(alignment: u8) -> Result<TextAlignment, GoudError> {
    match alignment {
        0 => Ok(TextAlignment::Left),
        1 => Ok(TextAlignment::Center),
        2 => Ok(TextAlignment::Right),
        _ => Err(GoudError::InvalidState(format!(
            "invalid alignment value: {}",
            alignment
        ))),
    }
}

pub(super) fn parse_text_direction(direction: u8) -> Result<TextDirection, GoudError> {
    TextDirection::from_u8(direction)
        .ok_or_else(|| GoudError::InvalidState(format!("invalid direction value: {}", direction)))
}
