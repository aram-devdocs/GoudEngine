//! FFI exports for animation layer stacks.
//!
//! Provides C-compatible functions for creating and managing
//! `AnimationLayerStack` components on entities.

use crate::core::error::{
    set_last_error, GoudError, ERR_COMPONENT_NOT_FOUND, ERR_ENTITY_NOT_FOUND, ERR_INTERNAL_ERROR,
    ERR_INVALID_CONTEXT, ERR_INVALID_STATE,
};
use crate::core::math::Rect;
use crate::ecs::components::animation_layer::{AnimationLayer, AnimationLayerStack};
use crate::ecs::components::sprite_animator::{AnimationClip, PlaybackMode};
use crate::ecs::systems::animation::BlendMode;
use crate::ecs::Entity;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::str_from_raw;

/// BlendMode discriminant: Override (replace previous).
const BLEND_MODE_OVERRIDE: u32 = 0;
/// BlendMode discriminant: Additive (add on top).
const BLEND_MODE_ADDITIVE: u32 = 1;
/// PlaybackMode discriminant: Loop.
const PLAYBACK_MODE_LOOP: u32 = 0;
/// PlaybackMode discriminant: OneShot.
const PLAYBACK_MODE_ONESHOT: u32 = 1;

/// Creates an empty `AnimationLayerStack` component on the given entity.
///
/// Returns `0` on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_animation_layer_stack_create(
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

    let stack = AnimationLayerStack::new();
    world.insert::<AnimationLayerStack>(entity, stack);
    0
}

/// Adds a new animation layer to the entity's `AnimationLayerStack`.
///
/// The layer is created with a default empty clip and weight `1.0`.
///
/// # Parameters
///
/// - `context_id`: Engine context handle.
/// - `entity_id`: Entity with an `AnimationLayerStack` component.
/// - `name_ptr` / `name_len`: Layer name (UTF-8 bytes).
/// - `blend_mode`: `0` = Override, `1` = Additive.
///
/// Returns `0` on success, or a negative error code.
///
/// # Safety
///
/// `name_ptr` must point to valid UTF-8 of `name_len` bytes.
/// Caller owns the string memory; this function copies the data.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_layer_add(
    context_id: GoudContextId,
    entity_id: u64,
    name_ptr: *const u8,
    name_len: u32,
    blend_mode: u32,
) -> i32 {
    let name = match str_from_raw(name_ptr, name_len as i32) {
        Ok(s) => s,
        Err(code) => return code,
    };

    let mode = match blend_mode {
        BLEND_MODE_OVERRIDE => BlendMode::Override,
        BLEND_MODE_ADDITIVE => BlendMode::Additive,
        _ => {
            set_last_error(GoudError::InvalidState(
                "invalid blend_mode (expected 0 or 1)".into(),
            ));
            return -ERR_INVALID_STATE;
        }
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

    if !world.is_alive(entity) {
        set_last_error(GoudError::EntityNotFound);
        return -ERR_ENTITY_NOT_FOUND;
    }

    let stack = match world.get_mut::<AnimationLayerStack>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    // Create a default empty clip for the layer.
    let clip = AnimationClip::new(Vec::new(), 0.1);
    let layer = AnimationLayer::new(name.to_string(), clip, mode);
    stack.layers.push(layer);

    0
}

/// Sets the blend weight of an animation layer by index.
///
/// Weight is clamped to `[0.0, 1.0]`.
///
/// Returns `0` on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_animation_layer_set_weight(
    context_id: GoudContextId,
    entity_id: u64,
    layer_index: u32,
    weight: f32,
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

    let stack = match world.get_mut::<AnimationLayerStack>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    let idx = layer_index as usize;
    if idx >= stack.layers.len() {
        set_last_error(GoudError::InvalidState("layer index out of range".into()));
        return -ERR_INVALID_STATE;
    }

    stack.set_layer_weight(idx, weight);
    0
}

/// Starts playback on an animation layer by index.
///
/// Sets `layer.playing = true`. Does nothing if the layer is
/// already finished (OneShot completed).
///
/// Returns `0` on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_animation_layer_play(
    context_id: GoudContextId,
    entity_id: u64,
    layer_index: u32,
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

    let stack = match world.get_mut::<AnimationLayerStack>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    let idx = layer_index as usize;
    if idx >= stack.layers.len() {
        set_last_error(GoudError::InvalidState("layer index out of range".into()));
        return -ERR_INVALID_STATE;
    }

    let layer = &mut stack.layers[idx];
    if !layer.finished {
        layer.playing = true;
    }

    0
}

/// Sets the animation clip on a layer by index.
///
/// Creates a new `AnimationClip` with `frame_count` empty `Rect` frames,
/// the given `frame_duration`, and playback mode.
///
/// # Parameters
///
/// - `context_id`: Engine context handle.
/// - `entity_id`: Entity with an `AnimationLayerStack` component.
/// - `layer_index`: Zero-based index of the layer to update.
/// - `frame_count`: Number of empty frames to initialize the clip with.
/// - `frame_duration`: Seconds per frame.
/// - `mode`: `0` = Loop, `1` = OneShot.
///
/// Returns `0` on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_animation_layer_set_clip(
    context_id: GoudContextId,
    entity_id: u64,
    layer_index: u32,
    frame_count: u32,
    frame_duration: f32,
    mode: u32,
) -> i32 {
    let playback_mode = match mode {
        PLAYBACK_MODE_LOOP => PlaybackMode::Loop,
        PLAYBACK_MODE_ONESHOT => PlaybackMode::OneShot,
        _ => {
            set_last_error(GoudError::InvalidState(
                "invalid playback mode (expected 0 or 1)".into(),
            ));
            return -ERR_INVALID_STATE;
        }
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

    if !world.is_alive(entity) {
        set_last_error(GoudError::EntityNotFound);
        return -ERR_ENTITY_NOT_FOUND;
    }

    let stack = match world.get_mut::<AnimationLayerStack>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    let idx = layer_index as usize;
    if idx >= stack.layers.len() {
        set_last_error(GoudError::InvalidState("layer index out of range".into()));
        return -ERR_INVALID_STATE;
    }

    let frames = vec![Rect::new(0.0, 0.0, 0.0, 0.0); frame_count as usize];
    let clip = AnimationClip::new(frames, frame_duration).with_mode(playback_mode);
    stack.layers[idx].clip = clip;

    0
}

/// Adds a source-rectangle frame to a layer's animation clip.
///
/// Appends a `Rect { x, y, width, height }` frame to the clip on the
/// layer at `layer_index`.
///
/// # Parameters
///
/// - `context_id`: Engine context handle.
/// - `entity_id`: Entity with an `AnimationLayerStack` component.
/// - `layer_index`: Zero-based index of the layer whose clip to extend.
/// - `x`, `y`, `w`, `h`: Source rectangle in pixel coordinates.
///
/// Returns `0` on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_animation_layer_add_frame(
    context_id: GoudContextId,
    entity_id: u64,
    layer_index: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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

    let stack = match world.get_mut::<AnimationLayerStack>(entity) {
        Some(s) => s,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    let idx = layer_index as usize;
    if idx >= stack.layers.len() {
        set_last_error(GoudError::InvalidState("layer index out of range".into()));
        return -ERR_INVALID_STATE;
    }

    stack.layers[idx].clip.frames.push(Rect::new(x, y, w, h));

    0
}
