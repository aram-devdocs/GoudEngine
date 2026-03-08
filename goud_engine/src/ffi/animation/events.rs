//! FFI exports for animation events.
//!
//! Provides C-compatible functions for adding events to animation clips
//! and reading fired animation events from the event queue.

use crate::core::error::{
    set_last_error, GoudError, ERR_COMPONENT_NOT_FOUND, ERR_ENTITY_NOT_FOUND, ERR_INTERNAL_ERROR,
    ERR_INVALID_CONTEXT, ERR_INVALID_STATE,
};
use crate::core::event::Events;
use crate::ecs::components::sprite_animator::events::{
    AnimationEvent, AnimationEventFired, EventPayload,
};
use crate::ecs::components::sprite_animator::SpriteAnimator;
use crate::ecs::Entity;
use crate::ffi::context::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::str_from_raw;

/// Payload type discriminant: no payload.
const PAYLOAD_NONE: u32 = 0;
/// Payload type discriminant: integer payload.
const PAYLOAD_INT: u32 = 1;
/// Payload type discriminant: float payload.
const PAYLOAD_FLOAT: u32 = 2;
/// Payload type discriminant: string payload.
const PAYLOAD_STRING: u32 = 3;

/// Adds an animation event to the clip of an entity's `SpriteAnimator`.
///
/// # Parameters
///
/// - `context_id`: Engine context handle.
/// - `entity_id`: Entity with a `SpriteAnimator` component.
/// - `clip_event_frame`: Frame index at which the event fires.
/// - `name_ptr` / `name_len`: Event name (UTF-8 bytes).
/// - `payload_type`: `0` = None, `1` = Int, `2` = Float, `3` = String.
/// - `payload_int`: Integer value (used when `payload_type == 1`).
/// - `payload_float`: Float value (used when `payload_type == 2`).
/// - `payload_str_ptr` / `payload_str_len`: String value (used when
///   `payload_type == 3`).
///
/// # Safety
///
/// - `name_ptr` must point to valid UTF-8 of `name_len` bytes.
/// - When `payload_type == 3`, `payload_str_ptr` must point to valid
///   UTF-8 of `payload_str_len` bytes.
/// - Caller owns the string memory; this function copies the data.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_clip_add_event(
    context_id: GoudContextId,
    entity_id: u64,
    clip_event_frame: u32,
    name_ptr: *const u8,
    name_len: u32,
    payload_type: u32,
    payload_int: i32,
    payload_float: f32,
    payload_str_ptr: *const u8,
    payload_str_len: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -ERR_INVALID_CONTEXT;
    }

    let name = match str_from_raw(name_ptr, name_len as i32) {
        Ok(s) => s,
        Err(code) => return code,
    };

    let payload = match payload_type {
        PAYLOAD_NONE => EventPayload::None,
        PAYLOAD_INT => EventPayload::Int(payload_int),
        PAYLOAD_FLOAT => EventPayload::Float(payload_float),
        PAYLOAD_STRING => {
            let s = match str_from_raw(payload_str_ptr, payload_str_len as i32) {
                Ok(s) => s,
                Err(code) => return code,
            };
            EventPayload::String(s.to_string())
        }
        _ => {
            set_last_error(GoudError::InvalidState(
                "invalid payload_type (expected 0-3)".into(),
            ));
            return -ERR_INVALID_STATE;
        }
    };

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

    let animator = match world.get_mut::<SpriteAnimator>(entity) {
        Some(a) => a,
        None => {
            set_last_error(GoudError::ComponentNotFound);
            return -ERR_COMPONENT_NOT_FOUND;
        }
    };

    animator.clip.events.push(AnimationEvent::new(
        clip_event_frame as usize,
        name.to_string(),
        payload,
    ));

    0
}

/// Returns the number of fired animation events available for reading.
///
/// The return value is the count of events in the read buffer of the
/// `Events<AnimationEventFired>` resource. Returns a negative error
/// code on failure.
#[no_mangle]
pub extern "C" fn goud_animation_events_count(context_id: GoudContextId) -> i32 {
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

    let world = context.world();
    match world.get_resource::<Events<AnimationEventFired>>() {
        Some(events) => events.read_len() as i32,
        None => 0,
    }
}

