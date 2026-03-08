//! Tests for error logging integration.

use crate::core::error::recovery::{recovery_class, RecoveryClass};
use crate::core::error::*;

#[test]
fn test_fatal_errors_map_to_fatal_class() {
    let fatal_errors = [
        GoudError::NotInitialized,
        GoudError::InvalidContext,
        GoudError::ContextDestroyed,
        GoudError::InvalidState("bad state".to_string()),
        GoudError::InternalError("bug".to_string()),
    ];
    for error in &fatal_errors {
        assert_eq!(
            recovery_class(error.error_code()),
            RecoveryClass::Fatal,
            "expected Fatal for {:?}",
            error
        );
    }
}

#[test]
fn test_degraded_errors_map_to_degraded_class() {
    let degraded_errors = [
        GoudError::BackendNotSupported("no vulkan".to_string()),
        GoudError::AudioInitFailed("no device".to_string()),
        GoudError::PhysicsInitFailed("bad config".to_string()),
    ];
    for error in &degraded_errors {
        assert_eq!(
            recovery_class(error.error_code()),
            RecoveryClass::Degraded,
            "expected Degraded for {:?}",
            error
        );
    }
}

#[test]
fn test_recoverable_errors_map_to_recoverable_class() {
    let recoverable_errors = [
        GoudError::ResourceNotFound("file.png".to_string()),
        GoudError::EntityNotFound,
        GoudError::ComponentNotFound,
    ];
    for error in &recoverable_errors {
        assert_eq!(
            recovery_class(error.error_code()),
            RecoveryClass::Recoverable,
            "expected Recoverable for {:?}",
            error
        );
    }
}

#[test]
fn test_init_logger_is_idempotent() {
    // Calling init_logger multiple times must not panic.
    init_logger();
    init_logger();
    init_logger();
}

#[test]
fn test_log_error_with_context_does_not_panic() {
    init_logger();
    let error = GoudError::ShaderCompilationFailed("syntax error".to_string());
    let ctx = GoudErrorContext::new("graphics", "shader_compile");
    // Should not panic regardless of recovery class.
    crate::core::error::logging::log_error(&error, Some(&ctx));
}

#[test]
fn test_log_error_without_context_does_not_panic() {
    init_logger();
    let error = GoudError::NotInitialized;
    crate::core::error::logging::log_error(&error, None);
}
