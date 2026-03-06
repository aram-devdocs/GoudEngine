//! FFI exports for error query functions.
//!
//! These functions allow language bindings to retrieve error details
//! from the thread-local error storage after an FFI call fails.

use crate::core::error::{clear_last_error, last_error_code, last_error_message, GoudErrorCode};

/// Returns the error code of the last error on the current thread.
/// Returns 0 (SUCCESS) if no error is set.
#[no_mangle]
pub extern "C" fn goud_last_error_code() -> GoudErrorCode {
    last_error_code()
}

/// Writes the last error message into a caller-provided buffer.
/// Returns the number of bytes written (excluding null terminator).
/// If the buffer is too small, returns the required size as a negative number.
/// Returns 0 if no error message is set.
///
/// # Safety
/// - `buf` must point to a valid buffer of at least `buf_len` bytes, or be null
#[no_mangle]
pub unsafe extern "C" fn goud_last_error_message(buf: *mut u8, buf_len: usize) -> i32 {
    if buf.is_null() || buf_len == 0 {
        return match last_error_message() {
            Some(msg) => -(msg.len() as i32 + 1),
            None => 0,
        };
    }

    match last_error_message() {
        Some(msg) => {
            let bytes = msg.as_bytes();
            let copy_len = bytes.len().min(buf_len - 1);
            // SAFETY: buf is valid for buf_len bytes per contract, copy_len < buf_len
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
            *buf.add(copy_len) = 0; // null terminator
            copy_len as i32
        }
        None => 0,
    }
}

/// Clears the last error on the current thread.
#[no_mangle]
pub extern "C" fn goud_clear_last_error() {
    clear_last_error();
}
