//! Tests for GoudError system variants (codes 500-599).

use crate::core::error::{
    GoudError, ERR_AUDIO_INIT_FAILED, ERR_PHYSICS_INIT_FAILED, ERR_PLATFORM_ERROR,
    ERR_WINDOW_CREATION_FAILED,
};

#[test]
fn test_window_creation_failed_error_code() {
    let error = GoudError::WindowCreationFailed("No display server found".to_string());
    assert_eq!(error.error_code(), ERR_WINDOW_CREATION_FAILED);
    assert_eq!(error.error_code(), 500);
}

#[test]
fn test_audio_init_failed_error_code() {
    let error = GoudError::AudioInitFailed("No audio devices found".to_string());
    assert_eq!(error.error_code(), ERR_AUDIO_INIT_FAILED);
    assert_eq!(error.error_code(), 510);
}

#[test]
fn test_physics_init_failed_error_code() {
    let error = GoudError::PhysicsInitFailed("Invalid gravity configuration".to_string());
    assert_eq!(error.error_code(), ERR_PHYSICS_INIT_FAILED);
    assert_eq!(error.error_code(), 520);
}

#[test]
fn test_platform_error_error_code() {
    let error = GoudError::PlatformError("macOS: Failed to acquire Metal device".to_string());
    assert_eq!(error.error_code(), ERR_PLATFORM_ERROR);
    assert_eq!(error.error_code(), 530);
}

#[test]
fn test_all_system_errors_in_system_category() {
    let errors: Vec<GoudError> = vec![
        GoudError::WindowCreationFailed("test".to_string()),
        GoudError::AudioInitFailed("test".to_string()),
        GoudError::PhysicsInitFailed("test".to_string()),
        GoudError::PlatformError("test".to_string()),
    ];

    for error in errors {
        assert_eq!(
            error.category(),
            "System",
            "Error {:?} should be in System category",
            error
        );
    }
}

#[test]
fn test_system_error_codes_in_valid_range() {
    let errors: Vec<GoudError> = vec![
        GoudError::WindowCreationFailed("test".to_string()),
        GoudError::AudioInitFailed("test".to_string()),
        GoudError::PhysicsInitFailed("test".to_string()),
        GoudError::PlatformError("test".to_string()),
    ];

    for error in errors {
        let code = error.error_code();
        assert!(
            code >= 500 && code < 600,
            "System error {:?} has code {} which is outside range 500-599",
            error,
            code
        );
    }
}

#[test]
fn test_system_errors_preserve_message() {
    let window_err = "Failed to create GLFW window: 800x600";
    if let GoudError::WindowCreationFailed(msg) =
        GoudError::WindowCreationFailed(window_err.to_string())
    {
        assert_eq!(msg, window_err);
    } else {
        panic!("Expected WindowCreationFailed variant");
    }

    let audio_err = "ALSA: Unable to open default audio device";
    if let GoudError::AudioInitFailed(msg) = GoudError::AudioInitFailed(audio_err.to_string()) {
        assert_eq!(msg, audio_err);
    } else {
        panic!("Expected AudioInitFailed variant");
    }

    let physics_err = "Box2D: Invalid world bounds";
    if let GoudError::PhysicsInitFailed(msg) = GoudError::PhysicsInitFailed(physics_err.to_string())
    {
        assert_eq!(msg, physics_err);
    } else {
        panic!("Expected PhysicsInitFailed variant");
    }

    let platform_err = "Linux: X11 display connection failed";
    if let GoudError::PlatformError(msg) = GoudError::PlatformError(platform_err.to_string()) {
        assert_eq!(msg, platform_err);
    } else {
        panic!("Expected PlatformError variant");
    }
}

#[test]
fn test_system_error_equality() {
    let err1 = GoudError::WindowCreationFailed("error".to_string());
    let err2 = GoudError::WindowCreationFailed("error".to_string());
    assert_eq!(err1, err2);

    let err3 = GoudError::WindowCreationFailed("different".to_string());
    assert_ne!(err1, err3);

    let err4 = GoudError::AudioInitFailed("error".to_string());
    assert_ne!(err1, err4);
}

#[test]
fn test_system_error_debug_format() {
    let error = GoudError::WindowCreationFailed("GLFW error 65543".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("WindowCreationFailed"));
    assert!(debug_str.contains("GLFW error 65543"));

    let error2 = GoudError::PlatformError("Win32 error".to_string());
    let debug_str2 = format!("{:?}", error2);
    assert!(debug_str2.contains("PlatformError"));
    assert!(debug_str2.contains("Win32 error"));
}

#[test]
fn test_system_error_codes_are_distinct() {
    let codes = vec![
        ERR_WINDOW_CREATION_FAILED,
        ERR_AUDIO_INIT_FAILED,
        ERR_PHYSICS_INIT_FAILED,
        ERR_PLATFORM_ERROR,
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
fn test_system_error_code_gaps_for_future_expansion() {
    assert!(ERR_WINDOW_CREATION_FAILED == 500);
    assert!(ERR_AUDIO_INIT_FAILED == 510);
    assert!(ERR_PHYSICS_INIT_FAILED == 520);
    assert!(ERR_PLATFORM_ERROR == 530);
}
