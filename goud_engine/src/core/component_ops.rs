//! # Core Component Operations
//!
//! This module provides the implementation logic for type-erased component
//! operations: register, add, remove, has, get, get_mut, and batch variants.
//!
//! These `_impl` functions contain the actual logic and are called by both
//! the FFI `#[no_mangle]` wrappers and the Rust SDK wrapper types.
//!
//! ## Design
//!
//! Component operations use raw byte pointers and type IDs because the FFI
//! layer does not know concrete Rust types at compile time. The raw component
//! storage uses a sparse set internally.
//!
//! ## Safety
//!
//! The caller MUST ensure:
//! - Pointers point to valid component data
//! - Size/alignment match the registered type
//! - Memory remains valid for the duration of the call

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::context_registry::{get_context_registry, GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::core::error::{set_last_error, GoudError};
use crate::core::types::{GoudEntityId, GoudResult};
use crate::ecs::Entity;

// ============================================================================
// Raw Component Storage
// ============================================================================

/// Type-erased component storage using raw bytes.
///
/// This storage is used for component operations where we don't know
/// the concrete Rust type at compile time. Components are stored as raw
/// bytes in a sparse set-like structure.
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
    dense: Vec<u64>,

    /// Packed array of raw component data.
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
    /// - `data_ptr` must point to valid memory of at least `component_size`
    ///   bytes
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
                // SAFETY: We allocated this pointer with the same layout.
                unsafe {
                    dealloc(ptr, self.layout());
                }
            }

            // Swap-remove from dense arrays
            let last_index = self.dense.len() - 1;
            if dense_index != last_index {
                self.dense.swap(dense_index, last_index);
                self.data.swap(dense_index, last_index);

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
        let layout = self.layout();
        for &ptr in &self.data {
            if !ptr.is_null() {
                // SAFETY: We allocated each pointer with the same layout.
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
static COMPONENT_TYPE_REGISTRY: Mutex<Option<HashMap<u64, ComponentTypeInfo>>> = Mutex::new(None);

/// Gets or initializes the component type registry.
fn get_type_registry() -> std::sync::MutexGuard<'static, Option<HashMap<u64, ComponentTypeInfo>>> {
    COMPONENT_TYPE_REGISTRY.lock().unwrap()
}

/// Registers a component type with the given information.
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

/// Packs a context ID into a u64 key for storage maps.
#[inline]
fn context_key(context_id: GoudContextId) -> u64 {
    (context_id.generation() as u64) << 32 | (context_id.index() as u64)
}

// ============================================================================
// Implementation Functions
// ============================================================================

/// Registers a component type with the engine.
///
/// # Safety
///
/// - `name_ptr` must be a valid pointer to UTF-8 data (or null)
/// - `name_len` must match the actual string length
/// - `size` and `align` must match the actual type layout
pub unsafe fn component_register_type_impl(
    type_id_hash: u64,
    name_ptr: *const u8,
    name_len: usize,
    size: usize,
    align: usize,
) -> bool {
    if name_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component type name pointer is null".to_string(),
        ));
        return false;
    }

    // SAFETY: Caller guarantees name_ptr points to valid memory of
    // name_len bytes.
    let name_slice = std::slice::from_raw_parts(name_ptr, name_len);
    if std::str::from_utf8(name_slice).is_err() {
        set_last_error(GoudError::InvalidState(
            "Component type name is not valid UTF-8".to_string(),
        ));
        return false;
    }

    register_component_type_internal(type_id_hash, size, align)
}

/// Adds a component to an entity.
///
/// # Safety
///
/// - `data_ptr` must point to valid component data
/// - `data_size` must match the registered component size
pub unsafe fn component_add_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
    data_ptr: *const u8,
    data_size: usize,
) -> GoudResult {
    if data_ptr.is_null() {
        set_last_error(GoudError::InvalidState(
            "Component data pointer is null".to_string(),
        ));
        return GoudResult::err(902);
    }

    let type_info = match get_component_type_info(type_id_hash) {
        Some(info) => info,
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Component type {} not registered",
                type_id_hash
            )));
            return GoudResult::err(101);
        }
    };

    if data_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component data size mismatch: expected {}, got {}",
            type_info.size, data_size
        )));
        return GoudResult::err(902);
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(3);
    }

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
            return GoudResult::err(300);
        }
    }

    let mut storage_map = get_context_storage_map();
    let map = storage_map.get_or_insert_with(HashMap::new);

    let key = context_key(context_id);
    let context_storage = map.entry(key).or_default();
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    if storage.insert(entity_id.bits(), data_ptr) {
        GoudResult::ok()
    } else {
        set_last_error(GoudError::InternalError(
            "Failed to allocate component storage".to_string(),
        ));
        GoudResult::err(900)
    }
}

