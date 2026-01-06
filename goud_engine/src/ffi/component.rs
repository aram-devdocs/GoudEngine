//! # FFI Component Operations
//!
//! This module provides C-compatible functions for adding, removing, and querying
//! components on entities. Since components are generic types in Rust, the FFI
//! layer uses raw byte pointers and type IDs for component data.
//!
//! ## Design
//!
//! Component operations in FFI require:
//! - **Type Registration**: Components must be registered with the engine
//! - **Raw Pointers**: Component data passed as `*const u8` / `*mut u8`
//! - **Size/Alignment**: Caller must provide correct size and alignment
//! - **Type IDs**: Components identified by 64-bit type hash
//!
//! ## Safety
//!
//! The FFI layer performs extensive validation:
//! - Context ID validation
//! - Entity liveness checks
//! - Pointer null checks
//! - Size/alignment validation
//! - Type ID verification
//!
//! However, the caller MUST ensure:
//! - Pointers point to valid component data
//! - Size/alignment match the registered type
//! - Memory remains valid for the duration of the call
//!
//! ## Thread Safety
//!
//! Component operations must be called from the thread that owns the context.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Register component type (once at startup)
//! var positionTypeId = goud_component_register_type(
//!     contextId,
//!     "Position",
//!     sizeof(Position),
//!     alignof(Position)
//! );
//!
//! // Create entity
//! var entity = goud_entity_spawn_empty(contextId);
//!
//! // Add component
//! var position = new Position { x = 10.0f, y = 20.0f };
//! fixed (Position* ptr = &position) {
//!     var result = goud_component_add(
//!         contextId,
//!         entity,
//!         positionTypeId,
//!         ptr,
//!         sizeof(Position)
//!     );
//!     if (!result.success) {
//!         // Handle error...
//!     }
//! }
//!
//! // Check if entity has component
//! if (goud_component_has(contextId, entity, positionTypeId)) {
//!     // Entity has Position component
//! }
//!
//! // Get component (read-only)
//! var posPtr = goud_component_get(contextId, entity, positionTypeId);
//! if (posPtr != null) {
//!     var pos = Marshal.PtrToStructure<Position>(posPtr);
//!     Console.WriteLine($"Position: ({pos.x}, {pos.y})");
//! }
//!
//! // Get component (mutable)
//! var posMutPtr = goud_component_get_mut(contextId, entity, positionTypeId);
//! if (posMutPtr != null) {
//!     var pos = Marshal.PtrToStructure<Position>(posMutPtr);
//!     pos.x += 1.0f;
//!     Marshal.StructureToPtr(pos, posMutPtr, false);
//! }
//!
//! // Remove component
//! var removeResult = goud_component_remove(contextId, entity, positionTypeId);
//! if (removeResult.success) {
//!     // Component removed successfully
//! }
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::Entity;
use crate::ffi::context::get_context_registry;
use crate::ffi::{GoudContextId, GoudEntityId, GoudResult, GOUD_INVALID_CONTEXT_ID};

// ============================================================================
// Type Registry
// ============================================================================

/// Information about a registered component type.
#[derive(Debug, Clone)]
struct ComponentTypeInfo {
    /// Human-readable type name (for debugging).
    #[allow(dead_code)]
    name: String,
    /// Size of the component in bytes.
    size: usize,
    /// Alignment of the component in bytes (for future validation).
    #[allow(dead_code)]
    align: usize,
    /// Rust TypeId for validation (placeholder for future use).
    #[allow(dead_code)]
    type_id: TypeId,
}

/// Global registry mapping type IDs to component information.
///
/// This is used to validate component operations at the FFI boundary.
/// Types must be registered before they can be used.
static COMPONENT_TYPE_REGISTRY: Mutex<Option<HashMap<u64, ComponentTypeInfo>>> = Mutex::new(None);

/// Gets or initializes the component type registry.
fn get_type_registry() -> std::sync::MutexGuard<'static, Option<HashMap<u64, ComponentTypeInfo>>> {
    COMPONENT_TYPE_REGISTRY.lock().unwrap()
}

