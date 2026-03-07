//! Tests for error recovery classification.

use crate::core::error::recovery::{
    is_fatal, is_recoverable, recovery_class, recovery_hint, RecoveryClass,
};
use crate::core::error::*;

// =============================================================================
// Recovery class tests
// =============================================================================

#[test]
fn test_fatal_codes_return_fatal() {
    let fatal_codes = [
        ERR_NOT_INITIALIZED,
        ERR_INVALID_CONTEXT,
        ERR_CONTEXT_DESTROYED,
        ERR_INVALID_STATE,
        ERR_INTERNAL_ERROR,
    ];
    for code in fatal_codes {
        assert_eq!(
            recovery_class(code),
            RecoveryClass::Fatal,
            "expected Fatal for code {code}"
        );
    }
}

#[test]
fn test_degraded_codes_return_degraded() {
    let degraded_codes = [
        ERR_BACKEND_NOT_SUPPORTED,
        ERR_AUDIO_INIT_FAILED,
        ERR_PHYSICS_INIT_FAILED,
    ];
    for code in degraded_codes {
        assert_eq!(
            recovery_class(code),
            RecoveryClass::Degraded,
            "expected Degraded for code {code}"
        );
    }
}

#[test]
fn test_recoverable_codes_return_recoverable() {
    let recoverable_codes = [
        ERR_ALREADY_INITIALIZED,
        ERR_INITIALIZATION_FAILED,
        ERR_RESOURCE_NOT_FOUND,
        ERR_RESOURCE_LOAD_FAILED,
        ERR_RESOURCE_INVALID_FORMAT,
        ERR_RESOURCE_ALREADY_EXISTS,
        ERR_INVALID_HANDLE,
        ERR_HANDLE_EXPIRED,
        ERR_HANDLE_TYPE_MISMATCH,
        ERR_SHADER_COMPILATION_FAILED,
        ERR_SHADER_LINK_FAILED,
        ERR_TEXTURE_CREATION_FAILED,
        ERR_BUFFER_CREATION_FAILED,
        ERR_RENDER_TARGET_FAILED,
        ERR_DRAW_CALL_FAILED,
        ERR_ENTITY_NOT_FOUND,
        ERR_ENTITY_ALREADY_EXISTS,
        ERR_COMPONENT_NOT_FOUND,
        ERR_COMPONENT_ALREADY_EXISTS,
        ERR_QUERY_FAILED,
        ERR_INPUT_DEVICE_NOT_FOUND,
        ERR_INVALID_INPUT_ACTION,
        ERR_WINDOW_CREATION_FAILED,
        ERR_PLATFORM_ERROR,
        ERR_PROVIDER_INIT_FAILED,
        ERR_PROVIDER_NOT_FOUND,
        ERR_PROVIDER_OPERATION_FAILED,
        ERR_NOT_IMPLEMENTED,
    ];
    for code in recoverable_codes {
        assert_eq!(
            recovery_class(code),
            RecoveryClass::Recoverable,
            "expected Recoverable for code {code}"
        );
    }
}

#[test]
fn test_success_returns_recoverable() {
    assert_eq!(recovery_class(SUCCESS), RecoveryClass::Recoverable);
}

#[test]
fn test_unknown_code_returns_recoverable() {
    assert_eq!(recovery_class(999), RecoveryClass::Recoverable);
    assert_eq!(recovery_class(12345), RecoveryClass::Recoverable);
    assert_eq!(recovery_class(-1), RecoveryClass::Recoverable);
}

// =============================================================================
// Recovery hint tests
// =============================================================================

#[test]
fn test_success_has_empty_hint() {
    assert_eq!(recovery_hint(SUCCESS), "");
}

