//! Tests for error context propagation.

use crate::core::error::context::subsystems;
use crate::core::error::{
    clear_last_error, last_error_operation, last_error_subsystem, set_last_error,
    set_last_error_with_context, take_last_error, GoudError, GoudErrorContext,
};

#[test]
fn test_set_last_error_with_context_populates_subsystem_and_operation() {
    let ctx = GoudErrorContext::new(subsystems::GRAPHICS, "shader_compile");
    set_last_error_with_context(GoudError::ShaderCompilationFailed("test".to_string()), ctx);

    assert_eq!(last_error_subsystem(), Some("graphics"));
    assert_eq!(last_error_operation(), Some("shader_compile"));

    // Clean up
    clear_last_error();
}

#[test]
fn test_set_last_error_without_context_clears_previous_context() {
    // First set an error with context
    let ctx = GoudErrorContext::new(subsystems::ECS, "entity_spawn");
    set_last_error_with_context(GoudError::EntityNotFound, ctx);
    assert_eq!(last_error_subsystem(), Some("ecs"));

    // Now set a plain error -- should clear context
    set_last_error(GoudError::NotInitialized);
    assert_eq!(last_error_subsystem(), None);
    assert_eq!(last_error_operation(), None);

    // Clean up
    clear_last_error();
}

#[test]
fn test_clear_last_error_clears_context() {
    let ctx = GoudErrorContext::new(subsystems::AUDIO, "play_sound");
    set_last_error_with_context(GoudError::AudioInitFailed("test".to_string()), ctx);
    assert_eq!(last_error_subsystem(), Some("audio"));

    clear_last_error();

    assert_eq!(last_error_subsystem(), None);
    assert_eq!(last_error_operation(), None);
}

#[test]
fn test_take_last_error_clears_context() {
    let ctx = GoudErrorContext::new(subsystems::RESOURCE, "load_texture");
    set_last_error_with_context(GoudError::ResourceNotFound("missing.png".to_string()), ctx);
    assert_eq!(last_error_subsystem(), Some("resource"));

    let err = take_last_error();
    assert!(err.is_some());

    assert_eq!(last_error_subsystem(), None);
    assert_eq!(last_error_operation(), None);
}

#[test]
fn test_context_defaults_to_none_when_not_provided() {
    clear_last_error();

    assert_eq!(last_error_subsystem(), None);
    assert_eq!(last_error_operation(), None);

    // Set error without context
    set_last_error(GoudError::NotInitialized);
    assert_eq!(last_error_subsystem(), None);
    assert_eq!(last_error_operation(), None);

    // Clean up
    clear_last_error();
}