/// Removes a component from an entity.
pub fn component_remove_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> GoudResult {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return GoudResult::err(101);
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GoudResult::err(3);
    }

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
            return GoudResult::err(300);
        }
    }

    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return GoudResult::ok(),
    };

    let key = context_key(context_id);
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return GoudResult::ok(),
    };

    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return GoudResult::ok(),
    };

    storage.remove(entity_id.bits());
    GoudResult::ok()
}

/// Checks if an entity has a specific component.
pub fn component_has_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> bool {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return false;
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

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

    let storage_map = get_context_storage_map();
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return false,
    };

    let key = context_key(context_id);
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return false,
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return false,
    };

    storage.contains(entity_id.bits())
}

/// Gets a read-only pointer to a component on an entity.
pub fn component_get_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *const u8 {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return std::ptr::null();
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null();
    }

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

    let storage_map = get_context_storage_map();
    let map = match storage_map.as_ref() {
        Some(m) => m,
        None => return std::ptr::null(),
    };

    let key = context_key(context_id);
    let context_storage = match map.get(&key) {
        Some(s) => s,
        None => return std::ptr::null(),
    };

    let storage = match context_storage.get_storage(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null(),
    };

    storage.get(entity_id.bits())
}

/// Gets a mutable pointer to a component on an entity.
pub fn component_get_mut_impl(
    context_id: GoudContextId,
    entity_id: GoudEntityId,
    type_id_hash: u64,
) -> *mut u8 {
    if get_component_type_info(type_id_hash).is_none() {
        set_last_error(GoudError::ResourceLoadFailed(format!(
            "Component type {} not registered",
            type_id_hash
        )));
        return std::ptr::null_mut();
    }

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return std::ptr::null_mut();
    }

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

    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return std::ptr::null_mut(),
    };

    let key = context_key(context_id);
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    storage.get_mut(entity_id.bits())
}

/// Adds the same component type to multiple entities in a batch.
///
/// # Safety
///
/// - `entity_ids` must point to valid memory with `count` u64 values
/// - `data_ptr` must point to `count * component_size` bytes of data
/// - `component_size` must match the registered component type size
pub unsafe fn component_add_batch_impl(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    data_ptr: *const u8,
    component_size: usize,
) -> u32 {
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

    let type_info = {
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
        match registry_map.get(&type_id_hash) {
            Some(info) => info.clone(),
            None => {
                set_last_error(GoudError::ResourceNotFound(format!(
                    "Component type {} not registered",
                    type_id_hash
                )));
                return 0;
            }
        }
    };

    if component_size != type_info.size {
        set_last_error(GoudError::InvalidState(format!(
            "Component size mismatch: expected {}, got {}",
            type_info.size, component_size
        )));
        return 0;
    }

    let mut storage_map = get_context_storage_map();
    let map = storage_map.get_or_insert_with(HashMap::new);

    let key = context_key(context_id);
    let context_storage = map.entry(key).or_default();
    let storage =
        context_storage.get_or_create_storage(type_id_hash, type_info.size, type_info.align);

    // SAFETY: Caller guarantees entity_ids points to count u64 values.
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

/// Removes the same component type from multiple entities in a batch.
///
/// # Safety
///
/// `entity_ids` must point to valid memory with `count` u64 values.
pub unsafe fn component_remove_batch_impl(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
) -> u32 {
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

    {
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
    }

    let mut storage_map = get_context_storage_map();
    let map = match storage_map.as_mut() {
        Some(m) => m,
        None => return 0,
    };

    let key = context_key(context_id);
    let context_storage = match map.get_mut(&key) {
        Some(s) => s,
        None => return 0,
    };

    let storage = match context_storage.get_storage_mut(type_id_hash) {
        Some(s) => s,
        None => return 0,
    };

    // SAFETY: Caller guarantees entity_ids points to count u64 values.
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
/// # Safety
///
/// - `entity_ids` must point to valid memory with `count` u64 values
/// - `out_results` must point to valid memory with `count` u8 values
pub unsafe fn component_has_batch_impl(
    context_id: GoudContextId,
    entity_ids: *const u64,
    count: u32,
    type_id_hash: u64,
    out_results: *mut u8,
) -> u32 {
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

    {
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
    }

    let storage_map = get_context_storage_map();

    let storage_exists = storage_map.as_ref().is_some_and(|map| {
        let key = context_key(context_id);
        map.get(&key)
            .and_then(|cs| cs.get_storage(type_id_hash))
            .is_some()
    });

    // SAFETY: Caller guarantees these pointers are valid for count elements.
    let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
    let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);

    if !storage_exists {
        for result in results_slice.iter_mut() {
            *result = 0;
        }
        return count;
    }

    let map = storage_map.as_ref().unwrap();
    let key = context_key(context_id);
    let context_storage = map.get(&key).unwrap();
    let storage = context_storage.get_storage(type_id_hash).unwrap();

    for (i, &entity_bits) in entity_slice.iter().enumerate() {
        results_slice[i] = if storage.contains(entity_bits) { 1 } else { 0 };
    }

    count
}
