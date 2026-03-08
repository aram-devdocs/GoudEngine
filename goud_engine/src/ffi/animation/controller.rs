//! AnimationController FFI functions.
//!
//! Provides C-compatible functions for creating and managing animation
//! state machine components on entities.

use crate::core::error::{
    set_last_error, GoudError, ERR_COMPONENT_NOT_FOUND, ERR_ENTITY_NOT_FOUND, ERR_INTERNAL_ERROR,
    ERR_INVALID_CONTEXT, ERR_INVALID_STATE,
};
use crate::ecs::components::animation_controller::AnimationController;
use crate::ecs::components::AnimationClip;
use crate::ecs::Entity;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::str_from_raw;

/// Creates an `AnimationController` component on `entity_id` with an
/// empty initial state named `""`.
#[no_mangle]
pub extern "C" fn goud_animation_controller_create(
    context_id: GoudContextId,
    entity_id: u64,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -ERR_INVALID_CONTEXT;
        }
    };

    let entity = Entity::from_bits(entity_id);
    let world = context.world_mut();

    if !world.is_alive(entity) {
        set_last_error(GoudError::EntityNotFound);
        return -ERR_ENTITY_NOT_FOUND;
    }

    let controller = AnimationController::new("");
    world.insert::<AnimationController>(entity, controller);
    0
}

/// Adds a named animation state backed by a clip with `clip_index` frames
/// of zero-size (placeholder). SDKs should configure the clip separately.
///
/// # Safety
///
/// `state_name_ptr` must point to valid UTF-8 of `state_name_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_controller_add_state(
    context_id: GoudContextId,
    entity_id: u64,
    state_name_ptr: *const u8,
    state_name_len: i32,
    clip_index: i32,
) -> i32 {
    let name = match str_from_raw(state_name_ptr, state_name_len) {
        Ok(s) => s,
        Err(code) => return code,
    };

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -ERR_INVALID_CONTEXT;
        }
    };

    let entity = Entity::from_bits(entity_id);
    let world = context.world_mut();

    let controller = match world.get_mut::<AnimationController>(entity) {
        Some(c) => c,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    // Create a placeholder clip with `clip_index` empty frames.
    let frame_count = clip_index.max(1) as usize;
    let frames = vec![crate::core::math::Rect::new(0.0, 0.0, 0.0, 0.0); frame_count];
    let clip = AnimationClip::new(frames, 0.1);
    controller.states.insert(
        name.to_string(),
        crate::ecs::components::animation_controller::AnimationState { clip },
    );
    0
}

/// Adds a transition from one state to another with a trigger name stored
/// as a boolean condition parameter.
///
/// # Safety
///
/// All string pointers must be valid UTF-8 of the given lengths.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_controller_add_transition(
    context_id: GoudContextId,
    entity_id: u64,
    from_ptr: *const u8,
    from_len: i32,
    to_ptr: *const u8,
    to_len: i32,
    trigger_ptr: *const u8,
    trigger_len: i32,
) -> i32 {
    let from = match str_from_raw(from_ptr, from_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let to = match str_from_raw(to_ptr, to_len) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let trigger = match str_from_raw(trigger_ptr, trigger_len) {
        Ok(s) => s,
        Err(code) => return code,
    };

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -ERR_INVALID_CONTEXT;
        }
    };

    let entity = Entity::from_bits(entity_id);
    let world = context.world_mut();

    let controller = match world.get_mut::<AnimationController>(entity) {
        Some(c) => c,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    use crate::ecs::components::animation_controller::{AnimationTransition, TransitionCondition};

    controller.transitions.push(AnimationTransition {
        from: from.to_string(),
        to: to.to_string(),
        conditions: vec![TransitionCondition::BoolEquals {
            param: trigger.to_string(),
            value: true,
        }],
        blend_duration: 0.0,
    });
    0
}

/// Sets the current state of the animation controller.
///
/// # Safety
///
/// `state_ptr` must point to valid UTF-8 of `state_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_controller_set_state(
    context_id: GoudContextId,
    entity_id: u64,
    state_ptr: *const u8,
    state_len: i32,
) -> i32 {
    let state = match str_from_raw(state_ptr, state_len) {
        Ok(s) => s,
        Err(code) => return code,
    };

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -ERR_INVALID_CONTEXT;
        }
    };

    let entity = Entity::from_bits(entity_id);
    let world = context.world_mut();

    let controller = match world.get_mut::<AnimationController>(entity) {
        Some(c) => c,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    controller.current_state = state.to_string();
    controller.transition_progress = None;
    0
}

/// Copies the current state name into `out_buf`. Returns the number of
/// bytes written on success, or a negative error code.
///
/// If `buf_len` is too small the name is truncated.
///
/// # Safety
///
/// `out_buf` must point to writable memory of at least `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_controller_get_state(
    context_id: GoudContextId,
    entity_id: u64,
    out_buf: *mut u8,
    buf_len: i32,
) -> i32 {
    if out_buf.is_null() || buf_len <= 0 {
        set_last_error(GoudError::InvalidState(
            "output buffer is null or empty".into(),
        ));
        return -ERR_INVALID_STATE;
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -ERR_INVALID_CONTEXT;
        }
    };

    let entity = Entity::from_bits(entity_id);
    let world = context.world();

    let controller = match world.get::<AnimationController>(entity) {
        Some(c) => c,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    let name_bytes = controller.current_state.as_bytes();
    let copy_len = name_bytes.len().min(buf_len as usize);
    // SAFETY: out_buf is non-null and has at least buf_len bytes.
    std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), out_buf, copy_len);
    copy_len as i32
}

