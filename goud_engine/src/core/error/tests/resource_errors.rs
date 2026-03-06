//! Tests for GoudError resource variants (codes 100-199).

use crate::core::error::{
    GoudError, ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH, ERR_INVALID_HANDLE,
    ERR_RESOURCE_ALREADY_EXISTS, ERR_RESOURCE_INVALID_FORMAT, ERR_RESOURCE_LOAD_FAILED,
    ERR_RESOURCE_NOT_FOUND,
};

#[test]
fn test_resource_not_found_error_code() {
    let error = GoudError::ResourceNotFound("textures/player.png".to_string());
    assert_eq!(error.error_code(), ERR_RESOURCE_NOT_FOUND);
    assert_eq!(error.error_code(), 100);
}

#[test]
fn test_resource_load_failed_error_code() {
    let error = GoudError::ResourceLoadFailed("I/O error reading file".to_string());
    assert_eq!(error.error_code(), ERR_RESOURCE_LOAD_FAILED);
    assert_eq!(error.error_code(), 101);
}

#[test]
fn test_resource_invalid_format_error_code() {
    let error = GoudError::ResourceInvalidFormat("Invalid PNG header".to_string());
    assert_eq!(error.error_code(), ERR_RESOURCE_INVALID_FORMAT);
    assert_eq!(error.error_code(), 102);
}

#[test]
fn test_resource_already_exists_error_code() {
    let error = GoudError::ResourceAlreadyExists("player_texture".to_string());
    assert_eq!(error.error_code(), ERR_RESOURCE_ALREADY_EXISTS);
    assert_eq!(error.error_code(), 103);
}

#[test]
fn test_invalid_handle_error_code() {
    let error = GoudError::InvalidHandle;
    assert_eq!(error.error_code(), ERR_INVALID_HANDLE);
    assert_eq!(error.error_code(), 110);
}

#[test]
fn test_handle_expired_error_code() {
    let error = GoudError::HandleExpired;
    assert_eq!(error.error_code(), ERR_HANDLE_EXPIRED);
    assert_eq!(error.error_code(), 111);
}

#[test]
fn test_handle_type_mismatch_error_code() {
    let error = GoudError::HandleTypeMismatch;
    assert_eq!(error.error_code(), ERR_HANDLE_TYPE_MISMATCH);
    assert_eq!(error.error_code(), 112);
}

#[test]
fn test_all_resource_errors_in_resource_category() {
    let errors: Vec<GoudError> = vec![
        GoudError::ResourceNotFound("test".to_string()),
        GoudError::ResourceLoadFailed("test".to_string()),
        GoudError::ResourceInvalidFormat("test".to_string()),
        GoudError::ResourceAlreadyExists("test".to_string()),
        GoudError::InvalidHandle,
        GoudError::HandleExpired,
        GoudError::HandleTypeMismatch,
    ];

    for error in errors {
        assert_eq!(
            error.category(),
            "Resource",
            "Error {:?} should be in Resource category",
            error
        );
    }
}

#[test]
fn test_resource_error_codes_in_valid_range() {
    let errors: Vec<GoudError> = vec![
        GoudError::ResourceNotFound("test".to_string()),
        GoudError::ResourceLoadFailed("test".to_string()),
        GoudError::ResourceInvalidFormat("test".to_string()),
        GoudError::ResourceAlreadyExists("test".to_string()),
        GoudError::InvalidHandle,
        GoudError::HandleExpired,
        GoudError::HandleTypeMismatch,
    ];

    for error in errors {
        let code = error.error_code();
        assert!(
            code >= 100 && code < 200,
            "Resource error {:?} has code {} which is outside range 100-199",
            error,
            code
        );
    }
}

#[test]
fn test_resource_errors_preserve_message() {
    let path = "assets/textures/missing.png";
    if let GoudError::ResourceNotFound(msg) = GoudError::ResourceNotFound(path.to_string()) {
        assert_eq!(msg, path);
    } else {
        panic!("Expected ResourceNotFound variant");
    }

    let reason = "Permission denied";
    if let GoudError::ResourceLoadFailed(msg) = GoudError::ResourceLoadFailed(reason.to_string()) {
        assert_eq!(msg, reason);
    } else {
        panic!("Expected ResourceLoadFailed variant");
    }

    let format_err = "Unsupported texture format: PVRTC";
    if let GoudError::ResourceInvalidFormat(msg) =
        GoudError::ResourceInvalidFormat(format_err.to_string())
    {
        assert_eq!(msg, format_err);
    } else {
        panic!("Expected ResourceInvalidFormat variant");
    }

    let resource_id = "main_shader";
    if let GoudError::ResourceAlreadyExists(msg) =
        GoudError::ResourceAlreadyExists(resource_id.to_string())
    {
        assert_eq!(msg, resource_id);
    } else {
        panic!("Expected ResourceAlreadyExists variant");
    }
}

#[test]
fn test_resource_error_equality() {
    let err1 = GoudError::ResourceNotFound("file.txt".to_string());
    let err2 = GoudError::ResourceNotFound("file.txt".to_string());
    assert_eq!(err1, err2);

    let err3 = GoudError::ResourceNotFound("other.txt".to_string());
    assert_ne!(err1, err3);

    let err4 = GoudError::ResourceLoadFailed("file.txt".to_string());
    assert_ne!(err1, err4);

    assert_eq!(GoudError::InvalidHandle, GoudError::InvalidHandle);
    assert_ne!(GoudError::InvalidHandle, GoudError::HandleExpired);
    assert_ne!(GoudError::HandleExpired, GoudError::HandleTypeMismatch);
}

#[test]
fn test_resource_error_debug_format() {
    let error = GoudError::ResourceNotFound("test.png".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ResourceNotFound"));
    assert!(debug_str.contains("test.png"));

    let handle_error = GoudError::InvalidHandle;
    let debug_str = format!("{:?}", handle_error);
    assert!(debug_str.contains("InvalidHandle"));
}

#[test]
fn test_handle_error_codes_are_distinct() {
    assert_ne!(ERR_INVALID_HANDLE, ERR_HANDLE_EXPIRED);
    assert_ne!(ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH);
    assert_ne!(ERR_INVALID_HANDLE, ERR_HANDLE_TYPE_MISMATCH);

    assert!(ERR_INVALID_HANDLE >= 110);
    assert!(ERR_HANDLE_EXPIRED >= 110);
    assert!(ERR_HANDLE_TYPE_MISMATCH >= 110);
}
