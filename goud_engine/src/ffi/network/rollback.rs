//! FFI exports for the rollback netcode system.
//!
//! The rollback engine is generic over `GameState`, but FFI callers cannot
//! provide Rust traits. Instead, the FFI layer uses an opaque-pointer +
//! function-pointer design: the caller supplies a `state_ptr` and callbacks
//! for `advance` and `state_hash`. The Rust side wraps these in a concrete
//! `FfiGameState` that implements `GameState`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::libs::networking::rollback::{GameState, PlayerId, RollbackConfig, RollbackNetcode};

// ---------------------------------------------------------------------------
// FFI-safe configuration struct
// ---------------------------------------------------------------------------

/// FFI-safe rollback configuration.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiRollbackConfig {
    /// Maximum number of rollback frames.
    pub max_rollback_frames: u32,
    /// Input delay frames.
    pub input_delay_frames: u32,
    /// Whether desync detection is enabled (0 = false, nonzero = true).
    pub desync_detection: u8,
}

// ---------------------------------------------------------------------------
// Opaque game state via function pointers
// ---------------------------------------------------------------------------

/// Function pointer type: advance the game state by one frame.
///
/// `state_ptr`: opaque pointer to the caller's game state.
/// `player_ids`: pointer to array of player IDs.
/// `input_ptrs`: pointer to array of pointers, one per player, to that player's input bytes.
/// `input_lens`: pointer to array of lengths for each player's input.
/// `num_players`: number of players.
pub type FfiAdvanceFn = extern "C" fn(
    state_ptr: *mut u8,
    player_ids: *const u8,
    input_ptrs: *const *const u8,
    input_lens: *const u32,
    num_players: u32,
);

/// Function pointer type: compute a hash of the game state.
///
/// `state_ptr`: opaque pointer to the caller's game state.
/// Returns a u64 hash.
pub type FfiHashFn = extern "C" fn(state_ptr: *mut u8) -> u64;

/// Function pointer type: clone the game state.
///
/// `state_ptr`: opaque pointer to the source state.
/// Returns a pointer to the cloned state. Caller owns the clone.
pub type FfiCloneFn = extern "C" fn(state_ptr: *mut u8) -> *mut u8;

/// Function pointer type: free a cloned game state.
///
/// `state_ptr`: pointer to a state previously returned by `FfiCloneFn`.
pub type FfiFreeFn = extern "C" fn(state_ptr: *mut u8);

/// Wraps an opaque caller-owned game state behind `GameState`.
struct FfiGameState {
    state_ptr: *mut u8,
    advance_fn: FfiAdvanceFn,
    hash_fn: FfiHashFn,
    clone_fn: FfiCloneFn,
    free_fn: FfiFreeFn,
}

// SAFETY: The FfiGameState is only accessed through the global registry Mutex,
// serializing all access. The raw pointer is managed by the caller's callbacks
// which are expected to be thread-safe or only called from one thread.
unsafe impl Send for FfiGameState {}

impl Clone for FfiGameState {
    fn clone(&self) -> Self {
        let cloned_ptr = (self.clone_fn)(self.state_ptr);
        Self {
            state_ptr: cloned_ptr,
            advance_fn: self.advance_fn,
            hash_fn: self.hash_fn,
            clone_fn: self.clone_fn,
            free_fn: self.free_fn,
        }
    }
}

impl Drop for FfiGameState {
    fn drop(&mut self) {
        if !self.state_ptr.is_null() {
            (self.free_fn)(self.state_ptr);
            self.state_ptr = std::ptr::null_mut();
        }
    }
}

impl GameState for FfiGameState {
    fn advance(&mut self, inputs: &HashMap<PlayerId, Vec<u8>>) {
        let mut player_ids: Vec<u8> = Vec::with_capacity(inputs.len());
        let mut input_ptrs: Vec<*const u8> = Vec::with_capacity(inputs.len());
        let mut input_lens: Vec<u32> = Vec::with_capacity(inputs.len());

        for (&player, data) in inputs {
            player_ids.push(player);
            input_ptrs.push(if data.is_empty() {
                std::ptr::null()
            } else {
                data.as_ptr()
            });
            input_lens.push(data.len() as u32);
        }

        (self.advance_fn)(
            self.state_ptr,
            player_ids.as_ptr(),
            input_ptrs.as_ptr(),
            input_lens.as_ptr(),
            player_ids.len() as u32,
        );
    }

    fn state_hash(&self) -> u64 {
        (self.hash_fn)(self.state_ptr)
    }
}

