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
        Err(_) => return -ERR_INTERNAL_ERROR,
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
        Err(_) => return -ERR_INTERNAL_ERROR,
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
        Err(_) => return -ERR_INTERNAL_ERROR,
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
        Err(_) => return -ERR_INTERNAL_ERROR,
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
        Err(_) => return -ERR_INTERNAL_ERROR,
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

/// Advances the animation controller system for a single entity by `dt`
/// seconds. Delegates to the existing system logic.
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
        Err(_) => return -ERR_INTERNAL_ERROR,
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

    if !world.has::<AnimationController>(entity) {
        set_last_error(GoudError::ComponentNotFound);
        return -ERR_COMPONENT_NOT_FOUND;
    }

    // Use the full system update (operates on all matching entities).
    // A per-entity update could be added later for finer control.
    crate::ecs::systems::update_animation_controllers(world, dt);
    0
}
