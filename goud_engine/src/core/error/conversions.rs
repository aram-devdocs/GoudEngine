//! Standard trait implementations and type conversions for [`GoudError`].
//!
//! Provides `Display`, `Error`, and `From` implementations to integrate
//! `GoudError` with the standard Rust error ecosystem.

use super::types::GoudError;

// =============================================================================
// Standard Trait Implementations
// =============================================================================

impl std::fmt::Display for GoudError {
    /// Formats the error for user-friendly display.
    ///
    /// Format: `"[GOUD-{code}] {category}: {message}"`
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error = GoudError::NotInitialized;
    /// let display = format!("{}", error);
    /// assert!(display.contains("[GOUD-1]"));
    /// assert!(display.contains("Context"));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[GOUD-{}] {}: {}",
            self.error_code(),
            self.category(),
            self.message()
        )
    }
}

impl std::error::Error for GoudError {
    /// Returns the source of this error, if any.
    ///
    /// Currently, `GoudError` does not wrap other errors, so this always returns `None`.
    /// Future versions may add error chaining for wrapped errors.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// =============================================================================
// Conversion Implementations
// =============================================================================

impl From<std::io::Error> for GoudError {
    /// Converts an I/O error into a `GoudError`.
    ///
    /// The I/O error is mapped to an appropriate `GoudError` variant based on its kind:
    /// - `NotFound` -> `ResourceNotFound`
    /// - `PermissionDenied` -> `ResourceLoadFailed`
    /// - Other -> `ResourceLoadFailed` with the error message
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    /// use std::io;
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let goud_error: GoudError = io_error.into();
    /// assert!(matches!(goud_error, GoudError::ResourceNotFound(_)));
    /// ```
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => GoudError::ResourceNotFound(error.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                GoudError::ResourceLoadFailed(format!("Permission denied: {}", error))
            }
            _ => GoudError::ResourceLoadFailed(error.to_string()),
        }
    }
}

impl From<String> for GoudError {
    /// Converts a string into a `GoudError::InternalError`.
    ///
    /// This is a convenience conversion for creating internal errors from strings.
    /// Use more specific error variants when the error category is known.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error: GoudError = "something went wrong".to_string().into();
    /// assert!(matches!(error, GoudError::InternalError(_)));
    /// ```
    fn from(msg: String) -> Self {
        GoudError::InternalError(msg)
    }
}

impl From<&str> for GoudError {
    /// Converts a string slice into a `GoudError::InternalError`.
    ///
    /// This is a convenience conversion for creating internal errors from string literals.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::error::GoudError;
    ///
    /// let error: GoudError = "something went wrong".into();
    /// assert!(matches!(error, GoudError::InternalError(_)));
    /// ```
    fn from(msg: &str) -> Self {
        GoudError::InternalError(msg.to_string())
    }
}
