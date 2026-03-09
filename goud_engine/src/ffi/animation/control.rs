//! FFI exports for animation control operations.

use crate::core::error::{
    set_last_error, GoudError, ERR_COMPONENT_NOT_FOUND, ERR_INTERNAL_ERROR, ERR_INVALID_CONTEXT,
};
use crate::ecs::components::animation_controller::AnimationController;
use crate::ecs::components::sprite_animator::SpriteAnimator;
use crate::ecs::Entity;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::str_from_raw;

/// Sets a lock-failure error and returns the internal error code.
fn lock_error() -> i32 {
    set_last_error(GoudError::InternalError(
        "Failed to lock context registry".to_string(),
    ));
    -ERR_INTERNAL_ERROR
}

/// Starts (or restarts) sprite animation playback from frame 0.
#[no_mangle]
pub extern "C" fn goud_animation_play(context_id: GoudContextId, entity_id: u64) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return lock_error(),
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

    let animator = match world.get_mut::<SpriteAnimator>(entity) {
        Some(a) => a,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    animator.play();
    0
}

/// Stops sprite animation playback and resets to frame 0.
#[no_mangle]
pub extern "C" fn goud_animation_stop(context_id: GoudContextId, entity_id: u64) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return lock_error(),
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

    let animator = match world.get_mut::<SpriteAnimator>(entity) {
        Some(a) => a,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    animator.reset();
    0
}

