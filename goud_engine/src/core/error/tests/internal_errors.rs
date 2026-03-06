//! Tests for GoudError internal variants (codes 900-999).

use crate::core::error::{GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE, ERR_NOT_IMPLEMENTED};

#[test]
fn test_internal_error_error_code() {
    let error = GoudError::InternalError("Unexpected null pointer in render queue".to_string());
    assert_eq!(error.error_code(), ERR_INTERNAL_ERROR);
    assert_eq!(error.error_code(), 900);
}

#[test]
fn test_not_implemented_error_code() {
    let error = GoudError::NotImplemented("Vulkan backend".to_string());
    assert_eq!(error.error_code(), ERR_NOT_IMPLEMENTED);
    assert_eq!(error.error_code(), 901);
}

#[test]
fn test_invalid_state_error_code() {
    let error = GoudError::InvalidState("Renderer called after shutdown".to_string());
    assert_eq!(error.error_code(), ERR_INVALID_STATE);
    assert_eq!(error.error_code(), 902);
}

#[test]
fn test_all_internal_errors_in_internal_category() {
    let errors: Vec<GoudError> = vec![
        GoudError::InternalError("test".to_string()),
        GoudError::NotImplemented("test".to_string()),
        GoudError::InvalidState("test".to_string()),
    ];

    for error in errors {
        assert_eq!(
            error.category(),
            "Internal",
            "Error {:?} should be in Internal category",
            error
        );
    }
}

#[test]
fn test_internal_error_codes_in_valid_range() {
    let errors: Vec<GoudError> = vec![
        GoudError::InternalError("test".to_string()),
        GoudError::NotImplemented("test".to_string()),
        GoudError::InvalidState("test".to_string()),
    ];

    for error in errors {
        let code = error.error_code();
        assert!(
            code >= 900 && code < 1000,
            "Internal error {:?} has code {} which is outside range 900-999",
            error,
            code
        );
    }
}

#[test]
fn test_internal_errors_preserve_message() {
    let internal_err = "FATAL: Inconsistent component storage state";
    if let GoudError::InternalError(msg) = GoudError::InternalError(internal_err.to_string()) {
        assert_eq!(msg, internal_err);
    } else {
        panic!("Expected InternalError variant");
    }

    let not_impl_err = "Feature 'ray tracing' is not yet implemented";
    if let GoudError::NotImplemented(msg) = GoudError::NotImplemented(not_impl_err.to_string()) {
        assert_eq!(msg, not_impl_err);
    } else {
        panic!("Expected NotImplemented variant");
    }

    let invalid_state_err = "Cannot add components while iterating";
    if let GoudError::InvalidState(msg) = GoudError::InvalidState(invalid_state_err.to_string()) {
        assert_eq!(msg, invalid_state_err);
    } else {
        panic!("Expected InvalidState variant");
    }
}

#[test]
fn test_internal_error_equality() {
    let err1 = GoudError::InternalError("bug".to_string());
    let err2 = GoudError::InternalError("bug".to_string());
    assert_eq!(err1, err2);

    let err3 = GoudError::InternalError("different bug".to_string());
    assert_ne!(err1, err3);

    let err4 = GoudError::NotImplemented("bug".to_string());
    assert_ne!(err1, err4);

    let err5 = GoudError::InvalidState("bug".to_string());
    assert_ne!(err1, err5);
    assert_ne!(err4, err5);
}

#[test]
fn test_internal_error_debug_format() {
    let error = GoudError::InternalError("assertion failed".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("InternalError"));
    assert!(debug_str.contains("assertion failed"));

    let error2 = GoudError::NotImplemented("3D audio".to_string());
    let debug_str2 = format!("{:?}", error2);
    assert!(debug_str2.contains("NotImplemented"));
    assert!(debug_str2.contains("3D audio"));

    let error3 = GoudError::InvalidState("already running".to_string());
    let debug_str3 = format!("{:?}", error3);
    assert!(debug_str3.contains("InvalidState"));
    assert!(debug_str3.contains("already running"));
}

#[test]
fn test_internal_error_codes_are_distinct() {
    let codes = vec![ERR_INTERNAL_ERROR, ERR_NOT_IMPLEMENTED, ERR_INVALID_STATE];

    for (i, code1) in codes.iter().enumerate() {
        for (j, code2) in codes.iter().enumerate() {
            if i != j {
                assert_ne!(
                    code1, code2,
                    "Error codes at index {} and {} should be different",
                    i, j
                );
            }
        }
    }
}

#[test]
fn test_internal_error_code_ordering() {
    assert_eq!(ERR_INTERNAL_ERROR, 900);
    assert_eq!(ERR_NOT_IMPLEMENTED, 901);
    assert_eq!(ERR_INVALID_STATE, 902);
}
