//! Raw component storage and type registry infrastructure.
//!
//! This module contains the type-erased storage types used to hold component
//! data as raw bytes, as well as the global type registry and per-context
//! storage statics.

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::sync::Mutex;

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
pub(crate) struct RawComponentStorage {
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
    pub(crate) fn new(component_size: usize, component_align: usize) -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new(),
            component_size,
            component_align,
        }
    }

    /// Resolves the dense index for an entity, verifying the full entity bits
    /// (index AND generation) match the slot. Because `sparse` is keyed only by
    /// entity index, a recycled index would otherwise alias a dead entity's
    /// slot; checking `dense[dense_index]` prevents that stale read.
    fn resolve(&self, entity_bits: u64) -> Option<usize> {
        let index = Entity::from_bits(entity_bits).index() as usize;
        let dense_index = (*self.sparse.get(index)?)?;
        if self.dense[dense_index] == entity_bits {
            Some(dense_index)
        } else {
            None
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
    pub(crate) unsafe fn insert(&mut self, entity_bits: u64, data_ptr: *const u8) -> bool {
        let entity = Entity::from_bits(entity_bits);
        let index = entity.index() as usize;

        // Grow sparse vec if needed
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }

        if let Some(dense_index) = self.sparse[index] {
            // The index is occupied - either the same entity replacing its
            // component, or a recycled index reclaiming a dead entity's slot.
            // Rebind the slot to the current entity bits so generation checks
            // resolve correctly afterwards.
            self.dense[dense_index] = entity_bits;
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
    pub(crate) fn remove(&mut self, entity_bits: u64) -> bool {
        // Only remove when the slot belongs to this exact entity (generation
        // included), so a recycled index cannot evict the live entity's slot.
        let dense_index = match self.resolve(entity_bits) {
            Some(d) => d,
            None => return false,
        };

        self.sparse[Entity::from_bits(entity_bits).index() as usize] = None;
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
    }

    /// Gets a pointer to the component data for the given entity.
    pub(crate) fn get(&self, entity_bits: u64) -> *const u8 {
        match self.resolve(entity_bits) {
            Some(dense_index) => self.data[dense_index],
            None => std::ptr::null(),
        }
    }

    /// Gets a mutable pointer to the component data for the given entity.
    pub(crate) fn get_mut(&mut self, entity_bits: u64) -> *mut u8 {
        match self.resolve(entity_bits) {
            Some(dense_index) => self.data[dense_index],
            None => std::ptr::null_mut(),
        }
    }

    /// Checks if the entity has this component.
    pub(crate) fn contains(&self, entity_bits: u64) -> bool {
        self.resolve(entity_bits).is_some()
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
pub(crate) struct ContextComponentStorage {
    /// Maps type_id_hash to raw component storage
    storages: HashMap<u64, RawComponentStorage>,
}

impl ContextComponentStorage {
    /// Removes the entity from every component storage in this context.
    ///
    /// Call this when an entity is despawned so its dynamic-component slots are
    /// freed. Without it, storages grow unbounded and a later entity that
    /// recycles the same index could otherwise alias the dead entity's data.
    pub(crate) fn purge_entity(&mut self, entity_bits: u64) {
        for storage in self.storages.values_mut() {
            storage.remove(entity_bits);
        }
    }

    /// Gets or creates storage for a component type.
    pub(crate) fn get_or_create_storage(
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
    pub(crate) fn get_storage(&self, type_id_hash: u64) -> Option<&RawComponentStorage> {
        self.storages.get(&type_id_hash)
    }

    /// Gets mutable storage for a component type if it exists.
    pub(crate) fn get_storage_mut(
        &mut self,
        type_id_hash: u64,
    ) -> Option<&mut RawComponentStorage> {
        self.storages.get_mut(&type_id_hash)
    }
}

// ============================================================================
// Global Statics and Accessors
// ============================================================================

/// Global storage for per-context component data.
pub(crate) static CONTEXT_COMPONENT_STORAGE: Mutex<Option<HashMap<u64, ContextComponentStorage>>> =
    Mutex::new(None);

/// Gets or initializes the context component storage map.
pub(crate) fn get_context_storage_map(
) -> std::sync::MutexGuard<'static, Option<HashMap<u64, ContextComponentStorage>>> {
    CONTEXT_COMPONENT_STORAGE.lock().unwrap()
}

/// Purges an entity's dynamic components from the given context's storage.
///
/// Call after despawning an entity so its component slots are freed. `context_key`
/// is the caller-computed key (see `helpers::context_key`); this keeps the FFI
/// context type out of this layer. A no-op if the context has no storage.
pub(crate) fn purge_context_entity(context_key: u64, entity_bits: u64) {
    let mut storage_map = get_context_storage_map();
    if let Some(map) = storage_map.as_mut() {
        if let Some(context_storage) = map.get_mut(&context_key) {
            context_storage.purge_entity(entity_bits);
        }
    }
}

// ============================================================================
// Type Registry
// ============================================================================

/// Information about a registered component type.
#[derive(Debug, Clone)]
pub(crate) struct ComponentTypeInfo {
    /// Size of the component in bytes.
    pub(crate) size: usize,
    /// Alignment of the component in bytes.
    pub(crate) align: usize,
}

/// Global registry mapping type IDs to component information.
pub(crate) static COMPONENT_TYPE_REGISTRY: Mutex<Option<HashMap<u64, ComponentTypeInfo>>> =
    Mutex::new(None);

/// Gets or initializes the component type registry.
pub(crate) fn get_type_registry(
) -> std::sync::MutexGuard<'static, Option<HashMap<u64, ComponentTypeInfo>>> {
    COMPONENT_TYPE_REGISTRY.lock().unwrap()
}

/// Registers a component type with the given information.
pub(crate) fn register_component_type_internal(
    type_id_hash: u64,
    size: usize,
    align: usize,
) -> bool {
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
pub(crate) fn get_component_type_info(type_id_hash: u64) -> Option<ComponentTypeInfo> {
    let registry = get_type_registry();
    registry.as_ref()?.get(&type_id_hash).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Entity;

    fn insert_u32(storage: &mut RawComponentStorage, entity: Entity, value: u32) -> bool {
        let bytes = value.to_ne_bytes();
        // SAFETY: `bytes` is 4 valid bytes matching the registered size/align (4/4).
        unsafe { storage.insert(entity.to_bits(), bytes.as_ptr()) }
    }

    fn read_u32(storage: &RawComponentStorage, entity: Entity) -> Option<u32> {
        let ptr = storage.get(entity.to_bits());
        if ptr.is_null() {
            return None;
        }
        let mut bytes = [0u8; 4];
        // SAFETY: `ptr` came from a slot sized for a u32.
        unsafe { std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 4) };
        Some(u32::from_ne_bytes(bytes))
    }

    #[test]
    fn recycled_index_does_not_read_stale_component() {
        let mut storage = RawComponentStorage::new(4, 4);
        let old = Entity::new(5, 1);
        assert!(insert_u32(&mut storage, old, 111));
        assert_eq!(read_u32(&storage, old), Some(111));

        // A recycled entity at the same index but a newer generation must not
        // see the dead entity's component data.
        let recycled = Entity::new(5, 2);
        assert_eq!(read_u32(&storage, recycled), None);
        assert!(!storage.contains(recycled.to_bits()));
    }

    #[test]
    fn insert_reclaims_a_recycled_slot() {
        let mut storage = RawComponentStorage::new(4, 4);
        insert_u32(&mut storage, Entity::new(5, 1), 111);

        let recycled = Entity::new(5, 2);
        assert!(insert_u32(&mut storage, recycled, 222));
        assert_eq!(read_u32(&storage, recycled), Some(222));
        // The dead entity's bits no longer resolve.
        assert_eq!(read_u32(&storage, Entity::new(5, 1)), None);
    }

    #[test]
    fn remove_only_evicts_the_matching_generation() {
        let mut storage = RawComponentStorage::new(4, 4);
        let live = Entity::new(5, 1);
        insert_u32(&mut storage, live, 111);

        // Removing a different generation at the same index is a no-op.
        assert!(!storage.remove(Entity::new(5, 2).to_bits()));
        assert_eq!(read_u32(&storage, live), Some(111));

        assert!(storage.remove(live.to_bits()));
        assert_eq!(read_u32(&storage, live), None);
    }

    #[test]
    fn storage_stays_bounded_across_respawn_cycles() {
        let mut storage = RawComponentStorage::new(4, 4);
        for generation in 0..1000u32 {
            let entity = Entity::new(0, generation);
            insert_u32(&mut storage, entity, generation);
            assert!(storage.remove(entity.to_bits()));
        }
        assert_eq!(
            storage.dense.len(),
            0,
            "despawn/respawn must not grow storage"
        );
    }

    #[test]
    fn purge_entity_clears_every_storage() {
        let mut ctx = ContextComponentStorage::default();
        let entity = Entity::new(3, 1);
        for type_id in [10u64, 20u64] {
            let storage = ctx.get_or_create_storage(type_id, 4, 4);
            insert_u32(storage, entity, 7);
        }

        ctx.purge_entity(entity.to_bits());

        for type_id in [10u64, 20u64] {
            assert!(!ctx.get_storage(type_id).unwrap().contains(entity.to_bits()));
        }
    }
}
