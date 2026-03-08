//! Tween FFI functions.
//!
//! Provides C-compatible functions for creating and managing standalone
//! value interpolation tweens with easing functions.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};
use crate::core::math::Easing;
use crate::core::math::Tweenable;
use crate::ffi::context::GoudContextId;

// =============================================================================
// Tween Storage (thread-local, handle-based)
// =============================================================================

struct TweenState {
    start: f32,
    end: f32,
    duration: f32,
    elapsed: f32,
    easing: Easing,
}

std::thread_local! {
    static TWEEN_STORE: std::cell::RefCell<HashMap<i64, TweenState>> =
        std::cell::RefCell::new(HashMap::new());
}

static NEXT_TWEEN_HANDLE: AtomicI64 = AtomicI64::new(1);

// =============================================================================
// FFI Functions
// =============================================================================

/// Creates a new tween interpolating from `start` to `end` over
/// `duration` seconds with the given easing type.
///
/// `easing_type`: 0=Linear, 1=EaseIn, 2=EaseOut, 3=EaseInOut,
///                4=EaseInBack, 5=EaseOutBounce.
///
/// Returns a positive handle on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_tween_create(
    _context_id: GoudContextId,
    start: f32,
    end: f32,
    duration: f32,
    easing_type: i32,
) -> i64 {
    let easing = match easing_type {
        0 => Easing::Linear,
        1 => Easing::EaseIn,
        2 => Easing::EaseOut,
        3 => Easing::EaseInOut,
        4 => Easing::EaseInBack,
        5 => Easing::EaseOutBounce,
        _ => {
            set_last_error(GoudError::InvalidState(format!(
                "unknown easing type {}",
                easing_type
            )));
            return -(ERR_INVALID_STATE as i64); // cast needed: i32 -> i64
        }
    };

    let handle = NEXT_TWEEN_HANDLE.fetch_add(1, Ordering::Relaxed);
    let state = TweenState {
        start,
        end,
        duration: duration.max(0.0),
        elapsed: 0.0,
        easing,
    };

    TWEEN_STORE.with(|store| {
        store.borrow_mut().insert(handle, state);
    });

    handle
}

/// Advances a tween by `dt` seconds.
#[no_mangle]
pub extern "C" fn goud_tween_update(_context_id: GoudContextId, handle: i64, dt: f32) -> i32 {
    TWEEN_STORE.with(|store| {
        let mut map = store.borrow_mut();
        match map.get_mut(&handle) {
            Some(tw) => {
                tw.elapsed = (tw.elapsed + dt).min(tw.duration);
                0
            }
            None => {
                set_last_error(GoudError::InvalidState("tween handle not found".into()));
                -ERR_INVALID_STATE
            }
        }
    })
}

/// Writes the current interpolated value of a tween into `out_value`.
///
/// # Safety
///
/// `out_value` must point to a writable `f32`.
#[no_mangle]
pub unsafe extern "C" fn goud_tween_value(
    _context_id: GoudContextId,
    handle: i64,
    out_value: *mut f32,
) -> i32 {
    if out_value.is_null() {
        set_last_error(GoudError::InvalidState("out_value is null".into()));
        return -ERR_INVALID_STATE;
    }

    TWEEN_STORE.with(|store| {
        let map = store.borrow();
        match map.get(&handle) {
            Some(tw) => {
                let t = if tw.duration > 0.0 {
                    (tw.elapsed / tw.duration).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let eased_t = tw.easing.apply(t);
                let value = tw.start.lerp(tw.end, eased_t);
                // SAFETY: out_value is non-null and points to a valid f32.
                *out_value = value;
                0
            }
            None => {
                set_last_error(GoudError::InvalidState("tween handle not found".into()));
                -ERR_INVALID_STATE
            }
        }
    })
}

/// Returns 1 if the tween is complete, 0 if not, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_tween_is_complete(_context_id: GoudContextId, handle: i64) -> i32 {
    TWEEN_STORE.with(|store| {
        let map = store.borrow();
        match map.get(&handle) {
            Some(tw) => i32::from(tw.elapsed >= tw.duration),
            None => {
                set_last_error(GoudError::InvalidState("tween handle not found".into()));
                -ERR_INVALID_STATE
            }
        }
    })
}

/// Resets a tween's elapsed time to zero.
#[no_mangle]
pub extern "C" fn goud_tween_reset(_context_id: GoudContextId, handle: i64) -> i32 {
    TWEEN_STORE.with(|store| {
        let mut map = store.borrow_mut();
        match map.get_mut(&handle) {
            Some(tw) => {
                tw.elapsed = 0.0;
                0
            }
            None => {
                set_last_error(GoudError::InvalidState("tween handle not found".into()));
                -ERR_INVALID_STATE
            }
        }
    })
}

/// Destroys a tween handle, freeing its storage.
/// Returns 0 on success, negative error code if handle not found.
#[no_mangle]
pub extern "C" fn goud_tween_destroy(_context_id: GoudContextId, handle: i64) -> i32 {
    TWEEN_STORE.with(|store| {
        let mut map = store.borrow_mut();
        if map.remove(&handle).is_some() {
            0
        } else {
            set_last_error(GoudError::InvalidState("tween handle not found".into()));
            -ERR_INVALID_STATE
        }
    })
}
