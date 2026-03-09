//! Collision event bridging for 2D physics FFI.
//!
//! Provides callback registration and deterministic pull APIs backed by events
//! captured from `PhysicsProvider::drain_collision_events()` during
//! `goud_physics_step`.

use std::ffi::c_void;

use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::types::CollisionEventKind;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::physics2d_common::with_provider_mut;
use super::physics2d_state::{
    capture_step_collision_events, collision_event_at, collision_event_count,
    set_collision_callback, CollisionCallback,
};

fn collision_kind_to_ffi(kind: CollisionEventKind) -> u32 {
    match kind {
        CollisionEventKind::Enter => 0,
        CollisionEventKind::Stay => 1,
        CollisionEventKind::Exit => 2,
    }
}

pub(super) fn step_and_dispatch_collision_events(ctx: GoudContextId, dt: f32) -> i32 {
    let mut drained_events = Vec::new();

    let code = with_provider_mut(ctx, |p| match p.step(dt) {
        Ok(()) => {
            drained_events = p.drain_collision_events();
            0
        }
        Err(e) => {
            let err_code = e.error_code();
            set_last_error(e);
            err_code
        }
    });

    if code < 0 {
        return code;
    }

    let (events, callback) = capture_step_collision_events(ctx, drained_events);
    if let Some((callback_fn, user_data_bits)) = callback {
        let user_data = user_data_bits as *mut c_void;
        for event in events {
            callback_fn(
                ctx,
                event.body_a.0,
                event.body_b.0,
                collision_kind_to_ffi(event.kind),
                user_data,
            );
        }
    }

    0
}

/// Registers or clears a per-context collision callback.
///
/// Pass a null `callback` pointer to clear the callback.
///
/// Callback signature: `extern "C" fn(ctx, body_a, body_b, kind, user_data)`,
/// where kind is `0 = Enter`, `1 = Stay`, `2 = Exit`.
///
/// Ownership: `user_data` remains owned by the caller and must outlive callback
/// invocation. The engine stores and forwards the raw pointer; it never frees it.
#[no_mangle]
pub extern "C" fn goud_physics_set_collision_callback(
    ctx: GoudContextId,
    callback: *mut c_void,
    user_data: *mut c_void,
) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let callback_fn = if callback.is_null() {
        None
    } else {
        // SAFETY: The caller must provide a valid function pointer with the
        // `CollisionCallback` ABI and signature.
        Some(unsafe { std::mem::transmute::<*mut c_void, CollisionCallback>(callback) })
    };

    set_collision_callback(ctx, callback_fn, user_data);
    0
}

fn collision_events_count_impl(ctx: GoudContextId) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    let count = collision_event_count(ctx);
    if count > i32::MAX as usize {
        set_last_error(GoudError::InternalError(
            "collision event count overflow".to_string(),
        ));
        return GoudError::InternalError(String::new()).error_code();
    }

    count as i32
}

/// Returns the number of filtered collision events captured by the last
/// `goud_physics_step` call for this context.
///
/// Returns a non-negative count or a negative error code.
#[no_mangle]
pub extern "C" fn goud_physics_collision_events_count(ctx: GoudContextId) -> i32 {
    collision_events_count_impl(ctx)
}

/// Backward-compatible alias for `goud_physics_collision_events_count`.
#[no_mangle]
pub extern "C" fn goud_physics_collision_event_count(ctx: GoudContextId) -> i32 {
    collision_events_count_impl(ctx)
}

fn collision_events_read_impl(
    ctx: GoudContextId,
    index: u32,
    out_body_a: *mut u64,
    out_body_b: *mut u64,
    out_kind: *mut u32,
) -> i32 {
    if ctx == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudError::InvalidContext.error_code();
    }

    if out_body_a.is_null() || out_body_b.is_null() || out_kind.is_null() {
        set_last_error(GoudError::InvalidState(
            "one or more output pointers are null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    let Some(event) = collision_event_at(ctx, index as usize) else {
        return 0;
    };

    // SAFETY: Null checks above guarantee output pointers are valid and writable.
    unsafe {
        *out_body_a = event.body_a.0;
        *out_body_b = event.body_b.0;
        *out_kind = collision_kind_to_ffi(event.kind);
    }
    1
}

/// Reads one filtered collision event by index.
///
/// # Safety
///
/// `out_body_a`, `out_body_b`, and `out_kind` must be valid, non-null
/// writable pointers. Ownership is not transferred.
/// Kind mapping: 0 = Enter, 1 = Stay, 2 = Exit.
///
/// # Returns
///
/// 1 if event exists, 0 if index is out of range, negative on error.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_collision_events_read(
    ctx: GoudContextId,
    index: u32,
    out_body_a: *mut u64,
    out_body_b: *mut u64,
    out_kind: *mut u32,
) -> i32 {
    collision_events_read_impl(ctx, index, out_body_a, out_body_b, out_kind)
}

