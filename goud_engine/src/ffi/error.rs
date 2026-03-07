//! FFI exports for error query functions.
//!
//! These functions allow language bindings to retrieve error details
//! from the thread-local error storage after an FFI call fails.

use crate::core::error::{
    clear_last_error, last_error_code, last_error_message, last_error_operation,
    last_error_subsystem, recovery_class, recovery_hint, GoudErrorCode,
};

/// Returns the error code of the last error on the current thread.
/// Returns 0 (SUCCESS) if no error is set.
#[no_mangle]
pub extern "C" fn goud_last_error_code() -> GoudErrorCode {
    last_error_code()
}

/// Writes the last error message into a caller-provided buffer.
///
/// Returns the number of bytes written (excluding null terminator).
/// If the buffer is too small, the message is truncated to fit and the
/// number of bytes actually written is returned. To query the required
/// buffer size without writing, pass a null pointer for `buf` — the
/// return value will be the negative required size (including null
/// terminator), e.g. `-26` means 26 bytes are needed.
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

/// Writes the last error's subsystem into a caller-provided buffer.
///
/// Returns the number of bytes written (excluding null terminator).
/// If the buffer is too small, the value is truncated to fit.
/// To query the required buffer size, pass a null pointer for `buf` --
/// the return value will be the negative required size (including null
/// terminator). Returns 0 if no error context is set.
///
/// # Safety
/// - `buf` must point to a valid buffer of at least `buf_len` bytes, or be null
#[no_mangle]
pub unsafe extern "C" fn goud_last_error_subsystem(buf: *mut u8, buf_len: usize) -> i32 {
    if buf.is_null() || buf_len == 0 {
        return match last_error_subsystem() {
            Some(s) => -(s.len() as i32 + 1),
            None => 0,
        };
    }

    match last_error_subsystem() {
        Some(s) => {
            let bytes = s.as_bytes();
            let copy_len = bytes.len().min(buf_len - 1);
            // SAFETY: buf is valid for buf_len bytes per contract, copy_len < buf_len
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
            *buf.add(copy_len) = 0; // null terminator
            copy_len as i32
        }
        None => 0,
    }
}

/// Writes the last error's operation into a caller-provided buffer.
///
/// Returns the number of bytes written (excluding null terminator).
/// If the buffer is too small, the value is truncated to fit.
/// To query the required buffer size, pass a null pointer for `buf` --
/// the return value will be the negative required size (including null
/// terminator). Returns 0 if no error context is set.
///
/// # Safety
/// - `buf` must point to a valid buffer of at least `buf_len` bytes, or be null
#[no_mangle]
pub unsafe extern "C" fn goud_last_error_operation(buf: *mut u8, buf_len: usize) -> i32 {
    if buf.is_null() || buf_len == 0 {
        return match last_error_operation() {
            Some(s) => -(s.len() as i32 + 1),
            None => 0,
        };
    }

    match last_error_operation() {
        Some(s) => {
            let bytes = s.as_bytes();
            let copy_len = bytes.len().min(buf_len - 1);
            // SAFETY: buf is valid for buf_len bytes per contract, copy_len < buf_len
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
            *buf.add(copy_len) = 0; // null terminator
            copy_len as i32
        }
        None => 0,
    }
}

/// Returns the recovery classification for an error code.
///
/// Returns 0 (Recoverable), 1 (Fatal), or 2 (Degraded).
/// This function is stateless and does not access thread-local storage.
#[no_mangle]
pub extern "C" fn goud_error_recovery_class(code: GoudErrorCode) -> i32 {
    recovery_class(code) as i32
}

