//! FFI-safe result type for engine operations.

use crate::core::error::GoudErrorCode;

// =============================================================================
// Result Type
// =============================================================================

/// FFI-safe result type for operations that can fail.
///
/// This is returned by FFI functions instead of throwing exceptions.
/// Callers must check the error code and retrieve error details via
/// `goud_get_last_error_message()` if needed.
///
/// # FFI Safety
///
/// - `#[repr(C)]` ensures predictable field layout
/// - Uses primitive types only (i32, bool)
/// - Always 8 bytes on all platforms (i32 + bool + padding)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoudResult {
    /// Error code (0 = success, non-zero = error).
    pub code: i32,

    /// True if operation succeeded (code == 0).
    pub success: bool,
}

impl GoudResult {
    /// Creates a success result.
    pub fn ok() -> Self {
        Self {
            code: 0,
            success: true,
        }
    }

    /// Creates an error result with the given code.
    pub fn err(code: GoudErrorCode) -> Self {
        Self {
            code,
            success: false,
        }
    }

    /// Returns true if this result is success.
    pub fn is_ok(&self) -> bool {
        self.success
    }

    /// Returns true if this result is an error.
    pub fn is_err(&self) -> bool {
        !self.success
    }
}

impl Default for GoudResult {
    fn default() -> Self {
        Self::ok()
    }
}

impl From<GoudErrorCode> for GoudResult {
    fn from(code: GoudErrorCode) -> Self {
        if code == 0 {
            Self::ok()
        } else {
            Self::err(code)
        }
    }
}

impl From<Result<(), crate::core::error::GoudError>> for GoudResult {
    fn from(result: Result<(), crate::core::error::GoudError>) -> Self {
        match result {
            Ok(()) => Self::ok(),
            Err(err) => Self::err(err.error_code()),
        }
    }
}

impl std::fmt::Display for GoudResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "GoudResult::Success")
        } else {
            write!(f, "GoudResult::Error(code={})", self.code)
        }
    }
}