// ---------------------------------------------------------------------------
// Registry for rollback instances
// ---------------------------------------------------------------------------

struct RollbackInstance {
    engine: RollbackNetcode<FfiGameState>,
}

struct RollbackRegistry {
    instances: HashMap<i64, RollbackInstance>,
    next_handle: i64,
}

impl RollbackRegistry {
    fn new() -> Self {
        Self {
            instances: HashMap::new(),
            next_handle: 1,
        }
    }

    fn insert(&mut self, inst: RollbackInstance) -> i64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.instances.insert(handle, inst);
        handle
    }
}

static ROLLBACK_REGISTRY: Mutex<Option<RollbackRegistry>> = Mutex::new(None);

fn with_rollback_registry<F, R>(f: F) -> Result<R, i32>
where
    F: FnOnce(&mut RollbackRegistry) -> Result<R, i32>,
{
    let mut guard = ROLLBACK_REGISTRY.lock().map_err(|_| {
        set_last_error(GoudError::InternalError(
            "Failed to lock rollback registry".to_string(),
        ));
        ERR_INTERNAL_ERROR
    })?;
    let reg = guard.get_or_insert_with(RollbackRegistry::new);
    f(reg)
}

fn with_rollback_instance<F, R>(handle: i64, f: F) -> Result<R, i32>
where
    F: FnOnce(&mut RollbackInstance) -> Result<R, i32>,
{
    with_rollback_registry(|reg| {
        let inst = reg.instances.get_mut(&handle).ok_or_else(|| {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown rollback handle {}",
                handle
            )));
            ERR_INVALID_STATE
        })?;
        f(inst)
    })
}

// ---------------------------------------------------------------------------
// FFI exports
// ---------------------------------------------------------------------------

/// Creates a new rollback netcode session.
///
/// Returns a positive handle on success, or -1 on failure.
///
/// # Parameters
///
/// - `config`: rollback configuration.
/// - `local_player`: the local player's ID.
/// - `player_ids_ptr`: pointer to array of all player IDs (including local).
/// - `num_players`: length of the player IDs array.
/// - `state_ptr`: opaque pointer to the initial game state. Ownership is
///   transferred to the rollback engine; it will be freed via `free_fn`.
/// - `advance_fn`, `hash_fn`, `clone_fn`, `free_fn`: callbacks.
///
/// # Safety
///
/// All pointers must be valid. `state_ptr` ownership transfers to the engine.
#[no_mangle]
pub unsafe extern "C" fn goud_rollback_create(
    config: FfiRollbackConfig,
    local_player: u8,
    player_ids_ptr: *const u8,
    num_players: u32,
    state_ptr: *mut u8,
    advance_fn: u64,
    hash_fn: u64,
    clone_fn: u64,
    free_fn: u64,
) -> i64 {
    if player_ids_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "player_ids_ptr is null".to_string(),
        ));
        return -1;
    }
    if state_ptr.is_null() {
        set_last_error(GoudError::InvalidState("state_ptr is null".to_string()));
        return -1;
    }
    if num_players == 0 {
        set_last_error(GoudError::InvalidState(
            "num_players must be > 0".to_string(),
        ));
        return -1;
    }
    if advance_fn == 0 || hash_fn == 0 || clone_fn == 0 || free_fn == 0 {
        set_last_error(GoudError::InvalidState(
            "callback function pointers must not be null".to_string(),
        ));
        return -1;
    }

    // SAFETY: Caller guarantees player_ids_ptr points to num_players valid bytes.
    let player_ids =
        unsafe { std::slice::from_raw_parts(player_ids_ptr, num_players as usize) }.to_vec();

    let rb_config = RollbackConfig {
        max_rollback_frames: config.max_rollback_frames as usize,
        input_delay_frames: config.input_delay_frames as usize,
        desync_detection: config.desync_detection != 0,
    };

    // SAFETY: Caller guarantees these u64 values are valid function pointers
    // cast from extern "C" fn types matching the FfiAdvanceFn/FfiHashFn/
    // FfiCloneFn/FfiFreeFn signatures.
    let game_state = FfiGameState {
        state_ptr,
        advance_fn: unsafe { std::mem::transmute::<u64, FfiAdvanceFn>(advance_fn) },
        hash_fn: unsafe { std::mem::transmute::<u64, FfiHashFn>(hash_fn) },
        clone_fn: unsafe { std::mem::transmute::<u64, FfiCloneFn>(clone_fn) },
        free_fn: unsafe { std::mem::transmute::<u64, FfiFreeFn>(free_fn) },
    };

    let engine = RollbackNetcode::new(rb_config, local_player, player_ids, game_state);

    let inst = RollbackInstance { engine };

    with_rollback_registry(|reg| Ok(reg.insert(inst))).unwrap_or(-1)
}