/// Advances the animation controller for a single entity by `dt` seconds.
///
/// Runs the same three-phase logic as the global system but scoped to one
/// entity, so only the specified entity is affected.
#[no_mangle]
pub extern "C" fn goud_animation_controller_update(
    context_id: GoudContextId,
    entity_id: u64,
    dt: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return -ERR_INTERNAL_ERROR;
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -ERR_INVALID_CONTEXT;
        }
    };

    let entity = Entity::from_bits(entity_id);
    let world = context.world_mut();

    // Phase 1: Read controller state and decide action.
    use crate::ecs::components::animation_controller::{
        AnimParam, TransitionCondition, TransitionProgress,
    };
    use crate::ecs::components::sprite_animator::SpriteAnimator;

    enum Action {
        None,
        AdvanceTransition { new_elapsed: f32 },
        CompleteTransition { to_state: String },
        StartTransition { to_state: String, duration: f32 },
    }

    let action = {
        let controller = match world.get::<AnimationController>(entity) {
            Some(c) => c,
            None => {
                set_last_error(GoudError::ComponentNotFound);
                return -ERR_COMPONENT_NOT_FOUND;
            }
        };

        if let Some(ref progress) = controller.transition_progress {
            let new_elapsed = progress.elapsed + dt;
            if new_elapsed >= progress.duration {
                Action::CompleteTransition {
                    to_state: progress.to_state.clone(),
                }
            } else {
                Action::AdvanceTransition { new_elapsed }
            }
        } else {
            // Check for matching transitions from current state
            let mut found = Action::None;
            for transition in &controller.transitions {
                if transition.from != controller.current_state {
                    continue;
                }
                let all_met = transition.conditions.iter().all(|c| match c {
                    TransitionCondition::BoolEquals { param, value } => {
                        matches!(
                            controller.parameters.get(param),
                            Some(AnimParam::Bool(v)) if *v == *value
                        )
                    }
                    TransitionCondition::FloatGreaterThan { param, threshold } => {
                        matches!(
                            controller.parameters.get(param),
                            Some(AnimParam::Float(v)) if *v > *threshold
                        )
                    }
                    TransitionCondition::FloatLessThan { param, threshold } => {
                        matches!(
                            controller.parameters.get(param),
                            Some(AnimParam::Float(v)) if *v < *threshold
                        )
                    }
                });
                if all_met {
                    found = Action::StartTransition {
                        to_state: transition.to.clone(),
                        duration: transition.blend_duration,
                    };
                    break;
                }
            }
            found
        }
    };

    // Phase 2: Apply the action.
    match action {
        Action::None => {}
        Action::AdvanceTransition { new_elapsed } => {
            if let Some(ctrl) = world.get_mut::<AnimationController>(entity) {
                if let Some(ref mut progress) = ctrl.transition_progress {
                    progress.elapsed = new_elapsed;
                }
            }
        }
        Action::CompleteTransition { to_state } => {
            let new_clip = {
                let ctrl = world.get_mut::<AnimationController>(entity);
                if let Some(ctrl) = ctrl {
                    ctrl.current_state = to_state;
                    ctrl.transition_progress = None;
                    ctrl.current_clip().cloned()
                } else {
                    None
                }
            };
            if let Some(clip) = new_clip {
                if let Some(animator) = world.get_mut::<SpriteAnimator>(entity) {
                    animator.clip = clip;
                    animator.current_frame = 0;
                    animator.elapsed = 0.0;
                }
            }
        }
        Action::StartTransition { to_state, duration } => {
            if duration <= 0.0 {
                let new_clip = {
                    let ctrl = world.get_mut::<AnimationController>(entity);
                    if let Some(ctrl) = ctrl {
                        ctrl.current_state = to_state;
                        ctrl.transition_progress = None;
                        ctrl.current_clip().cloned()
                    } else {
                        None
                    }
                };
                if let Some(clip) = new_clip {
                    if let Some(animator) = world.get_mut::<SpriteAnimator>(entity) {
                        animator.clip = clip;
                        animator.current_frame = 0;
                        animator.elapsed = 0.0;
                    }
                }
            } else if let Some(ctrl) = world.get_mut::<AnimationController>(entity) {
                ctrl.transition_progress = Some(TransitionProgress {
                    from_state: ctrl.current_state.clone(),
                    to_state,
                    elapsed: 0.0,
                    duration,
                });
            }
        }
    }

    // Phase 3: Sync animator clip with current state.
    let current_clip = {
        let ctrl = world.get::<AnimationController>(entity);
        if let Some(ctrl) = ctrl {
            if ctrl.transition_progress.is_some() {
                None
            } else {
                ctrl.current_clip().cloned()
            }
        } else {
            None
        }
    };
    if let Some(clip) = current_clip {
        if let Some(animator) = world.get_mut::<SpriteAnimator>(entity) {
            if animator.clip != clip {
                animator.clip = clip;
                animator.current_frame = 0;
                animator.elapsed = 0.0;
            }
        }
    }

    0
}
