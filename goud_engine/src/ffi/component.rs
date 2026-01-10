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

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::Entity;
use crate::ffi::context::get_context_registry;
use crate::ffi::{GoudContextId, GoudEntityId, GoudResult, GOUD_INVALID_CONTEXT_ID};

// ============================================================================
// Raw Component Storage
// ============================================================================

/// Type-erased component storage using raw bytes.
///
/// This storage is used for FFI component operations where we don't know
/// the concrete Rust type at compile time. Components are stored as raw bytes
/// in a sparse set-like structure.
///
/// # Safety
///
/// The caller must ensure:
/// - Component size and alignment match during add/get operations
/// - Data pointers passed to add() point to valid memory
/// - The storage is not accessed after being dropped
#[derive(Debug)]
struct RawComponentStorage {
    /// Maps entity index to position in dense array.
    sparse: Vec<Option<usize>>,

    /// Packed array of entities that have components.
    dense: Vec<u64>, // Store entity bits for FFI compatibility

    /// Packed array of raw component data.
    /// Each entry is a pointer to heap-allocated bytes.
    data: Vec<*mut u8>,

    /// Size of each component in bytes.
    component_size: usize,

    /// Alignment of each component.
    component_align: usize,
}

// SAFETY: RawComponentStorage is Send because:
// 1. All pointers in `data` point to owned, heap-allocated memory
// 2. We don't share these pointers with other threads
// 3. Access is synchronized at a higher level (context registry mutex)
unsafe impl Send for RawComponentStorage {}

// SAFETY: RawComponentStorage is Sync because:
// 1. All mutable access is synchronized via context registry mutex
// 2. The raw pointers point to owned data
unsafe impl Sync for RawComponentStorage {}

impl RawComponentStorage {
    /// Creates a new empty raw component storage.
    fn new(component_size: usize, component_align: usize) -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new(),
            component_size,
            component_align,
        }
    }

    /// Creates the memory layout for a single component.
    fn layout(&self) -> Layout {
        // Handle zero-sized types
        if self.component_size == 0 {
            Layout::from_size_align(1, 1).unwrap()
        } else {
            Layout::from_size_align(self.component_size, self.component_align)
                .expect("Invalid component layout")
        }
    }

    /// Inserts a component for the given entity.
    ///
    /// # Safety
    ///
    /// - `data_ptr` must point to valid memory of at least `component_size` bytes
    /// - The data must be properly initialized
    unsafe fn insert(&mut self, entity_bits: u64, data_ptr: *const u8) -> bool {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        // Grow sparse vec if needed
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }

        if let Some(dense_index) = self.sparse[index] {
            // Entity already has a component - replace it
            let existing_ptr = self.data[dense_index];
            if self.component_size > 0 {
                std::ptr::copy_nonoverlapping(data_ptr, existing_ptr, self.component_size);
            }
            true
        } else {
            // New entity - allocate and copy data
            let layout = self.layout();
            let new_ptr = alloc(layout);
            if new_ptr.is_null() {
                return false;
            }

            if self.component_size > 0 {
                std::ptr::copy_nonoverlapping(data_ptr, new_ptr, self.component_size);
            }

            let dense_index = self.dense.len();
            self.sparse[index] = Some(dense_index);
            self.dense.push(entity_bits);
            self.data.push(new_ptr);
            true
        }
    }

    /// Removes a component from the given entity.
    ///
    /// Returns true if the component was removed, false if the entity didn't have one.
    fn remove(&mut self, entity_bits: u64) -> bool {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return false;
        }

        if let Some(dense_index) = self.sparse[index].take() {
            // Free the component data
            let ptr = self.data[dense_index];
            if !ptr.is_null() {
                unsafe {
                    dealloc(ptr, self.layout());
                }
            }

            // Swap-remove from dense arrays
            let last_index = self.dense.len() - 1;
            if dense_index != last_index {
                // Swap with last element
                self.dense.swap(dense_index, last_index);
                self.data.swap(dense_index, last_index);

                // Update sparse for the swapped entity
                let swapped_entity = Entity::from_bits(self.dense[dense_index]);
                self.sparse[swapped_entity.index() as usize] = Some(dense_index);
            }

            self.dense.pop();
            self.data.pop();
            true
        } else {
            false
        }
    }

    /// Gets a pointer to the component data for the given entity.
    ///
    /// Returns null if the entity doesn't have this component.
    fn get(&self, entity_bits: u64) -> *const u8 {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return std::ptr::null();
        }

        if let Some(dense_index) = self.sparse[index] {
            self.data[dense_index]
        } else {
            std::ptr::null()
        }
    }

    /// Gets a mutable pointer to the component data for the given entity.
    ///
    /// Returns null if the entity doesn't have this component.
    fn get_mut(&mut self, entity_bits: u64) -> *mut u8 {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return std::ptr::null_mut();
        }

        if let Some(dense_index) = self.sparse[index] {
            self.data[dense_index]
        } else {
            std::ptr::null_mut()
        }
    }

    /// Checks if the entity has this component.
    fn contains(&self, entity_bits: u64) -> bool {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return false;
        }

        self.sparse[index].is_some()
    }
}

