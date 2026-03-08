//! Scene transition FFI functions.
//!
//! Provides C-compatible functions for starting, querying, and advancing
//! scene transitions within an engine context.

use crate::context_registry::scene::transition::TransitionType;
use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR};
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::types::GoudResult;

// ============================================================================
// Scene Transitions
// ============================================================================

/// Starts a transition between two scenes.
///
/// # Arguments
///
/// * `context_id` - The context containing both scenes
/// * `from_scene` - The scene to transition away from
/// * `to_scene` - The scene to transition to
/// * `transition_type` - Transition style as `u8` (0=Instant, 1=Fade, 2=Custom)
/// * `duration_secs` - Duration of the transition in seconds
///
/// # Returns
///
/// A `GoudResult` indicating success or failure.
#[no_mangle]
pub extern "C" fn goud_scene_transition_to(
    context_id: GoudContextId,
    from_scene: u32,
    to_scene: u32,
    transition_type: u8,
    duration_secs: f32,
) -> GoudResult {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    let tt = match TransitionType::try_from(transition_type) {
        Ok(t) => t,
        Err(_) => {
            set_last_error(GoudError::InvalidState(format!(
                "Invalid transition type: {}",
                transition_type
            )));
            return GoudResult::err(GoudError::InvalidState(String::new()).error_code());
        }
    };

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return GoudResult::err(ERR_INTERNAL_ERROR),
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    match context.start_transition(from_scene, to_scene, tt, duration_secs) {
        Ok(()) => GoudResult::ok(),
        Err(err) => {
            let code = err.error_code();
            set_last_error(err);
            GoudResult::err(code)
        }
    }
}

/// Returns the progress of the active transition.
///
/// # Arguments
///
/// * `context_id` - The context to query
///
/// # Returns
///
/// A value in `[0.0, 1.0]` representing transition progress, or `-1.0`
/// if no transition is active or on error.
#[no_mangle]
pub extern "C" fn goud_scene_transition_progress(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return -1.0;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return -1.0,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return -1.0,
    };

    context.transition_progress().unwrap_or(-1.0)
}

/// Returns whether a scene transition is currently active.
///
/// # Arguments
///
/// * `context_id` - The context to query
///
/// # Returns
///
/// `true` if a transition is in progress, `false` otherwise (including on error).
#[no_mangle]
pub extern "C" fn goud_scene_transition_is_active(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => return false,
    };

    context.is_transitioning()
}

/// Advances the active transition by `delta_time` seconds.
///
/// When the transition completes, the source scene is deactivated
/// and the transition state is cleared.
///
/// # Arguments
///
/// * `context_id` - The context containing the transition
/// * `delta_time` - Time in seconds to advance the transition
///
/// # Returns
///
/// A `GoudResult` indicating success or failure.
#[no_mangle]
pub extern "C" fn goud_scene_transition_tick(
    context_id: GoudContextId,
    delta_time: f32,
) -> GoudResult {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return GoudResult::err(ERR_INTERNAL_ERROR),
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    // tick_transition returns Some(TransitionComplete) when done, None otherwise.
    // Both are valid outcomes, so we always return ok.
    let _completed = context.tick_transition(delta_time);
    GoudResult::ok()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "scene_transition_tests.rs"]
mod tests;
