//! Tests for error codes, constants, and helper functions.

use crate::core::error::{
    error_category, is_error, is_success, CONTEXT_ERROR_BASE, ENTITY_ERROR_BASE,
    GRAPHICS_ERROR_BASE, INPUT_ERROR_BASE, INTERNAL_ERROR_BASE, RESOURCE_ERROR_BASE,
    SYSTEM_ERROR_BASE,
};
use crate::core::error::{
    ERR_DRAW_CALL_FAILED, ERR_ENTITY_NOT_FOUND, ERR_INITIALIZATION_FAILED, ERR_INTERNAL_ERROR,
    ERR_INVALID_INPUT_ACTION, ERR_INVALID_STATE, ERR_NOT_INITIALIZED, ERR_PLATFORM_ERROR,
    ERR_QUERY_FAILED, ERR_RESOURCE_NOT_FOUND, ERR_SHADER_COMPILATION_FAILED,
    ERR_WINDOW_CREATION_FAILED, SUCCESS,
};
use crate::core::error::{ERR_HANDLE_TYPE_MISMATCH, ERR_INPUT_DEVICE_NOT_FOUND};

#[test]
fn test_success_code_is_zero() {
    assert_eq!(SUCCESS, 0);
}

#[test]
fn test_error_code_ranges_are_non_overlapping() {
    assert!(CONTEXT_ERROR_BASE < RESOURCE_ERROR_BASE);
    assert!(RESOURCE_ERROR_BASE < GRAPHICS_ERROR_BASE);
    assert!(GRAPHICS_ERROR_BASE < ENTITY_ERROR_BASE);
    assert!(ENTITY_ERROR_BASE < INPUT_ERROR_BASE);
    assert!(INPUT_ERROR_BASE < SYSTEM_ERROR_BASE);
    assert!(SYSTEM_ERROR_BASE < INTERNAL_ERROR_BASE);
}

#[test]
fn test_error_category_classification() {
    assert_eq!(error_category(SUCCESS), "Success");
    assert_eq!(error_category(ERR_NOT_INITIALIZED), "Context");
    assert_eq!(error_category(ERR_RESOURCE_NOT_FOUND), "Resource");
    assert_eq!(error_category(ERR_SHADER_COMPILATION_FAILED), "Graphics");
    assert_eq!(error_category(ERR_ENTITY_NOT_FOUND), "Entity");
    assert_eq!(error_category(ERR_INPUT_DEVICE_NOT_FOUND), "Input");
    assert_eq!(error_category(ERR_WINDOW_CREATION_FAILED), "System");
    assert_eq!(error_category(ERR_INTERNAL_ERROR), "Internal");
}

#[test]
fn test_is_success_and_is_error() {
    assert!(is_success(SUCCESS));
    assert!(!is_error(SUCCESS));

    assert!(!is_success(ERR_NOT_INITIALIZED));
    assert!(is_error(ERR_NOT_INITIALIZED));

    assert!(!is_success(ERR_RESOURCE_NOT_FOUND));
    assert!(is_error(ERR_RESOURCE_NOT_FOUND));
}

#[test]
fn test_error_codes_within_category_bounds() {
    // Context errors: 1-99
    assert!(ERR_NOT_INITIALIZED >= 1 && ERR_NOT_INITIALIZED < 100);
    assert!(ERR_INITIALIZATION_FAILED >= 1 && ERR_INITIALIZATION_FAILED < 100);

    // Resource errors: 100-199
    assert!(ERR_RESOURCE_NOT_FOUND >= 100 && ERR_RESOURCE_NOT_FOUND < 200);
    assert!(ERR_HANDLE_TYPE_MISMATCH >= 100 && ERR_HANDLE_TYPE_MISMATCH < 200);

    // Graphics errors: 200-299
    assert!(ERR_SHADER_COMPILATION_FAILED >= 200 && ERR_SHADER_COMPILATION_FAILED < 300);
    assert!(ERR_DRAW_CALL_FAILED >= 200 && ERR_DRAW_CALL_FAILED < 300);

    // Entity errors: 300-399
    assert!(ERR_ENTITY_NOT_FOUND >= 300 && ERR_ENTITY_NOT_FOUND < 400);
    assert!(ERR_QUERY_FAILED >= 300 && ERR_QUERY_FAILED < 400);

    // Input errors: 400-499
    assert!(ERR_INPUT_DEVICE_NOT_FOUND >= 400 && ERR_INPUT_DEVICE_NOT_FOUND < 500);

    // System errors: 500-599
    assert!(ERR_WINDOW_CREATION_FAILED >= 500 && ERR_WINDOW_CREATION_FAILED < 600);
    assert!(ERR_PLATFORM_ERROR >= 500 && ERR_PLATFORM_ERROR < 600);

    // Internal errors: 900-999
    assert!(ERR_INTERNAL_ERROR >= 900 && ERR_INTERNAL_ERROR < 1000);
    assert!(ERR_INVALID_STATE >= 900 && ERR_INVALID_STATE < 1000);
}

#[test]
fn test_unknown_category_for_out_of_range() {
    assert_eq!(error_category(-1), "Unknown");
    assert_eq!(error_category(1000), "Unknown");
    assert_eq!(error_category(700), "Unknown");
}

#[test]
fn test_input_error_codes_in_valid_range() {
    assert!(ERR_INPUT_DEVICE_NOT_FOUND >= 400 && ERR_INPUT_DEVICE_NOT_FOUND < 500);
    assert!(ERR_INVALID_INPUT_ACTION >= 400 && ERR_INVALID_INPUT_ACTION < 500);
}
