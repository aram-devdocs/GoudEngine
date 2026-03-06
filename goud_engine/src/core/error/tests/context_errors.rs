//! Tests for GoudError context variants (codes 1-99).

use crate::core::error::{
    GoudError, ERR_ALREADY_INITIALIZED, ERR_CONTEXT_DESTROYED, ERR_INITIALIZATION_FAILED,
    ERR_INVALID_CONTEXT, ERR_NOT_INITIALIZED,
};

#[test]
fn test_not_initialized_error_code() {
    let error = GoudError::NotInitialized;
    assert_eq!(error.error_code(), ERR_NOT_INITIALIZED);
    assert_eq!(error.error_code(), 1);
}

#[test]
fn test_already_initialized_error_code() {
    let error = GoudError::AlreadyInitialized;
    assert_eq!(error.error_code(), ERR_ALREADY_INITIALIZED);
    assert_eq!(error.error_code(), 2);
}

#[test]
fn test_invalid_context_error_code() {
    let error = GoudError::InvalidContext;
    assert_eq!(error.error_code(), ERR_INVALID_CONTEXT);
    assert_eq!(error.error_code(), 3);
}

#[test]
fn test_context_destroyed_error_code() {
    let error = GoudError::ContextDestroyed;
    assert_eq!(error.error_code(), ERR_CONTEXT_DESTROYED);
    assert_eq!(error.error_code(), 4);
}

#[test]
fn test_initialization_failed_error_code() {
    let error = GoudError::InitializationFailed("GPU not found".to_string());
    assert_eq!(error.error_code(), ERR_INITIALIZATION_FAILED);
    assert_eq!(error.error_code(), 10);

    // Different messages should have same error code
    let error2 = GoudError::InitializationFailed("Missing dependency".to_string());
    assert_eq!(error2.error_code(), ERR_INITIALIZATION_FAILED);
}

#[test]
fn test_all_context_errors_in_context_category() {
    let errors = [
        GoudError::NotInitialized,
        GoudError::AlreadyInitialized,
        GoudError::InvalidContext,
        GoudError::ContextDestroyed,
        GoudError::InitializationFailed("test".to_string()),
    ];

    for error in errors {
        assert_eq!(
            error.category(),
            "Context",
            "Error {:?} should be in Context category",
            error
        );
    }
}

#[test]
fn test_context_error_codes_in_valid_range() {
    let errors = [
        GoudError::NotInitialized,
        GoudError::AlreadyInitialized,
        GoudError::InvalidContext,
        GoudError::ContextDestroyed,
        GoudError::InitializationFailed("test".to_string()),
    ];

    for error in errors {
        let code = error.error_code();
        assert!(
            code >= 1 && code < 100,
            "Context error {:?} has code {} which is outside range 1-99",
            error,
            code
        );
    }
}

#[test]
fn test_goud_error_derives() {
    // Test Debug
    let error = GoudError::NotInitialized;
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("NotInitialized"));

    // Test Clone
    let cloned = error.clone();
    assert_eq!(error, cloned);

    // Test PartialEq and Eq
    assert_eq!(GoudError::NotInitialized, GoudError::NotInitialized);
    assert_ne!(GoudError::NotInitialized, GoudError::AlreadyInitialized);

    // Test equality with message content
    let err1 = GoudError::InitializationFailed("msg1".to_string());
    let err2 = GoudError::InitializationFailed("msg1".to_string());
    let err3 = GoudError::InitializationFailed("msg2".to_string());
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn test_initialization_failed_preserves_message() {
    let message = "Failed to initialize OpenGL context: version 4.5 required";
    let error = GoudError::InitializationFailed(message.to_string());

    if let GoudError::InitializationFailed(msg) = error {
        assert_eq!(msg, message);
    } else {
        panic!("Expected InitializationFailed variant");
    }
}