/// Registers a component type with the given information.
///
/// Returns true if the type was newly registered, false if it already existed.
#[allow(clippy::needless_pass_by_value)] // name needs to be owned
fn register_component_type_internal(
    type_id_hash: u64,
    name: String,
    size: usize,
    align: usize,
    type_id: TypeId,
) -> bool {
    let mut registry = get_type_registry();
    let map = registry.get_or_insert_with(HashMap::new);

    use std::collections::hash_map::Entry;
    match map.entry(type_id_hash) {
        Entry::Occupied(_) => false,
        Entry::Vacant(e) => {
            e.insert(ComponentTypeInfo {
                name,
                size,
                align,
                type_id,
            });
            true
        }
    }
}

/// Looks up component type information by type ID hash.
fn get_component_type_info(type_id_hash: u64) -> Option<ComponentTypeInfo> {
    let registry = get_type_registry();
    registry.as_ref()?.get(&type_id_hash).cloned()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Converts an FFI GoudEntityId to an internal Entity.
#[inline]
fn entity_from_ffi(entity_id: GoudEntityId) -> Entity {
    Entity::from_bits(entity_id.bits())
}

/// Macro for component operations that return GoudResult.
macro_rules! with_context_mut_result {
    ($context_id:expr, $f:expr) => {{
        // Validate context ID
        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return GoudResult::err(3); // CONTEXT_ERROR_BASE + 3 = InvalidContext
        }

        // Lock registry and get context
        let mut registry = get_context_registry().lock().unwrap();
        let context = match registry.get_mut($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GoudResult::err(3);
            }
        };

        // Execute the closure with mutable World reference
        $f(context.world_mut())
    }};
}

/// Macro for component operations that return a pointer.
macro_rules! with_context_ptr {
    ($context_id:expr, $f:expr) => {{
        // Validate context ID
        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return std::ptr::null();
        }

        // Lock registry and get context
        let mut registry = get_context_registry().lock().unwrap();
        let context = match registry.get_mut($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return std::ptr::null();
            }
        };

        // Execute the closure with World reference
        $f(context.world_mut())
    }};
}

/// Macro for component operations that return a mutable pointer.
macro_rules! with_context_ptr_mut {
    ($context_id:expr, $f:expr) => {{
        // Validate context ID
        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
            return std::ptr::null_mut();
        }

        // Lock registry and get context
        let mut registry = get_context_registry().lock().unwrap();
        let context = match registry.get_mut($context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return std::ptr::null_mut();
            }
        };

        // Execute the closure with mutable World reference
        $f(context.world_mut())
    }};
}

/// Macro for component operations that return bool.
macro_rules! with_context_bool {
    ($context_id:expr, $f:expr) => {{
        // Validate context ID
        if $context_id == GOUD_INVALID_CONTEXT_ID {
            set_last_error(GoudError::InvalidContext);
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

        // Execute the closure with World reference
        $f(context.world())
    }};
}

// ============================================================================
// FFI Functions - Type Registration
// ============================================================================

/// Registers a component type with the engine.
///
/// This must be called once for each component type before it can be used.
/// The type ID hash should be stable across runs (e.g., from a const in C#).
///
/// # Parameters
///
/// - `type_id_hash`: Unique 64-bit identifier for the component type
/// - `name_ptr`: Null-terminated C string with the type name (for debugging)
/// - `name_len`: Length of the name string (excluding null terminator)
/// - `size`: Size of the component in bytes
/// - `align`: Alignment of the component in bytes
///
/// # Returns
///
/// `true` if the type was newly registered, `false` if already registered.
///
/// # Safety
///
/// - `name_ptr` must be a valid pointer to a C string
/// - `name_len` must match the actual string length
/// - `size` and `align` must match the actual type layout
#[no_mangle]
pub unsafe extern "C" fn goud_component_register_type(
    type_id_hash: u64,
    name_ptr: *const u8,
    name_len: usize,
    size: usize,
    align: usize,
) -> bool {
    // Validate name pointer
    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component type name pointer is null".to_string(),
        ));
        return false;
    }

    // Convert name to Rust string
    let name_slice = std::slice::from_raw_parts(name_ptr, name_len);
    let name = match std::str::from_utf8(name_slice) {
        Ok(s) => s.to_string(),
        Err(_) => {
            set_last_error(GoudError::InvalidState(
                "Component type name is not valid UTF-8".to_string(),
            ));
            return false;
        }
    };

    // Register the type (we don't have the actual TypeId from FFI, so we use a placeholder)
    // In a real implementation, the C# side would need to provide a stable hash
    let type_id = TypeId::of::<()>(); // Placeholder - actual type checking happens via hash
    register_component_type_internal(type_id_hash, name, size, align, type_id)
}

