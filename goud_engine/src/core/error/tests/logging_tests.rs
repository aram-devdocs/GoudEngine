//! Tests for error logging integration.

use crate::core::error::recovery::{recovery_class, RecoveryClass};
use crate::core::error::*;
use crate::core::error::logging::format_log_message;

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

/// Verifies that format_log_message produces correct output for every error
/// category, covering the full range of error types.
#[test]
fn test_format_log_message_covers_all_categories() {
    let test_cases: Vec<(GoudError, &str, &str)> = vec![
        // (error, expected_code_prefix, expected_content_substring)
        // Context errors (1-99)
        (GoudError::NotInitialized, "[GOUD-1]", "initialized"),
        (GoudError::InvalidContext, "[GOUD-3]", "context"),
        (GoudError::ContextDestroyed, "[GOUD-4]", "destroyed"),
        // Resource errors (100-199)
        (
            GoudError::ResourceNotFound("tex.png".into()),
            "[GOUD-100]",
            "tex.png",
        ),
        (
            GoudError::ResourceLoadFailed("bad.obj".into()),
            "[GOUD-101]",
            "bad.obj",
        ),
        // Graphics errors (200-299)
        (
            GoudError::ShaderCompilationFailed("syntax".into()),
            "[GOUD-200]",
            "syntax",
        ),
        // Entity errors (300-399)
        (GoudError::EntityNotFound, "[GOUD-300]", "Entity"),
        (GoudError::ComponentNotFound, "[GOUD-310]", "Component"),
        // Input errors (400-499)
        (GoudError::InputDeviceNotFound, "[GOUD-400]", "Input"),
        // System errors (500-599)
        (
            GoudError::AudioInitFailed("no device".into()),
            "[GOUD-510]",
            "no device",
        ),
        // Internal errors (900-999)
        (
            GoudError::InternalError("panic".into()),
            "[GOUD-900]",
            "panic",
        ),
    ];

    for (error, expected_code, expected_content) in &test_cases {
        let code = error.error_code();

        // Test with context
        let ctx = GoudErrorContext::new("test_subsystem", "test_op");
        let msg = format_log_message(error, code, Some(&ctx));
        assert!(
            msg.contains(expected_code),
            "expected {} in message for {:?}, got: {}",
            expected_code,
            error,
            msg
        );
        if cfg!(debug_assertions) {
            assert!(
                msg.contains("test_subsystem/test_op"),
                "expected context in debug format for {:?}, got: {}",
                error,
                msg
            );
            assert!(
                msg.contains(expected_content),
                "expected '{}' in debug format for {:?}, got: {}",
                expected_content,
                error,
                msg
            );
        }

        // Test without context
        let msg_no_ctx = format_log_message(error, code, None);
        assert!(
            msg_no_ctx.contains(expected_code),
            "expected {} in no-context message for {:?}, got: {}",
            expected_code,
            error,
            msg_no_ctx
        );
        if cfg!(debug_assertions) {
            assert!(
                msg_no_ctx.contains("unknown/unknown"),
                "expected unknown/unknown for {:?}, got: {}",
                error,
                msg_no_ctx
            );
        }
    }
}

/// Verifies correct severity mapping: Fatal -> error!, Degraded/Recoverable -> warn!
/// by checking that recovery_class returns the expected class for representative errors.
#[test]
fn test_severity_mapping_all_categories() {
    // Fatal: context/state errors
    assert_eq!(recovery_class(1), RecoveryClass::Fatal); // NOT_INITIALIZED
    assert_eq!(recovery_class(3), RecoveryClass::Fatal); // INVALID_CONTEXT
    assert_eq!(recovery_class(900), RecoveryClass::Fatal); // INTERNAL_ERROR

    // Degraded: subsystem init failures
    assert_eq!(recovery_class(230), RecoveryClass::Degraded); // BACKEND_NOT_SUPPORTED
    assert_eq!(recovery_class(510), RecoveryClass::Degraded); // AUDIO_INIT_FAILED

    // Recoverable: resource/entity/component errors
    assert_eq!(recovery_class(100), RecoveryClass::Recoverable); // RESOURCE_NOT_FOUND
    assert_eq!(recovery_class(300), RecoveryClass::Recoverable); // ENTITY_NOT_FOUND
    assert_eq!(recovery_class(310), RecoveryClass::Recoverable); // COMPONENT_NOT_FOUND
    assert_eq!(recovery_class(200), RecoveryClass::Recoverable); // SHADER_COMPILATION_FAILED
}
