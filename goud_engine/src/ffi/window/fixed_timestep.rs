//! # Fixed Timestep FFI
//!
//! FFI functions for driving a fixed timestep accumulator from SDK game loops.
//! These complement the per-frame `goud_window_poll_events` by letting SDKs
//! run deterministic simulation updates at a fixed rate.

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::state::WINDOW_STATES;

/// Begins the fixed timestep accumulator for this frame.
///
/// Feeds the current frame's delta time into the accumulator and resets the
/// per-frame step counter. Call this once per frame after
/// `goud_window_poll_events`.
///
/// Returns `true` if fixed timestep is enabled (caller should then call
/// `goud_fixed_timestep_step` in a loop), `false` if disabled.
#[no_mangle]
pub extern "C" fn goud_fixed_timestep_begin(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            if state.fixed_timestep <= 0.0 {
                return false;
            }
            state.accumulator += state.delta_time();
            state.fixed_steps_this_frame = 0;
            true
        } else {
            false
        }
    })
}

/// Consumes one fixed step from the accumulator.
///
/// Returns `true` if a step was consumed (caller should run one fixed
/// update tick). Returns `false` when the accumulator is exhausted or the
/// per-frame cap has been reached.
///
/// After this returns `false`, call `goud_fixed_timestep_alpha` to get the
/// interpolation value for render smoothing.
#[no_mangle]
pub extern "C" fn goud_fixed_timestep_step(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            if state.fixed_timestep <= 0.0 {
                return false;
            }
            if state.accumulator >= state.fixed_timestep
                && state.fixed_steps_this_frame < state.max_fixed_steps
            {
                state.accumulator -= state.fixed_timestep;
                state.fixed_steps_this_frame += 1;
                return true;
            }
            // All steps consumed — compute interpolation alpha
            state.interpolation_alpha = state.accumulator / state.fixed_timestep;
            false
        } else {
            false
        }
    })
}

/// Returns the interpolation alpha for render smoothing.
///
/// After all fixed steps have been consumed for a frame, this value
/// represents how far between the last and next fixed step the current
/// frame sits (0.0 to 1.0). Use it to interpolate visual positions.
#[no_mangle]
pub extern "C" fn goud_fixed_timestep_alpha(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0.0;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        states
            .get(index)
            .and_then(|opt| opt.as_ref())
            .map(|state| state.interpolation_alpha)
            .unwrap_or(0.0)
    })
}

/// Returns the configured fixed timestep step size in seconds.
///
/// Returns `0.0` if fixed timestep is disabled or the context is invalid.
#[no_mangle]
pub extern "C" fn goud_fixed_timestep_dt(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0.0;
    }

    WINDOW_STATES.with(|cell| {
        let states = cell.borrow();
        let index = context_id.index() as usize;
        states
            .get(index)
            .and_then(|opt| opt.as_ref())
            .map(|state| state.fixed_timestep)
            .unwrap_or(0.0)
    })
}

/// Sets the fixed timestep step size at runtime for an existing context.
///
/// Pass `0.0` to disable fixed timestep mode.
#[no_mangle]
pub extern "C" fn goud_fixed_timestep_set(context_id: GoudContextId, step: f32) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            state.fixed_timestep = step.max(0.0);
            true
        } else {
            false
        }
    })
}

/// Sets the maximum fixed steps per frame at runtime for an existing context.
#[no_mangle]
pub extern "C" fn goud_fixed_timestep_set_max_steps(context_id: GoudContextId, max: u32) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    WINDOW_STATES.with(|cell| {
        let mut states = cell.borrow_mut();
        let index = context_id.index() as usize;
        if let Some(Some(state)) = states.get_mut(index) {
            state.max_fixed_steps = max.max(1);
            true
        } else {
            false
        }
    })
}
