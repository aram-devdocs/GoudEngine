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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::{set_last_error, GoudError};

    #[test]
    fn test_goud_last_error_message_success_path() {
        set_last_error(GoudError::NotInitialized);

        let mut buf = [0u8; 256];
        let result =
            // SAFETY: buf is a valid buffer of 256 bytes
            unsafe { goud_last_error_message(buf.as_mut_ptr(), buf.len()) };

        assert!(result > 0, "expected positive byte count, got {result}");
        let msg = std::str::from_utf8(&buf[..result as usize]).unwrap();
        assert!(
            msg.contains("not been initialized"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn test_goud_last_error_message_buffer_too_small() {
        let long_msg = "A".repeat(100);
        set_last_error(GoudError::InternalError(long_msg.clone()));

        let mut buf = [0u8; 10]; // too small for 100-char message
        let result =
            // SAFETY: buf is a valid buffer of 10 bytes
            unsafe { goud_last_error_message(buf.as_mut_ptr(), buf.len()) };

        // The implementation truncates and returns copy_len (buf_len - 1 = 9)
        assert_eq!(
            result, 9,
            "expected truncated copy of 9 bytes, got {result}"
        );
        assert_eq!(buf[9], 0, "expected null terminator at position 9");
    }

    #[test]
    fn test_goud_last_error_message_null_buffer() {
        set_last_error(GoudError::NotInitialized);

        let result =
            // SAFETY: passing null is explicitly handled by the function
            unsafe { goud_last_error_message(std::ptr::null_mut(), 0) };

        // Null buffer returns negative required size (-(len + 1))
        assert!(result < 0, "expected negative required size, got {result}");
    }

    #[test]
    fn test_goud_last_error_message_no_error() {
        clear_last_error();

        let mut buf = [0u8; 256];
        let result =
            // SAFETY: buf is a valid buffer of 256 bytes
            unsafe { goud_last_error_message(buf.as_mut_ptr(), buf.len()) };

        assert_eq!(result, 0, "expected 0 when no error is set, got {result}");
    }

    #[test]
    fn test_goud_last_error_message_null_buffer_no_error() {
        clear_last_error();

        let result =
            // SAFETY: passing null is explicitly handled by the function
            unsafe { goud_last_error_message(std::ptr::null_mut(), 0) };

        assert_eq!(
            result, 0,
            "expected 0 for null buffer with no error, got {result}"
        );
    }
}
