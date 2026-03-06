//! Round-trip tests for GoudError <-> error code conversion.
//!
//! Verifies that `error_code()` and `from_error_code()` form a consistent
//! mapping for all error variants and all ERR_* constants.

use crate::core::error::{
    GoudError, ERR_ALREADY_INITIALIZED, ERR_AUDIO_INIT_FAILED, ERR_BACKEND_NOT_SUPPORTED,
    ERR_BUFFER_CREATION_FAILED, ERR_COMPONENT_ALREADY_EXISTS, ERR_COMPONENT_NOT_FOUND,
    ERR_CONTEXT_DESTROYED, ERR_DRAW_CALL_FAILED, ERR_ENTITY_ALREADY_EXISTS, ERR_ENTITY_NOT_FOUND,
    ERR_HANDLE_EXPIRED, ERR_HANDLE_TYPE_MISMATCH, ERR_INITIALIZATION_FAILED, ERR_INTERNAL_ERROR,
    ERR_INVALID_CONTEXT, ERR_INVALID_HANDLE, ERR_INVALID_STATE, ERR_NOT_IMPLEMENTED,
    ERR_NOT_INITIALIZED, ERR_PHYSICS_INIT_FAILED, ERR_PLATFORM_ERROR, ERR_QUERY_FAILED,
    ERR_RENDER_TARGET_FAILED, ERR_RESOURCE_ALREADY_EXISTS, ERR_RESOURCE_INVALID_FORMAT,
    ERR_RESOURCE_LOAD_FAILED, ERR_RESOURCE_NOT_FOUND, ERR_SHADER_COMPILATION_FAILED,
    ERR_SHADER_LINK_FAILED, ERR_TEXTURE_CREATION_FAILED, ERR_WINDOW_CREATION_FAILED, SUCCESS,
};

#[test]
fn test_success_returns_none() {
    assert!(GoudError::from_error_code(SUCCESS).is_none());
}

#[test]
fn test_unknown_code_returns_none() {
    assert!(GoudError::from_error_code(-1).is_none());
    assert!(GoudError::from_error_code(9999).is_none());
    assert!(GoudError::from_error_code(600).is_none());
}

#[test]
fn test_round_trip_context_errors() {
    let variants = vec![
        GoudError::NotInitialized,
        GoudError::AlreadyInitialized,
        GoudError::InvalidContext,
        GoudError::ContextDestroyed,
        GoudError::InitializationFailed("test".to_string()),
    ];
    for error in variants {
        let code = error.error_code();
        let recovered = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("from_error_code({code}) returned None"));
        assert_eq!(
            recovered.error_code(),
            code,
            "Round-trip failed for code {code}"
        );
    }
}

#[test]
fn test_round_trip_resource_errors() {
    let variants = vec![
        GoudError::ResourceNotFound("test".to_string()),
        GoudError::ResourceLoadFailed("test".to_string()),
        GoudError::ResourceInvalidFormat("test".to_string()),
        GoudError::ResourceAlreadyExists("test".to_string()),
        GoudError::InvalidHandle,
        GoudError::HandleExpired,
        GoudError::HandleTypeMismatch,
    ];
    for error in variants {
        let code = error.error_code();
        let recovered = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("from_error_code({code}) returned None"));
        assert_eq!(
            recovered.error_code(),
            code,
            "Round-trip failed for code {code}"
        );
    }
}

#[test]
fn test_round_trip_graphics_errors() {
    let variants = vec![
        GoudError::ShaderCompilationFailed("test".to_string()),
        GoudError::ShaderLinkFailed("test".to_string()),
        GoudError::TextureCreationFailed("test".to_string()),
        GoudError::BufferCreationFailed("test".to_string()),
        GoudError::RenderTargetFailed("test".to_string()),
        GoudError::BackendNotSupported("test".to_string()),
        GoudError::DrawCallFailed("test".to_string()),
    ];
    for error in variants {
        let code = error.error_code();
        let recovered = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("from_error_code({code}) returned None"));
        assert_eq!(
            recovered.error_code(),
            code,
            "Round-trip failed for code {code}"
        );
    }
}