/// Advances the rollback simulation by one frame with the given local input.
///
/// Returns 0 on success, or a negative error code.
///
/// # Safety
///
/// `input_ptr` must point to `input_len` valid bytes (not transferred).
#[no_mangle]
pub unsafe extern "C" fn goud_rollback_advance_frame(
    handle: i64,
    input_ptr: *const u8,
    input_len: u32,
) -> i32 {
    let local_input = if input_ptr.is_null() || input_len == 0 {
        Vec::new()
    } else {
        // SAFETY: Caller guarantees input_ptr is valid for input_len bytes.
        unsafe { std::slice::from_raw_parts(input_ptr, input_len as usize) }.to_vec()
    };

    let result = with_rollback_instance(handle, |inst| {
        inst.engine.advance_frame(local_input);
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Receives a confirmed remote input for a specific player and frame.
///
/// Returns 0 on success, or a negative error code.
///
/// # Safety
///
/// `input_ptr` must point to `input_len` valid bytes (not transferred).
#[no_mangle]
pub unsafe extern "C" fn goud_rollback_receive_remote_input(
    handle: i64,
    player_id: u8,
    frame: u64,
    input_ptr: *const u8,
    input_len: u32,
) -> i32 {
    let input = if input_ptr.is_null() || input_len == 0 {
        Vec::new()
    } else {
        // SAFETY: Caller guarantees input_ptr is valid for input_len bytes.
        unsafe { std::slice::from_raw_parts(input_ptr, input_len as usize) }.to_vec()
    };

    let result = with_rollback_instance(handle, |inst| {
        inst.engine.receive_remote_input(player_id, frame, input);
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}

/// Returns 1 if a rollback is pending, 0 otherwise, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_rollback_should_rollback(handle: i64) -> i32 {
    let result = with_rollback_instance(handle, |inst| {
        Ok(if inst.engine.should_rollback() { 1 } else { 0 })
    });
    result.unwrap_or_else(|e| e)
}

/// Performs rollback and resimulation. Returns the number of frames
/// resimulated (>= 0), or a negative error code.
#[no_mangle]
pub extern "C" fn goud_rollback_resimulate(handle: i64) -> i32 {
    let result = with_rollback_instance(handle, |inst| {
        let count = inst.engine.rollback_and_resimulate();
        Ok(count as i32)
    });
    result.unwrap_or_else(|e| e)
}

/// Returns the latest confirmed frame, or a negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_rollback_confirmed_frame(handle: i64) -> i64 {
    let result = with_rollback_instance(handle, |inst| Ok(inst.engine.confirmed_frame() as i64));
    result.unwrap_or_else(|e| e as i64)
}

/// Returns the current simulation frame, or a negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_rollback_current_frame(handle: i64) -> i64 {
    let result = with_rollback_instance(handle, |inst| Ok(inst.engine.current_frame() as i64));
    result.unwrap_or_else(|e| e as i64)
}

/// Checks for desync at the given frame by comparing with a remote hash.
///
/// Returns:
/// - 0: in sync
/// - 1: desync detected
/// - 2: frame not available in snapshot buffer
/// - negative: error
#[no_mangle]
pub extern "C" fn goud_rollback_check_desync(handle: i64, remote_hash: u64, frame: u64) -> i32 {
    use crate::libs::networking::rollback::DesyncResult;

    let result = with_rollback_instance(handle, |inst| {
        match inst.engine.check_desync(remote_hash, frame) {
            DesyncResult::InSync => Ok(0),
            DesyncResult::Desync { .. } => Ok(1),
            DesyncResult::FrameNotAvailable { .. } => Ok(2),
        }
    });
    result.unwrap_or_else(|e| e)
}

/// Destroys a rollback session and frees all associated resources.
///
/// Returns 0 on success, or a negative error code.
#[no_mangle]
pub extern "C" fn goud_rollback_destroy(handle: i64) -> i32 {
    let result = with_rollback_registry(|reg| {
        if reg.instances.remove(&handle).is_none() {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown rollback handle {}",
                handle
            )));
            return Err(ERR_INVALID_STATE);
        }
        Ok(0)
    });
    result.unwrap_or_else(|e| e)
}
