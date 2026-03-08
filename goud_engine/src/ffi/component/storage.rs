//! Raw component storage types used by the FFI component layer.
//!
//! This module provides type-erased, heap-allocated storage for FFI component
//! data. Components are stored as raw bytes in a sparse set structure, allowing
//! the engine to hold arbitrary component layouts without knowing their Rust type.

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError};
use crate::ecs::Entity;
use crate::ffi::GoudContextId;

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
pub(super) struct RawComponentStorage {
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
    pub(super) fn new(component_size: usize, component_align: usize) -> Self {
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
    pub(super) unsafe fn insert(&mut self, entity_bits: u64, data_ptr: *const u8) -> bool {
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
                set_last_error(GoudError::InternalError(
                    "Failed to allocate component storage".to_string(),
                ));
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
    pub(super) fn remove(&mut self, entity_bits: u64) -> bool {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            set_last_error(GoudError::EntityNotFound);
            return false;
        }

        if let Some(dense_index) = self.sparse[index].take() {
            // Free the component data
            let ptr = self.data[dense_index];
            if !ptr.is_null() {
                // SAFETY: ptr was allocated with the same layout via alloc() in insert(),
                // and is non-null as checked above.
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
    pub(super) fn get(&self, entity_bits: u64) -> *const u8 {
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
    pub(super) fn get_mut(&mut self, entity_bits: u64) -> *mut u8 {
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
    pub(super) fn contains(&self, entity_bits: u64) -> bool {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            set_last_error(GoudError::EntityNotFound);
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
                // SAFETY: Each ptr in self.data was allocated with this same layout via
                // alloc() in insert(), and is non-null as checked above.
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
pub(super) struct ContextComponentStorage {
    /// Maps type_id_hash to raw component storage
    storages: HashMap<u64, RawComponentStorage>,
}

impl ContextComponentStorage {
    /// Gets or creates storage for a component type.
    pub(super) fn get_or_create_storage(
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
    pub(super) fn get_storage(&self, type_id_hash: u64) -> Option<&RawComponentStorage> {
        self.storages.get(&type_id_hash)
    }

    /// Gets mutable storage for a component type if it exists.
    pub(super) fn get_storage_mut(
        &mut self,
        type_id_hash: u64,
    ) -> Option<&mut RawComponentStorage> {
        self.storages.get_mut(&type_id_hash)
    }
}

/// Global storage for per-context component data.
///
/// Maps context ID (as u64) to component storage for that context.
static CONTEXT_COMPONENT_STORAGE: Mutex<Option<HashMap<u64, ContextComponentStorage>>> =
    Mutex::new(None);

/// Gets or initializes the context component storage map.
pub(super) fn get_context_storage_map(
) -> Option<std::sync::MutexGuard<'static, Option<HashMap<u64, ContextComponentStorage>>>> {
    CONTEXT_COMPONENT_STORAGE.lock().ok()
}

/// Packs a `GoudContextId` into a `u64` map key.
pub(super) fn context_key(context_id: GoudContextId) -> u64 {
    (context_id.generation() as u64) << 32 | (context_id.index() as u64)
}

/// Converts an FFI `GoudEntityId` to an internal `Entity`.
#[inline]
pub(super) fn entity_from_ffi(entity_id: crate::ffi::GoudEntityId) -> crate::ecs::Entity {
    crate::ecs::Entity::from_bits(entity_id.bits())
}