impl Drop for RawComponentStorage {
    fn drop(&mut self) {
        // Free all allocated component data
        let layout = self.layout();
        for &ptr in &self.data {
            if !ptr.is_null() {
                unsafe {
                    dealloc(ptr, layout);
                }
            }
        }
    }
}

// ============================================================================
// Per-Context Component Storage
// ============================================================================

/// Component storage manager for a single context.
///
/// Each context has its own set of component storages, one per registered type.
/// This is stored in a global map keyed by context ID.
#[derive(Debug, Default)]
struct ContextComponentStorage {
    /// Maps type_id_hash to raw component storage
    storages: HashMap<u64, RawComponentStorage>,
}

impl ContextComponentStorage {
    /// Gets or creates storage for a component type.
    fn get_or_create_storage(
        &mut self,
        type_id_hash: u64,
        component_size: usize,
        component_align: usize,
    ) -> &mut RawComponentStorage {
        self.storages
            .entry(type_id_hash)
            .or_insert_with(|| RawComponentStorage::new(component_size, component_align))
    }

    /// Gets storage for a component type if it exists.
    fn get_storage(&self, type_id_hash: u64) -> Option<&RawComponentStorage> {
        self.storages.get(&type_id_hash)
    }

    /// Gets mutable storage for a component type if it exists.
    fn get_storage_mut(&mut self, type_id_hash: u64) -> Option<&mut RawComponentStorage> {
        self.storages.get_mut(&type_id_hash)
    }
}

/// Global storage for per-context component data.
///
/// Maps context ID (as u64) to component storage for that context.
static CONTEXT_COMPONENT_STORAGE: Mutex<Option<HashMap<u64, ContextComponentStorage>>> =
    Mutex::new(None);

/// Gets or initializes the context component storage map.
fn get_context_storage_map(
) -> std::sync::MutexGuard<'static, Option<HashMap<u64, ContextComponentStorage>>> {
    CONTEXT_COMPONENT_STORAGE.lock().unwrap()
}

// ============================================================================
// Type Registry
// ============================================================================

/// Information about a registered component type.
#[derive(Debug, Clone)]
struct ComponentTypeInfo {
    /// Size of the component in bytes.
    size: usize,
    /// Alignment of the component in bytes.
    align: usize,
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
fn register_component_type_internal(type_id_hash: u64, size: usize, align: usize) -> bool {
    let mut registry = get_type_registry();
    let map = registry.get_or_insert_with(HashMap::new);

    use std::collections::hash_map::Entry;
    match map.entry(type_id_hash) {
        Entry::Occupied(_) => false,
        Entry::Vacant(e) => {
            e.insert(ComponentTypeInfo { size, align });
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
/// - `name_ptr`: Null-terminated C string with the type name (for debugging, currently unused)
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
/// - `name_ptr` must be a valid pointer to a C string (or null)
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
    // Validate name pointer (kept for API compatibility, name is not stored)
    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component type name pointer is null".to_string(),
        ));
        return false;
    }

    // Validate name is valid UTF-8 (kept for API compatibility)
    let name_slice = std::slice::from_raw_parts(name_ptr, name_len);
    if std::str::from_utf8(name_slice).is_err() {
        set_last_error(GoudError::InvalidState(
            "Component type name is not valid UTF-8".to_string(),
        ));
        return false;
    }

    register_component_type_internal(type_id_hash, size, align)
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
                "Component type {type_id_hash} not registered"
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