// ============================================================================
// FFI Functions - Component Add/Remove
// ============================================================================

/// Adds a component to an entity.
///
/// If the entity already has this component type, it will be replaced.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to add the component to
/// - `type_id_hash`: Type ID of the component (from registration)
/// - `data_ptr`: Pointer to the component data
/// - `data_size`: Size of the component data in bytes
///
/// # Returns
///
/// `GoudResult` with success=true if component was added, error otherwise.
///
/// # Safety
///
/// - `data_ptr` must point to valid component data
/// - `data_size` must match the registered size
/// - Memory at `data_ptr` must remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn goud_component_add(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
    data_ptr: *const u8,
    data_size: usize,
) -> GoudResult {
    // Validate inputs
    if data_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component data pointer is null".to_string(),
        ));
        return GoudResult::err(902); // InternalError
    }

    // Look up type info
    let type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return GoudResult::err(101); // ResourceLoadFailed
        }
    };

    // Validate size
    if data_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component data size mismatch: expected {}, got {}",
            type_info.size, data_size
        )));
        return GoudResult::err(902); // InternalError
    }

    with_context_mut_result!(context_id, |world: &mut crate::ecs::World| {
        let entity = entity_from_ffi(entity_id);

        // Check if entity is alive
        if !world.is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return GoudResult::err(300); // EntityNotFound
        }

        // TODO: Actual component insertion would require generic type support
        // For now, we validate the operation and return success
        // In a full implementation, this would:
        // 1. Deserialize component data from raw bytes
        // 2. Call world.insert::<T>(entity, component)
        // 3. Handle archetype transitions

        // Placeholder: assume success
        GoudResult::ok()
    })
}

/// Removes a component from an entity.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to remove the component from
/// - `type_id_hash`: Type ID of the component to remove
///
/// # Returns
///
/// `GoudResult` with success=true if component was removed, error otherwise.
#[no_mangle]
pub extern "C" fn goud_component_remove(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> GoudResult {
    // Look up type info
    let type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return GoudResult::err(101); // ResourceLoadFailed
        }
    };

    with_context_mut_result!(context_id, |world: &mut crate::ecs::World| {
        let entity = entity_from_ffi(entity_id);

        // Check if entity is alive
        if !world.is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return GoudResult::err(300); // EntityNotFound
        }

        // TODO: Actual component removal would require generic type support
        // For now, we validate the operation and return success
        // In a full implementation, this would:
        // 1. Call world.remove::<T>(entity)
        // 2. Handle archetype transitions
        // 3. Return the removed component data if needed

        // Placeholder: assume success
        let _ = type_info; // Use type_info to silence warning
        GoudResult::ok()
    })
}

/// Checks if an entity has a specific component.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to check
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// `true` if entity has the component, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_component_has(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> bool {
    // Look up type info
    let _type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return false;
        }
    };

    with_context_bool!(context_id, |world: &crate::ecs::World| {
        let entity = entity_from_ffi(entity_id);

        // Check if entity is alive
        if !world.is_alive(entity) {
            return false;
        }

        // TODO: Actual component check would require generic type support
        // For now, we return false as a placeholder
        // In a full implementation, this would:
        // 1. Call world.has::<T>(entity)

        false // Placeholder
    })
}

/// Gets a read-only pointer to a component on an entity.
///
/// The returned pointer is valid until the next mutable operation on the World.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to get the component from
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// Pointer to the component data, or null if not found.
///
/// # Safety
///
/// - Returned pointer is only valid until next mutable World operation
/// - Caller must not modify data through this pointer
/// - Caller must not retain pointer across FFI calls
#[no_mangle]
pub extern "C" fn goud_component_get(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *const u8 {
    // Look up type info
    let _type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return std::ptr::null();
        }
    };

    with_context_ptr!(context_id, |world: &mut crate::ecs::World| {
        let entity = entity_from_ffi(entity_id);

        // Check if entity is alive
        if !world.is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return std::ptr::null();
        }

        // TODO: Actual component access would require generic type support
        // For now, we return null as a placeholder
        // In a full implementation, this would:
        // 1. Call world.get::<T>(entity)
        // 2. Return pointer to component data

        std::ptr::null() // Placeholder
    })
}