/// Writes the recovery hint for an error code into a caller-provided buffer.
///
/// Returns the number of bytes written (excluding null terminator).
/// If the buffer is too small, the hint is truncated to fit.
/// To query the required buffer size, pass a null pointer for `buf` --
/// the return value will be the negative required size (including null
/// terminator). Returns 0 if the code is SUCCESS (empty hint).
///
/// This function is stateless and does not access thread-local storage.
///
/// # Safety
/// - `buf` must point to a valid buffer of at least `buf_len` bytes, or be null
#[no_mangle]
pub unsafe extern "C" fn goud_error_recovery_hint(
    code: GoudErrorCode,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    let hint = recovery_hint(code);
    if hint.is_empty() {
        return 0;
    }

    if buf.is_null() || buf_len == 0 {
        return -(hint.len() as i32 + 1);
    }

    let bytes = hint.as_bytes();
    let copy_len = bytes.len().min(buf_len - 1);
    // SAFETY: buf is valid for buf_len bytes per contract, copy_len < buf_len
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
    *buf.add(copy_len) = 0; // null terminator
    copy_len as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::context::subsystems;
    use crate::core::error::{
        set_last_error, set_last_error_with_context, GoudError, GoudErrorContext,
    };

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
    fn test_goud_last_error_subsystem_with_context() {
        let ctx = GoudErrorContext::new(subsystems::GRAPHICS, "shader_compile");
        set_last_error_with_context(GoudError::ShaderCompilationFailed("test".to_string()), ctx);

        let mut buf = [0u8; 64];
        // SAFETY: buf is a valid buffer of 64 bytes
        let result = unsafe { goud_last_error_subsystem(buf.as_mut_ptr(), buf.len()) };
        assert!(result > 0);
        let s = std::str::from_utf8(&buf[..result as usize]).unwrap();
        assert_eq!(s, "graphics");

        clear_last_error();
    }

    #[test]
    fn test_goud_last_error_operation_with_context() {
        let ctx = GoudErrorContext::new(subsystems::ECS, "entity_spawn");
        set_last_error_with_context(GoudError::EntityNotFound, ctx);

        let mut buf = [0u8; 64];
        // SAFETY: buf is a valid buffer of 64 bytes
        let result = unsafe { goud_last_error_operation(buf.as_mut_ptr(), buf.len()) };
        assert!(result > 0);
        let s = std::str::from_utf8(&buf[..result as usize]).unwrap();
        assert_eq!(s, "entity_spawn");

        clear_last_error();
    }

    #[test]
    fn test_goud_last_error_subsystem_null_buffer_returns_neg_size() {
        let ctx = GoudErrorContext::new(subsystems::AUDIO, "play");
        set_last_error_with_context(GoudError::AudioInitFailed("test".to_string()), ctx);

        // SAFETY: passing null is explicitly handled
        let result = unsafe { goud_last_error_subsystem(std::ptr::null_mut(), 0) };
        // "audio" is 5 bytes, so required = -(5+1) = -6
        assert_eq!(result, -6);

        clear_last_error();
    }

    #[test]
    fn test_goud_last_error_subsystem_no_context() {
        set_last_error(GoudError::NotInitialized);

        let mut buf = [0u8; 64];
        // SAFETY: buf is a valid buffer of 64 bytes
        let result = unsafe { goud_last_error_subsystem(buf.as_mut_ptr(), buf.len()) };
        assert_eq!(result, 0);

        clear_last_error();
    }

    #[test]
    fn test_goud_last_error_operation_no_context() {
        set_last_error(GoudError::NotInitialized);

        // SAFETY: passing null is explicitly handled
        let result = unsafe { goud_last_error_operation(std::ptr::null_mut(), 0) };
        assert_eq!(result, 0);

        clear_last_error();
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

    // =========================================================================
    // Recovery classification FFI tests
    // =========================================================================

    #[test]
    fn test_goud_error_recovery_class_fatal() {
        use crate::core::error::ERR_NOT_INITIALIZED;
        assert_eq!(goud_error_recovery_class(ERR_NOT_INITIALIZED), 1);
    }

    #[test]
    fn test_goud_error_recovery_class_degraded() {
        use crate::core::error::ERR_AUDIO_INIT_FAILED;
        assert_eq!(goud_error_recovery_class(ERR_AUDIO_INIT_FAILED), 2);
    }

    #[test]
    fn test_goud_error_recovery_class_recoverable() {
        use crate::core::error::{ERR_RESOURCE_NOT_FOUND, SUCCESS};
        assert_eq!(goud_error_recovery_class(SUCCESS), 0);
        assert_eq!(goud_error_recovery_class(ERR_RESOURCE_NOT_FOUND), 0);
    }

    #[test]
    fn test_goud_error_recovery_hint_success_returns_zero() {
        use crate::core::error::SUCCESS;
        let mut buf = [0u8; 64];
        // SAFETY: buf is a valid buffer
        let result = unsafe { goud_error_recovery_hint(SUCCESS, buf.as_mut_ptr(), buf.len()) };
        assert_eq!(result, 0);
    }

    #[test]
    fn test_goud_error_recovery_hint_writes_hint() {
        use crate::core::error::ERR_RESOURCE_NOT_FOUND;
        let mut buf = [0u8; 256];
        // SAFETY: buf is a valid buffer
        let result = unsafe {
            goud_error_recovery_hint(ERR_RESOURCE_NOT_FOUND, buf.as_mut_ptr(), buf.len())
        };
        assert!(result > 0);
        let hint = std::str::from_utf8(&buf[..result as usize]).unwrap();
        assert!(hint.contains("file path"));
    }

    #[test]
    fn test_goud_error_recovery_hint_null_buffer() {
        use crate::core::error::ERR_NOT_INITIALIZED;
        // SAFETY: passing null is explicitly handled
        let result =
            unsafe { goud_error_recovery_hint(ERR_NOT_INITIALIZED, std::ptr::null_mut(), 0) };
        assert!(result < 0, "expected negative required size, got {result}");
    }

    #[test]
    fn test_goud_error_recovery_hint_truncates() {
        use crate::core::error::ERR_NOT_INITIALIZED;
        let mut buf = [0u8; 5]; // too small
                                // SAFETY: buf is a valid buffer
        let result =
            unsafe { goud_error_recovery_hint(ERR_NOT_INITIALIZED, buf.as_mut_ptr(), buf.len()) };
        assert_eq!(result, 4); // 5 - 1 for null terminator
        assert_eq!(buf[4], 0); // null terminator
    }
}