#[test]
fn test_round_trip_entity_errors() {
    let variants = vec![
        GoudError::EntityNotFound,
        GoudError::EntityAlreadyExists,
        GoudError::ComponentNotFound,
        GoudError::ComponentAlreadyExists,
        GoudError::QueryFailed("test".to_string()),
    ];
    for error in variants {
        let code = error.error_code();
        let recovered = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("from_error_code({code}) returned None"));
        assert_eq!(
            recovered.error_code(),
            code,
            "Round-trip failed for code {code}"
        );
    }
}

#[test]
fn test_round_trip_system_errors() {
    let variants = vec![
        GoudError::WindowCreationFailed("test".to_string()),
        GoudError::AudioInitFailed("test".to_string()),
        GoudError::PhysicsInitFailed("test".to_string()),
        GoudError::PlatformError("test".to_string()),
    ];
    for error in variants {
        let code = error.error_code();
        let recovered = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("from_error_code({code}) returned None"));
        assert_eq!(
            recovered.error_code(),
            code,
            "Round-trip failed for code {code}"
        );
    }
}

#[test]
fn test_round_trip_internal_errors() {
    let variants = vec![
        GoudError::InternalError("test".to_string()),
        GoudError::NotImplemented("test".to_string()),
        GoudError::InvalidState("test".to_string()),
    ];
    for error in variants {
        let code = error.error_code();
        let recovered = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("from_error_code({code}) returned None"));
        assert_eq!(
            recovered.error_code(),
            code,
            "Round-trip failed for code {code}"
        );
    }
}

#[test]
fn test_all_err_constants_have_mappings() {
    let all_codes = [
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
        ERR_WINDOW_CREATION_FAILED,
        ERR_AUDIO_INIT_FAILED,
        ERR_PHYSICS_INIT_FAILED,
        ERR_PLATFORM_ERROR,
        ERR_INTERNAL_ERROR,
        ERR_NOT_IMPLEMENTED,
        ERR_INVALID_STATE,
    ];

    for code in all_codes {
        let error = GoudError::from_error_code(code)
            .unwrap_or_else(|| panic!("ERR constant {code} has no from_error_code mapping"));
        assert_eq!(
            error.error_code(),
            code,
            "ERR constant {code} does not round-trip correctly"
        );
    }
}

#[test]
fn test_from_error_code_uses_empty_string_for_payloads() {
    let codes_with_payloads = [
        ERR_INITIALIZATION_FAILED,
        ERR_RESOURCE_NOT_FOUND,
        ERR_RESOURCE_LOAD_FAILED,
        ERR_RESOURCE_INVALID_FORMAT,
        ERR_RESOURCE_ALREADY_EXISTS,
        ERR_SHADER_COMPILATION_FAILED,
        ERR_SHADER_LINK_FAILED,
        ERR_TEXTURE_CREATION_FAILED,
        ERR_BUFFER_CREATION_FAILED,
        ERR_RENDER_TARGET_FAILED,
        ERR_BACKEND_NOT_SUPPORTED,
        ERR_DRAW_CALL_FAILED,
        ERR_QUERY_FAILED,
        ERR_WINDOW_CREATION_FAILED,
        ERR_AUDIO_INIT_FAILED,
        ERR_PHYSICS_INIT_FAILED,
        ERR_PLATFORM_ERROR,
        ERR_INTERNAL_ERROR,
        ERR_NOT_IMPLEMENTED,
        ERR_INVALID_STATE,
    ];

    for code in codes_with_payloads {
        let error = GoudError::from_error_code(code).unwrap();
        assert_eq!(
            error.message(),
            "",
            "from_error_code({code}) should use empty string for payload, got: '{}'",
            error.message()
        );
    }
}
