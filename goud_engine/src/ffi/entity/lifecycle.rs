//! Entity lifecycle FFI functions: spawning and despawning.
//!
//! Provides C-compatible functions for creating and destroying entities
//! in the engine world.

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::Entity;
use crate::ffi::{GoudContextId, GoudResult, GOUD_INVALID_CONTEXT_ID};

use super::GOUD_INVALID_ENTITY_ID;

// ============================================================================
// Private macros (lifecycle-local)
// ============================================================================

/// Executes a closure with a mutable world reference, returning an entity ID.
macro_rules! with_context_mut_entity {
    ($context_id:expr, $f:expr) => {{
        use crate::ffi::context::get_context_registry;

        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return GOUD_INVALID_ENTITY_ID;
        }

        let mut registry = match get_context_registry().lock() {
            Ok(r) => r,
            Err(_) => return GOUD_INVALID_ENTITY_ID,
        };
        let context = match registry.get_mut($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GOUD_INVALID_ENTITY_ID;
            }
        };

        let world = context.world_mut();
        let entity = $f(world);
        entity.to_bits()
    }};
}

// ============================================================================
// Spawn functions
// ============================================================================

/// Spawns a new empty entity in the world.
///
/// The entity is created with no components attached. Components can be
/// added later using `goud_entity_insert_component()`.
///
/// # Arguments
///
/// * `context_id` - The context to spawn the entity in
///
/// # Returns
///
/// The entity ID on success, or `GOUD_INVALID_ENTITY_ID` on failure.
/// Call `goud_get_last_error_message()` for error details.
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `INTERNAL_ERROR_BASE + 0` (InternalError) - Unexpected error during spawn
///
/// # Thread Safety
///
/// Must be called from the thread that owns the context.
#[no_mangle]
pub extern "C" fn goud_entity_spawn_empty(context_id: GoudContextId) -> u64 {
    use crate::ecs::World;
    with_context_mut_entity!(context_id, |world: &mut World| { world.spawn_empty() })
}

/// Spawns multiple empty entities in a single batch.
///
/// This is more efficient than calling `goud_entity_spawn_empty()` multiple
/// times because it pre-allocates slots and avoids repeated locking.
///
/// # Arguments
///
/// * `context_id` - The context to spawn entities in
/// * `count` - Number of entities to spawn
/// * `out_entities` - Output buffer to write entity IDs (must hold at least `count` u64s)
///
/// # Returns
///
/// The number of entities successfully spawned (may be less than `count` on partial failure).
///
/// # Safety
///
/// Caller must ensure `out_entities` points to valid memory with capacity for `count` u64 values.
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `INTERNAL_ERROR_BASE + 0` (InternalError) - Unexpected error during spawn
#[no_mangle]
pub unsafe extern "C" fn goud_entity_spawn_batch(
    context_id: GoudContextId,
    count: u32,
    out_entities: *mut u64,
) -> u32 {
    use crate::ffi::context::get_context_registry;

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    if out_entities.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_entities pointer is null".to_string(),
        ));
        return 0;
    }

    if count == 0 {
        return 0;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let context = match registry.get_mut(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    let world = context.world_mut();
    let entities = world.spawn_batch(count as usize);

    // SAFETY: caller guarantees out_entities is valid for count elements.
    let out_slice = std::slice::from_raw_parts_mut(out_entities, count as usize);
    for (i, entity) in entities.iter().enumerate() {
        out_slice[i] = entity.to_bits();
    }

    entities.len() as u32
}

// ============================================================================
// Despawn functions
// ============================================================================

/// Despawns an entity, removing it and all its components from the world.
///
/// After despawning, the entity ID becomes invalid and should not be used.
/// Any components attached to the entity are also removed.
///
/// # Arguments
///
/// * `context_id` - The context containing the entity
/// * `entity_id` - The entity to despawn
///
/// # Returns
///
/// A result indicating success or failure. Check `result.success`.
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `ENTITY_ERROR_BASE + 0` (EntityNotFound) - Entity does not exist or is already despawned
#[no_mangle]
pub extern "C" fn goud_entity_despawn(context_id: GoudContextId, entity_id: u64) -> GoudResult {
    use crate::ffi::context::get_context_registry;
    use crate::ffi::GoudEntityId;

    if context_id == GOUD_INVALID_CONTEXT_ID {
        return GoudResult::from_error(GoudError::InvalidContext);
    }

    if entity_id == GOUD_INVALID_ENTITY_ID {
        return GoudResult::from_error(GoudError::EntityNotFound);
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            return GoudResult::from_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
        }
    };
    let context = match registry.get_mut(context_id) {
        Some(guard) => guard,
        None => {
            return GoudResult::from_error(GoudError::InvalidContext);
        }
    };

    let entity = Entity::from_bits(GoudEntityId::new(entity_id).bits());
    let world = context.world_mut();

    if world.despawn(entity) {
        GoudResult::ok()
    } else {
        GoudResult::from_error(GoudError::EntityNotFound)
    }
}

/// Despawns multiple entities in a single batch.
///
/// This is more efficient than calling `goud_entity_despawn()` multiple times.
/// Invalid or already-despawned entities are skipped.
///
/// # Arguments
///
/// * `context_id` - The context containing the entities
/// * `entity_ids` - Pointer to array of entity IDs to despawn
/// * `count` - Number of entities in the array
///
/// # Returns
///
/// The number of entities successfully despawned.
///
/// # Safety
///
/// Caller must ensure `entity_ids` points to valid memory with at least `count` u64 values.
#[no_mangle]
pub unsafe extern "C" fn goud_entity_despawn_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
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

    if count == 0 {
        return 0;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let context = match registry.get_mut(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    // SAFETY: caller guarantees entity_ids is valid for count elements.
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let entities: Vec<Entity> = entity_slice
        .iter()
        .map(|&bits| Entity::from_bits(bits))
        .collect();

    let world = context.world_mut();
    world.despawn_batch(&entities) as u32
}