#[test]
fn test_every_error_code_has_nonempty_hint() {
    let all_error_codes = [
        ERR_NOT_INITIALIZED,
        ERR_ALREADY_INITIALIZED,
        ERR_INVALID_CONTEXT,
        ERR_CONTEXT_DESTROYED,
        ERR_INITIALIZATION_FAILED,
        ERR_RESOURCE_NOT_FOUND,
        ERR_RESOURCE_LOAD_FAILED,
        ERR_RESOURCE_INVALID_FORMAT,
        ERR_RESOURCE_ALREADY_EXISTS,
        ERR_INVALID_HANDLE,
        ERR_HANDLE_EXPIRED,
        ERR_HANDLE_TYPE_MISMATCH,
        ERR_SHADER_COMPILATION_FAILED,
        ERR_SHADER_LINK_FAILED,
        ERR_TEXTURE_CREATION_FAILED,
        ERR_BUFFER_CREATION_FAILED,
        ERR_RENDER_TARGET_FAILED,
        ERR_BACKEND_NOT_SUPPORTED,
        ERR_DRAW_CALL_FAILED,
        ERR_ENTITY_NOT_FOUND,
        ERR_ENTITY_ALREADY_EXISTS,
        ERR_COMPONENT_NOT_FOUND,
        ERR_COMPONENT_ALREADY_EXISTS,
        ERR_QUERY_FAILED,
        ERR_INPUT_DEVICE_NOT_FOUND,
        ERR_INVALID_INPUT_ACTION,
        ERR_WINDOW_CREATION_FAILED,
        ERR_AUDIO_INIT_FAILED,
        ERR_PHYSICS_INIT_FAILED,
        ERR_PLATFORM_ERROR,
        ERR_PROVIDER_INIT_FAILED,
        ERR_PROVIDER_NOT_FOUND,
        ERR_PROVIDER_OPERATION_FAILED,
        ERR_INTERNAL_ERROR,
        ERR_NOT_IMPLEMENTED,
        ERR_INVALID_STATE,
    ];
    for code in all_error_codes {
        let hint = recovery_hint(code);
        assert!(
            !hint.is_empty(),
            "expected non-empty hint for code {code}, got empty string"
        );
    }
}

#[test]
fn test_specific_recovery_hints() {
    assert_eq!(
        recovery_hint(ERR_NOT_INITIALIZED),
        "Call the initialization function first"
    );
    assert_eq!(
        recovery_hint(ERR_RESOURCE_NOT_FOUND),
        "Verify the file path and check the working directory"
    );
    assert_eq!(
        recovery_hint(ERR_INTERNAL_ERROR),
        "Report the error with full details; this is likely an engine bug"
    );
}

#[test]
fn test_unknown_code_has_fallback_hint() {
    let hint = recovery_hint(999);
    assert!(!hint.is_empty(), "unknown code should have a fallback hint");
}

// =============================================================================
// Helper function tests
// =============================================================================

#[test]
fn test_is_recoverable_helper() {
    assert!(is_recoverable(SUCCESS));
    assert!(is_recoverable(ERR_RESOURCE_NOT_FOUND));
    assert!(!is_recoverable(ERR_NOT_INITIALIZED));
    assert!(!is_recoverable(ERR_AUDIO_INIT_FAILED));
}

#[test]
fn test_is_fatal_helper() {
    assert!(is_fatal(ERR_NOT_INITIALIZED));
    assert!(is_fatal(ERR_INVALID_CONTEXT));
    assert!(is_fatal(ERR_INTERNAL_ERROR));
    assert!(!is_fatal(SUCCESS));
    assert!(!is_fatal(ERR_RESOURCE_NOT_FOUND));
    assert!(!is_fatal(ERR_AUDIO_INIT_FAILED));
}

// =============================================================================
// Repr tests
// =============================================================================

#[test]
fn test_recovery_class_repr_values() {
    assert_eq!(RecoveryClass::Recoverable as i32, 0);
    assert_eq!(RecoveryClass::Fatal as i32, 1);
    assert_eq!(RecoveryClass::Degraded as i32, 2);
}
