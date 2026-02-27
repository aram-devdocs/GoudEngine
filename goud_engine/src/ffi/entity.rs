//! # FFI Entity Operations
//!
//! This module provides C-compatible functions for entity lifecycle management:
//! spawning, despawning, and querying entity state.
//!
//! ## Design
//!
//! All functions:
//! - Accept `GoudContextId` as the first parameter
//! - Return error codes via thread-local storage
//! - Use `#[no_mangle]` and `extern "C"` for C ABI compatibility
//! - Validate all inputs before accessing the context
//! - Never panic - all errors are caught and converted to error codes
//!
//! ## Thread Safety
//!
//! Entity operations must be called from the thread that owns the context.
//! The context registry is thread-safe, but individual contexts are not.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Create context
//! var contextId = goud_context_create();
//!
//! // Spawn empty entity
//! var entity = goud_entity_spawn_empty(contextId);
//! if (entity == GOUD_INVALID_ENTITY_ID) {
//!     // Handle error...
//! }
//!
//! // Check if entity is alive
//! if (goud_entity_is_alive(contextId, entity)) {
//!     // Entity exists
//! }
//!
//! // Despawn entity
//! var result = goud_entity_despawn(contextId, entity);
//! if (!result.success) {
//!     // Handle error...
//! }
//! ```

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::Entity;
use crate::ffi::{GoudContextId, GoudEntityId, GoudResult, GOUD_INVALID_CONTEXT_ID};

/// Sentinel value for an invalid entity ID.
///
/// This is returned by entity spawn functions on failure.
/// Callers should check for this value before using the entity ID.
pub const GOUD_INVALID_ENTITY_ID: u64 = u64::MAX;

// ============================================================================
// Helper Functions
// ============================================================================

/// Converts an FFI GoudEntityId to an internal Entity.
#[inline]
fn entity_from_ffi(entity_id: GoudEntityId) -> Entity {
    Entity::from_bits(entity_id.bits())
}

/// Macro to safely execute code with a mutable context reference.
///
/// This handles all the boilerplate of:
/// 1. Validating the context ID
/// 2. Locking the registry
/// 3. Looking up the context
/// 4. Getting mutable access to the World
/// 5. Setting error on failure
///
/// # Usage
///
/// ```ignore
/// with_context_mut!(context_id, |world| {
///     // Your code here with &mut World
///     world.spawn_empty()
/// })
/// ```
/// Macro for operations that return an entity ID.
macro_rules! with_context_mut_entity {
    ($context_id:expr, $f:expr) => {{
        use crate::ffi::context::get_context_registry;

        // Validate context ID
        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return GOUD_INVALID_ENTITY_ID;
        }

        // Lock registry and get context
        let mut registry = get_context_registry().lock().unwrap();
        let context = match registry.get_mut($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GOUD_INVALID_ENTITY_ID;
            }
        };

        // Execute function with mutable world reference
        let world = context.world_mut();
        let entity = $f(world);
        entity.to_bits()
    }};
}

/// Macro for read-only operations that return a boolean.
macro_rules! with_context_bool {
    ($context_id:expr, $entity_id:expr, $f:expr) => {{
        use crate::ffi::context::get_context_registry;

        // Validate context ID
        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return false;
        }

        // Validate entity ID
        if $entity_id == GOUD_INVALID_ENTITY_ID {
            set_last_error(GoudError::EntityNotFound);
            return false;
        }

        // Lock registry and get context
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return false;
            }
        };

        // Execute function with immutable world reference
        let entity = entity_from_ffi(GoudEntityId::new($entity_id));
        let world = context.world();
        $f(world, entity)
    }};
}

