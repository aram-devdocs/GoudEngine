//! Entity query FFI functions: liveness checks and entity counting.
//!
//! Provides C-compatible read-only queries about entity state in the world.

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::Entity;
use crate::ffi::{GoudContextId, GoudEntityId, GOUD_INVALID_CONTEXT_ID};

use super::{entity_from_ffi, GOUD_INVALID_ENTITY_ID};

// ============================================================================
// Private macros (queries-local)
// ============================================================================

/// Executes a closure with an immutable world reference, returning a `bool`.
macro_rules! with_context_bool {
    ($context_id:expr, $entity_id:expr, $f:expr) => {{
        use crate::ffi::context::get_context_registry;

        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return false;
        }

        if $entity_id == GOUD_INVALID_ENTITY_ID {
            set_last_error(GoudError::EntityNotFound);
            return false;
        }

        let registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => {
                set_last_error(GoudError::InternalError(
                    "Failed to lock context registry".to_string(),
                ));
                return false;
            }
        };
        let context = match registry.get($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return false;
            }
        };

        let entity = entity_from_ffi(GoudEntityId::new($entity_id));
        let world = context.world();
        $f(world, entity)
    }};
}

// ============================================================================
// Query functions
// ============================================================================

/// Checks if an entity is currently alive in the world.
///
/// An entity is considered alive if:
/// - It was spawned and not yet despawned
/// - Its generation matches the current generation for that slot
///
/// # Arguments
///
/// * `context_id` - The context to check
/// * `entity_id` - The entity to check
///
/// # Returns
///
/// `true` if the entity is alive, `false` otherwise.
///
/// # Note
///
/// This function returns `false` on error (invalid context or entity).
/// Use `goud_get_last_error_code()` to distinguish errors from dead entities.
#[no_mangle]
pub extern "C" fn goud_entity_is_alive(context_id: GoudContextId, entity_id: u64) -> bool {
    use crate::ecs::World;
    with_context_bool!(context_id, entity_id, |world: &World, entity: Entity| {
        world.is_alive(entity)
    })
}

/// Gets the total number of alive entities in the world.
///
/// # Arguments
///
/// * `context_id` - The context to query
///
/// # Returns
///
/// The count of alive entities, or 0 on error.
#[no_mangle]
pub extern "C" fn goud_entity_count(context_id: GoudContextId) -> u32 {
    use crate::ffi::context::get_context_registry;

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return 0;
        }
    };
    let context = match registry.get(context_id) {
        Some(ctx) => ctx,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    context.world().entity_count() as u32
}

/// Checks if multiple entities are currently alive in the world.
///
/// Results are written to the `out_results` array, where each entry is:
/// - `1` (true) if the entity is alive
/// - `0` (false) if the entity is dead or invalid
///
/// This is more efficient than calling `goud_entity_is_alive()` multiple times.
///
/// # Arguments
///
/// * `context_id` - The context to check
/// * `entity_ids` - Pointer to array of entity IDs to check
/// * `count` - Number of entities in the array
/// * `out_results` - Pointer to array where results will be written (size = count)
///
/// # Returns
///
/// The number of results written (should equal `count` on success, 0 on error).
///
/// # Safety
///
/// Caller must ensure:
/// - `entity_ids` points to valid memory with at least `count` u64 values
/// - `out_results` points to valid memory with at least `count` u8 values
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
#[no_mangle]
pub unsafe extern "C" fn goud_entity_is_alive_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    out_results: *mut u8,
) -> u32 {
    use crate::ffi::context::get_context_registry;

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    if entity_ids.is_null() {
        set_last_error(GoudError::InvalidState(
            "entity_ids pointer is null".to_string(),
        ));
        return 0;
    }

    if out_results.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_results pointer is null".to_string(),
        ));
        return 0;
    }

    if count == 0 {
        return 0;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return 0;
        }
    };
    let context = match registry.get(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    // SAFETY: caller guarantees both slices are valid for count elements.
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);
    let world = context.world();

    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        let entity = Entity::from_bits(entity_bits);
        results_slice[i] = if world.is_alive(entity) { 1 } else { 0 };
    }

    count
}