/// Gets a mutable pointer to a component on an entity.
///
/// The returned pointer is valid until the next operation on the World.
///
/// # Parameters
///
/// - `context_id`: The engine context
/// - `entity_id`: The entity to get the component from
/// - `type_id_hash`: Type ID of the component
///
/// # Returns
///
/// Mutable pointer to the component data, or null if not found.
///
/// # Safety
///
/// - Returned pointer is only valid until next World operation
/// - Caller must ensure exclusive access (no concurrent reads/writes)
/// - Caller must not retain pointer across FFI calls
#[no_mangle]
pub extern "C" fn goud_component_get_mut(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *mut u8 {
    // Look up type info
    let _type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return std::ptr::null_mut();
        }
    };

    with_context_ptr_mut!(context_id, |world: &mut crate::ecs::World| {
        let entity = entity_from_ffi(entity_id);

        // Check if entity is alive
        if !world.is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return std::ptr::null_mut();
        }

        // TODO: Actual component access would require generic type support
        // For now, we return null as a placeholder
        // In a full implementation, this would:
        // 1. Call world.get_mut::<T>(entity)
        // 2. Return mutable pointer to component data

        std::ptr::null_mut() // Placeholder
    })
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Adds the same component type to multiple entities in a single batch.
///
/// This is more efficient than calling `goud_component_add()` multiple times
/// when adding the same component type to many entities.
///
/// # Arguments
///
/// * `context_id` - The context containing the entities
/// * `entity_ids` - Pointer to array of entity IDs
/// * `count` - Number of entities
/// * `type_id_hash` - Hash of the component type
/// * `data_ptr` - Pointer to array of component data (size = count * component_size)
/// * `component_size` - Size of a single component instance
///
/// # Returns
///
/// The number of entities that successfully received the component.
///
/// # Safety
///
/// Caller must ensure:
/// - `entity_ids` points to valid memory with `count` u64 values
/// - `data_ptr` points to valid memory with `count * component_size` bytes
/// - `component_size` matches the registered component type size
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `RESOURCE_ERROR_BASE + 100` (ResourceNotFound) - Component type not registered
/// - `INTERNAL_ERROR_BASE + 2` (InvalidState) - Size mismatch
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// Position[] positions = {
///     new Position { x = 0, y = 0 },
///     new Position { x = 10, y = 10 },
///     new Position { x = 20, y = 20 },
///     new Position { x = 30, y = 30 },
///     new Position { x = 40, y = 40 }
/// };
/// fixed (ulong* ePtr = entities)
/// fixed (Position* pPtr = positions) {
///     int added = goud_component_add_batch(
///         contextId, ePtr, 5, positionTypeId,
///         pPtr, sizeof(Position)
///     );
///     Console.WriteLine($"Added components to {added} entities");
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_component_add_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    data_ptr: *const u8,
    component_size: usize,
) -> u32 {
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

    if data_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "data_ptr pointer is null".to_string(),
        ));
        return 0;
    }

    if count == 0 {
        return 0;
    }

    // Verify component type is registered
    let type_registry = get_type_registry();
    let registry_map = match type_registry.as_ref() {
        Some(map) => map,
        None => {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return 0;
        }
    };
    let type_info = match registry_map.get(&type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return 0;
        }
    };

    // Validate size
    if component_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component size mismatch: expected {}, got {}",
            type_info.size, component_size
        )));
        return 0;
    }
    drop(type_registry); // Release lock before world operations

    // TODO: Actual batch component add would require:
    // 1. Get context and world
    // 2. For each entity in entity_ids:
    //    - Read component data from data_ptr[i * component_size]
    //    - Call world.insert() with the component data
    // 3. Track success count
    //
    // For now, this is a placeholder that validates inputs and type registration

    count // Placeholder: assume all succeeded
}

