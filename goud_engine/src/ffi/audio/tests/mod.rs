#[cfg(test)]
use super::*;

#[cfg(test)]
use crate::core::error::{clear_last_error, last_error_code, ERR_INVALID_CONTEXT, SUCCESS};

#[cfg(test)]
use crate::ffi::context::{goud_context_create, goud_context_destroy, GOUD_INVALID_CONTEXT_ID};

#[test]
fn test_goud_audio_activate_returns_success_for_valid_context() {
    // Arrange
    clear_last_error();
    let context_id = goud_context_create();
    assert_ne!(context_id, GOUD_INVALID_CONTEXT_ID);

    // Act
    let result = goud_audio_activate(context_id);

    // Assert
    assert_eq!(result, 0);
    assert_eq!(last_error_code(), SUCCESS);

    assert!(goud_context_destroy(context_id));
}

#[test]
fn test_goud_audio_activate_returns_error_for_invalid_context() {
    // Arrange
    clear_last_error();

    // Act
    let result = goud_audio_activate(GOUD_INVALID_CONTEXT_ID);

    // Assert
    assert_eq!(result, ERR_I32);
    assert_eq!(last_error_code(), ERR_INVALID_CONTEXT);
}

#[test]
fn test_goud_audio_activate_returns_error_for_destroyed_context() {
    // Arrange
    clear_last_error();
    let context_id = goud_context_create();
    assert_ne!(context_id, GOUD_INVALID_CONTEXT_ID);
    assert!(goud_context_destroy(context_id));

    // Act
    let result = goud_audio_activate(context_id);

    // Assert
    assert_eq!(result, ERR_I32);
    assert_eq!(last_error_code(), ERR_INVALID_CONTEXT);
}
