//! Tests for Display, Error trait, From conversions, and GoudResult.

use std::error::Error;

use crate::core::error::{
    GoudError, GoudResult, ERR_INITIALIZATION_FAILED, ERR_INTERNAL_ERROR, ERR_NOT_INITIALIZED,
    ERR_RESOURCE_LOAD_FAILED, ERR_RESOURCE_NOT_FOUND,
};

#[test]
fn test_display_format_context_errors() {
    let error = GoudError::NotInitialized;
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-1] Context: Engine has not been initialized");

    let error = GoudError::InitializationFailed("GPU not found".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-10] Context: GPU not found");
}

#[test]
fn test_display_format_resource_errors() {
    let error = GoudError::ResourceNotFound("textures/player.png".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-100] Resource: textures/player.png");

    let error = GoudError::InvalidHandle;
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-110] Resource: Invalid handle");
}

#[test]
fn test_display_format_graphics_errors() {
    let error = GoudError::ShaderCompilationFailed("syntax error at line 42".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-200] Graphics: syntax error at line 42");
}

#[test]
fn test_display_format_entity_errors() {
    let error = GoudError::EntityNotFound;
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-300] Entity: Entity not found");

    let error = GoudError::QueryFailed("conflicting access".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-320] Entity: conflicting access");
}

#[test]
fn test_display_format_system_errors() {
    let error = GoudError::WindowCreationFailed("no display".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-500] System: no display");
}

#[test]
fn test_display_format_internal_errors() {
    let error = GoudError::InternalError("unexpected state".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-900] Internal: unexpected state");

    let error = GoudError::NotImplemented("feature X".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "[GOUD-901] Internal: feature X");
}

#[test]
fn test_error_trait_implementation() {
    let error = GoudError::NotInitialized;

    let error_ref: &dyn Error = &error;
    assert!(error_ref.source().is_none());

    let display = format!("{}", error_ref);
    assert!(display.contains("GOUD-1"));
}

#[test]
fn test_message_method() {
    assert_eq!(
        GoudError::NotInitialized.message(),
        "Engine has not been initialized"
    );
    assert_eq!(GoudError::InvalidHandle.message(), "Invalid handle");
    assert_eq!(GoudError::EntityNotFound.message(), "Entity not found");

    let error = GoudError::InitializationFailed("custom message".to_string());
    assert_eq!(error.message(), "custom message");

    let error = GoudError::ResourceNotFound("path/to/file".to_string());
    assert_eq!(error.message(), "path/to/file");
}

#[test]
fn test_from_io_error_not_found() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let goud_error: GoudError = io_error.into();

    assert!(matches!(goud_error, GoudError::ResourceNotFound(_)));
    assert_eq!(goud_error.error_code(), ERR_RESOURCE_NOT_FOUND);
}

#[test]
fn test_from_io_error_permission_denied() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let goud_error: GoudError = io_error.into();

    assert!(matches!(goud_error, GoudError::ResourceLoadFailed(_)));
    assert_eq!(goud_error.error_code(), ERR_RESOURCE_LOAD_FAILED);
    assert!(goud_error.message().contains("Permission denied"));
}

#[test]
fn test_from_io_error_other() {
    let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "network error");
    let goud_error: GoudError = io_error.into();

    assert!(matches!(goud_error, GoudError::ResourceLoadFailed(_)));
    assert_eq!(goud_error.error_code(), ERR_RESOURCE_LOAD_FAILED);
}

#[test]
fn test_from_string() {
    let msg = "something went wrong".to_string();
    let error: GoudError = msg.into();

    assert!(matches!(error, GoudError::InternalError(_)));
    assert_eq!(error.message(), "something went wrong");
    assert_eq!(error.error_code(), ERR_INTERNAL_ERROR);
}

#[test]
fn test_from_str() {
    let error: GoudError = "oops".into();

    assert!(matches!(error, GoudError::InternalError(_)));
    assert_eq!(error.message(), "oops");
    assert_eq!(error.error_code(), ERR_INTERNAL_ERROR);
}

#[test]
fn test_goud_result_ok() {
    fn might_fail() -> GoudResult<i32> {
        Ok(42)
    }

    let result = might_fail();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_goud_result_err() {
    fn always_fails() -> GoudResult<i32> {
        Err(GoudError::NotInitialized)
    }

    let result = always_fails();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error, GoudError::NotInitialized);
}

#[test]
fn test_goud_result_with_question_mark() {
    fn inner() -> GoudResult<i32> {
        Err(GoudError::ResourceNotFound("missing.txt".to_string()))
    }

    fn outer() -> GoudResult<i32> {
        let value = inner()?;
        Ok(value + 1)
    }

    let result = outer();
    assert!(result.is_err());
    if let Err(GoudError::ResourceNotFound(msg)) = result {
        assert_eq!(msg, "missing.txt");
    } else {
        panic!("Expected ResourceNotFound error");
    }
}

#[test]
fn test_error_can_be_boxed() {
    let error: Box<dyn Error> = Box::new(GoudError::NotInitialized);
    let display = format!("{}", error);
    assert!(display.contains("GOUD-1"));
}

#[test]
fn test_display_versus_debug() {
    let error = GoudError::InitializationFailed("test message".to_string());

    let display = format!("{}", error);
    let debug = format!("{:?}", error);

    assert!(display.contains("[GOUD-10]"));
    assert!(display.contains("Context"));
    assert!(display.contains("test message"));

    assert!(debug.contains("InitializationFailed"));
    assert!(debug.contains("test message"));

    assert_ne!(display, debug);
}

#[test]
fn test_goud_error_not_initialized_has_correct_code() {
    let error = GoudError::NotInitialized;
    assert_eq!(error.error_code(), ERR_NOT_INITIALIZED);
}

#[test]
fn test_goud_error_initialization_failed_has_correct_code() {
    let error = GoudError::InitializationFailed("reason".to_string());
    assert_eq!(error.error_code(), ERR_INITIALIZATION_FAILED);
}