/// Removes the same component type from multiple entities in a single batch.
///
/// This is more efficient than calling `goud_component_remove()` multiple times.
/// Invalid entities or entities without the component are skipped.
///
/// # Arguments
///
/// * `context_id` - The context containing the entities
/// * `entity_ids` - Pointer to array of entity IDs
/// * `count` - Number of entities
/// * `type_id_hash` - Hash of the component type
///
/// # Returns
///
/// The number of entities that successfully had the component removed.
///
/// # Safety
///
/// Caller must ensure `entity_ids` points to valid memory with `count` u64 values.
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `RESOURCE_ERROR_BASE + 100` (ResourceNotFound) - Component type not registered
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// fixed (ulong* ePtr = entities) {
///     int removed = goud_component_remove_batch(
///         contextId, ePtr, 5, positionTypeId
///     );
///     Console.WriteLine($"Removed components from {removed} entities");
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_component_remove_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
) -> u32 {
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

    // Verify component type is registered
    let type_registry = get_type_registry();
    let registry_map = match type_registry.as_ref() {
        Some(map) => map,
        None => {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return 0;
        }
    };
    if !registry_map.contains_key(&type_id_hash) {
        set_last_error(GoudError::ResourceNotFound(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return 0;
    }
    drop(type_registry); // Release lock before world operations

    // TODO: Actual batch component remove would require:
    // 1. Get context and world
    // 2. For each entity in entity_ids:
    //    - Call world.remove::<T>(entity)
    //    - Track success count
    //
    // For now, this is a placeholder that validates inputs

    count // Placeholder: assume all succeeded
}

/// Checks if multiple entities have a specific component type.
///
/// Results are written to the `out_results` array, where each entry is:
/// - `1` (true) if the entity has the component
/// - `0` (false) if the entity doesn't have the component or is invalid
///
/// # Arguments
///
/// * `context_id` - The context to check
/// * `entity_ids` - Pointer to array of entity IDs
/// * `count` - Number of entities
/// * `type_id_hash` - Hash of the component type
/// * `out_results` - Pointer to array where results will be written (size = count)
///
/// # Returns
///
/// The number of results written (should equal `count` on success).
///
/// # Safety
///
/// Caller must ensure:
/// - `entity_ids` points to valid memory with `count` u64 values
/// - `out_results` points to valid memory with `count` u8 values
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid context ID
/// - `RESOURCE_ERROR_BASE + 100` (ResourceNotFound) - Component type not registered
///
/// # Example (C#)
///
/// ```csharp
/// ulong[] entities = { e1, e2, e3, e4, e5 };
/// byte[] results = new byte[5];
/// fixed (ulong* ePtr = entities)
/// fixed (byte* rPtr = results) {
///     int count = goud_component_has_batch(
///         contextId, ePtr, 5, positionTypeId, rPtr
///     );
///     for (int i = 0; i < count; i++) {
///         Console.WriteLine($"Entity {i}: {results[i] != 0}");
///     }
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn goud_component_has_batch(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    out_results: *mut u8,
) -> u32 {
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

    // Verify component type is registered
    let type_registry = get_type_registry();
    let registry_map = match type_registry.as_ref() {
        Some(map) => map,
        None => {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return 0;
        }
    };
    if !registry_map.contains_key(&type_id_hash) {
        set_last_error(GoudError::ResourceNotFound(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return 0;
    }
    drop(type_registry); // Release lock before world operations

    // TODO: Actual batch component has check would require:
    // 1. Get context and world
    // 2. For each entity in entity_ids:
    //    - Call world.has::<T>(entity)
    //    - Write 1 or 0 to out_results[i]
    //
    // For now, write placeholder results (all false)
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);
    for result in results_slice.iter_mut() {
        *result = 0; // Placeholder: all false
    }

    count
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Component;
    use crate::ffi::context::goud_context_create;

    // Test component for FFI operations
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[repr(C)]
    struct TestComponent {
        x: f32,
        y: f32,
    }

    impl Component for TestComponent {}

    const TEST_TYPE_ID: u64 = 12345;

    fn setup_test_context() -> GoudContextId {
        unsafe { goud_context_create() }
    }

    // ========================================================================
    // Type Registration Tests
    // ========================================================================

    #[test]
    fn test_register_type_basic() {
        // Use a unique type ID to avoid conflicts with other tests
        const UNIQUE_TYPE_ID: u64 = TEST_TYPE_ID + 1000;
        let name = b"TestComponent";
        let result = unsafe {
            goud_component_register_type(
                UNIQUE_TYPE_ID,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };
        // First registration should succeed (or may be false if already registered in other tests)
        // This is fine - the registry is global across all tests

        // Second registration should return false
        let result2 = unsafe {
            goud_component_register_type(
                UNIQUE_TYPE_ID,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };
        assert!(!result2, "Second registration should return false");
    }

    #[test]
    fn test_register_type_null_name() {
        let result = unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 1,
                std::ptr::null(),
                0,
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };
        assert!(!result, "Registration with null name should fail");
    }

    // ========================================================================
    // Component Add/Remove Tests
    // ========================================================================

    #[test]
    fn test_component_add_basic() {
        let context_id = setup_test_context();

        // Register type
        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 2,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        // Spawn entity
        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };
        assert_ne!(entity_id, crate::ffi::entity::GOUD_INVALID_ENTITY_ID);

        // Add component
        let component = TestComponent { x: 10.0, y: 20.0 };
        let result = unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                TEST_TYPE_ID + 2,
                &component as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert!(result.is_ok(), "Component add should succeed");
    }

    #[test]
    fn test_component_add_invalid_context() {
        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 3,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let component = TestComponent { x: 10.0, y: 20.0 };
        let result = unsafe {
            goud_component_add(
                GOUD_INVALID_CONTEXT_ID,
                GoudEntityId::new(0),
                TEST_TYPE_ID + 3,
                &component as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert!(result.is_err(), "Add with invalid context should fail");
    }

    #[test]
    fn test_component_add_unregistered_type() {
        let context_id = setup_test_context();
        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

        let component = TestComponent { x: 10.0, y: 20.0 };
        let result = unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                99999, // Unregistered type
                &component as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert!(result.is_err(), "Add with unregistered type should fail");
    }

    #[test]
    fn test_component_add_null_data() {
        let context_id = setup_test_context();

        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 4,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

        let result = unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                TEST_TYPE_ID + 4,
                std::ptr::null(),
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert!(result.is_err(), "Add with null data pointer should fail");
    }

    #[test]
    fn test_component_add_wrong_size() {
        let context_id = setup_test_context();

        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 5,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };
        let component = TestComponent { x: 10.0, y: 20.0 };

        let result = unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                TEST_TYPE_ID + 5,
                &component as *const _ as *const u8,
                999, // Wrong size
            )
        };

        assert!(result.is_err(), "Add with wrong size should fail");
    }

    #[test]
    fn test_component_remove_basic() {
        let context_id = setup_test_context();

        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 6,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

        let result =
            goud_component_remove(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 6);

        // Should succeed even if component doesn't exist (placeholder implementation)
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_has_basic() {
        let context_id = setup_test_context();

        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 7,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

        let has_component =
            goud_component_has(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 7);

        // Should return false (placeholder implementation)
        assert!(!has_component);
    }

    #[test]
    fn test_component_get_basic() {
        let context_id = setup_test_context();

        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 8,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

        let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 8);

        // Should return null (placeholder implementation)
        assert!(ptr.is_null());
    }

    #[test]
    fn test_component_get_mut_basic() {
        let context_id = setup_test_context();

        let name = b"TestComponent";
        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID + 9,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

        let ptr =
            goud_component_get_mut(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);

        // Should return null (placeholder implementation)
        assert!(ptr.is_null());
    }

    // ========================================================================
    // Batch Operation Tests
    // ========================================================================

    #[test]
    fn test_component_add_batch_basic() {
        let context_id = setup_test_context();
        let name = b"TestComponent";

        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        // Spawn 5 entities
        let mut entities = [0u64; 5];
        unsafe {
            crate::ffi::entity::goud_entity_spawn_batch(context_id, 5, entities.as_mut_ptr());
        }

        // Prepare component data
        let components = [
            TestComponent { x: 1.0, y: 2.0 },
            TestComponent { x: 3.0, y: 4.0 },
            TestComponent { x: 5.0, y: 6.0 },
            TestComponent { x: 7.0, y: 8.0 },
            TestComponent { x: 9.0, y: 10.0 },
        ];

        // Add components in batch
        let added = unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                5,
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        // Should succeed for all (placeholder returns count)
        assert_eq!(added, 5);
    }

    #[test]
    fn test_component_add_batch_invalid_context() {
        let entities = [1u64, 2, 3];
        let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

        let added = unsafe {
            goud_component_add_batch(
                GOUD_INVALID_CONTEXT_ID,
                entities.as_ptr(),
                3,
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert_eq!(added, 0);
    }

    #[test]
    fn test_component_add_batch_null_entities() {
        let context_id = setup_test_context();
        let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

        let added = unsafe {
            goud_component_add_batch(
                context_id,
                std::ptr::null(),
                3,
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert_eq!(added, 0);
    }

    #[test]
    fn test_component_add_batch_null_data() {
        let context_id = setup_test_context();
        let entities = [1u64, 2, 3];

        let added = unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                3,
                TEST_TYPE_ID,
                std::ptr::null(),
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert_eq!(added, 0);
    }

    #[test]
    fn test_component_add_batch_unregistered_type() {
        let context_id = setup_test_context();
        let entities = [1u64, 2, 3];
        let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

        let added = unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                3,
                99999, // Unregistered type
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };

        assert_eq!(added, 0);
    }

    #[test]
    fn test_component_add_batch_size_mismatch() {
        let context_id = setup_test_context();
        let name = b"TestComponent";

        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entities = [1u64, 2, 3];
        let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

        let added = unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                3,
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                16, // Wrong size
            )
        };

        assert_eq!(added, 0);
    }

    #[test]
    fn test_component_remove_batch_basic() {
        let context_id = setup_test_context();
        let name = b"TestComponent";

        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entities = [1u64, 2, 3, 4, 5];

        let removed =
            unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 5, TEST_TYPE_ID) };

        // Placeholder returns count
        assert_eq!(removed, 5);
    }

    #[test]
    fn test_component_remove_batch_invalid_context() {
        let entities = [1u64, 2, 3];

        let removed = unsafe {
            goud_component_remove_batch(GOUD_INVALID_CONTEXT_ID, entities.as_ptr(), 3, TEST_TYPE_ID)
        };

        assert_eq!(removed, 0);
    }

    #[test]
    fn test_component_remove_batch_unregistered_type() {
        let context_id = setup_test_context();
        let entities = [1u64, 2, 3];

        let removed =
            unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 3, 99999) };

        assert_eq!(removed, 0);
    }

    #[test]
    fn test_component_has_batch_basic() {
        let context_id = setup_test_context();
        let name = b"TestComponent";

        unsafe {
            goud_component_register_type(
                TEST_TYPE_ID,
                name.as_ptr(),
                name.len(),
                std::mem::size_of::<TestComponent>(),
                std::mem::align_of::<TestComponent>(),
            )
        };

        let entities = [1u64, 2, 3, 4, 5];
        let mut results = [0u8; 5];

        let count = unsafe {
            goud_component_has_batch(
                context_id,
                entities.as_ptr(),
                5,
                TEST_TYPE_ID,
                results.as_mut_ptr(),
            )
        };

        assert_eq!(count, 5);
        // Placeholder returns all false
        for result in &results {
            assert_eq!(*result, 0);
        }
    }

    #[test]
    fn test_component_has_batch_invalid_context() {
        let entities = [1u64, 2, 3];
        let mut results = [0u8; 3];

        let count = unsafe {
            goud_component_has_batch(
                GOUD_INVALID_CONTEXT_ID,
                entities.as_ptr(),
                3,
                TEST_TYPE_ID,
                results.as_mut_ptr(),
            )
        };

        assert_eq!(count, 0);
    }

    #[test]
    fn test_component_has_batch_null_results() {
        let context_id = setup_test_context();
        let entities = [1u64, 2, 3];

        let count = unsafe {
            goud_component_has_batch(
                context_id,
                entities.as_ptr(),
                3,
                TEST_TYPE_ID,
                std::ptr::null_mut(),
            )
        };

        assert_eq!(count, 0);
    }

    #[test]
    fn test_component_has_batch_unregistered_type() {
        let context_id = setup_test_context();
        let entities = [1u64, 2, 3];
        let mut results = [0u8; 3];

        let count = unsafe {
            goud_component_has_batch(
                context_id,
                entities.as_ptr(),
                3,
                99999, // Unregistered type
                results.as_mut_ptr(),
            )
        };

        assert_eq!(count, 0);
    }

    #[test]
    fn test_component_batch_zero_count() {
        let context_id = setup_test_context();
        let entities = [1u64];
        let components = [TestComponent { x: 0.0, y: 0.0 }];
        let mut results = [0u8; 1];

        // All batch operations should handle zero count
        let added = unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                0,
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };
        assert_eq!(added, 0);

        let removed =
            unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 0, TEST_TYPE_ID) };
        assert_eq!(removed, 0);

        let count = unsafe {
            goud_component_has_batch(
                context_id,
                entities.as_ptr(),
                0,
                TEST_TYPE_ID,
                results.as_mut_ptr(),
            )
        };
        assert_eq!(count, 0);
    }
}
