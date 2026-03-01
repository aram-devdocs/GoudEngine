//! # SDK Generic Component Operations API
//!
//! Provides static functions for type-erased component operations: register,
//! add, remove, has, get, get_mut, and batch variants. These are annotated
//! with `#[goud_api]` to auto-generate FFI wrappers that replace the
//! hand-written functions in `ffi/component.rs`.
//!
//! All methods are static (no `self` receiver) because they operate on the
//! global per-context component storage, not on a GoudGame instance.
//!
//! The raw component storage and type registry internals remain in
//! `ffi/component.rs` and are accessed via public helper functions.

use crate::core::component_ops::{
    component_add_impl, component_add_batch_impl, component_get_impl,
    component_get_mut_impl, component_has_batch_impl, component_has_impl,
    component_register_type_impl, component_remove_batch_impl,
    component_remove_impl,
};
use crate::core::context_registry::GoudContextId;
use crate::core::types::{GoudEntityId, GoudResult};

/// Zero-sized type hosting generic component FFI operations.
pub struct ComponentOps;

#[goud_engine_macros::goud_api(module = "component")]
impl ComponentOps {
    /// Registers a component type with the engine.
    ///
    /// # Safety
    ///
    /// - `name_ptr` must be a valid pointer to UTF-8 data (or null)
    /// - `size` and `align` must match the actual type layout
    pub unsafe fn register_type(
        type_id_hash: u64,
        name_ptr: *const u8,
        name_len: usize,
        size: usize,
        align: usize,
    ) -> bool {
        component_register_type_impl(type_id_hash, name_ptr, name_len, size, align)
    }

    /// Adds a component to an entity.
    ///
    /// # Safety
    ///
    /// - `data_ptr` must point to valid component data
    /// - `data_size` must match the registered component size
    pub unsafe fn add(
        context_id: GoudContextId,
        entity_id: GoudEntityId,
        type_id_hash: u64,
        data_ptr: *const u8,
        data_size: usize,
    ) -> GoudResult {
        component_add_impl(context_id, entity_id, type_id_hash, data_ptr, data_size)
    }

    /// Removes a component from an entity.
    pub fn remove(
        context_id: GoudContextId,
        entity_id: GoudEntityId,
        type_id_hash: u64,
    ) -> GoudResult {
        component_remove_impl(context_id, entity_id, type_id_hash)
    }

    /// Checks if an entity has a specific component.
    pub fn has(
        context_id: GoudContextId,
        entity_id: GoudEntityId,
        type_id_hash: u64,
    ) -> bool {
        component_has_impl(context_id, entity_id, type_id_hash)
    }

    /// Gets a read-only pointer to a component on an entity.
    pub fn get(
        context_id: GoudContextId,
        entity_id: GoudEntityId,
        type_id_hash: u64,
    ) -> *const u8 {
        component_get_impl(context_id, entity_id, type_id_hash)
    }

    /// Gets a mutable pointer to a component on an entity.
    pub fn get_mut(
        context_id: GoudContextId,
        entity_id: GoudEntityId,
        type_id_hash: u64,
    ) -> *mut u8 {
        component_get_mut_impl(context_id, entity_id, type_id_hash)
    }

    /// Adds the same component type to multiple entities in a batch.
    ///
    /// # Safety
    ///
    /// - `entity_ids` must point to valid memory with `count` u64 values
    /// - `data_ptr` must point to `count * component_size` bytes of data
    pub unsafe fn add_batch(
        context_id: GoudContextId,
        entity_ids: *const u64,
        count: u32,
        type_id_hash: u64,
        data_ptr: *const u8,
        component_size: usize,
    ) -> u32 {
        component_add_batch_impl(
            context_id, entity_ids, count, type_id_hash, data_ptr,
            component_size,
        )
    }

    /// Removes the same component type from multiple entities in a batch.
    ///
    /// # Safety
    ///
    /// `entity_ids` must point to valid memory with `count` u64 values.
    pub unsafe fn remove_batch(
        context_id: GoudContextId,
        entity_ids: *const u64,
        count: u32,
        type_id_hash: u64,
    ) -> u32 {
        component_remove_batch_impl(context_id, entity_ids, count, type_id_hash)
    }

    /// Checks if multiple entities have a specific component type.
    ///
    /// # Safety
    ///
    /// - `entity_ids` must point to valid memory with `count` u64 values
    /// - `out_results` must point to valid memory with `count` u8 values
    pub unsafe fn has_batch(
        context_id: GoudContextId,
        entity_ids: *const u64,
        count: u32,
        type_id_hash: u64,
        out_results: *mut u8,
    ) -> u32 {
        component_has_batch_impl(
            context_id, entity_ids, count, type_id_hash, out_results,
        )
    }
}
