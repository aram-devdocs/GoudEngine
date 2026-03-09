use super::*;
use crate::core::error::{
    clear_last_error, last_error_code, ERR_INVALID_HANDLE, ERR_INVALID_STATE,
};
use std::os::raw::c_char;

fn fake_context() -> GoudContextId {
    GoudContextId::new(7, 1)
}

#[test]
fn read_utf8_cstr_parses_valid_utf8() {
    let value = std::ffi::CString::new("hello").unwrap();
    let parsed = read_utf8_cstr(value.as_ptr()).unwrap();
    assert_eq!(parsed, "hello");
}

#[test]
fn read_utf8_cstr_rejects_invalid_utf8() {
    let bytes = [0x66u8, 0x6f, 0x80, 0x00];
    let ptr = bytes.as_ptr().cast::<c_char>();
    let result = read_utf8_cstr(ptr);
    assert!(result.is_err());
}

#[test]
fn font_load_rejects_null_path() {
    clear_last_error();
    // SAFETY: passing a null pointer is explicitly validated by goud_font_load.
    let ok = unsafe { goud_font_load(fake_context(), std::ptr::null()) };
    assert_eq!(ok, GOUD_INVALID_FONT);
    assert_eq!(last_error_code(), ERR_INVALID_STATE);
}

#[test]
fn font_destroy_rejects_invalid_handle() {
    clear_last_error();
    let ok = goud_font_destroy(fake_context(), GOUD_INVALID_FONT);
    assert!(!ok);
    assert_eq!(last_error_code(), ERR_INVALID_HANDLE);
}

#[test]
fn draw_text_rejects_null_pointer_before_gl() {
    clear_last_error();
    // SAFETY: passing a null text pointer is explicitly validated before use.
    let ok = unsafe {
        goud_renderer_draw_text(
            fake_context(),
            1,
            std::ptr::null(),
            0.0,
            0.0,
            16.0,
            0,
            0.0,
            1.0,
            0,
            1.0,
            1.0,
            1.0,
            1.0,
        )
    };
    assert!(!ok);
    assert_eq!(last_error_code(), ERR_INVALID_STATE);
}

#[test]
fn draw_text_rejects_invalid_utf8_before_gl() {
    clear_last_error();
    let bytes = [0x66u8, 0x6f, 0x80, 0x00];
    // SAFETY: bytes are nul-terminated; function validates UTF-8 and reports error.
    let ok = unsafe {
        goud_renderer_draw_text(
            fake_context(),
            1,
            bytes.as_ptr().cast::<c_char>(),
            0.0,
            0.0,
            16.0,
            0,
            0.0,
            1.0,
            0,
            1.0,
            1.0,
            1.0,
            1.0,
        )
    };
    assert!(!ok);
    assert_eq!(last_error_code(), ERR_INVALID_STATE);
}