/// Sets `AnimationController.current_state`.
/// # Safety
/// `state_ptr` must point to valid UTF-8 of `state_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_set_state(
    context_id: GoudContextId,
    entity_id: u64,
    state_ptr: *const u8,
    state_len: i32,
) -> i32 {
    // SAFETY: FFI caller provides pointer/length pair; `str_from_raw` performs
    // null, length, and UTF-8 validation before returning a borrowed `&str`.
    let state = match unsafe { str_from_raw(state_ptr, state_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return lock_error(),
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

/// Sets a boolean `AnimationController` parameter.
/// # Safety
/// `name_ptr` must point to valid UTF-8 of `name_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_set_parameter_bool(
    context_id: GoudContextId,
    entity_id: u64,
    name_ptr: *const u8,
    name_len: i32,
    value: bool,
) -> i32 {
    // SAFETY: FFI caller provides pointer/length pair; `str_from_raw` performs
    // null, length, and UTF-8 validation before returning a borrowed `&str`.
    let name = match unsafe { str_from_raw(name_ptr, name_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return lock_error(),
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

    controller.set_bool(name, value);
    0
}

/// Sets a float `AnimationController` parameter.
/// # Safety
/// `name_ptr` must point to valid UTF-8 of `name_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_set_parameter_float(
    context_id: GoudContextId,
    entity_id: u64,
    name_ptr: *const u8,
    name_len: i32,
    value: f32,
) -> i32 {
    // SAFETY: FFI caller provides pointer/length pair; `str_from_raw` performs
    // null, length, and UTF-8 validation before returning a borrowed `&str`.
    let name = match unsafe { str_from_raw(name_ptr, name_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return lock_error(),
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

    controller.set_float(name, value);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::{ERR_COMPONENT_NOT_FOUND, ERR_INVALID_CONTEXT};
    use crate::core::math::Rect;
    use crate::ecs::components::animation_controller::{AnimParam, TransitionProgress};
    use crate::ecs::components::sprite_animator::AnimationClip;
    use crate::ffi::context::{goud_context_create, goud_context_destroy};
    use crate::ffi::entity::goud_entity_spawn_empty;

    fn setup_context() -> GoudContextId {
        goud_context_create()
    }

    fn teardown_context(context_id: GoudContextId) {
        goud_context_destroy(context_id);
    }

    fn ffi_set_state(context_id: GoudContextId, entity_id: u64, state: &[u8]) -> i32 {
        // SAFETY: State bytes come from a valid test slice.
        unsafe {
            goud_animation_set_state(context_id, entity_id, state.as_ptr(), state.len() as i32)
        }
    }

    fn ffi_set_param_bool(
        context_id: GoudContextId,
        entity_id: u64,
        name: &[u8],
        value: bool,
    ) -> i32 {
        // SAFETY: Parameter name bytes come from a valid test slice.
        unsafe {
            goud_animation_set_parameter_bool(
                context_id,
                entity_id,
                name.as_ptr(),
                name.len() as i32,
                value,
            )
        }
    }

    fn ffi_set_param_float(
        context_id: GoudContextId,
        entity_id: u64,
        name: &[u8],
        value: f32,
    ) -> i32 {
        // SAFETY: Parameter name bytes come from a valid test slice.
        unsafe {
            goud_animation_set_parameter_float(
                context_id,
                entity_id,
                name.as_ptr(),
                name.len() as i32,
                value,
            )
        }
    }

    fn with_world_mut<F>(context_id: GoudContextId, f: F)
    where
        F: FnOnce(&mut crate::ecs::World),
    {
        let mut registry = get_context_registry()
            .lock()
            .expect("context registry lock should succeed in tests");
        let context = registry
            .get_mut(context_id)
            .expect("test context should exist");
        f(context.world_mut());
    }

    fn read_controller(context_id: GoudContextId, entity_id: u64) -> AnimationController {
        let registry = get_context_registry()
            .lock()
            .expect("context registry lock should succeed in tests");
        let context = registry.get(context_id).expect("test context should exist");
        let entity = Entity::from_bits(entity_id);
        context
            .world()
            .get::<AnimationController>(entity)
            .expect("controller should exist")
            .clone()
    }

    fn read_animator(context_id: GoudContextId, entity_id: u64) -> SpriteAnimator {
        let registry = get_context_registry()
            .lock()
            .expect("context registry lock should succeed in tests");
        let context = registry.get(context_id).expect("test context should exist");
        let entity = Entity::from_bits(entity_id);
        context
            .world()
            .get::<SpriteAnimator>(entity)
            .expect("animator should exist")
            .clone()
    }

    #[test]
    fn test_animation_control_invalid_context() {
        let entity_id = 1_u64;
        let state = b"idle";
        let param = b"speed";

        assert_eq!(
            goud_animation_play(GOUD_INVALID_CONTEXT_ID, entity_id),
            -ERR_INVALID_CONTEXT
        );
        assert_eq!(
            goud_animation_stop(GOUD_INVALID_CONTEXT_ID, entity_id),
            -ERR_INVALID_CONTEXT
        );
        assert_eq!(
            ffi_set_state(GOUD_INVALID_CONTEXT_ID, entity_id, state),
            -ERR_INVALID_CONTEXT
        );
        assert_eq!(
            ffi_set_param_bool(GOUD_INVALID_CONTEXT_ID, entity_id, param, true),
            -ERR_INVALID_CONTEXT
        );
        assert_eq!(
            ffi_set_param_float(GOUD_INVALID_CONTEXT_ID, entity_id, param, 1.0),
            -ERR_INVALID_CONTEXT
        );
    }

    #[test]
    fn test_animation_control_missing_required_component() {
        let context_id = setup_context();
        let entity_id = goud_entity_spawn_empty(context_id);
        let state = b"run";
        let param = b"moving";

        assert_eq!(
            goud_animation_play(context_id, entity_id),
            -ERR_COMPONENT_NOT_FOUND
        );
        assert_eq!(
            goud_animation_stop(context_id, entity_id),
            -ERR_COMPONENT_NOT_FOUND
        );
        assert_eq!(
            ffi_set_state(context_id, entity_id, state),
            -ERR_COMPONENT_NOT_FOUND
        );
        assert_eq!(
            ffi_set_param_bool(context_id, entity_id, param, true),
            -ERR_COMPONENT_NOT_FOUND
        );
        assert_eq!(
            ffi_set_param_float(context_id, entity_id, param, 2.5),
            -ERR_COMPONENT_NOT_FOUND
        );

        teardown_context(context_id);
    }

    #[test]
    fn test_animation_control_set_state_and_parameters_happy_path() {
        let context_id = setup_context();
        let entity_id = goud_entity_spawn_empty(context_id);
        let entity = Entity::from_bits(entity_id);

        with_world_mut(context_id, |world| {
            let mut controller = AnimationController::new("idle");
            controller.transition_progress = Some(TransitionProgress {
                from_state: "idle".to_string(),
                to_state: "run".to_string(),
                elapsed: 0.25,
                duration: 1.0,
            });
            world.insert::<AnimationController>(entity, controller);
        });

        let state = b"run";
        let bool_name = b"is_grounded";
        let float_name = b"speed";

        assert_eq!(ffi_set_state(context_id, entity_id, state), 0);
        assert_eq!(
            ffi_set_param_bool(context_id, entity_id, bool_name, true),
            0
        );
        assert_eq!(
            ffi_set_param_float(context_id, entity_id, float_name, 3.5),
            0
        );

        let controller = read_controller(context_id, entity_id);
        assert_eq!(controller.current_state_name(), "run");
        assert!(controller.transition_progress.is_none());
        assert_eq!(
            controller.get_param("is_grounded"),
            Some(&AnimParam::Bool(true))
        );
        assert_eq!(controller.get_param("speed"), Some(&AnimParam::Float(3.5)));

        teardown_context(context_id);
    }

    #[test]
    fn test_animation_control_play_stop_mutates_sprite_animator() {
        let context_id = setup_context();
        let entity_id = goud_entity_spawn_empty(context_id);
        let entity = Entity::from_bits(entity_id);

        with_world_mut(context_id, |world| {
            let clip = AnimationClip::new(vec![Rect::new(0.0, 0.0, 16.0, 16.0)], 0.1);
            let mut animator = SpriteAnimator::new(clip);
            animator.current_frame = 2;
            animator.previous_frame = 1;
            animator.elapsed = 0.75;
            animator.playing = false;
            animator.finished = true;
            world.insert::<SpriteAnimator>(entity, animator);
        });

        assert_eq!(goud_animation_play(context_id, entity_id), 0);

        let animator = read_animator(context_id, entity_id);
        assert_eq!(animator.current_frame, 0);
        assert_eq!(animator.previous_frame, 0);
        assert!((animator.elapsed - 0.0).abs() < f32::EPSILON);
        assert!(animator.playing);
        assert!(!animator.finished);

        with_world_mut(context_id, |world| {
            let animator = world
                .get_mut::<SpriteAnimator>(entity)
                .expect("animator should exist");
            animator.current_frame = 4;
            animator.previous_frame = 3;
            animator.elapsed = 1.25;
            animator.playing = true;
            animator.finished = true;
        });

        assert_eq!(goud_animation_stop(context_id, entity_id), 0);

        let animator = read_animator(context_id, entity_id);
        assert_eq!(animator.current_frame, 0);
        assert_eq!(animator.previous_frame, 0);
        assert!((animator.elapsed - 0.0).abs() < f32::EPSILON);
        assert!(!animator.playing);
        assert!(!animator.finished);

        teardown_context(context_id);
    }
}