// ============================================================================
// FFI Functions
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
///
/// # Example (C#)
///
/// ```csharp
/// var entity = goud_entity_spawn_empty(contextId);
/// if (entity == GOUD_INVALID_ENTITY_ID) {
///     var error = goud_get_last_error_message();
///     Console.WriteLine($"Failed to spawn entity: {error}");
/// }
/// ```
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
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = new ulong[100];
/// int spawned = goud_entity_spawn_batch(contextId, 100, entities);
/// Console.WriteLine($"Spawned {spawned} entities");
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_entity_spawn_batch(
    context_id: GoudContextId,
    count: u32,
    out_entities: *mut u64,
) -> u32 {
    use crate::ffi::context::get_context_registry;

    // Validate inputs
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

    // Lock registry and get context
    let mut registry = get_context_registry().lock().unwrap();
    let context = match registry.get_mut(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    // Spawn entities in batch
    let world = context.world_mut();
    let entities = world.spawn_batch(count as usize);

    // Write entity IDs to output buffer
    let out_slice = std::slice::from_raw_parts_mut(out_entities, count as usize);
    for (i, entity) in entities.iter().enumerate() {
        out_slice[i] = entity.to_bits();
    }

    entities.len() as u32
}

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
///
/// # Example (C#)
///
/// ```csharp
/// var result = goud_entity_despawn(contextId, entity);
/// if (!result.success) {
///     Console.WriteLine($"Failed to despawn entity: code {result.code}");
/// }
/// ```
#[no_mangle]
pub extern "C" fn goud_entity_despawn(context_id: GoudContextId, entity_id: u64) -> GoudResult {
    use crate::ffi::context::get_context_registry;

    // Validate context ID
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(GoudError::InvalidContext.error_code());
    }

    // Validate entity ID
    if entity_id == GOUD_INVALID_ENTITY_ID {
        set_last_error(GoudError::EntityNotFound);
        return GoudResult::err(GoudError::EntityNotFound.error_code());
    }

    // Lock registry and get context
    let mut registry = get_context_registry().lock().unwrap();
    let context = match registry.get_mut(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(GoudError::InvalidContext.error_code());
        }
    };

    // Despawn entity
    let entity = entity_from_ffi(GoudEntityId::new(entity_id));
    let world = context.world_mut();

    if world.despawn(entity) {
        GoudResult::ok()
    } else {
        set_last_error(GoudError::EntityNotFound);
        GoudResult::err(GoudError::EntityNotFound.error_code())
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
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { entity1, entity2, entity3 };
/// int despawned = goud_entity_despawn_batch(contextId, entities, 3);
/// Console.WriteLine($"Despawned {despawned} entities");
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_entity_despawn_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
) -> u32 {
    use crate::ffi::context::get_context_registry;

    // Validate inputs
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

    // Lock registry and get context
    let mut registry = get_context_registry().lock().unwrap();
    let context = match registry.get_mut(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    // Convert entity IDs
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let entities: Vec<Entity> = entity_slice
        .iter()
        .map(|&bits| Entity::from_bits(bits))
        .collect();

    // Despawn entities in batch
    let world = context.world_mut();
    world.despawn_batch(&entities) as u32
}

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
///
/// # Example (C#)
///
/// ```csharp
/// if (goud_entity_is_alive(contextId, entity)) {
///     Console.WriteLine("Entity is alive");
/// } else {
///     Console.WriteLine("Entity is dead or invalid");
/// }
/// ```
#[no_mangle]
pub extern "C" fn goud_entity_is_alive(context_id: GoudContextId, entity_id: u64) -> bool {
    use crate::ecs::{Entity, World};
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
///
/// # Example (C#)
///
/// ```csharp
/// int count = goud_entity_count(contextId);
/// Console.WriteLine($"World has {count} entities");
/// ```
#[no_mangle]
pub extern "C" fn goud_entity_count(context_id: GoudContextId) -> u32 {
    use crate::ffi::context::get_context_registry;

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return 0;
    }

    let registry = get_context_registry().lock().unwrap();
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
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// byte[] results = new byte[5];
/// fixed (ulong* ePtr = entities)
/// fixed (byte* rPtr = results) {
///     int count = goud_entity_is_alive_batch(contextId, ePtr, 5, rPtr);
///     for (int i = 0; i < count; i++) {
///         Console.WriteLine($"Entity {i} alive: {results[i] != 0}");
///     }
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_entity_is_alive_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    out_results: *mut u8,
) -> u32 {
    use crate::ffi::context::get_context_registry;

    // Validate inputs
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

    // Lock registry and get context
    let registry = get_context_registry().lock().unwrap();
    let context = match registry.get(context_id) {
        Some(guard) => guard,
        None => {
            set_last_error(GoudError::InvalidContext);
            return 0;
        }
    };

    // Check each entity
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);
    let world = context.world();

    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        let entity = Entity::from_bits(entity_bits);
        results_slice[i] = if world.is_alive(entity) { 1 } else { 0 };
    }

    count
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::context::goud_context_create;
    use crate::ffi::context::goud_context_destroy;

    // ========================================================================
    // Entity Spawn Tests
    // ========================================================================

    #[test]
    fn test_spawn_empty_basic() {
        let ctx = goud_context_create();
        assert_ne!(ctx, GOUD_INVALID_CONTEXT_ID);

        let entity = goud_entity_spawn_empty(ctx);
        assert_ne!(entity, GOUD_INVALID_ENTITY_ID);
        assert!(goud_entity_is_alive(ctx, entity));

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_spawn_empty_invalid_context() {
        let entity = goud_entity_spawn_empty(GOUD_INVALID_CONTEXT_ID);
        assert_eq!(entity, GOUD_INVALID_ENTITY_ID);
    }

    #[test]
    fn test_spawn_empty_multiple() {
        let ctx = goud_context_create();

        let e1 = goud_entity_spawn_empty(ctx);
        let e2 = goud_entity_spawn_empty(ctx);
        let e3 = goud_entity_spawn_empty(ctx);

        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);

        assert!(goud_entity_is_alive(ctx, e1));
        assert!(goud_entity_is_alive(ctx, e2));
        assert!(goud_entity_is_alive(ctx, e3));

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_spawn_batch_basic() {
        let ctx = goud_context_create();
        let mut entities = vec![0u64; 10];

        let count = unsafe { goud_entity_spawn_batch(ctx, 10, entities.as_mut_ptr()) };

        assert_eq!(count, 10);
        for entity in &entities {
            assert_ne!(*entity, GOUD_INVALID_ENTITY_ID);
            assert!(goud_entity_is_alive(ctx, *entity));
        }

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_spawn_batch_zero_count() {
        let ctx = goud_context_create();
        let mut entities = vec![0u64; 1];

        let count = unsafe { goud_entity_spawn_batch(ctx, 0, entities.as_mut_ptr()) };

        assert_eq!(count, 0);
        goud_context_destroy(ctx);
    }

    #[test]
    fn test_spawn_batch_invalid_context() {
        let mut entities = vec![0u64; 10];

        let count =
            unsafe { goud_entity_spawn_batch(GOUD_INVALID_CONTEXT_ID, 10, entities.as_mut_ptr()) };

        assert_eq!(count, 0);
    }

    #[test]
    fn test_spawn_batch_null_pointer() {
        let ctx = goud_context_create();

        let count = unsafe { goud_entity_spawn_batch(ctx, 10, std::ptr::null_mut()) };

        assert_eq!(count, 0);
        goud_context_destroy(ctx);
    }

    #[test]
    fn test_spawn_batch_large() {
        let ctx = goud_context_create();
        let mut entities = vec![0u64; 1000];

        let count = unsafe { goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr()) };

        assert_eq!(count, 1000);

        // Verify all are unique
        let unique: std::collections::HashSet<_> = entities.iter().copied().collect();
        assert_eq!(unique.len(), 1000);

        goud_context_destroy(ctx);
    }

    // ========================================================================
    // Entity Despawn Tests
    // ========================================================================

    #[test]
    fn test_despawn_basic() {
        let ctx = goud_context_create();
        let entity = goud_entity_spawn_empty(ctx);

        assert!(goud_entity_is_alive(ctx, entity));

        let result = goud_entity_despawn(ctx, entity);
        assert!(result.success);
        assert!(!goud_entity_is_alive(ctx, entity));

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_despawn_invalid_context() {
        let result = goud_entity_despawn(GOUD_INVALID_CONTEXT_ID, 123);
        assert!(!result.success);
    }

    #[test]
    fn test_despawn_invalid_entity() {
        let ctx = goud_context_create();

        let result = goud_entity_despawn(ctx, GOUD_INVALID_ENTITY_ID);
        assert!(!result.success);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_despawn_already_despawned() {
        let ctx = goud_context_create();
        let entity = goud_entity_spawn_empty(ctx);

        let result1 = goud_entity_despawn(ctx, entity);
        assert!(result1.success);

        let result2 = goud_entity_despawn(ctx, entity);
        assert!(!result2.success);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_despawn_batch_basic() {
        let ctx = goud_context_create();
        let mut entities = vec![0u64; 5];

        unsafe {
            goud_entity_spawn_batch(ctx, 5, entities.as_mut_ptr());
        }

        let count = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), 5) };

        assert_eq!(count, 5);
        for entity in &entities {
            assert!(!goud_entity_is_alive(ctx, *entity));
        }

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_despawn_batch_partial_invalid() {
        let ctx = goud_context_create();
        let entities = vec![
            goud_entity_spawn_empty(ctx),
            GOUD_INVALID_ENTITY_ID,
            goud_entity_spawn_empty(ctx),
        ];

        let count = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), 3) };

        // Should despawn 2 valid entities, skip 1 invalid
        assert_eq!(count, 2);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_despawn_batch_zero_count() {
        let ctx = goud_context_create();
        let entities = vec![0u64; 1];

        let count = unsafe { goud_entity_despawn_batch(ctx, entities.as_ptr(), 0) };

        assert_eq!(count, 0);
        goud_context_destroy(ctx);
    }

    // ========================================================================
    // Entity Query Tests
    // ========================================================================

    #[test]
    fn test_is_alive_basic() {
        let ctx = goud_context_create();
        let entity = goud_entity_spawn_empty(ctx);

        assert!(goud_entity_is_alive(ctx, entity));

        goud_entity_despawn(ctx, entity);
        assert!(!goud_entity_is_alive(ctx, entity));

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_invalid_context() {
        let alive = goud_entity_is_alive(GOUD_INVALID_CONTEXT_ID, 123);
        assert!(!alive);
    }

    #[test]
    fn test_is_alive_invalid_entity() {
        let ctx = goud_context_create();
        let alive = goud_entity_is_alive(ctx, GOUD_INVALID_ENTITY_ID);
        assert!(!alive);
        goud_context_destroy(ctx);
    }

    #[test]
    fn test_entity_count_basic() {
        let ctx = goud_context_create();

        assert_eq!(goud_entity_count(ctx), 0);

        goud_entity_spawn_empty(ctx);
        assert_eq!(goud_entity_count(ctx), 1);

        goud_entity_spawn_empty(ctx);
        assert_eq!(goud_entity_count(ctx), 2);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_entity_count_after_despawn() {
        let ctx = goud_context_create();

        let e1 = goud_entity_spawn_empty(ctx);
        let e2 = goud_entity_spawn_empty(ctx);
        assert_eq!(goud_entity_count(ctx), 2);

        goud_entity_despawn(ctx, e1);
        assert_eq!(goud_entity_count(ctx), 1);

        goud_entity_despawn(ctx, e2);
        assert_eq!(goud_entity_count(ctx), 0);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_entity_count_invalid_context() {
        let count = goud_entity_count(GOUD_INVALID_CONTEXT_ID);
        assert_eq!(count, 0);
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_spawn_despawn_respawn() {
        let ctx = goud_context_create();

        // Spawn, despawn, then spawn again (should reuse slot)
        let e1 = goud_entity_spawn_empty(ctx);
        goud_entity_despawn(ctx, e1);
        let e2 = goud_entity_spawn_empty(ctx);

        // e1 should be dead, e2 should be alive
        assert!(!goud_entity_is_alive(ctx, e1));
        assert!(goud_entity_is_alive(ctx, e2));

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_mixed_operations() {
        let ctx = goud_context_create();

        // Spawn some entities
        let mut entities = vec![0u64; 10];
        unsafe {
            goud_entity_spawn_batch(ctx, 10, entities.as_mut_ptr());
        }

        assert_eq!(goud_entity_count(ctx), 10);

        // Despawn half of them
        unsafe {
            goud_entity_despawn_batch(ctx, entities.as_ptr(), 5);
        }

        assert_eq!(goud_entity_count(ctx), 5);

        // Spawn more
        let e = goud_entity_spawn_empty(ctx);
        assert_eq!(goud_entity_count(ctx), 6);
        assert!(goud_entity_is_alive(ctx, e));

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_stress_spawn_despawn() {
        let ctx = goud_context_create();

        // Spawn 1000 entities
        let mut entities = vec![0u64; 1000];
        unsafe {
            goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr());
        }

        assert_eq!(goud_entity_count(ctx), 1000);

        // Despawn all
        unsafe {
            goud_entity_despawn_batch(ctx, entities.as_ptr(), 1000);
        }

        assert_eq!(goud_entity_count(ctx), 0);

        // Spawn again
        unsafe {
            goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr());
        }

        assert_eq!(goud_entity_count(ctx), 1000);

        goud_context_destroy(ctx);
    }

    // ========================================================================
    // Batch Is Alive Tests
    // ========================================================================

    #[test]
    fn test_is_alive_batch_basic() {
        let ctx = goud_context_create();

        // Spawn 5 entities
        let mut entities = [0u64; 5];
        unsafe {
            goud_entity_spawn_batch(ctx, 5, entities.as_mut_ptr());
        }

        // Check all are alive
        let mut results = [0u8; 5];
        let count =
            unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 5, results.as_mut_ptr()) };

        assert_eq!(count, 5);
        for result in &results {
            assert_eq!(*result, 1); // All alive
        }

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_batch_mixed() {
        let ctx = goud_context_create();

        // Spawn 5 entities
        let mut entities = [0u64; 5];
        unsafe {
            goud_entity_spawn_batch(ctx, 5, entities.as_mut_ptr());
        }

        // Despawn entities 1 and 3 (indices 1 and 3)
        let _ = goud_entity_despawn(ctx, entities[1]);
        let _ = goud_entity_despawn(ctx, entities[3]);

        // Check alive status
        let mut results = [0u8; 5];
        let count =
            unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 5, results.as_mut_ptr()) };

        assert_eq!(count, 5);
        assert_eq!(results[0], 1); // Alive
        assert_eq!(results[1], 0); // Despawned
        assert_eq!(results[2], 1); // Alive
        assert_eq!(results[3], 0); // Despawned
        assert_eq!(results[4], 1); // Alive

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_batch_invalid_context() {
        let entities = [1u64, 2, 3];
        let mut results = [0u8; 3];

        let count = unsafe {
            goud_entity_is_alive_batch(
                GOUD_INVALID_CONTEXT_ID,
                entities.as_ptr(),
                3,
                results.as_mut_ptr(),
            )
        };

        assert_eq!(count, 0);
    }

    #[test]
    fn test_is_alive_batch_null_entities() {
        let ctx = goud_context_create();
        let mut results = [0u8; 3];

        let count =
            unsafe { goud_entity_is_alive_batch(ctx, std::ptr::null(), 3, results.as_mut_ptr()) };

        assert_eq!(count, 0);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_batch_null_results() {
        let ctx = goud_context_create();
        let entities = [1u64, 2, 3];

        let count =
            unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 3, std::ptr::null_mut()) };

        assert_eq!(count, 0);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_batch_zero_count() {
        let ctx = goud_context_create();
        let entities = [1u64];
        let mut results = [0u8; 1];

        let count =
            unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 0, results.as_mut_ptr()) };

        assert_eq!(count, 0);

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_batch_invalid_entities() {
        let ctx = goud_context_create();

        // Use entity IDs that were never spawned
        let entities = [GOUD_INVALID_ENTITY_ID, u64::MAX - 1, 123456];
        let mut results = [1u8; 3]; // Initialize to 1 to see they're cleared

        let count =
            unsafe { goud_entity_is_alive_batch(ctx, entities.as_ptr(), 3, results.as_mut_ptr()) };

        assert_eq!(count, 3);
        for result in &results {
            assert_eq!(*result, 0); // All should be not alive
        }

        goud_context_destroy(ctx);
    }

    #[test]
    fn test_is_alive_batch_large() {
        let ctx = goud_context_create();

        // Spawn 1000 entities
        let mut entities = vec![0u64; 1000];
        unsafe {
            goud_entity_spawn_batch(ctx, 1000, entities.as_mut_ptr());
        }

        // Check all are alive
        let mut results = vec![0u8; 1000];
        let count = unsafe {
            goud_entity_is_alive_batch(ctx, entities.as_ptr(), 1000, results.as_mut_ptr())
        };

        assert_eq!(count, 1000);
        for result in &results {
            assert_eq!(*result, 1);
        }

        goud_context_destroy(ctx);
    }
}
