//! Tests for GoudError entity variants (codes 300-399).

use crate::core::error::{
    GoudError, ERR_COMPONENT_ALREADY_EXISTS, ERR_COMPONENT_NOT_FOUND, ERR_ENTITY_ALREADY_EXISTS,
    ERR_ENTITY_NOT_FOUND, ERR_QUERY_FAILED,
};

#[test]
fn test_entity_not_found_error_code() {
    let error = GoudError::EntityNotFound;
    assert_eq!(error.error_code(), ERR_ENTITY_NOT_FOUND);
    assert_eq!(error.error_code(), 300);
}

#[test]
fn test_entity_already_exists_error_code() {
    let error = GoudError::EntityAlreadyExists;
    assert_eq!(error.error_code(), ERR_ENTITY_ALREADY_EXISTS);
    assert_eq!(error.error_code(), 301);
}

#[test]
fn test_component_not_found_error_code() {
    let error = GoudError::ComponentNotFound;
    assert_eq!(error.error_code(), ERR_COMPONENT_NOT_FOUND);
    assert_eq!(error.error_code(), 310);
}

#[test]
fn test_component_already_exists_error_code() {
    let error = GoudError::ComponentAlreadyExists;
    assert_eq!(error.error_code(), ERR_COMPONENT_ALREADY_EXISTS);
    assert_eq!(error.error_code(), 311);
}

#[test]
fn test_query_failed_error_code() {
    let error = GoudError::QueryFailed("conflicting access on Position".to_string());
    assert_eq!(error.error_code(), ERR_QUERY_FAILED);
    assert_eq!(error.error_code(), 320);
}

#[test]
fn test_all_entity_errors_in_entity_category() {
    let errors: Vec<GoudError> = vec![
        GoudError::EntityNotFound,
        GoudError::EntityAlreadyExists,
        GoudError::ComponentNotFound,
        GoudError::ComponentAlreadyExists,
        GoudError::QueryFailed("test".to_string()),
    ];

    for error in errors {
        assert_eq!(
            error.category(),
            "Entity",
            "Error {:?} should be in Entity category",
            error
        );
    }
}

#[test]
fn test_entity_error_codes_in_valid_range() {
    let errors: Vec<GoudError> = vec![
        GoudError::EntityNotFound,
        GoudError::EntityAlreadyExists,
        GoudError::ComponentNotFound,
        GoudError::ComponentAlreadyExists,
        GoudError::QueryFailed("test".to_string()),
    ];

    for error in errors {
        let code = error.error_code();
        assert!(
            code >= 300 && code < 400,
            "Entity error {:?} has code {} which is outside range 300-399",
            error,
            code
        );
    }
}

#[test]
fn test_query_failed_preserves_message() {
    let query_err = "Conflicting access: &mut Position and &Position on same entity";
    if let GoudError::QueryFailed(msg) = GoudError::QueryFailed(query_err.to_string()) {
        assert_eq!(msg, query_err);
    } else {
        panic!("Expected QueryFailed variant");
    }
}

#[test]
fn test_entity_error_equality() {
    assert_eq!(GoudError::EntityNotFound, GoudError::EntityNotFound);
    assert_eq!(GoudError::ComponentNotFound, GoudError::ComponentNotFound);

    assert_ne!(GoudError::EntityNotFound, GoudError::EntityAlreadyExists);
    assert_ne!(
        GoudError::ComponentNotFound,
        GoudError::ComponentAlreadyExists
    );

    let err1 = GoudError::QueryFailed("error".to_string());
    let err2 = GoudError::QueryFailed("error".to_string());
    assert_eq!(err1, err2);

    let err3 = GoudError::QueryFailed("different".to_string());
    assert_ne!(err1, err3);
}

#[test]
fn test_entity_error_debug_format() {
    let error = GoudError::EntityNotFound;
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("EntityNotFound"));

    let error2 = GoudError::QueryFailed("access conflict".to_string());
    let debug_str2 = format!("{:?}", error2);
    assert!(debug_str2.contains("QueryFailed"));
    assert!(debug_str2.contains("access conflict"));
}

#[test]
fn test_entity_error_codes_are_distinct() {
    let codes = vec![
        ERR_ENTITY_NOT_FOUND,
        ERR_ENTITY_ALREADY_EXISTS,
        ERR_COMPONENT_NOT_FOUND,
        ERR_COMPONENT_ALREADY_EXISTS,
        ERR_QUERY_FAILED,
    ];

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
fn test_entity_error_code_gaps_for_future_expansion() {
    assert!(ERR_ENTITY_NOT_FOUND == 300);
    assert!(ERR_ENTITY_ALREADY_EXISTS == 301);
    assert!(ERR_COMPONENT_NOT_FOUND == 310);
    assert!(ERR_COMPONENT_ALREADY_EXISTS == 311);
    assert!(ERR_QUERY_FAILED == 320);
}
