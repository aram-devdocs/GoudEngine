//! FFI error bridge infrastructure.
//!
//! Provides thread-local error storage and the [`GoudFFIResult`] type for passing
//! errors across the FFI boundary to C#, Python, and other language bindings.
//!
//! # Usage Pattern
//!
//! 1. Rust function encounters an error
//! 2. Rust function calls `set_last_error(error)`
//! 3. Rust function returns error code via `GoudFFIResult`
//! 4. Language binding checks if `success` is false
//! 5. Language binding calls `goud_last_error_code()` and `goud_last_error_message()`
//! 6. Language binding calls `take_last_error()` to clear the error
//!
//! # Thread Safety
//!
//! Each thread has its own error storage. Errors do not cross thread boundaries.
//! This matches the behavior of `errno` in C and is safe for multi-threaded use.

use std::cell::RefCell;

use super::codes::{GoudErrorCode, SUCCESS};
use super::context::GoudErrorContext;
use super::types::GoudError;

thread_local! {
    /// Thread-local storage for the last error.
    ///
    /// Each thread has its own error storage, ensuring that errors from one
    /// thread do not affect another. This is critical for thread-safe FFI.
    static LAST_ERROR: RefCell<Option<GoudError>> = const { RefCell::new(None) };

    /// Thread-local storage for error context (subsystem + operation metadata).
    ///
    /// Set alongside LAST_ERROR when `set_last_error_with_context` is used.
    /// Cleared by `set_last_error`, `clear_last_error`, and `take_last_error`.
    static LAST_ERROR_CONTEXT: RefCell<Option<GoudErrorContext>> = const { RefCell::new(None) };
}

/// Sets the last error for the current thread.
///
/// This function stores the error in thread-local storage where it can be
/// retrieved by `last_error_code()` and `last_error_message()`.
///
/// # Thread Safety
///
/// The error is stored in thread-local storage and will not affect other threads.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, last_error_code, ERR_NOT_INITIALIZED};
///
/// set_last_error(GoudError::NotInitialized);
/// assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);
/// ```
pub fn set_last_error(error: GoudError) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
    LAST_ERROR_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
}

/// Sets the last error and its context for the current thread.
///
/// This function stores both the error and context metadata (subsystem, operation)
/// in thread-local storage. Use `last_error_subsystem()` and `last_error_operation()`
/// to retrieve the context.
///
/// # Thread Safety
///
/// Both values are stored in thread-local storage and will not affect other threads.
pub fn set_last_error_with_context(error: GoudError, ctx: GoudErrorContext) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
    LAST_ERROR_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(ctx);
    });
}

/// Takes the last error from the current thread, clearing it.
///
/// This function removes the error from thread-local storage and returns it.
/// Subsequent calls will return `None` until a new error is set.
///
/// # Thread Safety
///
/// Only affects the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, take_last_error};
///
/// set_last_error(GoudError::NotInitialized);
/// let error = take_last_error();
/// assert!(error.is_some());
/// assert!(take_last_error().is_none()); // Cleared after take
/// ```
pub fn take_last_error() -> Option<GoudError> {
    LAST_ERROR_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
    LAST_ERROR.with(|e| e.borrow_mut().take())
}

/// Gets the last error from the current thread without clearing it.
///
/// This function clones the error from thread-local storage. Use `take_last_error()`
/// if you want to clear the error after retrieval.
///
/// # Thread Safety
///
/// Only accesses the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, get_last_error};
///
/// set_last_error(GoudError::NotInitialized);
/// let error1 = get_last_error();
/// let error2 = get_last_error();
/// assert_eq!(error1, error2); // Error not cleared
/// ```
pub fn get_last_error() -> Option<GoudError> {
    LAST_ERROR.with(|e| e.borrow().clone())
}

/// Returns the error code of the last error for the current thread.
///
/// Returns `SUCCESS` (0) if no error is set. This does not clear the error.
///
/// # Thread Safety
///
/// Only accesses the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{set_last_error, last_error_code, clear_last_error, GoudError, SUCCESS, ERR_NOT_INITIALIZED};
///
/// clear_last_error();
/// assert_eq!(last_error_code(), SUCCESS);
///
/// set_last_error(GoudError::NotInitialized);
/// assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);
/// ```
pub fn last_error_code() -> GoudErrorCode {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|err| err.error_code())
            .unwrap_or(SUCCESS)
    })
}

/// Returns the error message of the last error for the current thread.
///
/// Returns `None` if no error is set. This does not clear the error.
/// The returned string is a copy, safe to use across FFI.
///
/// # Thread Safety
///
/// Only accesses the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, last_error_message};
///
/// set_last_error(GoudError::InitializationFailed("GPU not found".to_string()));
/// let msg = last_error_message();
/// assert_eq!(msg, Some("GPU not found".to_string()));
/// ```
pub fn last_error_message() -> Option<String> {
    LAST_ERROR.with(|e| e.borrow().as_ref().map(|err| err.message().to_string()))
}

/// Clears the last error for the current thread.
///
/// After calling this, `last_error_code()` will return `SUCCESS` and
/// `last_error_message()` will return `None`.
///
/// # Thread Safety
///
/// Only affects the current thread's error storage.
///
/// # Example
///
/// ```
/// use goud_engine::core::error::{GoudError, set_last_error, clear_last_error, last_error_code, SUCCESS};
///
/// set_last_error(GoudError::NotInitialized);
/// clear_last_error();
/// assert_eq!(last_error_code(), SUCCESS);
/// ```
pub fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
    LAST_ERROR_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
}

