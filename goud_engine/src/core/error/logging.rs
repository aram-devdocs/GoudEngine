//! Error logging integration for GoudEngine.
//!
//! Provides automatic logging of errors as they occur, with log levels
//! determined by the error's [`RecoveryClass`]. Debug builds include full
//! context (subsystem, operation, message); release builds log a summary.

use std::sync::Once;

use super::codes::{error_category, GoudErrorCode};
use super::context::GoudErrorContext;
use super::recovery::{recovery_class, RecoveryClass};
use super::types::GoudError;

/// Guard ensuring the logger is initialized at most once.
static INIT_ONCE: Once = Once::new();

/// Initializes the global logger (env_logger) if not already initialized.
///
/// On native builds, calls `env_logger::try_init()` behind a `Once` guard
/// so multiple calls are safe. On non-native builds this is a no-op.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::error::init_logger;
/// init_logger(); // safe to call multiple times
/// ```
pub fn init_logger() {
    INIT_ONCE.call_once(|| {
        #[cfg(feature = "native")]
        {
            let _ = env_logger::try_init();
        }
    });
}

/// Logs an error with a severity level determined by its [`RecoveryClass`].
///
/// - [`RecoveryClass::Fatal`] errors are logged at `error!` level.
/// - [`RecoveryClass::Degraded`] and [`RecoveryClass::Recoverable`] errors
///   are logged at `warn!` level.
///
/// In debug builds the log message includes the full format:
/// `[GOUD-{code}] {subsystem}/{operation}: {message}`.
///
/// In release builds only a summary is logged:
/// `[GOUD-{code}] {category}`.
pub fn log_error(error: &GoudError, context: Option<&GoudErrorContext>) {
    let code: GoudErrorCode = error.error_code();
    let class = recovery_class(code);

    let formatted = format_log_message(error, code, context);

    match class {
        RecoveryClass::Fatal => log::error!("{}", formatted),
        RecoveryClass::Degraded | RecoveryClass::Recoverable => {
            log::warn!("{}", formatted);
        }
    }
}

/// Formats the log message for an error.
///
/// Debug builds produce `[GOUD-{code}] {subsystem}/{operation}: {message}`.
/// Release builds produce `[GOUD-{code}] {category}`.
pub(crate) fn format_log_message(
    error: &GoudError,
    code: GoudErrorCode,
    context: Option<&GoudErrorContext>,
) -> String {
    if cfg!(debug_assertions) {
        let subsystem = context.map_or("unknown", |c| c.subsystem);
        let operation = context.map_or("unknown", |c| c.operation);
        format!(
            "[GOUD-{}] {}/{}: {}",
            code,
            subsystem,
            operation,
            error.message()
        )
    } else {
        format!("[GOUD-{}] {}", code, error_category(code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_log_message_debug_with_context() {
        let error = GoudError::NotInitialized;
        let ctx = GoudErrorContext::new("graphics", "shader_compile");
        let msg = format_log_message(&error, error.error_code(), Some(&ctx));

        // Debug build: full format
        assert!(msg.contains("[GOUD-1]"));
        assert!(msg.contains("graphics/shader_compile"));
        assert!(msg.contains("Engine has not been initialized"));
    }

    #[test]
    fn test_format_log_message_debug_without_context() {
        let error = GoudError::ResourceNotFound("missing.png".to_string());
        let msg = format_log_message(&error, error.error_code(), None);

        assert!(msg.contains("[GOUD-100]"));
        assert!(msg.contains("unknown/unknown"));
        assert!(msg.contains("missing.png"));
    }

    #[test]
    fn test_log_error_selects_correct_level() {
        init_logger();

        // Fatal -> error! (verified via recovery_class)
        let fatal = GoudError::NotInitialized;
        assert_eq!(recovery_class(fatal.error_code()), RecoveryClass::Fatal);
        log_error(&fatal, None);

        // Degraded -> warn!
        let degraded = GoudError::AudioInitFailed("no device".to_string());
        assert_eq!(
            recovery_class(degraded.error_code()),
            RecoveryClass::Degraded
        );
        log_error(&degraded, None);

        // Recoverable -> warn!
        let recoverable = GoudError::ResourceNotFound("file.png".to_string());
        assert_eq!(
            recovery_class(recoverable.error_code()),
            RecoveryClass::Recoverable
        );
        log_error(&recoverable, None);
    }
}