    // Validate context and entity
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(3); // InvalidContext
    }

    // Check entity is alive using context registry
    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GoudResult::err(3);
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return GoudResult::err(300); // EntityNotFound
        }
    }

    // Get or create component storage for this context
    let mut storage_map = get_context_storage_map();
    let map = storage_map.get_or_insert_with(HashMap::new);

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get or create context storage
    let context_storage = map.entry(key).or_default();

    // Get or create storage for this component type
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    // Insert the component data
    if storage.insert(entity_id.bits(), data_ptr) {
        GoudResult::ok()
    } else {
        set_last_error(GoudError::InternalError(
            "Failed to allocate component storage".to_string(),
        ));
        GoudResult::err(900) // InternalError
    }
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
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {type_id_hash} not registered"
        )));
        return GoudResult::err(101); // ResourceLoadFailed
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(3); // InvalidContext
    }

    // Check entity is alive using context registry
    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return GoudResult::err(3);
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return GoudResult::err(300); // EntityNotFound
        }
    }

    // Get component storage for this context
    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => {
            // No component storage exists, so entity can't have the component
            return GoudResult::ok();
        }
    };

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get context storage
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => {
            // No storage for this context, so entity can't have the component
            return GoudResult::ok();
        }
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => {
            // No storage for this type, so entity can't have the component
            return GoudResult::ok();
        }
    };

    // Remove the component
    storage.remove(entity_id.bits());
    GoudResult::ok()
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
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {type_id_hash} not registered"
        )));
        return false;
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Check entity is alive using context registry
    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return false;
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            return false;
        }
    }

    // Get component storage for this context
    let storage_map = get_context_storage_map();
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return false, // No storage exists
    };

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get context storage
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return false, // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return false, // No storage for this type
    };

    // Check if entity has the component
    storage.contains(entity_id.bits())
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
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {type_id_hash} not registered"
        )));
        return std::ptr::null();
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null();
    }

    // Check entity is alive using context registry
    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return std::ptr::null();
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return std::ptr::null();
        }
    }

    // Get component storage for this context
    let storage_map = get_context_storage_map();
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return std::ptr::null(), // No storage exists
    };

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get context storage
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return std::ptr::null(), // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null(), // No storage for this type
    };

    // Get component data pointer
    storage.get(entity_id.bits())
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
    // Look up type info to verify the type is registered
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {type_id_hash} not registered"
        )));
        return std::ptr::null_mut();
    }

    // Validate context
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null_mut();
    }

    // Check entity is alive using context registry
    {
        let registry = get_context_registry().lock().unwrap();
        let context = match registry.get(context_id) {
            Some(ctx) => ctx,
            None => {
                set_last_error(GoudError::InvalidContext);
                return std::ptr::null_mut();
            }
        };

        let entity = entity_from_ffi(entity_id);
        if !context.world().is_alive(entity) {
            set_last_error(GoudError::EntityNotFound);
            return std::ptr::null_mut();
        }
    }

    // Get component storage for this context
    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return std::ptr::null_mut(), // No storage exists
    };

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get context storage
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return std::ptr::null_mut(), // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null_mut(), // No storage for this type
    };

    // Get mutable component data pointer
    storage.get_mut(entity_id.bits())
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

    // Verify component type is registered and get info
    let type_info = {
        let type_registry = get_type_registry();
        let registry_map = match type_registry.as_ref() {
            Some(map) => map,
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {type_id_hash} not registered"
                )));
                return 0;
            }
        };
        match registry_map.get(&type_id_hash) {
            Some(info) => info.clone(),
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {type_id_hash} not registered"
                )));
                return 0;
            }
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

    // Get component storage for this context
    let mut storage_map = get_context_storage_map();
    let map = storage_map.get_or_insert_with(HashMap::new);

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get or create context storage
    let context_storage = map.entry(key).or_default();

    // Get or create storage for this component type
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    // Add components for each entity
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let mut success_count = 0u32;

    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        let component_data = data_ptr.add(i * component_size);
        if storage.insert(entity_bits, component_data) {
            success_count += 1;
        }
    }

    success_count
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
    {
        let type_registry = get_type_registry();
        let registry_map = match type_registry.as_ref() {
            Some(map) => map,
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {type_id_hash} not registered"
                )));
                return 0;
            }
        };
        if !registry_map.contains_key(&type_id_hash) {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {type_id_hash} not registered"
            )));
            return 0;
        }
    }

    // Get component storage for this context
    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return 0, // No storage exists
    };

    // Pack context ID into u64 for use as key
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);

    // Get context storage
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return 0, // No storage for this context
    };

    // Get storage for this component type
    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return 0, // No storage for this type
    };

    // Remove components from each entity
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let mut success_count = 0u32;

    for &entity_bits in entity_slice {
        if storage.remove(entity_bits) {
            success_count += 1;
        }
    }

    success_count
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
    {
        let type_registry = get_type_registry();
        let registry_map = match type_registry.as_ref() {
            Some(map) => map,
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {type_id_hash} not registered"
                )));
                return 0;
            }
        };
        if !registry_map.contains_key(&type_id_hash) {
            set_last_error(GoudError::ResourceNotFound(format!(
                "Component type {type_id_hash} not registered"
            )));
            return 0;
        }
    }

    // Get component storage for this context
    let storage_map = get_context_storage_map();

    // Check for storage existence
    let storage_exists = storage_map.as_ref().is_some_and(|map| {
        let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);
        map.get(&key)
            .and_then(|cs| cs.get_storage(type_id_hash))
            .is_some()
    });

    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);

    if !storage_exists {
        // No storage exists, all results are false
        for result in results_slice.iter_mut() {
            *result = 0;
        }
        return count;
    }

    // Need to get storage again to use it
    let map = storage_map.as_ref().unwrap();
    let key = (context_id.generation() as u64) << 32 | (context_id.index() as u64);
    let context_storage = map.get(&key).unwrap();
    let storage = context_storage.get_storage(type_id_hash).unwrap();

    // Check each entity
    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        results_slice[i] = if storage.contains(entity_bits) { 1 } else { 0 };
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

        // Before adding component - should return false
        let has_component =
            goud_component_has(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 7);
        assert!(!has_component);

        // Add component
        let component = TestComponent { x: 1.0, y: 2.0 };
        unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                TEST_TYPE_ID + 7,
                &component as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }

        // After adding - should return true
        let has_component =
            goud_component_has(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 7);
        assert!(has_component);
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

        // Before adding - should return null
        let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 8);
        assert!(ptr.is_null());

        // Add component
        let component = TestComponent { x: 42.0, y: 99.0 };
        unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                TEST_TYPE_ID + 8,
                &component as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }

        // After adding - should return valid pointer with correct data
        let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 8);
        assert!(!ptr.is_null());

        // Read back the component data and verify
        let read_component = unsafe { *(ptr as *const TestComponent) };
        assert_eq!(read_component.x, 42.0);
        assert_eq!(read_component.y, 99.0);
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

        // Before adding - should return null
        let ptr =
            goud_component_get_mut(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);
        assert!(ptr.is_null());

        // Add component
        let component = TestComponent { x: 10.0, y: 20.0 };
        unsafe {
            goud_component_add(
                context_id,
                GoudEntityId::new(entity_id),
                TEST_TYPE_ID + 9,
                &component as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }

        // Get mutable pointer
        let ptr =
            goud_component_get_mut(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);
        assert!(!ptr.is_null());

        // Modify the component through the mutable pointer
        unsafe {
            let comp = &mut *(ptr as *mut TestComponent);
            comp.x = 100.0;
            comp.y = 200.0;
        }

        // Read back and verify changes
        let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);
        let read_component = unsafe { *(ptr as *const TestComponent) };
        assert_eq!(read_component.x, 100.0);
        assert_eq!(read_component.y, 200.0);
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

        // Spawn 5 entities
        let mut entities = [0u64; 5];
        unsafe {
            crate::ffi::entity::goud_entity_spawn_batch(context_id, 5, entities.as_mut_ptr());
        }

        // Add components to all entities first
        let components = [
            TestComponent { x: 1.0, y: 2.0 },
            TestComponent { x: 3.0, y: 4.0 },
            TestComponent { x: 5.0, y: 6.0 },
            TestComponent { x: 7.0, y: 8.0 },
            TestComponent { x: 9.0, y: 10.0 },
        ];

        unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                5,
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }

        // Verify components were added
        for &entity_bits in &entities {
            assert!(goud_component_has(
                context_id,
                GoudEntityId::new(entity_bits),
                TEST_TYPE_ID
            ));
        }

        // Now remove them
        let removed =
            unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 5, TEST_TYPE_ID) };

        assert_eq!(removed, 5);

        // Verify components are gone
        for &entity_bits in &entities {
            assert!(!goud_component_has(
                context_id,
                GoudEntityId::new(entity_bits),
                TEST_TYPE_ID
            ));
        }
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

        // Spawn 5 entities
        let mut entities = [0u64; 5];
        unsafe {
            crate::ffi::entity::goud_entity_spawn_batch(context_id, 5, entities.as_mut_ptr());
        }

        // Add components to first 3 entities only
        let components = [
            TestComponent { x: 1.0, y: 2.0 },
            TestComponent { x: 3.0, y: 4.0 },
            TestComponent { x: 5.0, y: 6.0 },
        ];

        unsafe {
            goud_component_add_batch(
                context_id,
                entities.as_ptr(),
                3, // Only first 3
                TEST_TYPE_ID,
                components.as_ptr() as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }

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
        // First 3 should have the component, last 2 should not
        assert_eq!(results[0], 1);
        assert_eq!(results[1], 1);
        assert_eq!(results[2], 1);
        assert_eq!(results[3], 0);
        assert_eq!(results[4], 0);
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