/// Returns the subsystem from the last error's context, if set.
///
/// Returns `None` if no error context has been set or if the error was
/// set without context via `set_last_error()`.
pub fn last_error_subsystem() -> Option<&'static str> {
    LAST_ERROR_CONTEXT.with(|c| c.borrow().as_ref().map(|ctx| ctx.subsystem))
}

/// Returns the operation from the last error's context, if set.
///
/// Returns `None` if no error context has been set or if the error was
/// set without context via `set_last_error()`.
pub fn last_error_operation() -> Option<&'static str> {
    LAST_ERROR_CONTEXT.with(|c| c.borrow().as_ref().map(|ctx| ctx.operation))
}

// =============================================================================
// FFI Result Type
// =============================================================================

/// FFI-safe result type for returning success/failure status across the FFI boundary.
///
/// This struct is designed to be passed by value across FFI and provides both
/// a boolean success flag and the error code for detailed error handling.
///
/// # Memory Layout
///
/// Uses `#[repr(C)]` for predictable memory layout across language boundaries.
/// The struct is 8 bytes (4 bytes for code, 4 bytes for success with padding).
///
/// # Usage
///
/// ```
/// use goud_engine::core::error::{GoudFFIResult, GoudError, SUCCESS};
///
/// // Success case
/// let result = GoudFFIResult::success();
/// assert!(result.success);
/// assert_eq!(result.code, SUCCESS);
///
/// // Error case
/// let result = GoudFFIResult::from_error(GoudError::NotInitialized);
/// assert!(!result.success);
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoudFFIResult {
    /// The error code. `SUCCESS` (0) on success, error code on failure.
    pub code: GoudErrorCode,
    /// True if the operation succeeded, false otherwise.
    pub success: bool,
}

impl GoudFFIResult {
    /// Creates a successful result.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, SUCCESS};
    ///
    /// let result = GoudFFIResult::success();
    /// assert!(result.success);
    /// assert_eq!(result.code, SUCCESS);
    /// ```
    #[inline]
    pub const fn success() -> Self {
        Self {
            code: SUCCESS,
            success: true,
        }
    }

    /// Creates a result from an error code.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, ERR_NOT_INITIALIZED};
    ///
    /// let result = GoudFFIResult::from_code(ERR_NOT_INITIALIZED);
    /// assert!(!result.success);
    /// assert_eq!(result.code, ERR_NOT_INITIALIZED);
    /// ```
    #[inline]
    pub const fn from_code(code: GoudErrorCode) -> Self {
        Self {
            code,
            success: code == SUCCESS,
        }
    }

    /// Creates a result from a `GoudError`.
    ///
    /// This also sets the thread-local last error for message retrieval.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, GoudError, ERR_NOT_INITIALIZED, last_error_message};
    ///
    /// let result = GoudFFIResult::from_error(GoudError::NotInitialized);
    /// assert!(!result.success);
    /// assert_eq!(result.code, ERR_NOT_INITIALIZED);
    /// ```
    #[inline]
    pub fn from_error(error: GoudError) -> Self {
        let code = error.error_code();
        set_last_error(error);
        Self {
            code,
            success: false,
        }
    }

    /// Creates a result from a `GoudResult<T>`.
    ///
    /// On success, clears any previous error. On error, sets the last error.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::{GoudFFIResult, GoudResult, GoudError, SUCCESS, ERR_NOT_INITIALIZED};
    ///
    /// let ok_result: GoudResult<i32> = Ok(42);
    /// let ffi_result = GoudFFIResult::from_result(ok_result);
    /// assert!(ffi_result.success);
    ///
    /// let err_result: GoudResult<i32> = Err(GoudError::NotInitialized);
    /// let ffi_result = GoudFFIResult::from_result(err_result);
    /// assert!(!ffi_result.success);
    /// assert_eq!(ffi_result.code, ERR_NOT_INITIALIZED);
    /// ```
    #[inline]
    pub fn from_result<T>(result: super::GoudResult<T>) -> Self {
        match result {
            Ok(_) => {
                clear_last_error();
                Self::success()
            }
            Err(error) => Self::from_error(error),
        }
    }

    /// Returns true if the result indicates success.
    #[inline]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the result indicates failure.
    #[inline]
    pub const fn is_error(&self) -> bool {
        !self.success
    }

    // =========================================================================
    // Compatibility aliases (used by GoudResult type alias and proc macros)
    // =========================================================================

    /// Alias for `success()` — creates a success result.
    #[inline]
    pub const fn ok() -> Self {
        Self::success()
    }

    /// Alias for `from_code()` — creates an error result from a code.
    #[inline]
    pub const fn err(code: GoudErrorCode) -> Self {
        Self::from_code(code)
    }

    /// Alias for `is_success()` — returns true if the result is ok.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        self.success
    }

    /// Alias for `is_error()` — returns true if the result is an error.
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.success
    }
}

impl Default for GoudFFIResult {
    /// Default is success.
    fn default() -> Self {
        Self::success()
    }
}

impl From<GoudError> for GoudFFIResult {
    fn from(error: GoudError) -> Self {
        Self::from_error(error)
    }
}

impl From<GoudErrorCode> for GoudFFIResult {
    fn from(code: GoudErrorCode) -> Self {
        if code == SUCCESS {
            Self::success()
        } else {
            Self::from_code(code)
        }
    }
}

impl<T> From<super::GoudResult<T>> for GoudFFIResult {
    fn from(result: super::GoudResult<T>) -> Self {
        Self::from_result(result)
    }
}

impl std::fmt::Display for GoudFFIResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "GoudResult::Success")
        } else {
            write!(f, "GoudResult::Error(code={})", self.code)
        }
    }
}
