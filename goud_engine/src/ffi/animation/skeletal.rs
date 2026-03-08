//! Skeleton2D FFI functions.
//!
//! Provides C-compatible functions for creating and managing 2D skeletal
//! animation components on entities.

use crate::core::error::{
    set_last_error, GoudError, ERR_COMPONENT_NOT_FOUND, ERR_ENTITY_NOT_FOUND, ERR_INTERNAL_ERROR,
    ERR_INVALID_CONTEXT, ERR_INVALID_STATE,
};
use crate::core::math::Vec2;
use crate::ecs::components::skeleton2d::{
    Bone2D, BoneTransform, SkeletalAnimation, SkeletalAnimator, Skeleton2D,
};
use crate::ecs::Entity;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::str_from_raw;

/// Creates an empty `Skeleton2D` component on the given entity.
#[no_mangle]
pub extern "C" fn goud_skeleton_create(context_id: GoudContextId, entity_id: u64) -> i32 {
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

    let skeleton = Skeleton2D::new(Vec::new());
    world.insert::<Skeleton2D>(entity, skeleton);
    0
}

/// Adds a bone to the skeleton on `entity_id`. Returns the bone index
/// (>= 0) on success, or a negative error code.
///
/// `parent_index` of -1 means root bone (no parent).
///
/// # Safety
///
/// `bone_name_ptr` must point to valid UTF-8 of `bone_name_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_skeleton_add_bone(
    context_id: GoudContextId,
    entity_id: u64,
    bone_name_ptr: *const u8,
    bone_name_len: i32,
    parent_index: i32,
    x: f32,
    y: f32,
    rotation: f32,
) -> i32 {
    let name = match str_from_raw(bone_name_ptr, bone_name_len) {
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

    let skeleton = match world.get_mut::<Skeleton2D>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    let bone_id = skeleton.bones.len();
    let parent_id = if parent_index < 0 {
        None
    } else {
        Some(parent_index as usize)
    };

    let local_transform = BoneTransform {
        position: Vec2::new(x, y),
        rotation,
        scale: Vec2::one(),
    };

    let bone = Bone2D {
        id: bone_id,
        name: name.to_string(),
        parent_id,
        local_transform,
        bind_pose_inverse: BoneTransform::default(),
    };

    // Compute world transform for the new bone.
    let world_transform = match parent_id {
        Some(pid) if pid < skeleton.world_transforms.len() => {
            BoneTransform::combine(&skeleton.world_transforms[pid], &local_transform)
        }
        _ => local_transform,
    };

    skeleton.bones.push(bone);
    skeleton.world_transforms.push(world_transform);

    bone_id as i32
}

/// Sets the local transform of a bone by index.
#[no_mangle]
pub extern "C" fn goud_skeleton_set_bone_transform(
    context_id: GoudContextId,
    entity_id: u64,
    bone_index: i32,
    x: f32,
    y: f32,
    rotation: f32,
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

    let skeleton = match world.get_mut::<Skeleton2D>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    if bone_index < 0 || (bone_index as usize) >= skeleton.bones.len() {
        set_last_error(GoudError::InvalidState(format!(
            "bone index {} out of range",
            bone_index
        )));
        return -ERR_INVALID_STATE;
    }

    let idx = bone_index as usize;
    skeleton.bones[idx].local_transform = BoneTransform {
        position: Vec2::new(x, y),
        rotation,
        scale: skeleton.bones[idx].local_transform.scale,
    };
    0
}

/// Plays a skeletal animation clip by name on the entity. Always creates a
/// new `SkeletalAnimator`, replacing any existing one. Previous animator
/// state (loaded clips, playback position) is discarded.
/// The `looping` flag controls whether the clip loops.
///
/// # Safety
///
/// `clip_name_ptr` must point to valid UTF-8 of `clip_name_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_skeleton_play_clip(
    context_id: GoudContextId,
    entity_id: u64,
    clip_name_ptr: *const u8,
    clip_name_len: i32,
    looping: bool,
) -> i32 {
    let clip_name = match str_from_raw(clip_name_ptr, clip_name_len) {
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

    if !world.is_alive(entity) {
        set_last_error(GoudError::EntityNotFound);
        return -ERR_ENTITY_NOT_FOUND;
    }

    let animation = SkeletalAnimation {
        name: clip_name.to_string(),
        duration: 1.0, // Default duration; real clips loaded from assets.
        tracks: Vec::new(),
        looping,
    };

    let mut animator = SkeletalAnimator::new(animation);
    animator.play();
    world.insert::<SkeletalAnimator>(entity, animator);
    0
}