/// Backward-compatible alias for `goud_physics_collision_events_read`.
///
/// # Safety
///
/// Same as `goud_physics_collision_events_read`.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_collision_event_read(
    ctx: GoudContextId,
    index: u32,
    out_body_a: *mut u64,
    out_body_b: *mut u64,
    out_kind: *mut u32,
) -> i32 {
    collision_events_read_impl(ctx, index, out_body_a, out_body_b, out_kind)
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::ptr::null_mut;

    use super::goud_physics_set_collision_callback;
    use crate::ffi::context::{
        goud_context_create, goud_context_destroy, GoudContextId, GOUD_INVALID_CONTEXT_ID,
    };
    use crate::ffi::physics::physics2d::{
        goud_physics_add_rigid_body, goud_physics_create, goud_physics_destroy,
        goud_physics_set_velocity, goud_physics_step,
    };
    use crate::ffi::physics::physics2d_ex::goud_physics_add_collider_ex;

    #[derive(Debug, Clone, Copy)]
    struct CallbackEvent {
        ctx: GoudContextId,
        body_a: u64,
        body_b: u64,
        kind: u32,
    }

    struct Physics2DContextGuard {
        ctx: GoudContextId,
        has_physics: bool,
    }

    impl Physics2DContextGuard {
        fn new() -> Self {
            let ctx = goud_context_create();
            Self {
                ctx,
                has_physics: false,
            }
        }
    }

    impl Drop for Physics2DContextGuard {
        fn drop(&mut self) {
            if self.has_physics {
                let _ = goud_physics_destroy(self.ctx);
            }
            let _ = goud_context_destroy(self.ctx);
        }
    }

    fn body_pair_matches(event: &CallbackEvent, a: u64, b: u64) -> bool {
        (event.body_a == a && event.body_b == b) || (event.body_a == b && event.body_b == a)
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    extern "C" fn record_collision_event(
        ctx: GoudContextId,
        body_a: u64,
        body_b: u64,
        kind: u32,
        user_data: *mut c_void,
    ) {
        if user_data.is_null() {
            return;
        }

        // SAFETY: The test passes a valid `*mut Vec<CallbackEvent>` pointer
        // as user_data that remains alive for the callback lifetime.
        let events = unsafe { &mut *(user_data as *mut Vec<CallbackEvent>) };
        events.push(CallbackEvent {
            ctx,
            body_a,
            body_b,
            kind,
        });
    }

    #[test]
    fn test_collision_callback_registration_delivers_enter_stay_exit_lifecycle() {
        let mut guard = Physics2DContextGuard::new();
        assert_ne!(guard.ctx, GOUD_INVALID_CONTEXT_ID);
        assert_eq!(goud_physics_create(guard.ctx, 0.0, 0.0), 0);
        guard.has_physics = true;

        let sensor_body = goud_physics_add_rigid_body(guard.ctx, 0, 0.0, 0.0, 0.0);
        assert!(
            sensor_body > 0,
            "expected sensor body creation to succeed, got {sensor_body}"
        );
        let sensor_collider = goud_physics_add_collider_ex(
            guard.ctx,
            sensor_body as u64,
            0,
            0.0,
            0.0,
            2.0,
            0.5,
            0.0,
            true,
            0b0001,
            u32::MAX,
        );
        assert!(
            sensor_collider > 0,
            "expected sensor collider creation to succeed, got {sensor_collider}"
        );

        let dynamic_body = goud_physics_add_rigid_body(guard.ctx, 1, 0.0, 0.0, 0.0);
        assert!(
            dynamic_body > 0,
            "expected dynamic body creation to succeed, got {dynamic_body}"
        );
        let dynamic_collider = goud_physics_add_collider_ex(
            guard.ctx,
            dynamic_body as u64,
            0,
            0.0,
            0.0,
            0.5,
            0.5,
            0.0,
            false,
            0b0001,
            u32::MAX,
        );
        assert!(
            dynamic_collider > 0,
            "expected dynamic collider creation to succeed, got {dynamic_collider}"
        );

        let mut callback_events: Vec<CallbackEvent> = Vec::new();
        let callback_fn_ptr = record_collision_event as *const () as *mut c_void;
        let user_data_ptr = &mut callback_events as *mut Vec<CallbackEvent> as *mut c_void;
        assert_eq!(
            goud_physics_set_collision_callback(guard.ctx, callback_fn_ptr, user_data_ptr),
            0
        );

        assert_eq!(goud_physics_step(guard.ctx, 1.0 / 60.0), 0);
        assert_eq!(goud_physics_step(guard.ctx, 1.0 / 60.0), 0);
        assert_eq!(
            goud_physics_set_velocity(guard.ctx, dynamic_body as u64, 300.0, 0.0),
            0
        );
        assert_eq!(goud_physics_step(guard.ctx, 1.0 / 60.0), 0);

        assert_eq!(
            callback_events.len(),
            3,
            "expected exactly enter/stay/exit callback events"
        );
        assert_eq!(
            callback_events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![0, 1, 2],
            "expected callback kinds in enter/stay/exit order"
        );
        assert!(
            callback_events
                .iter()
                .all(|event| event.ctx == guard.ctx && body_pair_matches(event, sensor_body as u64, dynamic_body as u64)),
            "all callback events should belong to the expected context and body pair, got {callback_events:?}"
        );

        assert_eq!(
            goud_physics_set_collision_callback(guard.ctx, null_mut(), null_mut()),
            0
        );
    }
}