/// Reads a fired animation event by index from the read buffer.
///
/// # Parameters
///
/// - `context_id`: Engine context handle.
/// - `index`: Zero-based index into the event read buffer.
/// - `out_entity`: Receives the entity ID whose animation fired the event.
/// - `out_name_ptr`: Receives a pointer to the event name (UTF-8, NOT
///   null-terminated). Valid until the next event system update.
/// - `out_name_len`: Receives the byte length of the event name.
/// - `out_frame`: Receives the frame index that triggered the event.
/// - `out_payload_type`: Receives the payload discriminant (`0`=None,
///   `1`=Int, `2`=Float, `3`=String).
/// - `out_payload_int`: Receives the integer payload (valid when type==1).
/// - `out_payload_float`: Receives the float payload (valid when type==2).
/// - `out_payload_str_ptr`: Receives a pointer to the string payload
///   (valid when type==3). Borrows from the event buffer.
/// - `out_payload_str_len`: Receives the byte length of the string payload.
///
/// Returns `0` on success, or a negative error code.
///
/// # Safety
///
/// All output pointers must be non-null and point to writable memory.
/// The returned `out_name_ptr` and `out_payload_str_ptr` borrow from
/// the event buffer and are only valid until
/// `Events<AnimationEventFired>::update()` is called.
#[no_mangle]
pub unsafe extern "C" fn goud_animation_events_read(
    context_id: GoudContextId,
    index: u32,
    out_entity: *mut u64,
    out_name_ptr: *mut *const u8,
    out_name_len: *mut u32,
    out_frame: *mut u32,
    out_payload_type: *mut u32,
    out_payload_int: *mut i32,
    out_payload_float: *mut f32,
    out_payload_str_ptr: *mut *const u8,
    out_payload_str_len: *mut u32,
) -> i32 {
    // Null-check all output pointers.
    if out_entity.is_null() {
        set_last_error(GoudError::InvalidState("out_entity pointer is null".into()));
        return -ERR_INVALID_STATE;
    }
    if out_name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_name_ptr pointer is null".into(),
        ));
        return -ERR_INVALID_STATE;
    }
    if out_name_len.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_name_len pointer is null".into(),
        ));
        return -ERR_INVALID_STATE;
    }
    if out_frame.is_null() {
        set_last_error(GoudError::InvalidState("out_frame pointer is null".into()));
        return -ERR_INVALID_STATE;
    }
    if out_payload_type.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_payload_type pointer is null".into(),
        ));
        return -ERR_INVALID_STATE;
    }
    if out_payload_int.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_payload_int pointer is null".into(),
        ));
        return -ERR_INVALID_STATE;
    }
    if out_payload_float.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_payload_float pointer is null".into(),
        ));
        return -ERR_INVALID_STATE;
    }
    if out_payload_str_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_payload_str_ptr pointer is null".into(),
        ));
        return -ERR_INVALID_STATE;
    }
    if out_payload_str_len.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_payload_str_len pointer is null".into(),
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

    let world = context.world();
    let events = match world.get_resource::<Events<AnimationEventFired>>() {
        Some(e) => e,
        None => {
            set_last_error(GoudError::InvalidState(
                "AnimationEventFired events resource not found".into(),
            ));
            return -ERR_INVALID_STATE;
        }
    };

    let buffer = events.read_buffer();
    let idx = index as usize;
    if idx >= buffer.len() {
        set_last_error(GoudError::InvalidState("event index out of range".into()));
        return -ERR_INVALID_STATE;
    }

    let event = &buffer[idx];

    // SAFETY: All output pointers were validated as non-null above.
    // Caller is responsible for keeping them valid.
    *out_entity = event.entity.to_bits();
    *out_name_ptr = event.event_name.as_ptr();
    *out_name_len = event.event_name.len() as u32;
    *out_frame = event.frame_index as u32;

    // Write payload fields based on the variant.
    match &event.payload {
        EventPayload::None => {
            *out_payload_type = PAYLOAD_NONE;
            *out_payload_int = 0;
            *out_payload_float = 0.0;
            *out_payload_str_ptr = std::ptr::null();
            *out_payload_str_len = 0;
        }
        EventPayload::Int(v) => {
            *out_payload_type = PAYLOAD_INT;
            *out_payload_int = *v;
            *out_payload_float = 0.0;
            *out_payload_str_ptr = std::ptr::null();
            *out_payload_str_len = 0;
        }
        EventPayload::Float(v) => {
            *out_payload_type = PAYLOAD_FLOAT;
            *out_payload_int = 0;
            *out_payload_float = *v;
            *out_payload_str_ptr = std::ptr::null();
            *out_payload_str_len = 0;
        }
        EventPayload::String(s) => {
            *out_payload_type = PAYLOAD_STRING;
            *out_payload_int = 0;
            *out_payload_float = 0.0;
            *out_payload_str_ptr = s.as_ptr();
            *out_payload_str_len = s.len() as u32;
        }
    }

    0
}
